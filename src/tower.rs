use crate::map::{TILE, tile_center};
use macroquad::prelude::*;

pub const MAX_LEVEL: u8 = 3;
pub const SELL_RATIO: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerKind {
    Arrow,
    Cannon,
    Frost,
    Sniper,
}

impl TowerKind {
    pub const ALL: [TowerKind; 4] = [Self::Arrow, Self::Cannon, Self::Frost, Self::Sniper];

    pub fn name(self) -> &'static str {
        match self {
            Self::Arrow => "Arrow",
            Self::Cannon => "Cannon",
            Self::Frost => "Frost",
            Self::Sniper => "Sniper",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Arrow => "Fast single target",
            Self::Cannon => "Slow, splash damage",
            Self::Frost => "Slows enemies down",
            Self::Sniper => "Huge range and damage",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Self::Arrow => 50,
            Self::Cannon => 90,
            Self::Frost => 70,
            Self::Sniper => 120,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Arrow => Color::from_rgba(230, 190, 70, 255),
            Self::Cannon => Color::from_rgba(200, 90, 60, 255),
            Self::Frost => Color::from_rgba(110, 200, 240, 255),
            Self::Sniper => Color::from_rgba(170, 120, 220, 255),
        }
    }

    pub fn stats(self, level: u8) -> Stats {
        let l = (level.clamp(1, MAX_LEVEL) - 1) as usize;
        match self {
            Self::Arrow => Stats {
                damage: [10.0, 18.0, 30.0][l],
                range: [2.6, 2.9, 3.2][l] * TILE,
                cooldown: [0.45, 0.38, 0.3][l],
                projectile_speed: 520.0,
                ..Stats::default()
            },
            Self::Cannon => Stats {
                damage: [24.0, 44.0, 80.0][l],
                range: [2.4, 2.6, 2.8][l] * TILE,
                cooldown: [1.3, 1.15, 1.0][l],
                splash: [0.9, 1.05, 1.2][l] * TILE,
                projectile_speed: 300.0,
                ..Stats::default()
            },
            Self::Frost => Stats {
                damage: [4.0, 8.0, 14.0][l],
                range: [2.2, 2.5, 2.8][l] * TILE,
                cooldown: [0.8, 0.7, 0.6][l],
                slow: [0.55, 0.45, 0.35][l],
                slow_time: [1.5, 1.8, 2.2][l],
                projectile_speed: 380.0,
                ..Stats::default()
            },
            Self::Sniper => Stats {
                damage: [70.0, 140.0, 260.0][l],
                range: [5.5, 6.2, 7.0][l] * TILE,
                cooldown: [1.8, 1.6, 1.4][l],
                projectile_speed: 1400.0,
                ..Stats::default()
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub damage: f32,
    /// Range in pixels.
    pub range: f32,
    /// Seconds between shots.
    pub cooldown: f32,
    /// Splash radius in pixels; 0 for single target.
    pub splash: f32,
    /// Speed multiplier applied to enemies that are hit; 1 for none.
    pub slow: f32,
    pub slow_time: f32,
    pub projectile_speed: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            damage: 0.0,
            range: 0.0,
            cooldown: 1.0,
            splash: 0.0,
            slow: 1.0,
            slow_time: 0.0,
            projectile_speed: 400.0,
        }
    }
}

pub struct Tower {
    pub kind: TowerKind,
    pub level: u8,
    pub col: i32,
    pub row: i32,
    pub cooldown: f32,
    pub angle: f32,
    /// Gold spent on this tower so far, used for the sell price.
    pub invested: u32,
    /// Time since the last shot, used for the muzzle flash.
    pub since_shot: f32,
}

impl Tower {
    pub fn new(kind: TowerKind, col: i32, row: i32) -> Self {
        Self {
            kind,
            level: 1,
            col,
            row,
            cooldown: 0.0,
            angle: 0.0,
            invested: kind.cost(),
            since_shot: 10.0,
        }
    }

    pub fn center(&self) -> Vec2 {
        tile_center(self.col, self.row)
    }

    pub fn stats(&self) -> Stats {
        self.kind.stats(self.level)
    }

    pub fn upgrade_cost(&self) -> Option<u32> {
        let base = self.kind.cost() as f32;
        match self.level {
            1 => Some((base * 0.8) as u32),
            2 => Some((base * 1.4) as u32),
            _ => None,
        }
    }

    pub fn sell_value(&self) -> u32 {
        (self.invested as f32 * SELL_RATIO) as u32
    }
}
