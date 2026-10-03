use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            Self::Grunt => 60.0,
            Self::Runner => 110.0,
            Self::Tank => 38.0,
            Self::Boss => 30.0,
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

    /// How strongly slows affect this enemy (1 = fully).
    fn slow_resistance(self) -> f32 {
        match self {
            Self::Boss => 0.5,
            _ => 1.0,
        }
    }
}

pub struct Enemy {
    pub id: u32,
    pub kind: EnemyKind,
    pub pos: Vec2,
    pub next_waypoint: usize,
    pub hp: f32,
    pub max_hp: f32,
    pub slow_factor: f32,
    pub slow_timer: f32,
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

    pub fn speed(&self) -> f32 {
        let factor = if self.slowed() { self.slow_factor } else { 1.0 };
        self.kind.speed() * factor
    }

    pub fn hit(&mut self, damage: f32) {
        self.hp -= damage;
        self.since_hit = 0.0;
    }

    pub fn apply_slow(&mut self, factor: f32, time: f32) {
        let factor = 1.0 - (1.0 - factor) * self.kind.slow_resistance();
        self.slow_factor = if self.slowed() {
            self.slow_factor.min(factor)
        } else {
            factor
        };
        self.slow_timer = self.slow_timer.max(time);
    }

    /// Moves along the path. Returns true once the enemy has reached the end.
    pub fn advance(&mut self, dt: f32, waypoints: &[Vec2]) -> bool {
        self.slow_timer = (self.slow_timer - dt).max(0.0);
        self.since_hit += dt;
        let mut remaining = self.speed() * dt;
        while remaining > 0.0 && self.next_waypoint < waypoints.len() {
            let target = waypoints[self.next_waypoint];
            let to_target = target - self.pos;
            let dist = to_target.length();
            if dist <= remaining {
                self.pos = target;
                self.traveled += dist;
                remaining -= dist;
                self.next_waypoint += 1;
            } else {
                self.pos += to_target / dist * remaining;
                self.traveled += remaining;
                remaining = 0.0;
            }
        }
        self.next_waypoint >= waypoints.len()
    }
}
