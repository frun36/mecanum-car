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

pub struct Drive {
    motors: [motor::Motor; 4],
    pwm_frequency: f64,
}

impl Drive {
    pub fn new(gpio: &Gpio, motor_pins: [(u8, u8, u8); 4], pwm_frequency: f64) -> Self {
        Self {
            motors: [
                motor::Motor::new(&gpio, motor_pins[0].0, motor_pins[0].1, motor_pins[0].2),
                motor::Motor::new(&gpio, motor_pins[1].0, motor_pins[1].1, motor_pins[1].2),
                motor::Motor::new(&gpio, motor_pins[2].0, motor_pins[2].1, motor_pins[2].2),
                motor::Motor::new(&gpio, motor_pins[3].0, motor_pins[3].1, motor_pins[3].2),
            ],
            pwm_frequency,
        }
    }

    pub fn fwd(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        for m in self.motors.iter_mut() {
            m.enable_fwd(self.pwm_frequency, speed);
        }
    }

    pub fn bwd(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        for m in self.motors.iter_mut() {
            m.enable_bwd(self.pwm_frequency, speed);
        }
    }

    pub fn rwd(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_bwd(self.pwm_frequency, speed);
        self.motors[1].enable_fwd(self.pwm_frequency, speed);
        self.motors[2].enable_bwd(self.pwm_frequency, speed);
        self.motors[3].enable_fwd(self.pwm_frequency, speed);
    }

    pub fn lwd(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_fwd(self.pwm_frequency, speed);
        self.motors[1].enable_bwd(self.pwm_frequency, speed);
        self.motors[2].enable_fwd(self.pwm_frequency, speed);
        self.motors[3].enable_bwd(self.pwm_frequency, speed);
    }

    pub fn l_rot(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_bwd(self.pwm_frequency, speed);
        self.motors[1].enable_fwd(self.pwm_frequency, speed);
        self.motors[2].enable_fwd(self.pwm_frequency, speed);
        self.motors[3].enable_bwd(self.pwm_frequency, speed);
    }

    pub fn r_rot(&mut self, speed: &Speed) {
        let speed = get_speed(speed);
        self.motors[0].enable_fwd(self.pwm_frequency, speed);
        self.motors[1].enable_bwd(self.pwm_frequency, speed);
        self.motors[2].enable_bwd(self.pwm_frequency, speed);
        self.motors[3].enable_fwd(self.pwm_frequency, speed);
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
