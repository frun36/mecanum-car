use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web::Data;
use actix_web_actors::ws;

use log::error;
use serde::{Deserialize, Serialize};

use crate::distance_scan::{Scanner, ScannerMessage};
use crate::drive::{Drive, DriveMessage, DriveResponse};
use crate::error::Error;
use crate::hc_sr04::{HcSr04, HcSr04Measurement, HcSr04Message, HcSr04Response, Recipient};
use crate::movement_calibration::{Calibrator, CalibratorMessage};
use crate::Device;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
    drive_data: Data<Mutex<Addr<Drive>>>,
    hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    calibrator_addr: Option<Addr<Calibrator>>,
    scanner_addr: Option<Addr<Scanner>>,
}

#[derive(Deserialize)]
#[serde(tag = "message")]
enum SocketMessage {
    Move(DriveMessage),
    MeasureDistance,
    CalibrateMovement(CalibratorMessage),
    ScanDistance(ScannerMessage),
}

impl WebSocket {
    pub fn new(
        drive_data: Data<Mutex<Addr<Drive>>>,
        hc_sr04_data: Data<Mutex<Addr<HcSr04>>>,
    ) -> Self {
        Self {
            hb: Instant::now(),
            drive_data,
            hc_sr04_data,
            calibrator_addr: None,
            scanner_addr: None,
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
        message: DriveMessage,
        _ctx: &mut <WebSocket as Actor>::Context,
    ) -> Result<(), Error> {
        let drive_addr = self.drive_data.lock()?;
        drive_addr.try_send(message)?;
        Ok(())
    }

    fn measure_distance_handler(
        &mut self,
        ctx: &mut <Self as Actor>::Context,
    ) -> Result<(), Error> {
        let hc_sr04_addr = self.hc_sr04_data.lock()?;
        hc_sr04_addr.try_send(HcSr04Message::Single(Recipient::WebSocket(ctx.address())))?;
        Ok(())
    }

    fn calibrator_handler(&mut self, msg: CalibratorMessage) -> Result<(), Error> {
        match msg {
            CalibratorMessage::Start(params) => {
                // Send message to calibrator if it exists
                if let Some(addr) = &self.calibrator_addr {
                    addr.try_send(msg)?;
                    return Ok(());
                }
                // Create new calibrator otherwise
                let calibrator =
                    Calibrator::new(self.drive_data.clone(), self.hc_sr04_data.clone(), params);
                let addr = calibrator.start();
                self.calibrator_addr = Some(addr.clone());
                addr.try_send(msg)?;
            }
            CalibratorMessage::Stop => {
                // Stop calibration if calibrator already exists
                if let Some(addr) = &self.calibrator_addr {
                    addr.try_send(CalibratorMessage::Stop)?;
                    self.calibrator_addr = None;
                }
            }
        }
        Ok(())
    }

    fn scanner_handler(&mut self, msg: ScannerMessage) -> Result<(), Error> {
        match msg {
            ScannerMessage::Start {
                speed,
                slip,
                resolution,
            } => {
                // Send message to scanner if it exists
                if let Some(addr) = &self.scanner_addr {
                    addr.try_send(msg)?;
                    return Ok(());
                }
                // Create new scanner otherwise
                let scanner = Scanner::new(
                    self.drive_data.clone(),
                    self.hc_sr04_data.clone(),
                    speed,
                    slip,
                    resolution,
                );
                let addr = scanner.start();
                self.scanner_addr = Some(addr.clone());
                addr.try_send(msg)?;
            }
            ScannerMessage::Stop => {
                // Stop scanning if scanner already exists
                if let Some(addr) = &self.scanner_addr {
                    addr.try_send(ScannerMessage::Stop)?;
                    self.calibrator_addr = None;
                }
            }
        }
        Ok(())
    }
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket actor started");
        let drive_addr = self.drive_data.lock().unwrap();
        drive_addr.do_send(AddrMessage(ctx.address()));
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
                let message: SocketMessage =
                    serde_json::from_str(&text).expect("Failed to deserialize message");
                match message {
                    SocketMessage::Move(message) => self.motion_handler(message, ctx),
                    SocketMessage::MeasureDistance => self.measure_distance_handler(ctx),
                    SocketMessage::CalibrateMovement(message) => self.calibrator_handler(message),
                    SocketMessage::ScanDistance(message) => self.scanner_handler(message),
                }
                .unwrap_or_else(|e| error!("{e}"));
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

// Device actor response handling

impl Handler<DriveResponse> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: DriveResponse, ctx: &mut Self::Context) -> Self::Result {
        let response = serde_json::to_string(&SocketResponses::Move {
            description: match msg {
                DriveResponse::Ok(m) => match m {
                    DriveMessage::Enable { motion, speed } => {
                        format!("Moving {:?} with {:?} speed", motion, speed)
                    }
                    DriveMessage::Disable => "Stopped".to_string(),
                    DriveMessage::Move {
                        motion,
                        speed,
                        distance,
                    } => format!("Moving {:?} {} m with {:?} speed", motion, distance, speed),
                    DriveMessage::Rotate {
                        motion,
                        speed,
                        angle,
                    } => format!("Rotating {:?} {} deg with {:?} speed", motion, angle, speed),
                },
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
            HcSr04Response::Ok(dist) => serde_json::to_string(&SocketResponses::MeasureDistance {
                measurement: match dist {
                    HcSr04Measurement::Single(d) => d.distance,
                    HcSr04Measurement::Multiple(d_vec) => {
                        d_vec.iter().map(|x| x.distance).sum::<f32>() / d_vec.len() as f32
                    }
                },
            })
            .unwrap(),
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
