use std::{
    f64::consts::PI,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use actix::{fut::wrap_future, prelude::*};
use actix_web::{rt::time, web::Data};
use serde::Deserialize;

use crate::{
    drive::{Drive, DriveMessage, Motion, Speed, ROBOT_RADIUS},
    hc_sr04::{HcSr04, HcSr04Measurement, HcSr04Message, HcSr04Response, Recipient},
};

pub struct Scanner {
    drive_data: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    speed: Speed,
    slip: f64,
    resolution: usize,
    start_time: Option<Duration>,
}

impl Scanner {
    pub fn new(
        drive_data: Data<Mutex<Addr<Drive>>>,
        hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
        speed: Speed,
        slip: f64,
        resolution: usize,
    ) -> Self {
        Self {
            drive_data,
            hc_sr04_data,
            speed,
            slip,
            resolution,
            start_time: None,
        }
    }

    fn scan(&mut self, ctx: &mut <Self as Actor>::Context) {
        // Clone necessary data
        let speed = self.speed;
        let resolution = self.resolution;
        let addr = ctx.address();
        let drive_addr = self.drive_data.lock().unwrap().to_owned();
        let hc_sr04_addr = self.hc_sr04_data.lock().unwrap().to_owned();

        // Compute time between measurements
        let distance = 2. * PI * ROBOT_RADIUS * (1. + self.slip);
        let time_s = distance / (self.speed.get_velocity() * resolution as f64);
        let time = Duration::from_secs_f64(time_s);

        // Define task
        let fut = async move {
            drive_addr.do_send(DriveMessage::Enable {
                motion: Motion::RightRot,
                speed,
            });

            for _ in 0..resolution {
                hc_sr04_addr.do_send(HcSr04Message::Single(Recipient::Scanner(addr.clone())));
                time::sleep(time).await;
            }

            drive_addr.do_send(DriveMessage::Disable);
        };

        // Run task
        self.start_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards"),
        );
        ctx.spawn(wrap_future(fut));
    }
}

impl Actor for Scanner {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Scanner actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let drive_addr = self.drive_data.lock().unwrap();
        drive_addr.do_send(DriveMessage::Disable);
    }
}

#[derive(Message, Deserialize)]
#[serde(tag = "variant")]
#[rtype(result = "()")]
pub enum ScannerMessage {
    Start {
        speed: Speed,
        slip: f64,
        resolution: usize,
    },
    Stop,
}

impl Handler<ScannerMessage> for Scanner {
    type Result = ();

    fn handle(&mut self, msg: ScannerMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ScannerMessage::Start {
                speed,
                slip,
                resolution,
            } => {
                self.speed = speed;
                self.slip = slip;
                self.resolution = resolution;
                self.scan(ctx);
            }
            ScannerMessage::Stop => ctx.stop(),
        };
    }
}

impl Handler<HcSr04Response> for Scanner {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Response, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            HcSr04Response::Ok(measurement) => match measurement {
                HcSr04Measurement::Single(result) => println!(
                    "{},{}",
                    result.time.as_millis() - self.start_time.unwrap().as_millis(),
                    result.distance
                ),
                HcSr04Measurement::Multiple(_) => (),
            },
            HcSr04Response::Err(e) => println!("{:?}", e),
        };
    }
}
