use rppal::gpio::Gpio;

mod motor;

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

pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    R,
    L,
}

pub struct Drive {
    motors: [motor::Motor; 4],
    pwm_frequency: f64,
}

impl Drive {
    pub fn new(gpio: &Gpio, motor_pins: [(u8, u8, u8); 4], pwm_frequency: f64) -> Self {
        Self {
            motors: [
                motor::Motor::new(gpio, motor_pins[0].0, motor_pins[0].1, motor_pins[0].2),
                motor::Motor::new(gpio, motor_pins[1].0, motor_pins[1].1, motor_pins[1].2),
                motor::Motor::new(gpio, motor_pins[2].0, motor_pins[2].1, motor_pins[2].2),
                motor::Motor::new(gpio, motor_pins[3].0, motor_pins[3].1, motor_pins[3].2),
            ],
            pwm_frequency,
        }
    }

    fn enable_motors(&mut self, motor_speeds: &[f64]) {
        (0..4).for_each(|i| {
            if motor_speeds[i] > 0. {
                self.motors[i].enable_fwd(self.pwm_frequency, motor_speeds[i]);
            } else if motor_speeds[i] < 0. {
                self.motors[i].enable_bwd(self.pwm_frequency, -motor_speeds[i]);
            }
        });
    }

    pub fn move_robot(&mut self, direction: &Direction, speed: &Speed) {
        let speed = get_speed(speed);
        let motor_speeds = match *direction {
            Direction::N => [speed, speed, speed, speed],
            Direction::NE => [0., speed, 0., speed],
            Direction::E => [-speed, speed, -speed, speed],
            Direction::SE => [-speed, 0., -speed, 0.],
            Direction::S => [-speed, -speed, -speed, -speed],
            Direction::SW => [0., -speed, 0., -speed],
            Direction::W => [speed, -speed, speed, -speed],
            Direction::NW => [speed, 0., speed, 0.],
            Direction::R => [speed, -speed, -speed, speed],
            Direction::L => [-speed, speed, speed, -speed],
        };
        self.enable_motors(&motor_speeds);
    }

    pub fn l_turn(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_fwd(self.pwm_frequency, speed - 0.3);
        self.motors[1].enable_fwd(self.pwm_frequency, speed);
        self.motors[2].enable_fwd(self.pwm_frequency, speed);
        self.motors[3].enable_fwd(self.pwm_frequency, speed - 0.3);
    }

    pub fn r_turn(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_fwd(self.pwm_frequency, speed);
        self.motors[1].enable_fwd(self.pwm_frequency, speed - 0.3);
        self.motors[2].enable_fwd(self.pwm_frequency, speed - 0.3);
        self.motors[3].enable_fwd(self.pwm_frequency, speed);
    }

    pub fn stop(&mut self) {
        for m in self.motors.iter_mut() {
            m.stop();
        }
    }
}
