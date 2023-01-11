use crate::messages;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

const TICKT_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Serialize, Deserialize, Debug)]
struct PlayerInput {
    step_forward: bool,
    step_backward: bool,
    step_left: bool,
    step_right: bool,
}

#[derive(Debug)]
pub struct Server {
    sessions: HashMap<usize, Recipient<messages::GameStateMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
}

impl Server {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> Server {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        Server {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
        }
    }

    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(messages::GameStateMessage(message.to_owned()));
                    }
                }
            }
        }
    }

    /// send message to all clients
    fn broadcast(&self, msg: &str) {
        for recipient in self.sessions.values() {
            recipient.do_send(messages::GameStateMessage(msg.to_owned()));
        }
    }

    fn tick(&self) {
        println!("server tick, sending to all sessions");
        self.broadcast("tick")
    }

    fn start_tick_interval(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(TICKT_INTERVAL, |act, _ctx| act.tick());
    }

    fn apply_player_input(&self, player_id: usize, player_input: PlayerInput) {
        println!(
            "in game, apply_player_input, id: {}, step_forward: {}",
            player_id, player_input.step_forward
        )
    }
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_tick_interval(ctx)
    }
}

impl Handler<messages::PlayerJoinMessage> for Server {
    type Result = usize;

    fn handle(&mut self, msg: messages::PlayerJoinMessage, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        self.send_message("main", &format!("Total visitors {count}"), 0);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<messages::PlayerDisconnectMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: messages::PlayerDisconnectMessage, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        /*
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
        */
    }
}

impl Handler<messages::PlayerInputMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: messages::PlayerInputMessage, _: &mut Context<Self>) {
        let res: Result<PlayerInput, serde_json::Error> = serde_json::from_str(&msg.msg);
        match res {
            Ok(player_input) => self.apply_player_input(msg.id, player_input),
            Err(_) => println!("bad player_input parse"),
        }
        //self.send_message(&msg.room, msg.msg.as_str(), msg.id);
    }
}

impl Handler<messages::ListRooms> for Server {
    type Result = MessageResult<messages::ListRooms>;

    fn handle(&mut self, _: messages::ListRooms, _: &mut Context<Self>) -> Self::Result {
        let mut rooms = Vec::new();

        for key in self.rooms.keys() {
            rooms.push(key.to_owned())
        }

        MessageResult(rooms)
    }
}

impl Handler<messages::PlayerJoinRoomMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: messages::PlayerJoinRoomMessage, _: &mut Context<Self>) {
        let messages::PlayerJoinRoomMessage { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, "Someone connected", id);
    }
}
