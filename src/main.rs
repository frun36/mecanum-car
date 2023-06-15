use std::thread;
use std::time::Duration;
use std::io;

use rppal::gpio::Gpio;

mod drive;

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

fn main() {
    let gpio = Gpio::new().unwrap();

    let mut drive = drive::Drive::new(
        &gpio,
        [
            (MOTOR0_FWD, MOTOR0_BWD, MOTOR0_PWM),
            (MOTOR1_FWD, MOTOR1_BWD, MOTOR1_PWM),
            (MOTOR2_FWD, MOTOR2_BWD, MOTOR2_PWM),
            (MOTOR3_FWD, MOTOR3_BWD, MOTOR3_PWM),
        ],
        MOTOR_PWM_FREQUENCY
    );

    let mut input = String::new();
    let stdin = io::stdin();
    loop {
        stdin.read_line(&mut input).expect("Couldn't get user input");
        match input.trim() {
            "fwd" => drive.fwd(0.5),
            "bwd" => drive.bwd(0.5),
            "rwd" => drive.rwd(0.5),
            "lwd" => drive.lwd(0.5),
            "rrot" => drive.r_rot(0.5),
            "lrot" => drive.l_rot(0.5),
            "q" => break,
            _ => println!("Invalid command"),
        }
        thread::sleep(Duration::from_millis(1000));
        drive.stop();
        input.clear();
    }
}
