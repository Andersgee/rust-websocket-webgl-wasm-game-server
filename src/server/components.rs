use gl_matrix::common::{Mat4, Quat, Vec2, Vec3};
use gl_matrix::{mat4, quat, vec2, vec3};
use rand::{self, Rng};
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
pub struct Projectile {
    pub ticks: u32,
    pub ticks_lifetime: u32,
    pub transform: Transform,
    pub renderable: Renderable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub attributes: Attributes,
    pub transform: Transform,
    pub renderable: Renderable,
    pub player_input: PlayerInput,
    pub anim_target_id: AnimTargetId,
    pub anim_ticks: u32,
    pub projectile: Option<Projectile>,
}

impl Player {
    pub fn new(pos: [f32; 3]) -> Self {
        Self {
            player_input: PlayerInput::new(),
            attributes: Attributes {
                move_speed: 0.05,
                health: 100.0,
            },
            transform: Transform::new(pos),
            renderable: Renderable::new(Vao::Guy),
            anim_target_id: AnimTargetId::Idle,
            anim_ticks: 0,
            projectile: None,
        }
    }

    pub fn respawn(&mut self) {
        let randvec3: [f32; 3] = rand::thread_rng().gen();
        let pos = [(randvec3[0] - 0.5) * 16.0, 0.0, (randvec3[2] - 0.5) * 16.0];
        self.transform.pos = pos;
    }

    /// apply self.player_input all the way to self.renderable
    pub fn apply(&mut self) {
        let prev_anim_target_id = self.anim_target_id.clone();
        let mut is_ability = false;
        self.anim_target_id = AnimTargetId::Idle;

        if self.player_input.kick {
            self.anim_target_id = AnimTargetId::Kick;
            is_ability = true;
        } else if self.player_input.punch {
            self.anim_target_id = AnimTargetId::Punch;
            is_ability = true;
        }
        if is_ability == false {
            let is_walking = self.transform.apply(&self.player_input, &self.attributes);
            if is_walking {
                self.anim_target_id = AnimTargetId::Walk;
            }
        }
        self.renderable.apply(&self.transform);

        if prev_anim_target_id != self.anim_target_id {
            self.anim_ticks = 0;
            self.projectile = None;
        } else {
            self.anim_ticks += 1;
            match &mut self.projectile {
                Some(proj) => {
                    proj.ticks += 1;
                    if proj.ticks > proj.ticks_lifetime {
                        self.projectile = None;
                    }
                }
                None => (),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    pub move_speed: f32,
    pub health: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Vao {
    Guy,
    Floor,
    Unitcube,
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
    pub fn new(pos: [f32; 3]) -> Self {
        Self {
            pos,
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
        let is_walking = v[0] != 0. || v[1] != 0.;

        //vec2_rotate_around_origin(&mut v, player_input.facing_rad);
        vec2_normalize(&mut v);
        vec2_scale(&mut v, attributes.move_speed);

        //update pos
        self.pos[0] += v[0];
        self.pos[2] += v[1];
        //update quat
        //quat_from_rad(&mut self.quat, 0.0, -player_input.facing_rad, 0.0);
        if is_walking {
            let mut targetquat = quat::create();
            let prev_quat =
                quat::from_values(self.quat[0], self.quat[1], self.quat[2], self.quat[3]);
            quat_from_vec2(&mut targetquat, &v);
            quat::slerp(&mut self.quat, &prev_quat, &targetquat, 0.1);
        }

        //gravity
        /*
        if self.pos[0] < -10.0 || self.pos[0] > 10.0 || self.pos[2] < -10.0 || self.pos[2] > 10.0 {
            self.pos[1] -= 1.0;
        }
         */

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
