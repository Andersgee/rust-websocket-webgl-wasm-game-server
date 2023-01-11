use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    x: u32,
    y: u32,
    anim_frame: u32,
}
