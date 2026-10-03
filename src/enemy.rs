use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

/// After thawing, an enemy can't be frozen again for this long.
pub const FREEZE_IMMUNITY: f32 = 1.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyKind {
    Grunt,
    Runner,
    Tank,
    Boss,
}

impl EnemyKind {
    pub fn base_hp(self) -> f32 {
        match self {
            Self::Grunt => 30.0,
            Self::Runner => 18.0,
            Self::Tank => 130.0,
            Self::Boss => 900.0,
        }
    }

    /// Pixels per second.
    pub fn speed(self) -> f32 {
        match self {
            // Everyone 5% slower than v4.0, bosses 15% slower.
            Self::Grunt => 90.0 * 0.95,
            Self::Runner => 160.0 * 0.95,
            Self::Tank => 56.0 * 0.95,
            Self::Boss => 35.0 * 0.85,
        }
    }

    pub fn reward(self) -> u32 {
        match self {
            Self::Grunt => 5,
            Self::Runner => 6,
            Self::Tank => 14,
            Self::Boss => 90,
        }
    }

    /// Lives lost when this enemy reaches the end of the path.
    pub fn damage(self) -> u32 {
        match self {
            Self::Boss => 10,
            Self::Tank => 2,
            _ => 1,
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Grunt => 11.0,
            Self::Runner => 8.0,
            Self::Tank => 15.0,
            Self::Boss => 21.0,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Grunt => Color::from_rgba(120, 200, 90, 255),
            Self::Runner => Color::from_rgba(240, 220, 90, 255),
            Self::Tank => Color::from_rgba(130, 130, 150, 255),
            Self::Boss => Color::from_rgba(200, 50, 70, 255),
        }
    }

