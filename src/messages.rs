use actix::prelude::{Message, Recipient};
use serde::{Deserialize, Serialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct GameStateMessage(pub String);

#[derive(Message)]
#[rtype(usize)]
pub struct PlayerConnectMessage {
    pub addr: Recipient<GameStateMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerDisconnectMessage {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "Vec<String>")]
pub struct ListRooms;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerJoinRoomMessage {
    pub id: usize,
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub id: usize,
    pub text: String,
}

#[derive(Message, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct PlayerInput {
    /// Id of the client session
    pub id: usize,
    pub step_forward: bool,
    pub step_backward: bool,
    pub step_left: bool,
    pub step_right: bool,
    pub facing_rad: f32,
}

impl PlayerInput {
    pub fn new() -> Self {
        Self {
            id: 0,
            step_forward: false,
            step_backward: false,
            step_left: false,
            step_right: false,
            facing_rad: 0.,
        }
    }
}

#[derive(Deserialize)]
pub struct PlayerInputWithoutId {
    pub step_forward: bool,
    pub step_backward: bool,
    pub step_left: bool,
    pub step_right: bool,
    pub facing_rad: f32,
}
