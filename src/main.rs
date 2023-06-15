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
    let mut current_speed = drive::Speed::Medium;
    loop {
        stdin.read_line(&mut input).expect("Couldn't get user input");
        match input.trim() {
            "lo" => current_speed = drive::Speed::Low,
            "med" => current_speed = drive::Speed::Medium,
            "hi" => current_speed = drive::Speed::High,
            "fwd" => drive.fwd(&current_speed),
            "bwd" => drive.bwd(&current_speed),
            "rwd" => drive.rwd(&current_speed),
            "lwd" => drive.lwd(&current_speed),
            "rrot" => drive.r_rot(&current_speed),
            "lrot" => drive.l_rot(&current_speed),
            "rturn" => drive.r_turn(&current_speed),
            "lturn" => drive.l_turn(&current_speed),
            "q" => break,
            _ => println!("Invalid command"),
        }
        thread::sleep(Duration::from_millis(1000));
        drive.stop();
        input.clear();
    }
}
