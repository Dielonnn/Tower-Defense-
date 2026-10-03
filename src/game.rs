use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::enemy::{Enemy, EnemyKind};
use crate::map::{Map, tile_center};
use crate::tower::{Tower, TowerKind};
use crate::wave::{self, MAX_WAVES, Spawn};

pub const START_GOLD: u32 = 150;
pub const START_LIVES: u32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    GameOver,
    Victory,
}

pub struct Projectile {
    pub kind: TowerKind,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub target: u32,
    pub target_pos: Vec2,
    pub speed: f32,
    pub damage: f32,
    pub splash: f32,
    pub slow: f32,
    pub slow_time: f32,
}

/// A short-lived expanding ring, used for impacts and deaths.
pub struct Effect {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Color,
    pub life: f32,
    pub max_life: f32,
}

/// Floating text such as "+5" when an enemy dies.
pub struct Floater {
    pub pos: Vec2,
    pub text: String,
    pub color: Color,
    pub life: f32,
}

pub struct Game {
    pub map: Map,
    pub towers: Vec<Tower>,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    pub effects: Vec<Effect>,
    pub floaters: Vec<Floater>,
    pub gold: u32,
    pub lives: u32,
    pub wave: u32,
    pub wave_active: bool,
    pub state: GameState,
    pub speed: u32,
    pub paused: bool,
    pub build_choice: Option<TowerKind>,
    pub selected: Option<usize>,
    pub time: f32,
    pub message: Option<(String, f32)>,
    spawn_queue: VecDeque<Spawn>,
    spawn_timer: f32,
    next_id: u32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            map: Map::new(),
            towers: Vec::new(),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            effects: Vec::new(),
            floaters: Vec::new(),
            gold: START_GOLD,
            lives: START_LIVES,
            wave: 0,
            wave_active: false,
            state: GameState::Playing,
            speed: 1,
            paused: false,
            build_choice: None,
            selected: None,
            time: 0.0,
            message: None,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            next_id: 0,
        }
    }

    pub fn tower_at(&self, col: i32, row: i32) -> Option<usize> {
        self.towers
            .iter()
            .position(|t| t.col == col && t.row == row)
    }

    pub fn can_build(&self, col: i32, row: i32) -> bool {
        crate::map::in_bounds(col, row)
            && !self.map.is_path(col, row)
            && self.tower_at(col, row).is_none()
    }

    pub fn build(&mut self, kind: TowerKind, col: i32, row: i32) -> bool {
        if !self.can_build(col, row) {
            return false;
        }
        if self.gold < kind.cost() {
            self.flash("Not enough gold");
            return false;
        }
        self.gold -= kind.cost();
        self.towers.push(Tower::new(kind, col, row));
        true
    }

    pub fn upgrade_selected(&mut self) {
        let Some(tower) = self.selected.and_then(|i| self.towers.get_mut(i)) else {
            return;
        };
        let Some(cost) = tower.upgrade_cost() else {
            return;
        };
        if self.gold < cost {
            self.flash("Not enough gold");
            return;
        }
        self.gold -= cost;
        tower.level += 1;
        tower.invested += cost;
        let pos = tower.center();
        self.effects.push(Effect {
            pos,
            radius: 30.0,
            color: GOLD,
            life: 0.4,
            max_life: 0.4,
        });
    }

    pub fn sell_selected(&mut self) {
        let Some(i) = self.selected.take() else {
            return;
        };
        let tower = self.towers.remove(i);
        let value = tower.sell_value();
        self.gold += value;
        self.floaters.push(Floater {
            pos: tower.center(),
            text: format!("+{value}"),
            color: GOLD,
            life: 1.0,
        });
    }

    pub fn can_start_wave(&self) -> bool {
        self.state == GameState::Playing && !self.wave_active && self.wave < MAX_WAVES
    }

    pub fn start_wave(&mut self) {
        if !self.can_start_wave() {
            return;
        }
        self.wave += 1;
        self.wave_active = true;
        self.spawn_queue = wave::generate(self.wave).into();
        self.spawn_timer = self.spawn_queue.front().map_or(0.0, |s| s.delay);
        self.flash(&format!("Wave {}", self.wave));
    }

    fn flash(&mut self, text: &str) {
        self.message = Some((text.to_string(), 1.5));
    }

    fn spawn(&mut self, kind: EnemyKind) {
        let enemy = Enemy::new(
            self.next_id,
            kind,
            wave::hp_multiplier(self.wave),
            &self.map.waypoints,
        );
        self.next_id += 1;
        self.enemies.push(enemy);
    }

    pub fn update(&mut self, dt: f32) {
        if let Some((_, t)) = &mut self.message {
            *t -= dt;
            if *t <= 0.0 {
                self.message = None;
            }
        }
        if self.paused || self.state != GameState::Playing {
            return;
        }
        self.time += dt;

        self.update_spawning(dt);
        self.update_enemies(dt);
        self.update_towers(dt);
        self.update_projectiles(dt);
        self.collect_dead();
        self.update_effects(dt);

        if self.lives == 0 {
            self.state = GameState::GameOver;
        } else if self.wave_active && self.spawn_queue.is_empty() && self.enemies.is_empty() {
            self.wave_active = false;
            let bonus = wave::clear_bonus(self.wave);
            self.gold += bonus;
            if self.wave >= MAX_WAVES {
                self.state = GameState::Victory;
            } else {
                self.flash(&format!("Wave cleared! +{bonus} gold"));
            }
        }
    }

    fn update_spawning(&mut self, dt: f32) {
        if self.spawn_queue.is_empty() {
            return;
        }
        self.spawn_timer -= dt;
        while self.spawn_timer <= 0.0 {
            let Some(spawn) = self.spawn_queue.pop_front() else {
                break;
            };
            self.spawn(spawn.kind);
            match self.spawn_queue.front() {
                Some(next) => self.spawn_timer += next.delay,
                None => break,
            }
        }
    }

    fn update_enemies(&mut self, dt: f32) {
        let waypoints = &self.map.waypoints;
        let mut leaked = 0;
        self.enemies.retain_mut(|e| {
            if e.advance(dt, waypoints) {
                leaked += e.kind.damage();
                false
            } else {
                true
            }
        });
        self.lives = self.lives.saturating_sub(leaked);
    }

    fn update_towers(&mut self, dt: f32) {
        for tower in &mut self.towers {
            let stats = tower.stats();
            let center = tower.center();
            tower.cooldown = (tower.cooldown - dt).max(0.0);
            tower.since_shot += dt;

            let target = self
                .enemies
                .iter()
                .filter(|e| e.alive() && e.pos.distance(center) <= stats.range)
                .max_by(|a, b| a.traveled.total_cmp(&b.traveled));
            let Some(target) = target else {
                continue;
            };

            let aim = target.pos - center;
            tower.angle = aim.y.atan2(aim.x);
            if tower.cooldown > 0.0 {
                continue;
            }
            tower.cooldown = stats.cooldown;
            tower.since_shot = 0.0;
            let muzzle = center + aim.normalize_or_zero() * 18.0;
            self.projectiles.push(Projectile {
                kind: tower.kind,
                pos: muzzle,
                prev_pos: muzzle,
                target: target.id,
                target_pos: target.pos,
                speed: stats.projectile_speed,
                damage: stats.damage,
                splash: stats.splash,
                slow: stats.slow,
                slow_time: stats.slow_time,
            });
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        let enemies = &mut self.enemies;
        let effects = &mut self.effects;
        self.projectiles.retain_mut(|p| {
            if let Some(e) = enemies.iter().find(|e| e.id == p.target && e.alive()) {
                p.target_pos = e.pos;
            }
            p.prev_pos = p.pos;
            let to_target = p.target_pos - p.pos;
            let step = p.speed * dt;
            if to_target.length() > step.max(4.0) {
                p.pos += to_target.normalize() * step;
                return true;
            }
            p.pos = p.target_pos;

            if p.splash > 0.0 {
                for e in enemies.iter_mut().filter(|e| e.alive()) {
                    if e.pos.distance(p.pos) <= p.splash + e.kind.radius() {
                        e.hit(p.damage);
                    }
                }
                effects.push(Effect {
                    pos: p.pos,
                    radius: p.splash,
                    color: ORANGE,
                    life: 0.3,
                    max_life: 0.3,
                });
            } else if let Some(e) = enemies.iter_mut().find(|e| e.id == p.target && e.alive()) {
                e.hit(p.damage);
                if p.slow < 1.0 {
                    e.apply_slow(p.slow, p.slow_time);
                    effects.push(Effect {
                        pos: p.pos,
                        radius: 16.0,
                        color: SKYBLUE,
                        life: 0.25,
                        max_life: 0.25,
                    });
                }
            }
            false
        });
    }

    fn collect_dead(&mut self) {
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].alive() {
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            let reward = e.kind.reward();
            self.gold += reward;
            self.effects.push(Effect {
                pos: e.pos,
                radius: e.kind.radius() * 2.0,
                color: e.kind.color(),
                life: 0.35,
                max_life: 0.35,
            });
            self.floaters.push(Floater {
                pos: e.pos,
                text: format!("+{reward}"),
                color: GOLD,
                life: 0.9,
            });
        }
    }

    fn update_effects(&mut self, dt: f32) {
        self.effects.retain_mut(|e| {
            e.life -= dt;
            e.life > 0.0
        });
        self.floaters.retain_mut(|f| {
            f.life -= dt;
            f.pos.y -= 30.0 * dt;
            f.life > 0.0
        });
    }
}

