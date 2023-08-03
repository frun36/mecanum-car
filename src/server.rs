use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};

use crate::drive::{Drive, DriveMessage, DriveResponse, Motion, Speed};
use crate::hc_sr04::{HcSr04, HcSr04Message, HcSr04Response};
use crate::movement_calibration::Calibrator;
use crate::Device;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
    drive_addr: Addr<Drive>,
    hc_sr04_addr: Addr<HcSr04>,
}

#[derive(Deserialize)]
#[serde(tag = "variant")]
enum SocketMessages {
    Move { motion: Motion, speed: Speed },
    MeasureDistance,
    CalibrateMovement,
}

impl WebSocket {
    pub fn new(drive_addr: Addr<Drive>, hc_sr04_addr: Addr<HcSr04>) -> Self {
        Self {
            hb: Instant::now(),
            drive_addr,
            hc_sr04_addr,
        }
    }

    /// Starts heartbeat process
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    fn motion_handler(
        &mut self,
        motion: Motion,
        speed: Speed,
        _ctx: &mut <WebSocket as Actor>::Context,
    ) {
        self.drive_addr.do_send(DriveMessage { motion, speed });
    }

    fn measure_distance_handler(&mut self, _ctx: &mut <Self as Actor>::Context) {
        self.hc_sr04_addr.do_send(HcSr04Message);
    }

    // fn calibrate_distance_handler(&mut self, ctx: &mut <Self as Actor>::Context) {
    //     let drive = self.drive_data.clone();
    //     let distance_sensor = self.hc_sr04_data.clone();
    //     let fut = async move {
    //         task::spawn_blocking(move || {
    //             let mut drive = drive.lock().unwrap();
    //             let mut distance_sensor = distance_sensor.lock().unwrap();
    //             let mut cal =
    //                 Calibrator::new(&mut drive, &mut distance_sensor, 0.4, 0.5, 0.1, 300, 2);

    //             cal.calibrate();
    //         })
    //         .await
    //         .unwrap();
    //     };

    //     ctx.spawn(wrap_future(fut));
    // }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket actor started");
        self.drive_addr.do_send(AddrMessage(ctx.address()));
        self.hc_sr04_addr.do_send(AddrMessage(ctx.address()));
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
                    SocketMessages::Move { motion, speed } => {
                        self.motion_handler(motion, speed, ctx)
                    }
                    SocketMessages::MeasureDistance => {
                        self.measure_distance_handler(ctx);
                    }
                    SocketMessages::CalibrateMovement => {
                        // self.calibrate_distance_handler(ctx);
                    }
                };
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

// Send address to device actors
#[derive(Message)]
#[rtype(result = "()")]
pub struct AddrMessage(Addr<WebSocket>);

impl Handler<AddrMessage> for Drive {
    type Result = ();

    fn handle(&mut self, msg: AddrMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.set_websocket_addr(msg.0);
        println!("Set WebSocket address for Drive");
    }
}

impl Handler<AddrMessage> for HcSr04 {
    type Result = ();

    fn handle(&mut self, msg: AddrMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.set_websocket_addr(msg.0);
        println!("Set WebSocket address for HcSr04");
    }
}

// Device actor response handling

impl Handler<DriveResponse> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: DriveResponse, ctx: &mut Self::Context) -> Self::Result {
        let response = serde_json::to_string(&SocketResponses::Move {
            description: match msg {
                DriveResponse::Ok(m) => format!("Moving {:?} with {:?} speed", m.motion, m.speed),
                DriveResponse::Err(e) => format!("Drive error: {:?}", e),
            },
        })
        .unwrap();
        ctx.text(response);
    }
}

impl Handler<HcSr04Response> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: HcSr04Response, ctx: &mut Self::Context) {
        // Handle the measurement result
        let response = match msg {
            HcSr04Response::Ok(dist) => {
                serde_json::to_string(&SocketResponses::MeasureDistance { measurement: dist })
                    .unwrap()
            }
            HcSr04Response::Err(e) => format!("HcSr04 error: {:?}", e),
        };
        // Send the response back to the WebSocket client
        ctx.text(response);
    }
}

/// WebSocket to client responses
#[derive(Serialize)]
#[serde(tag = "variant")]
enum SocketResponses {
    Move { description: String },
    MeasureDistance { measurement: f32 },
}
