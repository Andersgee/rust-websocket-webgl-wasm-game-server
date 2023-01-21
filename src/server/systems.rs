use gl_matrix::{quat, vec3};
use rand::{self, Rng};
use std::collections::HashMap;

use crate::server::components::{AnimTargetId, Player, Projectile, Vao};

use super::components::{Renderable, Transform};

pub fn run(players: &mut HashMap<usize, Player>) {
    gravity(players);
    attack(players);
}

fn gravity(players: &mut HashMap<usize, Player>) {
    for (_id, player) in players {
        let p = player.transform.pos;
        if p[1] < 0.0 || p[0] < -10.0 || p[0] > 10.0 || p[2] < -10.0 || p[2] > 10.0 {
            player.transform.pos[1] -= 0.1;
        }

        if player.transform.pos[1] < -8.0 {
            let randvec3: [f32; 3] = rand::thread_rng().gen();
            let pos = [(randvec3[0] - 0.5) * 16.0, 0.0, (randvec3[2] - 0.5) * 16.0];
            player.transform.pos = pos;
        }
    }
}

fn attack(players: &mut HashMap<usize, Player>) {
    for (_id, player) in players {
        match player.anim_target_id {
            AnimTargetId::Kick => {
                if player.anim_ticks == 20 {
                    //use same quat (player.transform.quat) but translate 2 units or smth in the direction that quat is pointing?
                    let offset = vec3::from_values(0.0, 1.0, 0.0);
                    let transform = Transform {
                        pos: vec3::add(&mut vec3::create(), &player.transform.pos, &offset),
                        quat: player.transform.quat,
                    };
                    player.projectile = Some(Projectile {
                        ticks: 0,
                        ticks_lifetime: 15,
                        transform,
                        renderable: Renderable::new(Vao::Unitcube),
                    })
                }
            }
            AnimTargetId::Punch => (),
            _ => (),
        }

        match &mut player.projectile {
            Some(proj) => proj.renderable.apply(&proj.transform),
            _ => (),
        }
    }
}
