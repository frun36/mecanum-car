use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web::Data;
use actix_web_actors::ws;

use crate::drive::{Drive, Motion, Speed};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
    drive: Data<Mutex<Drive>>,
}

impl WebSocket {
    pub fn new(drive: Data<Mutex<Drive>>) -> Self {
        Self {
            hb: Instant::now(),
            drive,
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
                motion_handler(&self.drive, &text);
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

fn motion_handler(drive_data: &Data<Mutex<Drive>>, message: &str) {
    let mut drive = drive_data.lock().unwrap();
    let mut motion = &Motion::Stop;
    let speed = &Speed::Low;
    match message {
        "move_forwardleft" => {
            motion = &Motion::ForwardLeft;
        }
        "move_forward" => {
            motion = &Motion::Forward;
        }
        "move_forwardright" => {
            motion = &Motion::ForwardRight;
        }
        "move_right" => {
            motion = &Motion::Right;
        }
        "move_backwardright" => {
            motion = &Motion::BackwardRight;
        }
        "move_backward" => {
            motion = &Motion::Backward;
        }
        "move_backwardleft" => {
            motion = &Motion::BackwardLeft;
        }
        "move_left" => {
            motion = &Motion::Left;
        }
        "move_leftrot" => {
            motion = &Motion::LeftRot;
        }
        "move_rightrot" => {
            motion = &Motion::RightRot;
        }
        "stop" => {
            motion = &Motion::Stop;
        }
        _ => {
            println!("Invalid message")
        }
    };
    drive.move_robot(motion, speed).unwrap();
}
