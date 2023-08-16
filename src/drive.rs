use std::{f64::consts::PI, fmt::Display, time::Duration};

use actix_web::rt::time;
use rppal::gpio::{Error, Gpio};

use serde::{Deserialize, Serialize};

use actix::{fut::wrap_future, prelude::*};

use crate::{server::WebSocket, Device};

mod motor;

pub const WHEEL_CIRCUMFERENCE: f64 = 0.25; // in meters
pub const ROBOT_RADIUS: f64 = 0.11; // in meters

/// Provides simple API for speed control
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Speed {
    Low,
    Medium,
    High,
    Manual(f64),
}

impl Speed {
    /// Converts `Speed` values to pwm frequencies
    pub fn get_duty_cycle(&self) -> f64 {
        match self {
            Speed::Low => 0.3,
            Speed::Medium => 0.6,
            Speed::High => 1.0,
            Speed::Manual(v) => *v,
        }
    }

    /// Converts `Speed` values to approximate robot velocity
    pub fn get_velocity(&self) -> f64 {
        let duty_cycle = self.get_duty_cycle();
        f64::powf(f64::ln(0.3474 * duty_cycle + 0.9077), 0.25)
    }

    /// Converts `Speed` values to approximate wheel rpm
    #[allow(dead_code)]
    pub fn get_rpm(&self) -> f64 {
        let velocity = self.get_velocity();
        60. * velocity / WHEEL_CIRCUMFERENCE
    }
}

/// Supported robot motions
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum Motion {
    Forward,
    ForwardRight,
    Right,
    BackwardRight,
    Backward,
    BackwardLeft,
    Left,
    ForwardLeft,
    RightRot,
    LeftRot,
    Stop,
}

impl Display for Motion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let alias = match self {
            Motion::Forward => "f",
            Motion::ForwardRight => "fr",
            Motion::Right => "r",
            Motion::BackwardRight => "br",
            Motion::Backward => "b",
            Motion::BackwardLeft => "bl",
            Motion::Left => "l",
            Motion::ForwardLeft => "fl",
            Motion::RightRot => "rr",
            Motion::LeftRot => "lr",
            Motion::Stop => "s",
        };
        write!(f, "{}", alias)
    }
}

/// Allows control of the motion of a 4-wheeled mecanum robot<br>
/// Wheel layout:<br>
/// \\------/<br>
/// \\-3--2-/<br>
/// \\------/<br>
/// <br>
/// <br>
/// /------\\<br>
/// /-0--1-\\<br>
/// /------\\<br>
pub struct Drive {
    motors: [motor::Motor; 4],
    pwm_frequency: f64,
    websocket_addr: Option<Addr<WebSocket>>,
}

