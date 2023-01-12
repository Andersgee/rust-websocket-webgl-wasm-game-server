use crate::messages;
use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::time::Duration;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use self::components::Player;
mod components;

const TICK_INTERVAL: Duration = Duration::from_millis(17);
//const TICK_INTERVAL: Duration = Duration::from_millis(1000);

#[derive(Debug)]
pub struct Server {
    players: HashMap<usize, Player>,
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
            players: HashMap::with_capacity(10),
            sessions: HashMap::with_capacity(10),
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

    fn tick(&mut self) {
        if self.players.len() < 1 {
            return;
        }
        //println!("server tick, sending to all sessions");
        for (_id, player) in &mut self.players {
            player.apply();
        }
        let renderable = self
            .players
            .iter()
            .map(|(_id, player)| player.renderable)
            .collect::<Vec<components::Renderable>>();

        let serialized = serde_json::to_string(&renderable);
        match serialized {
            Ok(serialized_renderable) => self.broadcast(&serialized_renderable),
            Err(_) => println!("failed to serialize renderable, not broadcasting anything"),
        }
    }

    fn start_tick_interval(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(TICK_INTERVAL, |act, _ctx| act.tick());
    }

    fn apply_player_input(&mut self, player_id: usize, player_input: messages::PlayerInput) {
        //let player = self.players.entry(player_id).or_insert(Player::new());
        //player.player_input = player_input;
        self.players
            .entry(player_id)
            .and_modify(|p| p.player_input = player_input);

        println!("apply_player_input, player_id: {}", player_id);
    }
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_tick_interval(ctx)
    }
}

impl Handler<messages::PlayerConnectMessage> for Server {
    type Result = usize;

    fn handle(
        &mut self,
        msg: messages::PlayerConnectMessage,
        _: &mut Context<Self>,
    ) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        //self.send_message("main", "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);
        self.players.insert(id, Player::new());
        let _oldcount = self.visitor_count.fetch_add(1, Ordering::SeqCst);

        /*
        // auto join session to main room
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);
        */
        //self.send_message("main", &format!("Total visitors {count}"), 0);

        // send id back
        id
    }
}

impl Handler<messages::PlayerDisconnectMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: messages::PlayerDisconnectMessage, _: &mut Context<Self>) {
        println!("Someone disconnected, id:{}", msg.id);

        self.sessions.remove(&msg.id);
        self.players.remove(&msg.id);
        let _oldcount = self.visitor_count.fetch_sub(1, Ordering::SeqCst);

        /*
        let mut rooms: Vec<String> = Vec::new();
        // remove session from all rooms
        if self.sessions.remove(&msg.id).is_some() {
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        */

        // send message to other users
        /*
        for room in rooms {
            self.send_message(&room, "Someone disconnected", 0);
        }
        */
    }
}

impl Handler<messages::PlayerInput> for Server {
    type Result = ();

    fn handle(&mut self, msg: messages::PlayerInput, _: &mut Context<Self>) {
        self.apply_player_input(msg.id, msg)
    }
}

impl Handler<messages::ListRooms> for Server {
    type Result = MessageResult<messages::ListRooms>;

    fn handle(&mut self, _: messages::ListRooms, _: &mut Context<Self>) -> Self::Result {
        let names: Vec<String> = self.rooms.keys().map(|k| k.to_owned()).collect();
        MessageResult(names)
    }
}

impl Handler<messages::PlayerJoinRoomMessage> for Server {
    type Result = ();

    fn handle(&mut self, _msg: messages::PlayerJoinRoomMessage, _: &mut Context<Self>) {
        //let messages::PlayerJoinRoomMessage { id, name } = msg;

        //self.send_message(&name, "Someone connected", id);
    }
}
