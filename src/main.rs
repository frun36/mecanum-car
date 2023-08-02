use std::{io, sync::Mutex};

use actix::prelude::*;

use actix_files as fs;
use actix_files::NamedFile;
use actix_web::{
    get,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;

use rppal::gpio::Gpio;

use drive::Drive;
use hc_sr04::HcSr04;
use server::WebSocket;

mod drive;
mod hc_sr04;
mod movement_calibration;
mod server;

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

const DISTANCE_SENSOR_TRIG: u8 = 26;
const DISTANCE_SENSOR_ECHO: u8 = 20;

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("static/index.html").await.unwrap()
}

/// Websocket handshake, start `WebSocket` actor
#[get("/ws")]
async fn ws_connect(
    req: HttpRequest,
    stream: web::Payload,
    drive_addr: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<HcSr04>>,
) -> Result<HttpResponse, actix_web::Error> {
    ws::start(
        WebSocket::new(drive_addr.lock().unwrap().to_owned(), hc_sr04_data),
        &req,
        stream,
    )
}

#[actix_web::main]
async fn main() -> Result<(), io::Error> {
    let gpio = Gpio::new().unwrap();

    let drive = Drive::new(
        &gpio,
        [
            (MOTOR0_FWD, MOTOR0_BWD, MOTOR0_PWM),
            (MOTOR1_FWD, MOTOR1_BWD, MOTOR1_PWM),
            (MOTOR2_FWD, MOTOR2_BWD, MOTOR2_PWM),
            (MOTOR3_FWD, MOTOR3_BWD, MOTOR3_PWM),
        ],
        MOTOR_PWM_FREQUENCY,
    )
    .unwrap()
    .start();
    // drive.list_motors();

    let mut distance_sensor = HcSr04::new(&gpio, DISTANCE_SENSOR_TRIG, DISTANCE_SENSOR_ECHO, 25.0);

    // For some reason without this line the distance measurement doesn't work
    println!("{}", distance_sensor.measure_distance());

    let drive_mutex = Mutex::new(drive);
    let drive_data = Data::new(drive_mutex);

    let distance_sensor_mutex = Mutex::new(distance_sensor);
    let distance_sensor_data = Data::new(distance_sensor_mutex);

    HttpServer::new(move || {
        App::new()
            .app_data(drive_data.clone())
            .app_data(distance_sensor_data.clone())
            .service(index)
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(ws_connect)
    })
    .workers(2)
    .bind(("0.0.0.0", 7878))?
    .run()
    .await
}
