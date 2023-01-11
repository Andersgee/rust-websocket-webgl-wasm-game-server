use actix::prelude::*;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct GameStateMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct PlayerJoinMessage {
    pub addr: Recipient<GameStateMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerDisconnectMessage {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerJoinRoomMessage {
    pub id: usize,
    pub name: String,
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct PlayerInputMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Room name
    pub room: String,
}
