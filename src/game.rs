use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::enemy::{Enemy, EnemyKind};
use crate::map::{Map, PATH_WIDTH, TILE, TOWER_RADIUS, distance_to_segment, in_map};
use crate::tower::{Stats, Tower, TowerKind};
use crate::wave::{self, MAX_WAVES, Spawn};

pub const START_GOLD: u32 = 150;
pub const START_LIVES: u32 = 20;
/// Pause between waves when auto play is on.
pub const AUTO_DELAY: f32 = 1.5;
/// How far a sniper shot can bounce to the next enemy.
const BOUNCE_RANGE: f32 = 2.5 * TILE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    GameOver,
    Victory,
}

pub enum Shot {
    /// Flies straight, hitting every enemy it passes until it runs out.
    Arrow {
        dir: Vec2,
        remaining: f32,
        hit: Vec<u32>,
    },
    /// Homes in on its target and explodes.
    Shell { target: u32, target_pos: Vec2 },
}

pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub stats: Stats,
    pub shot: Shot,
}

pub enum EffectShape {
    Ring(f32),
    Line(Vec2),
}

pub struct Effect {
    pub pos: Vec2,
    pub shape: EffectShape,
    pub color: Color,
    pub life: f32,
    pub max_life: f32,
}

impl Effect {
    fn ring(pos: Vec2, radius: f32, color: Color, life: f32) -> Self {
        Self {
            pos,
            shape: EffectShape::Ring(radius),
            color,
            life,
            max_life: life,
        }
    }

