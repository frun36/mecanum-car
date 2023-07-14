use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web::Data;
use actix_web_actors::ws;

use serde::Deserialize;

use crate::drive::{Drive, Motion, Speed};
use crate::hc_sr04::HcSr04;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
    drive_data: Data<Mutex<Drive>>,
    hc_sr04_data: Data<Mutex<HcSr04>>,
}

#[derive(Deserialize)]
#[serde(tag = "variant")]
enum SocketMessages {
    Move { motion: Motion, speed: Speed },
    MeasureDistance,
}

impl WebSocket {
    pub fn new(drive_data: Data<Mutex<Drive>>, hc_sr04_data: Data<Mutex<HcSr04>>) -> Self {
        Self {
            hb: Instant::now(),
            drive_data,
            hc_sr04_data,
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // Process websocket message
        println!("WS: {msg:?}");
        match msg {
            // Respond to pings with pong
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            // Reset clock after receiving heartbeat pong
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            // Text message
            Ok(ws::Message::Text(text)) => {
                let message: SocketMessages =
                    serde_json::from_str(&text).expect("Failed to deserialize message");
                match message {
                    SocketMessages::Move { motion, speed } => motion_handler(&self.drive_data, &motion, &speed),
                    SocketMessages::MeasureDistance => measure_distance_handler(&self.hc_sr04_data),
                }
                ctx.text(text)
            }
            // Binary message
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            // Close page
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

fn motion_handler(drive_data: &Data<Mutex<Drive>>, motion: &Motion, speed: &Speed) {
    let mut drive = drive_data.lock().unwrap();
    drive.move_robot(motion, speed).unwrap();
}

fn measure_distance_handler(hc_sr04_data: &Data<Mutex<HcSr04>>) {
    let mut sensor = hc_sr04_data.lock().unwrap();
    println!("Distance: {}", sensor.measure_distance());
}
