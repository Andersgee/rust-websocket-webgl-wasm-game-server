use crate::{messages, server};
use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Session {
    pub id: usize,
    pub hb: Instant,
    pub room: String,
    pub name: Option<String>,
    pub server_addr: Addr<server::ChatServer>,
}

impl Session {
    pub fn new(server_addr: Addr<server::ChatServer>) -> Self {
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
                println!("Websocket Client heartbeat failed, disconnecting!");
                act.server_addr.do_send(messages::Disconnect { id: act.id });
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
            .send(messages::Connect {
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
            .do_send(messages::Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<messages::Message> for Session {
    type Result = ();

    fn handle(&mut self, msg: messages::Message, ctx: &mut Self::Context) {
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

        println!("WEBSOCKET MESSAGE: {msg:?}");
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
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // send() is for when we want to pause processing of new messages until response returned
                            // do_send() is for when we dont care about the response
                            println!("List rooms");
                            self.server_addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                self.server_addr.do_send(messages::Join {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {m:?}")),
                    }
                } else {
                    self.server_addr.do_send(messages::ClientMessage {
                        id: self.id,
                        msg: m.to_owned(),
                        room: self.room.clone(),
                    })
                }
            }
        }
    }
}