/// Screen position of the end of the path, where the base is drawn.
pub fn base_position() -> Vec2 {
    tile_center(19, 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(game: &mut Game, seconds: f32) {
        let dt = 1.0 / 60.0;
        for _ in 0..(seconds / dt) as usize {
            game.update(dt);
        }
    }

    #[test]
    fn cannot_build_on_path_or_twice() {
        let mut game = Game::new();
        assert!(!game.build(TowerKind::Arrow, 0, 2));
        assert!(game.build(TowerKind::Arrow, 0, 0));
        assert!(!game.build(TowerKind::Arrow, 0, 0));
        assert_eq!(game.gold, START_GOLD - TowerKind::Arrow.cost());
    }

    #[test]
    fn cannot_build_without_gold() {
        let mut game = Game::new();
        game.gold = 10;
        assert!(!game.build(TowerKind::Sniper, 0, 0));
        assert!(game.towers.is_empty());
    }

    #[test]
    fn upgrade_and_sell() {
        let mut game = Game::new();
        game.gold = 1000;
        game.build(TowerKind::Arrow, 0, 0);
        game.selected = Some(0);
        game.upgrade_selected();
        assert_eq!(game.towers[0].level, 2);
        let invested = game.towers[0].invested;
        let before = game.gold;
        game.sell_selected();
        assert!(game.towers.is_empty());
        assert_eq!(game.gold - before, (invested as f32 * 0.7) as u32);
    }

    #[test]
    fn undefended_wave_costs_lives() {
        let mut game = Game::new();
        game.start_wave();
        run(&mut game, 60.0);
        assert!(!game.wave_active);
        assert_eq!(game.lives, START_LIVES - 7);
    }

    #[test]
    fn towers_defend_first_wave() {
        let mut game = Game::new();
        game.gold = 1000;
        for (c, r) in [(3, 1), (5, 3), (5, 5), (3, 6)] {
            assert!(game.build(TowerKind::Arrow, c, r));
        }
        let gold = game.gold;
        game.start_wave();
        run(&mut game, 60.0);
        assert_eq!(game.lives, START_LIVES);
        assert!(!game.wave_active);
        assert_eq!(game.wave, 1);
        assert_eq!(game.gold, gold + 7 * 5 + wave::clear_bonus(1));
    }

    #[test]
    fn running_out_of_lives_ends_game() {
        let mut game = Game::new();
        game.lives = 1;
        game.start_wave();
        run(&mut game, 60.0);
        assert_eq!(game.state, GameState::GameOver);
    }
}
