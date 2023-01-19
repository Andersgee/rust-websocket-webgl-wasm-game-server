use gl_matrix::common::{Mat4, Quat, Vec2, Vec3};
use gl_matrix::{mat4, quat, vec2, vec3};
use serde::{Deserialize, Serialize};

use crate::messages::PlayerInput;

//note to self:
//anything in this file should be identical on server and client?

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

//inline version
pub fn vec2_rotate(out: &mut Vec2, origin: &Vec2, rad: f32) -> Vec2 {
    let p0 = out[0] - origin[0];
    let p1 = out[1] - origin[1];

    let sin_c = f32::sin(rad);
    let cos_c = f32::cos(rad);

    out[0] = p0 * cos_c - p1 * sin_c + origin[0];
    out[1] = p0 * sin_c + p1 * cos_c + origin[1];

    *out
}
pub fn vec2_rotate_around_origin(out: &mut Vec2, rad: f32) -> Vec2 {
    let p0 = out[0];
    let p1 = out[1];

    let sin_c = f32::sin(rad);
    let cos_c = f32::cos(rad);

    out[0] = p0 * cos_c - p1 * sin_c;
    out[1] = p0 * sin_c + p1 * cos_c;

    *out
}

/// [1,0] would give rotation 0 around y axis
/// [0,1] would give rotation pi/2 around y axis
/// [-1,0] would give rotation pi around y axis
fn quat_from_vec2(out: &mut Quat, v: &Vec2) {
    quat_from_rad(out, 0.0, v[0].atan2(v[1]), 0.0)
}

pub fn quat_from_rad(out: &mut Quat, x: f32, y: f32, z: f32) {
    let half_to_rad = 0.5;

    let x = x * half_to_rad;
    let y = y * half_to_rad;
    let z = z * half_to_rad;

    let sx = f32::sin(x);
    let cx = f32::cos(x);
    let sy = f32::sin(y);
    let cy = f32::cos(y);
    let sz = f32::sin(z);
    let cz = f32::cos(z);

    out[0] = sx * cy * cz - cx * sy * sz;
    out[1] = cx * sy * cz + sx * cy * sz;
    out[2] = cx * cy * sz - sx * sy * cz;
    out[3] = cx * cy * cz + sx * sy * sz;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    attributes: Attributes,
    transform: Transform,
    pub renderable: Renderable,
    pub player_input: PlayerInput,
    anim_target_id: AnimTargetId,
}

impl Player {
    pub fn new() -> Self {
        Self {
            player_input: PlayerInput::new(),
            attributes: Attributes { move_speed: 0.05 },
            transform: Transform::new(),
            renderable: Renderable::new(Vao::Guy),
            anim_target_id: AnimTargetId::Idle,
        }
    }

    /// apply self.player_input all the way to self.renderable
    pub fn apply(&mut self) {
        let is_waling = self.transform.apply(&self.player_input, &self.attributes);
        self.renderable.apply(&self.transform);

        if is_waling {
            self.anim_target_id = AnimTargetId::Walk;
        } else if self.player_input.kick {
            self.anim_target_id = AnimTargetId::Kick;
        } else if self.player_input.punch {
            self.anim_target_id = AnimTargetId::Punch;
        } else {
            self.anim_target_id = AnimTargetId::Idle;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    move_speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Vao {
    Guy,
    Floor,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnimTargetId {
    Idle,
    Walk,
    Kick,
    Punch,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vec3,
    pub quat: Quat,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            pos: vec3::create(),
            quat: quat::create(),
        }
    }
}

impl Transform {
    /// update velocity and position
    pub fn apply(&mut self, player_input: &PlayerInput, attributes: &Attributes) -> bool {
        let mut is_walking = false;
        let mut v = vec2::create();
        let right = [1.0, 0.0];
        let left = [-1.0, 0.0];
        let forward = [0.0, -1.0];
        let backward = [0.0, 1.0];
        if player_input.step_forward {
            vec2_add(&mut v, &forward);
            is_walking = true;
        }
        if player_input.step_backward {
            vec2_add(&mut v, &backward);
            is_walking = true;
        }
        if player_input.step_right {
            vec2_add(&mut v, &right);
            is_walking = true;
        }
        if player_input.step_left {
            vec2_add(&mut v, &left);
            is_walking = true;
        }

        //vec2_rotate_around_origin(&mut v, player_input.facing_rad);
        vec2_normalize(&mut v);
        vec2_scale(&mut v, attributes.move_speed);

        //update pos
        self.pos[0] += v[0];
        self.pos[2] += v[1];
        //update quat
        //quat_from_rad(&mut self.quat, 0.0, -player_input.facing_rad, 0.0);
        if v[0] != 0. || v[1] != 0. {
            quat_from_vec2(&mut self.quat, &v);
        }
        is_walking
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
        //mat4::from_translation(&mut self.model_mat, &transform.pos);
        mat4::from_rotation_translation(&mut self.model_mat, &transform.quat, &transform.pos);
    }
}
