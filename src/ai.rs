use rand::{prelude::SmallRng, Rng};

use crate::game_state::Input;

#[derive(Debug, Clone, Default)]
pub(crate) struct Ai {
    movement: i32,
    turning: i32,
    firing: bool,
}

impl Ai {
    pub(crate) fn input(&mut self, rng: &mut SmallRng) -> Input {
        if rng.gen_bool(0.05) {
            self.movement = rng.gen_range(-1, 2);
        }

        if rng.gen_bool(0.05) {
            self.turning = rng.gen_range(-1, 2);
        }

        if !self.firing && rng.gen_bool(0.01) {
            self.firing = true;
        } else if self.firing && rng.gen_bool(0.05) {
            self.firing = false;
        }

        Input {
            up: self.movement == 1,
            down: self.movement == -1,
            left: self.turning == -1,
            right: self.turning == 1,
            turret_left: rng.gen_bool(0.001),
            turret_right: rng.gen_bool(0.001),
            prev_weapon: rng.gen_bool(0.001),
            next_weapon: rng.gen_bool(0.001),
            fire: self.firing,
            mine: rng.gen_bool(0.001),
            self_destruct: rng.gen_bool(0.0001),
            horn: rng.gen_bool(0.0001),
            chat: false,
        }
    }
}
