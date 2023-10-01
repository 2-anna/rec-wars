use crate::prelude::*;

/// Things that change during the game
/// and might need to be taken back during frame interpolation / reconciliation.
#[derive(Debug, Clone)]
pub struct GameState {
    /// Inclusive range [-1.0, 1.0].
    /// Creating it once and saving it here might be faster than using gen_range according to docs.
    pub range_uniform11: Uniform<f64>,

    pub frame_num: usize,

    /// This gamelogic frame's time in seconds.
    pub game_time: f64,

    /// The previous gamelogic frame's time in seconds.
    pub game_time_prev: f64,

    /// Delta time since last gamelogic frame in seconds.
    pub dt: f64,

    pub ais: Arena<Ai>,
    pub players: Arena<Player>,
    pub vehicles: Arena<Vehicle>,
    pub projectiles: Arena<Projectile>,

    /// Map of projectile handles to vehicle handles.
    /// Prevents rail hitting the same vehicle twice
    /// when one segment ends inside the hitbox and the next starts inside it the next frame.
    /// This can for now only happen with railguns since all other projectiles get removed on hit.
    /// LATER This is still not perfect since one segment can hit multiple vehicles in any order
    /// and there's no guarantee the last vehicle is the one where the beam ends.
    /// LATER This is a can of worms:
    ///     1) Make sure (add test) one beam can kill the player and hit him again if he's unlucky enough to respawn in its path.
    ///     2) Remove the entry after the projectile exits the hitbox - e.g. guided missiles that can pass through several times.
    ///     3) Make sure the HashMap doesn't grow indefinitely in case we forgot to remove in some cases.
    ///     4) Why is this even a hashmap? Keep this as SmallVec/Set on projectile?
    pub rail_hits: FnvHashMap<Index, Index>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            range_uniform11: Uniform::new_inclusive(-1.0, 1.0),
            frame_num: 0,
            game_time: 0.0,
            game_time_prev: 0.0,
            dt: 0.0,

            ais: Arena::new(),
            players: Arena::new(),
            vehicles: Arena::new(),
            projectiles: Arena::new(),

            rail_hits: FnvHashMap::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RailBeam {
    pub begin: Vec2f,
    pub end: Vec2f,
    pub start_time: f64,
}

impl RailBeam {
    pub fn new(begin: Vec2f, end: Vec2f, start_time: f64) -> Self {
        Self {
            begin,
            end,
            start_time,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Explosion {
    pub pos: Vec2f,
    pub scale: f64,
    pub start_time: f64,
    pub bfg: bool,
}

impl Explosion {
    pub fn new(pos: Vec2f, scale: f64, start_time: f64, bfg: bool) -> Self {
        Self {
            pos,
            scale,
            start_time,
            bfg,
        }
    }
}