    /// How strongly slows, freezes and stuns affect this enemy (1 = fully).
    fn control_resistance(self) -> f32 {
        match self {
            Self::Boss => 0.5,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub pos: Vec2,
    pub next_waypoint: usize,
    pub hp: f32,
    pub max_hp: f32,
    pub slow_factor: f32,
    pub slow_timer: f32,
    pub freeze_timer: f32,
    /// Counts down the freeze plus `FREEZE_IMMUNITY`; no refreezing until 0.
    pub freeze_immunity: f32,
    /// Extra damage fraction taken while `brittle_timer` runs.
    pub brittle: f32,
    pub brittle_timer: f32,
    pub stun_timer: f32,
    pub burn_dps: f32,
    pub burn_timer: f32,
    /// Distance walked along the path; towers target the enemy furthest ahead.
    pub traveled: f32,
    /// Time since last hit, used for a hit flash.
    pub since_hit: f32,
}

impl Enemy {
    pub fn new(id: u32, kind: EnemyKind, hp_mult: f32, waypoints: &[Vec2]) -> Self {
        let hp = kind.base_hp() * hp_mult;
        Self {
            id,
            kind,
            pos: waypoints[0],
            next_waypoint: 1,
            hp,
            max_hp: hp,
            slow_factor: 1.0,
            slow_timer: 0.0,
            freeze_timer: 0.0,
            freeze_immunity: 0.0,
            brittle: 0.0,
            brittle_timer: 0.0,
            stun_timer: 0.0,
            burn_dps: 0.0,
            burn_timer: 0.0,
            traveled: 0.0,
            since_hit: 10.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.hp > 0.0
    }

    pub fn slowed(&self) -> bool {
        self.slow_timer > 0.0
    }

    pub fn frozen(&self) -> bool {
        self.freeze_timer > 0.0
    }

    /// Frozen recently and still thawing out, so it can't be refrozen yet.
    pub fn freeze_immune(&self) -> bool {
        !self.frozen() && self.freeze_immunity > 0.0
    }

    pub fn can_freeze(&self) -> bool {
        self.freeze_immunity <= 0.0
    }

    pub fn stunned(&self) -> bool {
        self.stun_timer > 0.0
    }

    pub fn burning(&self) -> bool {
        self.burn_timer > 0.0
    }

    pub fn speed(&self) -> f32 {
        if self.frozen() || self.stunned() {
            return 0.0;
        }
        let factor = if self.slowed() { self.slow_factor } else { 1.0 };
        self.kind.speed() * factor
    }

    pub fn hit(&mut self, damage: f32) {
        let mult = if self.brittle_timer > 0.0 {
            1.0 + self.brittle
        } else {
            1.0
        };
        self.hp -= damage * mult;
        self.since_hit = 0.0;
    }

    pub fn apply_slow(&mut self, factor: f32, time: f32) {
        let factor = 1.0 - (1.0 - factor) * self.kind.control_resistance();
        self.slow_factor = if self.slowed() {
            self.slow_factor.min(factor)
        } else {
            factor
        };
        self.slow_timer = self.slow_timer.max(time);
    }

    /// Freezes the enemy unless it is still immune from its last freeze.
    pub fn apply_freeze(&mut self, time: f32) -> bool {
        if !self.can_freeze() {
            return false;
        }
        let time = time * self.kind.control_resistance();
        self.freeze_timer = time;
        self.freeze_immunity = time + FREEZE_IMMUNITY;
        true
    }

    pub fn apply_brittle(&mut self, amount: f32, time: f32) {
        self.brittle = if self.brittle_timer > 0.0 {
            self.brittle.max(amount)
        } else {
            amount
        };
        self.brittle_timer = self.brittle_timer.max(time);
    }

    pub fn apply_stun(&mut self, time: f32) {
        let time = time * self.kind.control_resistance();
        self.stun_timer = self.stun_timer.max(time);
    }

    pub fn apply_burn(&mut self, dps: f32, time: f32) {
        self.burn_dps = if self.burning() {
            self.burn_dps.max(dps)
        } else {
            dps
        };
        self.burn_timer = self.burn_timer.max(time);
    }

    /// Where this enemy will be after `t` seconds at its current speed.
    pub fn predict(&self, t: f32, waypoints: &[Vec2]) -> Vec2 {
        let mut pos = self.pos;
        let mut next = self.next_waypoint;
        walk(&mut pos, &mut next, self.speed() * t, waypoints);
        pos
    }

    /// Moves along the path and ticks status effects. Returns true once the
    /// enemy has reached the end.
    pub fn advance(&mut self, dt: f32, waypoints: &[Vec2]) -> bool {
        if self.burning() {
            self.hp -= self.burn_dps * dt.min(self.burn_timer);
        }
        let distance = self.speed() * dt;
        // Slows only tick down once the enemy has thawed out.
        if !self.frozen() {
            self.slow_timer = (self.slow_timer - dt).max(0.0);
        }
        self.freeze_timer = (self.freeze_timer - dt).max(0.0);
        self.freeze_immunity = (self.freeze_immunity - dt).max(0.0);
        self.brittle_timer = (self.brittle_timer - dt).max(0.0);
        self.stun_timer = (self.stun_timer - dt).max(0.0);
        self.burn_timer = (self.burn_timer - dt).max(0.0);
        self.since_hit += dt;
        self.traveled += walk(&mut self.pos, &mut self.next_waypoint, distance, waypoints);
        self.next_waypoint >= waypoints.len()
    }
}

/// Moves `pos` up to `distance` along the path, returning how far it went.
fn walk(pos: &mut Vec2, next: &mut usize, distance: f32, waypoints: &[Vec2]) -> f32 {
    let mut remaining = distance;
    while remaining > 0.0 && *next < waypoints.len() {
        let to_target = waypoints[*next] - *pos;
        let dist = to_target.length();
        if dist <= remaining {
            *pos = waypoints[*next];
            remaining -= dist;
            *next += 1;
        } else {
            *pos += to_target / dist * remaining;
            remaining = 0.0;
        }
    }
    distance - remaining
}
