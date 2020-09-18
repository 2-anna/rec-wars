use vek::Clamp;

use crate::{cvars::Cvars, weapons::Weapon};
use crate::{
    map::{Map, Vec2f},
    Input,
};

#[derive(Debug, Clone)]
pub struct GuidedMissile {
    pub pos: Vec2f,
    pub vel: Vec2f,
    /// Kinda like angular momentum, except more special-casey.
    /// TODO Might wanna revisit when i have proper physics.
    pub turn_rate: f64,
}

impl GuidedMissile {
    #[must_use]
    pub fn spawn(cvars: &Cvars, pos: Vec2f, angle: f64) -> GuidedMissile {
        // example of GM pasing through wall:
        // pos: Vec2f::new(640.0, 640.0),
        // vel: Vec2f::new(0.3, 0.2),

        GuidedMissile {
            pos,
            vel: Vec2f::new(cvars.g_guided_missile_speed_initial, 0.0).rotated_z(angle),
            turn_rate: 0.0,
        }
    }

    /// Returns if it hit something.
    pub fn tick(&mut self, dt: f64, cvars: &Cvars, input: &Input, map: &Map) -> bool {
        // Accel / decel
        let accel_input = input.up * cvars.g_guided_missile_speed_change
            - input.down * cvars.g_guided_missile_speed_change;
        let accel = accel_input * dt;
        let dir = self.vel.normalized();
        let speed_old = self.vel.magnitude();
        let speed_new = (speed_old + accel).clamped(
            cvars.g_guided_missile_speed_min,
            cvars.g_guided_missile_speed_max,
        );
        self.vel = speed_new * dir;

        // Turning
        // TODO this doesn't feel like flying a missile - probably needs to carry some sideways momentum
        let tr_input: f64 = input.right * cvars.g_guided_missile_turn_rate_increase * dt
            - input.left * cvars.g_guided_missile_turn_rate_increase * dt;

        // Without input, turn rate should gradually decrease towards 0.
        let tr_old = self.turn_rate;
        let tr = if tr_input == 0.0 {
            // With a fixed timestep, this would multiply tr_old each frame.
            let tr_after_friction =
                tr_old * (1.0 - cvars.g_guided_missile_turn_rate_friction).powf(dt);
            let linear = (tr_old - tr_after_friction).abs();
            // With a fixed timestep, this would subtract from tr_old each frame.
            let constant = cvars.g_guided_missile_turn_rate_decrease * dt;
            // Don't auto-decay faster than turning in the other dir would.
            let max_change = cvars.g_guided_missile_turn_rate_increase * dt;
            let decrease = (linear + constant).min(max_change);
            // Don't cross 0 and start turning in the other dir
            let tr_new = if tr_old > 0.0 {
                (tr_old - decrease).max(0.0)
            } else {
                (tr_old + decrease).min(0.0)
            };

            tr_new
        } else {
            (tr_old + tr_input).clamped(
                -cvars.g_guided_missile_turn_rate_max,
                cvars.g_guided_missile_turn_rate_max,
            )
        };

        self.vel.rotate_z(tr * dt);
        self.turn_rate = tr;

        // TODO this is broken when minimized (collision detection, etc.)
        self.pos += self.vel * dt;
        map.collision(self.pos)
    }
}

#[derive(Debug, Clone)]
pub struct Tank {
    pub pos: Vec2f,
    pub vel: Vec2f,
    pub angle: f64,
    pub turn_rate: f64,
    pub turret_angle: f64,
    /// Fraction of full
    pub hp: f64,
    /// Each weapon has a separate reload status even if they all reload at the same time.
    /// I plan to generalize this and have a cvar to choose between multiple reload mechanisms.
    pub ammos: Vec<Ammo>,
}

