use super::components::{Renderable, Transform};
use crate::server::components::{AnimTargetId, Player, Projectile, Vao};
use gl_matrix::vec3;
use rand::{self, Rng};
use std::collections::HashMap;

pub fn run(players: &mut HashMap<usize, Player>) {
    gravity(players);
    spawn_attack_projectiles(players);
    recievedmg(players);
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

fn spawn_attack_projectiles(players: &mut HashMap<usize, Player>) {
    for (_id, player) in players {
        match player.anim_target_id {
            AnimTargetId::Kick => {
                if player.anim_ticks == 20 {
                    let mut offset = vec3::create();
                    vec3::transform_quat(
                        &mut offset,
                        &vec3::from_values(0.0, 0.7, 1.2),
                        &player.transform.quat,
                    );
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
            AnimTargetId::Punch => {
                if player.anim_ticks == 20 {
                    let mut offset = vec3::create();
                    vec3::transform_quat(
                        &mut offset,
                        &vec3::from_values(0.0, 0.7, 1.2),
                        &player.transform.quat,
                    );
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
            _ => (),
        }

        match &mut player.projectile {
            Some(proj) => proj.renderable.apply(&proj.transform),
            _ => (),
        }
    }
}

fn recievedmg(players: &mut HashMap<usize, Player>) {
    let projectiles: Vec<(usize, Projectile)> = players
        .iter()
        .filter_map(|(id, player)| match player.projectile {
            Some(proj) => Some((*id, proj)),
            None => None,
        })
        .collect();

    for (id, player) in players {
        for (attacker_id, projectile) in &projectiles {
            if id == attacker_id {
                continue;
            }

            if vec3::dist(&projectile.transform.pos, &player.transform.pos) < 1.0 {
                player.attributes.health -= 10.0;
            }
        }
    }
}