    fn line(from: Vec2, to: Vec2, color: Color) -> Self {
        Self {
            pos: from,
            shape: EffectShape::Line(to),
            color,
            life: 0.15,
            max_life: 0.15,
        }
    }
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
    pub map_index: usize,
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
    pub auto_play: bool,
    auto_timer: f32,
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
        Self::with_map(0)
    }

    pub fn with_map(map_index: usize) -> Self {
        Self {
            map: Map::new(map_index),
            map_index,
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
            auto_play: false,
            auto_timer: 0.0,
            build_choice: None,
            selected: None,
            time: 0.0,
            message: None,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            next_id: 0,
        }
    }

    /// Switches to the next map. Only allowed before the first wave.
    pub fn next_map(&mut self) {
        if self.wave == 0 {
            let (speed, auto) = (self.speed, self.auto_play);
            *self = Self::with_map(self.map_index + 1);
            self.speed = speed;
            self.auto_play = auto;
        }
    }

    pub fn tower_at(&self, pos: Vec2) -> Option<usize> {
        self.towers
            .iter()
            .position(|t| t.pos.distance(pos) <= TOWER_RADIUS + 2.0)
    }

    pub fn can_place(&self, pos: Vec2) -> bool {
        in_map(pos, TOWER_RADIUS)
            && self.map.distance_to_path(pos) >= PATH_WIDTH / 2.0 + TOWER_RADIUS - 2.0
            && self
                .towers
                .iter()
                .all(|t| t.pos.distance(pos) >= TOWER_RADIUS * 2.0)
    }

    pub fn place(&mut self, kind: TowerKind, pos: Vec2) -> bool {
        if !self.can_place(pos) {
            return false;
        }
        if self.gold < kind.cost() {
            self.flash("Not enough gold");
            return false;
        }
        self.gold -= kind.cost();
        self.towers.push(Tower::new(kind, pos));
        true
    }

    pub fn upgrade_selected(&mut self, path: usize) {
        let Some(tower) = self.selected.and_then(|i| self.towers.get_mut(i)) else {
            return;
        };
        let Some(cost) = tower.upgrade_cost(path) else {
            return;
        };
        if self.gold < cost {
            self.flash("Not enough gold");
            return;
        }
        self.gold -= cost;
        tower.tiers[path] += 1;
        tower.invested += cost;
        let pos = tower.pos;
        self.effects.push(Effect::ring(pos, 30.0, GOLD, 0.4));
    }

    pub fn sell_selected(&mut self) {
        let Some(i) = self.selected.take() else {
            return;
        };
        let tower = self.towers.remove(i);
        let value = tower.sell_value();
        self.gold += value;
        self.floaters.push(Floater {
            pos: tower.pos,
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

    pub fn toggle_auto(&mut self) {
        self.auto_play = !self.auto_play;
        self.auto_timer = 0.0;
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

        if self.auto_play && self.can_start_wave() {
            self.auto_timer -= dt;
            if self.auto_timer <= 0.0 {
                self.start_wave();
            }
        }

        self.update_spawning(dt);
        self.update_enemies(dt);
        self.update_towers(dt);
        self.update_projectiles(dt);
        self.collect_dead();
        self.update_effects(dt);

        if self.lives == 0 {
            self.state = GameState::GameOver;
        } else if self.wave_active && self.spawn_queue.is_empty() && self.enemies.is_empty() {
            self.finish_wave();
        }
    }

    fn finish_wave(&mut self) {
        self.wave_active = false;
        self.auto_timer = AUTO_DELAY;
        let bonus = wave::clear_bonus(self.wave);
        let mut income = 0;
        for tower in &self.towers {
            let stats = tower.stats();
            income += stats.income;
            self.lives += stats.lives_per_wave;
            let mut parts = Vec::new();
            if stats.income > 0 {
                parts.push(format!("+{}g", stats.income));
            }
            if stats.lives_per_wave > 0 {
                parts.push(format!("+{} hp", stats.lives_per_wave));
            }
            if !parts.is_empty() {
                self.floaters.push(Floater {
                    pos: tower.pos - vec2(0.0, 20.0),
                    text: parts.join(" "),
                    color: GOLD,
                    life: 1.5,
                });
            }
        }
        self.gold += bonus + income;
        if self.wave >= MAX_WAVES {
            self.state = GameState::Victory;
        } else if income > 0 {
            self.flash(&format!("Wave cleared! +{bonus} bonus, +{income} income"));
        } else {
            self.flash(&format!("Wave cleared! +{bonus} gold"));
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
            if e.advance(dt, waypoints) && e.alive() {
                leaked += e.kind.damage();
                false
            } else {
                true
            }
        });
        self.lives = self.lives.saturating_sub(leaked);
    }

    fn update_towers(&mut self, dt: f32) {
        let Self {
            towers,
            enemies,
            projectiles,
            effects,
            map,
            ..
        } = self;

        for tower in towers.iter_mut() {
            tower.cooldown = (tower.cooldown - dt).max(0.0);
            tower.since_shot += dt;
            let stats = tower.stats();
            let in_range = |e: &Enemy| {
                e.alive() && in_map(e.pos, 0.0) && e.pos.distance(tower.pos) <= stats.range
            };

            match tower.kind {
                TowerKind::Farm => {}

                TowerKind::Frost => {
                    // Debuff tower: only pulse when someone in range isn't slowed yet,
                    // then chill everything in range.
                    let fresh = enemies.iter().any(|e| in_range(e) && !e.slowed());
                    if tower.cooldown > 0.0 || (tower.kind.is_debuff() && !fresh) {
                        continue;
                    }
                    for e in enemies.iter_mut().filter(|e| in_range(e)) {
                        e.hit(stats.damage);
                        e.apply_slow(stats.slow, stats.slow_time, stats.brittle);
                        if stats.freeze_time > 0.0 {
                            e.apply_stun(stats.freeze_time);
                        }
                    }
                    tower.cooldown = stats.cooldown;
                    tower.since_shot = 0.0;
                    effects.push(Effect::ring(tower.pos, stats.range, SKYBLUE, 0.4));
                }

                TowerKind::Sniper => {
                    let Some(first) = furthest(enemies, &in_range) else {
                        continue;
                    };
                    let aim = enemies[first].pos - tower.pos;
                    tower.angle = aim.y.atan2(aim.x);
                    if tower.cooldown > 0.0 {
                        continue;
                    }
                    tower.cooldown = stats.cooldown;
                    tower.since_shot = 0.0;
                    let muzzle = tower.pos + aim.normalize_or_zero() * 24.0;
                    sniper_shot(enemies, effects, first, muzzle, &stats);
                }

                TowerKind::Arrow | TowerKind::Cannon => {
                    let Some(i) = furthest(enemies, &in_range) else {
                        continue;
                    };
                    let target = &enemies[i];
                    // Lead the target so straight-flying arrows connect.
                    let mut aim = target.pos;
                    for _ in 0..2 {
                        let t = aim.distance(tower.pos) / stats.projectile_speed;
                        aim = target.predict(t, &map.waypoints);
                    }
                    let to_aim = aim - tower.pos;
                    tower.angle = to_aim.y.atan2(to_aim.x);
                    if tower.cooldown > 0.0 {
                        continue;
                    }
                    tower.cooldown = stats.cooldown;
                    tower.since_shot = 0.0;

                    if tower.kind == TowerKind::Cannon {
                        let muzzle = tower.pos + to_aim.normalize_or_zero() * 18.0;
                        projectiles.push(Projectile {
                            pos: muzzle,
                            prev_pos: muzzle,
                            stats,
                            shot: Shot::Shell {
                                target: target.id,
                                target_pos: target.pos,
                            },
                        });
                        continue;
                    }

                    let n = stats.arrows.max(1);
                    for k in 0..n {
                        let spread = (k as f32 - (n - 1) as f32 / 2.0) * 0.12;
                        let angle = tower.angle + spread;
                        let dir = vec2(angle.cos(), angle.sin());
                        let muzzle = tower.pos + dir * 16.0;
                        projectiles.push(Projectile {
                            pos: muzzle,
                            prev_pos: muzzle,
                            stats,
                            shot: Shot::Arrow {
                                dir,
                                remaining: to_aim.length() - 16.0 + stats.pierce,
                                hit: Vec::new(),
                            },
                        });
                    }
                }
            }
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        let enemies = &mut self.enemies;
        let effects = &mut self.effects;
        self.projectiles.retain_mut(|p| {
            p.prev_pos = p.pos;
            let s = p.stats;
            match &mut p.shot {
                Shot::Arrow {
                    dir,
                    remaining,
                    hit,
                } => {
                    let step = (s.projectile_speed * dt).min(*remaining);
                    p.pos += *dir * step;
                    *remaining -= step;
                    for e in enemies.iter_mut() {
                        if !e.alive() || hit.contains(&e.id) {
                            continue;
                        }
                        if distance_to_segment(e.pos, p.prev_pos, p.pos) <= e.kind.radius() + 3.0 {
                            e.hit(s.damage);
                            hit.push(e.id);
                        }
                    }
                    *remaining > 0.0 && in_map(p.pos, -40.0)
                }
                Shot::Shell { target, target_pos } => {
                    if let Some(e) = enemies.iter().find(|e| e.id == *target && e.alive()) {
                        *target_pos = e.pos;
                    }
                    let to_target = *target_pos - p.pos;
                    let step = s.projectile_speed * dt;
                    if to_target.length() > step.max(4.0) {
                        p.pos += to_target.normalize() * step;
                        return true;
                    }
                    p.pos = *target_pos;
                    for e in enemies.iter_mut().filter(|e| e.alive()) {
                        if e.pos.distance(p.pos) <= s.splash + e.kind.radius() {
                            e.hit(s.damage);
                            if s.burn_dps > 0.0 {
                                e.apply_burn(s.burn_dps, s.burn_time);
                            }
                            if s.stun_time > 0.0 {
                                e.apply_stun(s.stun_time);
                            }
                        }
                    }
                    let color = if s.burn_dps > 0.0 { RED } else { ORANGE };
                    effects.push(Effect::ring(p.pos, s.splash, color, 0.3));
                    false
                }
            }
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
            let bounty: u32 = self
                .towers
                .iter()
                .map(|t| t.stats())
                .zip(&self.towers)
                .filter(|(s, t)| s.kill_bounty > 0 && t.pos.distance(e.pos) <= s.bounty_range)
                .map(|(s, _)| s.kill_bounty)
                .sum();
            let reward = e.kind.reward() + bounty;
            self.gold += reward;
            self.effects.push(Effect::ring(
                e.pos,
                e.kind.radius() * 2.0,
                e.kind.color(),
                0.35,
            ));
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

/// Index of the enemy furthest along the path that passes `filter`.
fn furthest(enemies: &[Enemy], filter: &impl Fn(&Enemy) -> bool) -> Option<usize> {
    enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| filter(e))
        .max_by(|a, b| a.1.traveled.total_cmp(&b.1.traveled))
        .map(|(i, _)| i)
}

/// Instant hit on `first`, then bounces to nearby enemies.
fn sniper_shot(
    enemies: &mut [Enemy],
    effects: &mut Vec<Effect>,
    first: usize,
    muzzle: Vec2,
    s: &Stats,
) {
    let tracer = Color::from_rgba(230, 200, 255, 255);
    let mut hit = vec![first];
    let mut from = muzzle;
    let mut current = first;
    loop {
        let e = &mut enemies[current];
        let mult = if e.kind == EnemyKind::Boss {
            s.boss_mult
        } else {
            1.0
        };
        e.hit(s.damage * mult);
        effects.push(Effect::line(from, e.pos, tracer));
        from = e.pos;
        if hit.len() > s.bounces as usize {
            break;
        }
        let next = enemies
            .iter()
            .enumerate()
            .filter(|(i, e)| e.alive() && !hit.contains(i) && e.pos.distance(from) <= BOUNCE_RANGE)
            .min_by(|a, b| a.1.pos.distance(from).total_cmp(&b.1.pos.distance(from)));
        let Some((i, _)) = next else {
            break;
        };
        hit.push(i);
        current = i;
    }
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

    /// An open grass spot on the default map, away from the path.
    fn grass() -> Vec2 {
        vec2(560.0, 300.0)
    }

    fn enemy_at(game: &mut Game, kind: EnemyKind, pos: Vec2) -> u32 {
        game.spawn(kind);
        let e = game.enemies.last_mut().unwrap();
        e.pos = pos;
        e.next_waypoint = game.map.waypoints.len() - 1;
        e.id
    }

    #[test]
    fn placement_rules() {
        let mut game = Game::new();
        let on_path = game.map.waypoints[1];
        assert!(!game.place(TowerKind::Arrow, on_path));
        assert!(game.place(TowerKind::Arrow, grass()));
        assert!(
            !game.place(TowerKind::Arrow, grass() + vec2(10.0, 0.0)),
            "overlap"
        );
        assert!(
            game.place(TowerKind::Arrow, grass() + vec2(40.0, 0.0)),
            "free placement"
        );
        assert_eq!(game.gold, START_GOLD - 2 * TowerKind::Arrow.cost());
    }

    #[test]
    fn cannot_place_without_gold() {
        let mut game = Game::new();
        game.gold = 10;
        assert!(!game.place(TowerKind::Sniper, grass()));
        assert!(game.towers.is_empty());
    }

    #[test]
    fn upgrade_and_sell() {
        let mut game = Game::new();
        game.gold = 10_000;
        game.place(TowerKind::Arrow, grass());
        game.selected = Some(0);
        game.upgrade_selected(0);
        game.upgrade_selected(1);
        game.upgrade_selected(2); // blocked: only two paths
        assert_eq!(game.towers[0].tiers, [1, 1, 0]);
        let invested = game.towers[0].invested;
        let before = game.gold;
        game.sell_selected();
        assert!(game.towers.is_empty());
        assert_eq!(game.gold - before, (invested as f32 * 0.7) as u32);
    }

    #[test]
    fn arrows_pierce_two_tiles_behind() {
        let mut game = Game::new();
        game.gold = 1000;
        let origin = vec2(500.0, 300.0);
        game.towers.push(Tower::new(TowerKind::Arrow, origin));
        // Stun everyone so they stand still in a line to the right.
        let near = enemy_at(&mut game, EnemyKind::Tank, origin + vec2(100.0, 0.0));
        let behind = enemy_at(
            &mut game,
            EnemyKind::Tank,
            origin + vec2(100.0 + 1.5 * TILE, 0.0),
        );
        let far = enemy_at(
            &mut game,
            EnemyKind::Tank,
            origin + vec2(100.0 + 3.0 * TILE, 0.0),
        );
        for e in &mut game.enemies {
            e.stun_timer = 100.0;
        }
        // Make sure the tower aims at the near one.
        game.enemies
            .iter_mut()
            .find(|e| e.id == near)
            .unwrap()
            .traveled = 10.0;
        game.update_towers(0.0);
        assert_eq!(game.projectiles.len(), 1);
        for _ in 0..60 {
            game.update_projectiles(1.0 / 60.0);
        }
        let hp = |id| game.enemies.iter().find(|e| e.id == id).unwrap();
        assert!(hp(near).hp < hp(near).max_hp);
        assert!(hp(behind).hp < hp(behind).max_hp);
        assert_eq!(hp(far).hp, hp(far).max_hp);
    }

    #[test]
    fn frost_hits_everything_in_range_but_skips_when_all_slowed() {
        let mut game = Game::new();
        let origin = grass();
        game.towers.push(Tower::new(TowerKind::Frost, origin));
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(40.0, 0.0));
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(-40.0, 20.0));
        game.update_towers(0.0);
        assert!(game.enemies.iter().all(|e| e.slowed() && e.hp < e.max_hp));

        // Everything in range is already slowed, so the next pulse is held.
        let hp: Vec<f32> = game.enemies.iter().map(|e| e.hp).collect();
        game.towers[0].cooldown = 0.0;
        game.update_towers(0.0);
        let after: Vec<f32> = game.enemies.iter().map(|e| e.hp).collect();
        assert_eq!(hp, after);
        assert_eq!(game.towers[0].cooldown, 0.0);
    }

    #[test]
    fn sniper_hits_across_the_map() {
        let mut game = Game::new();
        game.towers
            .push(Tower::new(TowerKind::Sniper, vec2(30.0, 700.0)));
        let id = enemy_at(&mut game, EnemyKind::Tank, vec2(900.0, 80.0));
        game.update_towers(0.0);
        let e = game.enemies.iter().find(|e| e.id == id).unwrap();
        assert!(e.hp < e.max_hp);
    }

    #[test]
    fn farm_pays_each_wave() {
        let mut game = Game::new();
        game.place(TowerKind::Farm, grass());
        game.lives = 100;
        let gold = game.gold;
        game.start_wave();
        run(&mut game, 120.0);
        assert!(!game.wave_active);
        let kills = 0; // no defenses, so nothing was killed
        assert_eq!(game.gold, gold + kills + wave::clear_bonus(1) + 30);
    }

    #[test]
    fn auto_play_starts_next_wave() {
        let mut game = Game::new();
        game.lives = 1000;
        game.toggle_auto();
        run(&mut game, 0.1);
        assert_eq!(game.wave, 1);
        run(&mut game, 120.0);
        assert!(game.wave >= 2);
    }

    #[test]
    fn undefended_wave_costs_lives() {
        let mut game = Game::new();
        game.start_wave();
        run(&mut game, 120.0);
        assert!(!game.wave_active);
        assert_eq!(game.lives, START_LIVES - 7);
    }

    #[test]
    fn running_out_of_lives_ends_game() {
        let mut game = Game::new();
        game.lives = 1;
        game.start_wave();
        run(&mut game, 120.0);
        assert_eq!(game.state, GameState::GameOver);
    }
}