impl Tank {
    #[must_use]
    pub fn spawn(cvars: &Cvars, pos: Vec2f, angle: f64) -> Tank {
        let ammos = vec![
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Mg)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rail)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Cb)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Rockets)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Hm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Gm)),
            Ammo::Loaded(0.0, cvars.g_weapon_reload_ammo(Weapon::Bfg)),
        ];

        Tank {
            pos,
            vel: Vec2f::zero(),
            angle,
            turn_rate: 0.0,
            turret_angle: 0.0,
            hp: 1.0,
            ammos,
        }
    }

    pub fn tick(&mut self, dt: f64, cvars: &Cvars, input: &Input, map: &Map) {
        // Turn rate
        dbg_textf!("tank orig tr: {}", self.turn_rate);
        let tr_input = cvars.g_tank_turn_rate_increase * input.right
            - cvars.g_tank_turn_rate_increase * input.left;
        let tr_change = tr_input * dt;
        dbg_textd!(tr_change);
        self.turn_rate += tr_change;

        let tr_fric_const = cvars.g_tank_turn_rate_friction_const * dt;
        dbg_textd!(tr_fric_const);
        if self.turn_rate >= 0.0 {
            self.turn_rate = (self.turn_rate - tr_fric_const).max(0.0);
        } else {
            self.turn_rate = (self.turn_rate + tr_fric_const).min(0.0);
        }

        let tr_new = self.turn_rate * (1.0 - cvars.g_tank_turn_rate_friction_linear).powf(dt);
        dbg_textf!("diff: {:?}", self.turn_rate - tr_new);
        self.turn_rate = tr_new.clamped(-cvars.g_tank_turn_rate_max, cvars.g_tank_turn_rate_max);
        dbg_textd!(self.turn_rate);

        // Accel / decel
        // TODO lateral friction
        dbg_textf!("tank orig speed: {}", self.vel.magnitude());
        let vel_input =
            cvars.g_tank_accel_forward * input.up - cvars.g_tank_accel_backward * input.down;
        let vel_change = vel_input * dt;
        dbg_textd!(vel_change);
        self.vel += Vec2f::unit_x().rotated_z(self.angle) * vel_change;

        let vel_fric_const = cvars.g_tank_friction_const * dt;
        dbg_textd!(vel_fric_const);
        let vel_norm = self.vel.try_normalized().unwrap_or_default();
        self.vel -= (vel_fric_const).min(self.vel.magnitude()) * vel_norm;

        let vel_new = self.vel * (1.0 - cvars.g_tank_friction_linear).powf(dt);
        dbg_textf!("diff: {:?}", (self.vel - vel_new).magnitude());
        self.vel = vel_new;
        if self.vel.magnitude_squared() > cvars.g_tank_speed_max.powi(2) {
            self.vel = vel_norm * cvars.g_tank_speed_max;
        }
        dbg_textd!(self.vel.magnitude());

        // Turning - part of vel gets rotated to simulate steering
        // TODO cvar to set turning origin - original RW turned around turret center
        let vel_rotation = self.turn_rate * cvars.g_tank_turn_effectiveness;
        self.vel.rotate_z(vel_rotation);
        let new_angle = self.angle + self.turn_rate; // TODO * dt
        if Self::corners(cvars, self.pos, new_angle)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            self.turn_rate = 0.0;
        } else {
            self.angle = new_angle;
        }

        // TODO unify order with missile / input

        // Moving
        let new_pos = self.pos + self.vel * dt;
        if Self::corners(cvars, new_pos, self.angle)
            .iter()
            .any(|&corner| map.collision(corner))
        {
            self.vel = Vec2f::zero();
        } else {
            self.pos = new_pos;
        }
    }

    pub fn corners(cvars: &Cvars, pos: Vec2f, angle: f64) -> [Vec2f; 4] {
        let back_left = pos + Vec2f::new(cvars.g_tank_mins_x, cvars.g_tank_mins_y).rotated_z(angle);
        let front_left =
            pos + Vec2f::new(cvars.g_tank_maxs_x, cvars.g_tank_mins_y).rotated_z(angle);
        let front_right =
            pos + Vec2f::new(cvars.g_tank_maxs_x, cvars.g_tank_maxs_y).rotated_z(angle);
        let back_right =
            pos + Vec2f::new(cvars.g_tank_mins_x, cvars.g_tank_maxs_y).rotated_z(angle);
        [back_left, front_left, front_right, back_right]
    }
}

#[derive(Debug, Clone)]
pub enum Ammo {
    /// Refire delay end time, ammo count remaining
    Loaded(f64, u32),
    /// Start time, end time
    Reloading(f64, f64),
}
