use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Vao {
    Guy,
    Floor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Renderable {
    pub vao: Vao,
    pub model_mat: f32, //Mat4,
}
