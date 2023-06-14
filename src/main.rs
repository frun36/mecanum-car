use std::thread;
use std::time::Duration;

use rppal::gpio::Gpio;
use rppal::gpio::OutputPin;

const MOTOR0_FWD: u8 = 4;
const MOTOR0_BWD: u8 = 17;
const MOTOR0_PWM: u8 = 12;

const MOTOR1_FWD: u8 = 27;
const MOTOR1_BWD: u8 = 22;
const MOTOR1_PWM: u8 = 5;

const MOTOR2_FWD: u8 = 18;
const MOTOR2_BWD: u8 = 23;
const MOTOR2_PWM: u8 = 13;

const MOTOR3_FWD: u8 = 14;
const MOTOR3_BWD: u8 = 15;
const MOTOR3_PWM: u8 = 6;

const MOTOR_PWM_FREQUENCY: f64 = 100.0;

struct Motor {
    fwd_pin: OutputPin,
    bwd_pin: OutputPin,
    pwm_pin: OutputPin,
}

impl Motor {
    fn new(gpio: &Gpio, fwd_pin_number: u8, bwd_pin_number: u8, pwm_pin_number: u8) -> Self {
        Self {
            fwd_pin: gpio.get(fwd_pin_number).expect("Error: Couldn't get gpio").into_output(),
            bwd_pin: gpio.get(bwd_pin_number).expect("Error: Couldn't get gpio").into_output(),
            pwm_pin: gpio.get(pwm_pin_number).expect("Error: Couldn't get gpio").into_output(),
        }
    }

    fn enable_fwd(&mut self, duty_cycle: f64) {
        self.bwd_pin.set_low();
        self.pwm_pin.set_high();
        self.fwd_pin.set_high();
        self.pwm_pin.set_pwm_frequency(MOTOR_PWM_FREQUENCY, duty_cycle);
    }
    
    fn enable_bwd(&mut self, duty_cycle: f64) {
        self.fwd_pin.set_low();
        self.pwm_pin.set_high();
        self.bwd_pin.set_high();
        self.pwm_pin.set_pwm_frequency(MOTOR_PWM_FREQUENCY, duty_cycle);
    }

    fn stop(&mut self) {
        self.pwm_pin.set_low();
        self.fwd_pin.set_low();
        self.bwd_pin.set_low();
    }
}

fn main() {
    let gpio = Gpio::new().unwrap();
    let mut motors = [Motor::new(&gpio, MOTOR0_FWD, MOTOR0_BWD, MOTOR0_PWM), 
                Motor::new(&gpio, MOTOR1_FWD, MOTOR1_BWD, MOTOR1_PWM),
                Motor::new(&gpio, MOTOR2_FWD, MOTOR2_BWD, MOTOR2_PWM),
                Motor::new(&gpio, MOTOR3_FWD, MOTOR3_BWD, MOTOR3_PWM)];

    // let mut motor = Motor::new(&gpio, MOTOR0_FWD, MOTOR0_BWD, MOTOR0_PWM);
    /*for v in 1..=3 {
        for motor in motors.iter_mut() {
            motor.enable_fwd((v as f64) * 0.1);
        }
        thread::sleep(Duration::from_millis(2000));
        for motor in motors.iter_mut() {
            motor.stop();
        }
        thread::sleep(Duration::from_millis(1000));
    }*/
    motors[0].enable_bwd(0.5);
    motors[1].enable_fwd(0.5);
    motors[2].enable_bwd(0.5);
    motors[3].enable_fwd(0.5);

    thread::sleep(Duration::from_millis(1000));
    for motor in motors.iter_mut() {
        motor.stop();
    }
}

    /*
    let mut motor = (
            gpio.get(MOTOR0_FWD).expect("Error: Couldn't get gpio").into_output(),
            gpio.get(MOTOR0_BWD).expect("Error: Couldn't get gpio").into_output(),
            gpio.get(MOTOR0_PWM).expect("Error: Couldn't get gpio").into_output(),
        );
    thread::sleep(Duration::from_millis(2000));
    motor.1.set_low();
    // motor.2.set_high();
    motor.2.set_pwm_frequency(100.0, 0.25);
    motor.0.set_high();
    thread::sleep(Duration::from_millis(1500));
    motor.2.set_pwm_frequency(100.0, 1.0);
    thread::sleep(Duration::from_millis(1500));
    motor.0.set_low();
    motor.2.set_low(); */
/*
    for motor in motors.iter_mut() {
        motor.enable_fwd();
    }
    
    thread::sleep(Duration::from_millis(1000));

    for motor in motors.iter_mut() {
        motor.stop();
    }
    
    thread::sleep(Duration::from_millis(100));
        
    for motor in motors.iter_mut() {
        motor.enable_bwd();
    }
    
    thread::sleep(Duration::from_millis(1000));

    for motor in motors.iter_mut() {
        motor.stop();
    }
*/
