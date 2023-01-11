use crate::{messages, server};
use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Session {
    pub id: usize,
    pub hb: Instant, //ping-pong heartbeat for timing out clients
    pub room: String,
    pub name: Option<String>,
    pub server_addr: Addr<server::Server>,
}

impl Session {
    pub fn new(server_addr: Addr<server::Server>) -> Self {
        Self {
            id: 0, //owerwrite this on actor started
            hb: Instant::now(),
            room: String::from("main"),
            name: None, //
            server_addr,
        }
    }
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                //println!("Websocket Client heartbeat failed, disconnecting!");
                act.server_addr
                    .do_send(messages::PlayerDisconnectMessage { id: act.id });
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.server_addr
            .send(messages::PlayerJoinMessage {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.server_addr
            .do_send(messages::PlayerDisconnectMessage { id: self.id });
        Running::Stop
    }
}

impl Handler<messages::GameStateMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: messages::GameStateMessage, ctx: &mut Self::Context) {
        //println!("session id: {}, sending msg: {}", self.id, msg.0);
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        //println!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
            ws::Message::Text(text) => {
                let m = text.trim();
                // check for special commands, else just send text to server to process
                if m.starts_with('/') {
                    let (cmd, arg) = m.split_once(" ").unwrap_or((m, ""));
                    match cmd {
                        "/list" => {
                            //list rooms
                            // send() is for when we want to pause processing of new messages until response returned
                            // do_send() is for when we dont care about the response
                            println!("List rooms");
                            self.server_addr
                                .send(messages::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            //send multiple strings to client
                                            //for room in rooms {
                                            //  ctx.text(room);
                                            //}
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                        }

                        "/join" => {
                            //join room
                            if arg != "" {
                                self.room = arg.to_owned();
                                self.server_addr.do_send(messages::PlayerJoinRoomMessage {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                //ctx.text("joined");
                            }
                        }
                        "/name" => {
                            //change name
                            if arg != "" {
                                self.name = Some(arg.to_owned());
                            }
                        }
                        _ => println!("unknown slash command recieved, cmd: {cmd}, arg: {arg}"),
                    }
                } else {
                    let res: Result<messages::PlayerInputWithoutId, serde_json::Error> =
                        serde_json::from_str(m); //this fails unless sending id...
                    match res {
                        Ok(p) => {
                            //is there a clean way use remaining fields (from a different type)?
                            let player_input = messages::PlayerInput {
                                id: self.id,
                                step_forward: p.step_forward,
                                step_backward: p.step_backward,
                                step_left: p.step_left,
                                step_right: p.step_right,
                            };
                            self.server_addr.do_send(player_input);
                        }
                        Err(_) => {
                            println!("bad PlayerInputWithoutId json string: {m}");
                            //ctx.stop();
                        }
                    }
                }
            }
        }
    }
}
