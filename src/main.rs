use std::{io, sync::Mutex};

use actix_web::{web, App, HttpServer};

use rppal::gpio::Gpio;

use hc_sr04::HcSr04;

mod api;
mod drive;
mod hc_sr04;

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

const DISTANCE_SENSOR_TRIG: u8 = 19;
const DISTANCE_SENSOR_ECHO: u8 = 16;

#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    let gpio = Gpio::new().unwrap();

    let drive = drive::Drive::new(
        &gpio,
        [
            (MOTOR0_FWD, MOTOR0_BWD, MOTOR0_PWM),
            (MOTOR1_FWD, MOTOR1_BWD, MOTOR1_PWM),
            (MOTOR2_FWD, MOTOR2_BWD, MOTOR2_PWM),
            (MOTOR3_FWD, MOTOR3_BWD, MOTOR3_PWM),
        ],
        MOTOR_PWM_FREQUENCY,
    )
    .unwrap();

    let distance_sensor = HcSr04::new(&gpio, DISTANCE_SENSOR_TRIG, DISTANCE_SENSOR_ECHO, 25.0);

    let drive_mutex = Mutex::new(drive);
    let drive_data = web::Data::new(drive_mutex);

    let distance_sensor_mutex = Mutex::new(distance_sensor);
    let distance_sensor_data = web::Data::new(distance_sensor_mutex);

    HttpServer::new(move || {
        App::new()
            .app_data(drive_data.clone())
            .app_data(distance_sensor_data.clone())
            .configure(api::init_routes)
    })
    .bind(("0.0.0.0", 7878))?
    .run()
    .await
}