impl Drive {
    /// Creates new `Drive` instance
    pub fn new(
        gpio: &Gpio,
        motor_pins: [(u8, u8, u8); 4],
        pwm_frequency: f64,
        websocket_addr: Option<Addr<WebSocket>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            motors: [
                motor::Motor::new(gpio, motor_pins[0].0, motor_pins[0].1, motor_pins[0].2)?,
                motor::Motor::new(gpio, motor_pins[1].0, motor_pins[1].1, motor_pins[1].2)?,
                motor::Motor::new(gpio, motor_pins[2].0, motor_pins[2].1, motor_pins[2].2)?,
                motor::Motor::new(gpio, motor_pins[3].0, motor_pins[3].1, motor_pins[3].2)?,
            ],
            pwm_frequency,
            websocket_addr,
        })
    }

    /// Enables all motors, speeds specified in `motor_speeds` (positive: forward, negative: backward)
    fn enable_motors(&mut self, motor_speeds: &[f64]) -> Result<(), Error> {
        motor_speeds
            .iter()
            .enumerate()
            .try_for_each(|(i, duty_cycle)| -> Result<(), Error> {
                let duty_cycle = *duty_cycle;
                if duty_cycle > 0. {
                    self.motors[i].enable_fwd(self.pwm_frequency, duty_cycle)?;
                } else if duty_cycle < 0. {
                    self.motors[i].enable_bwd(self.pwm_frequency, -duty_cycle)?;
                } else {
                    self.motors[i].stop();
                }
                Ok(())
            })?;
        Ok(())
    }

    /// Starts specified `motion` with specified PWM `duty_cycle`
    fn enable_with_duty_cycle(&mut self, motion: Motion, duty_cycle: f64) -> Result<(), Error> {
        let motor_speeds = match motion {
            Motion::Forward => [duty_cycle, duty_cycle, duty_cycle, duty_cycle],
            Motion::ForwardRight => [0., duty_cycle, 0., duty_cycle],
            Motion::Right => [-duty_cycle, duty_cycle, -duty_cycle, duty_cycle],
            Motion::BackwardRight => [-duty_cycle, 0., -duty_cycle, 0.],
            Motion::Backward => [-duty_cycle, -duty_cycle, -duty_cycle, -duty_cycle],
            Motion::BackwardLeft => [0., -duty_cycle, 0., -duty_cycle],
            Motion::Left => [duty_cycle, -duty_cycle, duty_cycle, -duty_cycle],
            Motion::ForwardLeft => [duty_cycle, 0., duty_cycle, 0.],
            Motion::RightRot => [duty_cycle, -duty_cycle, -duty_cycle, duty_cycle],
            Motion::LeftRot => [-duty_cycle, duty_cycle, duty_cycle, -duty_cycle],
            Motion::Stop => [0., 0., 0., 0.],
        };
        self.enable_motors(&motor_speeds)?;
        Ok(())
    }

    /// Starts specified `motion` with specified `speed`
    fn enable(&mut self, motion: Motion, speed: Speed) -> Result<(), Error> {
        let duty_cycle = speed.get_duty_cycle();
        self.enable_with_duty_cycle(motion, duty_cycle)?;
        Ok(())
    }

    /// Move robot with `motion` a specified `distance` with specified `speed`. Wildly inaccurate:
    /// * Based only on forward/backward measurements,
    /// * Doesn't take into account the time it takes the motors to accelerate
    fn move_distance(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        motion: Motion,
        speed: Speed,
        distance: f64,
    ) -> Result<(), Error> {
        if motion == Motion::RightRot || motion == Motion::LeftRot {
            panic!("Invalid motion for moving robot a specified distance");
        }
        let time_s = distance / speed.get_velocity();
        let time = Duration::from_secs_f64(time_s);
        let addr = ctx.address();
        let fut = async move {
            addr.do_send(DriveMessage::Enable { motion, speed });
            time::sleep(time).await;
            addr.do_send(DriveMessage::Disable);
        };

        ctx.spawn(wrap_future(fut));
        Ok(())
    }

    /// Rotate robot with `motion` a specified `angle` with specified `speed`. Wildly inaccurate:
    /// * Based only on forward/backward measurements,
    /// * Doesn't take into account the time it takes the motors to accelerate
    /// Angle can be calibrated manually using the frontend `Slip` slider
    fn rotate_angle(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
        motion: Motion,
        speed: Speed,
        angle: f64,
    ) -> Result<(), Error> {
        if motion != Motion::RightRot && motion != Motion::LeftRot {
            panic!("Invalid motion for rotating robot a specified angle");
        }
        let distance = 2. * PI * ROBOT_RADIUS * angle / 360.;
        let time_s = distance / speed.get_velocity();
        let time = Duration::from_secs_f64(time_s);
        let addr = ctx.address();
        let fut = async move {
            addr.do_send(DriveMessage::Enable { motion, speed });
            time::sleep(time).await;
            addr.do_send(DriveMessage::Disable);
        };
        
        ctx.spawn(wrap_future(fut));
        Ok(())
    }

    /// Prints all motor pins
    pub fn list_motors(&self) {
        self.motors.iter().enumerate().for_each(|(i, m)| {
            println!("Motor {}:", i);
            m.print_pins();
            println!();
        });
    }
}

// Actor communication

impl Actor for Drive {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Drive actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.enable(Motion::Stop, Speed::Low).unwrap();
    }
}

impl Device for Drive {
    fn set_websocket_addr(&mut self, addr: Addr<WebSocket>) {
        self.websocket_addr = Some(addr);
    }
}

#[derive(Deserialize, Message)]
#[rtype(result = "()")]
#[serde(tag = "variant")]
pub enum DriveMessage {
    Enable {
        motion: Motion,
        speed: Speed,
    },
    Disable,
    Move {
        motion: Motion,
        speed: Speed,
        distance: f64,
    },
    Rotate {
        motion: Motion,
        speed: Speed,
        angle: f64,
    },
}

impl Handler<DriveMessage> for Drive {
    type Result = ();

    fn handle(&mut self, msg: DriveMessage, ctx: &mut Self::Context) -> Self::Result {
        let response = match msg {
            DriveMessage::Enable { motion, speed } => match self.enable(motion, speed) {
                Ok(_) => DriveResponse::Ok(msg),
                Err(e) => DriveResponse::Err(e),
            },
            DriveMessage::Disable => match self.enable(Motion::Stop, Speed::Low) {
                Ok(_) => DriveResponse::Ok(msg),
                Err(e) => DriveResponse::Err(e),
            },
            DriveMessage::Move {
                motion,
                speed,
                distance,
            } => match self.move_distance(ctx, motion, speed, distance) {
                Ok(_) => DriveResponse::Ok(msg),
                Err(e) => DriveResponse::Err(e),
            },
            DriveMessage::Rotate {
                motion,
                speed,
                angle,
            } => match self.rotate_angle(ctx, motion, speed, angle) {
                Ok(_) => DriveResponse::Ok(msg),
                Err(e) => DriveResponse::Err(e),
            },
        };

        self.websocket_addr.as_ref().unwrap().do_send(response);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum DriveResponse {
    Ok(DriveMessage),
    Err(Error),
}
