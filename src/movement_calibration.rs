use actix::fut::wrap_future;
use actix::prelude::*;
use actix_web::rt::time;
use actix_web::web::Data;

use crate::drive::{Drive, DriveMessage, Motion, Speed};
use crate::hc_sr04::{HcSr04, HcSr04Measurement, HcSr04Message, HcSr04Response};

use std::fs::File;
use std::io::Write;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct CalibratorParams {
    min_duty_cycle: f64,
    max_duty_cycle: f64,
    step: f64,
    measurements_per_repetition: usize,
    repetitions: usize,
}

impl CalibratorParams {
    pub fn new(
        min_duty_cycle: f64,
        max_duty_cycle: f64,
        step: f64,
        measurements_per_repetition: usize,
        repetitions: usize,
    ) -> CalibratorParams {
        Self {
            min_duty_cycle,
            max_duty_cycle,
            step,
            measurements_per_repetition,
            repetitions,
        }
    }
}

#[derive(Debug)]
struct CalibratorState {
    motion: Motion,
    duty_cycle: f64,
    repetition: usize,
}

impl CalibratorState {
    fn new(min_duty_cycle: f64) -> Self {
        Self {
            motion: Motion::Forward,
            duty_cycle: min_duty_cycle,
            repetition: 0,
        }
    }
}

pub struct Calibrator {
    drive_data: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    params: CalibratorParams,
    state: CalibratorState,
}

impl Calibrator {
    pub fn new(
        drive_data: Data<Mutex<Addr<Drive>>>,
        hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
        params: CalibratorParams,
    ) -> Self {
        Self {
            drive_data,
            hc_sr04_data,
            params,
            state: CalibratorState::new(params.min_duty_cycle),
        }
    }
}

impl Actor for Calibrator {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Calibrator actor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let drive_addr = self.drive_data.lock().unwrap();
        // Stop the robot
        drive_addr.do_send(DriveMessage {
            motion: Motion::Stop,
            speed: Speed::Low,
        });
        println!("Calibrator actor stopped");
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum CalibratorMessage {
    Start(CalibratorParams),
    Stop,
}

impl Handler<CalibratorMessage> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: CalibratorMessage, ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        let hc_sr04_addr = self.hc_sr04_data.lock().unwrap();
        match msg {
            CalibratorMessage::Start(params) => {
                self.params = params;
                println!("Performing calibration: {:?}", self.state);
                // Move robot
                drive_addr.do_send(DriveMessage {
                    motion: self.state.motion,
                    speed: Speed::Manual(self.state.duty_cycle),
                });
                // Start measurement
                hc_sr04_addr.do_send(HcSr04Message::Multiple(
                    self.params.measurements_per_repetition,
                    crate::hc_sr04::Recipient::Calibrator(ctx.address()),
                ));
            }
            CalibratorMessage::Stop => {
                ctx.stop();
            }
        };
    }
}

impl Handler<HcSr04Response> for Calibrator {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Response, ctx: &mut Self::Context) -> Self::Result {
        let drive_addr = self.drive_data.lock().unwrap();
        // Stop the robot
        drive_addr.do_send(DriveMessage {
            motion: Motion::Stop,
            speed: Speed::Low,
        });
        // Process the result
        let result = match msg {
            HcSr04Response::Ok(measurement) => match measurement {
                HcSr04Measurement::Single(d) => format!("{},{}", d.time.as_millis(), d.distance),
                HcSr04Measurement::Multiple(d_vec) => d_vec
                    .iter()
                    .map(|x| format!("{},{}", x.time.as_millis(), x.distance))
                    .collect::<Vec<String>>()
                    .join("\n"),
            },
            HcSr04Response::Err(e) => format!("{}", e),
        };
        // Save the result to file
        let mut file = File::create(format!(
            "measurements/{}_{:.2}_{:02}.csv",
            self.state.motion, self.state.duty_cycle, self.state.repetition
        ))
        .unwrap();
        write!(file, "{}", result).unwrap();
        // Update params for next measurement
        self.state.motion = match self.state.motion {
            Motion::Forward => Motion::Backward,
            Motion::Backward => {
                self.state.repetition += 1;
                Motion::Forward
            }
            _ => panic!("Invalid calibration motion"),
        };
        // Check if all repetitions were done
        if self.state.repetition >= self.params.repetitions {
            self.state.repetition = 0;
            self.state.duty_cycle += self.params.step;
        }
        // Return if measurement is completed
        if self.state.duty_cycle > self.params.max_duty_cycle {
            return;
        }
        // Send message after some time for next measurement if not
        ctx.wait(wrap_future(time::sleep(Duration::from_millis(3000))));
        ctx.address().do_send(CalibratorMessage::Start(self.params));
    }
}
