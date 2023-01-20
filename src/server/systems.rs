use rand::{self, Rng};
use std::collections::HashMap;

use crate::server::components::Player;

pub fn run(players: &mut HashMap<usize, Player>) {
    gravity(players);
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
