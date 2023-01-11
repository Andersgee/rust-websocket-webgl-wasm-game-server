use gl_matrix::common::{Mat4, Quat, Vec2, Vec3};
use gl_matrix::{mat4, quat, vec2, vec3};
use serde::{Deserialize, Serialize};

use crate::messages::PlayerInput;

//note to self:
//anything in this file should be identical on server and client

//inline version
pub fn vec2_add(out: &mut Vec2, v: &Vec2) -> Vec2 {
    out[0] += v[0];
    out[1] += v[1];

    *out
}

/// inline version
pub fn vec2_normalize(out: &mut Vec2) -> Vec2 {
    let x = out[0];
    let y = out[1];

    let mut len = x * x + y * y;

    if len > 0_f32 {
        //TODO: evaluate use of glm_invsqrt here?
        len = 1_f32 / f32::sqrt(len);
    }

    out[0] *= len;
    out[1] *= len;

    *out
}

//inline version
pub fn vec2_scale(out: &mut Vec2, k: f32) -> Vec2 {
    out[0] *= k;
    out[1] *= k;

    *out
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    vao: Vao,
    transform: Transform,
    renderable: Renderable,
    player_input: PlayerInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Vao {
    Guy,
    Floor,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vec3,
    pub vel: Vec3,
    pub quat: Quat,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            pos: vec3::create(),
            vel: vec3::create(),
            quat: quat::create(),
        }
    }
}

impl Transform {
    /// update velocity and position
    pub fn apply(&mut self, player_input: &PlayerInput) {
        let mut v = vec2::create();
        let right = vec2::from_values(1., 0.);
        let left = vec2::from_values(-1., 0.);
        let forward = vec2::from_values(0., 1.);
        let backward = vec2::from_values(0., -1.);
        if player_input.step_forward {
            vec2_add(&mut v, &forward);
        }
        if player_input.step_backward {
            vec2_add(&mut v, &backward);
        }
        if player_input.step_right {
            vec2_add(&mut v, &right);
        }
        if player_input.step_left {
            vec2_add(&mut v, &left);
        }

        vec2_normalize(&mut v);
        let move_speed = 2.0;
        vec2_scale(&mut v, move_speed);

        self.vel[0] = v[0];
        self.vel[1] = v[1];
        self.pos[0] += v[0];
        self.pos[1] += v[1];
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Renderable {
    pub vao: Vao,
    pub model_mat: Mat4,
}

impl Renderable {
    pub fn new(vao: Vao) -> Self {
        Self {
            vao,
            model_mat: mat4::create(),
        }
    }

    pub fn apply(&mut self, transform: &Transform) {
        mat4::from_translation(&mut self.model_mat, &transform.pos);
    }
}
