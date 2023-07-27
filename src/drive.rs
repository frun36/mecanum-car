use rppal::gpio::{Error, Gpio};

use serde::{Deserialize, Serialize};

mod motor;

/// Provides simple API for speed control
#[derive(Debug, Deserialize, Serialize)]
pub enum Speed {
    Low,
    Medium,
    High,
}

/// Converts `Speed` values to pwm frequencies
fn get_speed(speed: &Speed) -> f64 {
    match *speed {
        Speed::Low => 0.3,
        Speed::Medium => 0.6,
        Speed::High => 1.0,
    }
}

/// Supported robot motions
#[derive(Debug, Deserialize, Serialize)]
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
}

impl Drive {
    /// Creates new `Drive` instance
    pub fn new(
        gpio: &Gpio,
        motor_pins: [(u8, u8, u8); 4],
        pwm_frequency: f64,
    ) -> Result<Self, Error> {
        Ok(Self {
            motors: [
                motor::Motor::new(gpio, motor_pins[0].0, motor_pins[0].1, motor_pins[0].2)?,
                motor::Motor::new(gpio, motor_pins[1].0, motor_pins[1].1, motor_pins[1].2)?,
                motor::Motor::new(gpio, motor_pins[2].0, motor_pins[2].1, motor_pins[2].2)?,
                motor::Motor::new(gpio, motor_pins[3].0, motor_pins[3].1, motor_pins[3].2)?,
            ],
            pwm_frequency,
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
    pub fn move_robot_pwm_speed(&mut self, motion: &Motion, duty_cycle: f64) -> Result<(), Error> {
        let motor_speeds = match *motion {
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
    pub fn move_robot(&mut self, motion: &Motion, speed: &Speed) -> Result<(), Error> {
        let duty_cycle = get_speed(speed);
        self.move_robot_pwm_speed(motion, duty_cycle)?;
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
