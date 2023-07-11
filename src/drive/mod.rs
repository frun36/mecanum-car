use rppal::gpio::{Gpio, Error};

mod motor;

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Speed {
    Low,
    Medium,
    High,
}

fn get_speed(speed: &Speed) -> f64 {
    match *speed {
        Speed::Low => 0.3,
        Speed::Medium => 0.6,
        Speed::High => 1.0,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
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

pub struct Drive {
    motors: [motor::Motor; 4],
    pwm_frequency: f64,
}

impl Drive {
    pub fn new(gpio: &Gpio, motor_pins: [(u8, u8, u8); 4], pwm_frequency: f64) -> Result<Self, Error> {
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

    fn enable_motors(&mut self, motor_speeds: &[f64]) -> Result<(), Error> {
        for i in 0..4  {
            if motor_speeds[i] > 0. {
                self.motors[i].enable_fwd(self.pwm_frequency, motor_speeds[i])?;
            } else if motor_speeds[i] < 0. {
                self.motors[i].enable_bwd(self.pwm_frequency, -motor_speeds[i])?;
            } else {
                self.motors[i].stop();
            }
        }
        Ok(())
    }

    pub fn move_robot(&mut self, motion: &Motion, speed: &Speed) -> Result<(), Error> {
        let speed = get_speed(speed);
        let motor_speeds = match *motion {
            Motion::Forward => [speed, speed, speed, speed],
            Motion::ForwardRight => [0., speed, 0., speed],
            Motion::Right => [-speed, speed, -speed, speed],
            Motion::BackwardRight => [-speed, 0., -speed, 0.],
            Motion::Backward => [-speed, -speed, -speed, -speed],
            Motion::BackwardLeft => [0., -speed, 0., -speed],
            Motion::Left => [speed, -speed, speed, -speed],
            Motion::ForwardLeft => [speed, 0., speed, 0.],
            Motion::RightRot => [speed, -speed, -speed, speed],
            Motion::LeftRot => [-speed, speed, speed, -speed],
            Motion::Stop => [0., 0., 0., 0.],
        };
        self.enable_motors(&motor_speeds)?;
        Ok(())
    }
}
