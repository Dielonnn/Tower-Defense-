use std::collections::VecDeque;

use macroquad::prelude::*;

use crate::enemy::{Enemy, EnemyKind};
use crate::map::{Map, PATH_WIDTH, TILE, TOWER_RADIUS, distance_to_segment, in_map};
use crate::tower::{Stats, Targeting, Tower, TowerKind};
use crate::wave::{self, MAX_WAVES, Spawn};

pub const START_GOLD: u32 = 150;
pub const START_LIVES: u32 = 20;
/// Pause between waves when auto play is on.
pub const AUTO_DELAY: f32 = 1.5;
/// How far a sniper shot can bounce to the next enemy.
const BOUNCE_RANGE: f32 = 2.5 * TILE;
/// Highest wave the sandbox level selector goes to.
pub const SANDBOX_MAX_WAVE: u32 = 99;

/// Full-screen menus that pause the game while open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Settings,
    /// Tower guide, showing the tower at this index of `TowerKind::ALL`.
    Guide(usize),
}

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
    /// Sandbox: everything is free, lives never drop, waves never end the game.
    pub sandbox: bool,
    pub night: bool,
    pub menu: Option<Menu>,
    /// Time since an enemy last reached the castle, for the hit flash.
    pub since_base_hit: f32,
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
            sandbox: false,
            night: false,
            menu: None,
            since_base_hit: 10.0,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            next_id: 0,
        }
    }

    /// Starts a fresh game, keeping the player's settings.
    pub fn restart(&mut self, map_index: usize, sandbox: bool) {
        let (speed, auto, night) = (self.speed, self.auto_play, self.night);
        *self = Self::with_map(map_index);
        self.speed = speed;
        self.auto_play = auto;
        self.night = night;
        self.sandbox = sandbox;
        if sandbox {
            self.flash("Sandbox: everything is free");
        }
    }

    /// Switches to the next map. Only allowed before the first wave.
    pub fn next_map(&mut self) {
        if self.wave == 0 {
            self.restart(self.map_index + 1, self.sandbox);
        }
    }

    /// Pays `cost` if possible. Everything is free in sandbox mode.
    fn charge(&mut self, cost: u32) -> bool {
        if self.sandbox {
            return true;
        }
        if self.gold < cost {
            self.flash("Not enough gold");
            return false;
        }
        self.gold -= cost;
        true
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.sandbox || self.gold >= cost
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
        if !self.can_place(pos) || !self.charge(kind.cost()) {
            return false;
        }
        self.towers.push(Tower::new(kind, pos));
        true
    }

    pub fn upgrade_selected(&mut self, path: usize) {
        let Some(i) = self.selected.filter(|&i| i < self.towers.len()) else {
            return;
        };
        let Some(cost) = self.towers[i].upgrade_cost(path) else {
            return;
        };
        if !self.charge(cost) {
            return;
        }
        let tower = &mut self.towers[i];
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

    pub fn cycle_targeting(&mut self) {
        if let Some(t) = self.selected.and_then(|i| self.towers.get_mut(i))
            && t.kind.has_targeting()
        {
            t.targeting = t.targeting.next();
        }
    }

    pub fn max_wave(&self) -> u32 {
        if self.sandbox {
            SANDBOX_MAX_WAVE
        } else {
            MAX_WAVES
        }
    }

    pub fn can_start_wave(&self) -> bool {
        self.state == GameState::Playing && !self.wave_active && self.wave < self.max_wave()
    }

    /// Sandbox: spawn a single enemy at the current wave's strength.
    pub fn sandbox_spawn(&mut self, kind: EnemyKind) {
        if self.sandbox {
            self.spawn(kind);
        }
    }

    /// Sandbox: change which wave comes next (and how tough spawns are).
    pub fn sandbox_set_wave(&mut self, wave: i32) {
        if self.sandbox && !self.wave_active {
            self.wave = wave.clamp(0, SANDBOX_MAX_WAVE as i32 - 1) as u32;
        }
    }

    /// Sandbox: remove every enemy and cancel the current wave.
    pub fn sandbox_clear(&mut self) {
        if self.sandbox {
            self.enemies.clear();
            self.projectiles.clear();
            self.spawn_queue.clear();
            self.wave_active = false;
        }
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
            wave::hp_multiplier(self.wave.max(1)),
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
        if self.paused || self.menu.is_some() || self.state != GameState::Playing {
            return;
        }
        self.time += dt;
        self.since_base_hit += dt;

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
        let savings = self.gold;
        for tower in &self.towers {
            let stats = tower.stats();
            let interest = ((savings as f32 * stats.interest) as u32).min(stats.interest_cap);
            let earned = stats.income + interest;
            income += earned;
            self.lives += stats.lives_per_wave;
            let mut parts = Vec::new();
            if earned > 0 {
                parts.push(format!("+{earned}g"));
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
        if self.wave >= MAX_WAVES && !self.sandbox {
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
        if leaked == 0 {
            return;
        }
        // Enemies hit the castle at the end of the path.
        let base = self.map.base;
        self.since_base_hit = 0.0;
        self.effects.push(Effect::ring(base, 36.0, RED, 0.4));
        self.floaters.push(Floater {
            pos: base - vec2(0.0, 30.0),
            text: format!("-{leaked}"),
            color: RED,
            life: 1.0,
        });
        if !self.sandbox {
            self.lives = self.lives.saturating_sub(leaked);
        }
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
                    // Debuff tower: only pulse when someone in range isn't frozen,
                    // then freeze everything in range.
                    let fresh = enemies.iter().any(|e| in_range(e) && !e.frozen());
                    if tower.cooldown > 0.0 || (tower.kind.is_debuff() && !fresh) {
                        continue;
                    }
                    for e in enemies.iter_mut().filter(|e| in_range(e)) {
                        e.hit(stats.damage);
                        e.apply_freeze(stats.freeze_time);
                        // Slows kick in once the enemy thaws.
                        if stats.slow < 1.0 {
                            e.apply_slow(stats.slow, stats.slow_time);
                        }
                        if stats.brittle > 0.0 {
                            e.apply_brittle(stats.brittle, stats.freeze_time + 2.0);
                        }
                    }
                    tower.cooldown = stats.cooldown;
                    tower.since_shot = 0.0;
                    effects.push(Effect::ring(tower.pos, stats.range, SKYBLUE, 0.4));
                }

                TowerKind::Sniper => {
                    let Some(first) = pick_target(enemies, &in_range, tower.targeting, tower.pos)
                    else {
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
                    let Some(i) = pick_target(enemies, &in_range, tower.targeting, tower.pos)
                    else {
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
            let reward = e.kind.reward();
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

/// Index of the enemy passing `filter` that `mode` prefers most.
fn pick_target(
    enemies: &[Enemy],
    filter: &impl Fn(&Enemy) -> bool,
    mode: Targeting,
    origin: Vec2,
) -> Option<usize> {
    // Higher is better; the second value breaks ties.
    let score = |e: &Enemy| -> (f32, f32) {
        match mode {
            Targeting::First => (e.traveled, 0.0),
            Targeting::Last => (-e.traveled, 0.0),
            Targeting::Strongest => (e.hp, e.traveled),
            Targeting::Closest => (-e.pos.distance(origin), e.traveled),
            Targeting::Boss => {
                let rank = match e.kind {
                    EnemyKind::Boss => 3.0,
                    EnemyKind::Tank => 2.0,
                    EnemyKind::Runner => 1.0,
                    EnemyKind::Grunt => 0.0,
                };
                (rank, e.traveled)
            }
        }
    };
    enemies
        .iter()
        .enumerate()
        .filter(|(_, e)| filter(e))
        .max_by(|a, b| {
            let (sa, sb) = (score(a.1), score(b.1));
            sa.0.total_cmp(&sb.0).then(sa.1.total_cmp(&sb.1))
        })
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
    fn frost_freezes_everything_in_range_but_skips_when_all_frozen() {
        let mut game = Game::new();
        let origin = grass();
        game.towers.push(Tower::new(TowerKind::Frost, origin));
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(40.0, 0.0));
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(-40.0, 20.0));
        game.update_towers(0.0);
        assert!(
            game.enemies
                .iter()
                .all(|e| e.frozen() && !e.slowed() && e.hp < e.max_hp)
        );

        // Everything in range is already frozen, so the next pulse is held.
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
        assert_eq!(game.gold, gold + kills + wave::clear_bonus(1) + 40);
    }

    #[test]
    fn farm_bank_pays_capped_interest() {
        let mut game = Game::new();
        game.lives = 100;
        game.towers.push(Tower::new(TowerKind::Farm, grass()));
        game.towers[0].tiers = [0, 1, 0]; // 3% interest, max 40g
        game.gold = 10_000;
        game.start_wave();
        run(&mut game, 120.0);
        assert_eq!(game.gold, 10_000 + wave::clear_bonus(1) + 40 + 40);
    }

    #[test]
    fn frostbite_slows_after_thawing() {
        let mut game = Game::new();
        let origin = grass();
        game.towers.push(Tower::new(TowerKind::Frost, origin));
        game.towers[0].tiers = [0, 1, 0];
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(40.0, 0.0));
        game.update_towers(0.0);
        let e = &mut game.enemies[0];
        assert!(e.frozen() && e.slowed());
        // The slow outlasts the freeze.
        e.advance(0.7, &game.map.waypoints);
        assert!(!e.frozen() && e.slowed());
    }

    #[test]
    fn lone_frost_cannot_lock_a_runner_wave() {
        let mut game = Game::new();
        game.lives = 1000;
        // Right beside the first stretch of path.
        game.towers.push(Tower::new(
            TowerKind::Frost,
            game.map.waypoints[1] + vec2(-60.0, -45.0),
        ));
        for _ in 0..12 {
            game.spawn(EnemyKind::Runner);
            run(&mut game, 0.2);
        }
        let mut clear = Game::new();
        clear.lives = 1000;
        for _ in 0..12 {
            clear.spawn(EnemyKind::Runner);
            run(&mut clear, 0.2);
        }
        // Run both until every runner has reached the castle.
        let mut frost_time = 0.0;
        while !game.enemies.is_empty() && frost_time < 120.0 {
            run(&mut game, 0.5);
            frost_time += 0.5;
        }
        let mut clear_time = 0.0;
        while !clear.enemies.is_empty() {
            run(&mut clear, 0.5);
            clear_time += 0.5;
        }
        assert!(
            game.enemies.is_empty(),
            "runners got stuck at the frost tower"
        );
        assert!(
            frost_time < clear_time + 6.0,
            "{frost_time} vs {clear_time}"
        );
    }

    #[test]
    fn targeting_modes() {
        let mut game = Game::new();
        let origin = grass();
        let near = enemy_at(&mut game, EnemyKind::Grunt, origin + vec2(30.0, 0.0));
        let tank = enemy_at(&mut game, EnemyKind::Tank, origin + vec2(90.0, 0.0));
        let boss = enemy_at(&mut game, EnemyKind::Boss, origin + vec2(120.0, 0.0));
        let ahead = enemy_at(&mut game, EnemyKind::Runner, origin + vec2(0.0, 100.0));
        let travel = [(near, 10.0), (tank, 20.0), (boss, 5.0), (ahead, 50.0)];
        for (id, d) in travel {
            game.enemies
                .iter_mut()
                .find(|e| e.id == id)
                .unwrap()
                .traveled = d;
        }
        let pick = |mode| {
            let i = pick_target(&game.enemies, &|_: &Enemy| true, mode, origin).unwrap();
            game.enemies[i].id
        };
        assert_eq!(pick(Targeting::First), ahead);
        assert_eq!(pick(Targeting::Last), boss);
        assert_eq!(pick(Targeting::Strongest), boss);
        assert_eq!(pick(Targeting::Closest), near);
        assert_eq!(pick(Targeting::Boss), boss);
    }

    #[test]
    fn enemies_damage_the_castle() {
        let mut game = Game::new();
        game.spawn(EnemyKind::Tank);
        let e = game.enemies.last_mut().unwrap();
        let end = game.map.waypoints.len() - 1;
        e.pos = game.map.waypoints[end] - vec2(0.0, 5.0);
        e.next_waypoint = end;
        run(&mut game, 0.5);
        assert!(game.enemies.is_empty());
        assert_eq!(game.lives, START_LIVES - 2);
        assert!(game.since_base_hit < 1.0);
    }

    #[test]
    fn sandbox_is_free_and_lives_never_drop() {
        let mut game = Game::new();
        game.restart(0, true);
        game.gold = 0;
        assert!(game.place(TowerKind::Sniper, grass()));
        game.selected = Some(0);
        game.upgrade_selected(0);
        assert_eq!(game.towers[0].tiers[0], 1);
        game.sandbox_set_wave(40);
        game.sandbox_spawn(EnemyKind::Boss);
        assert!(game.enemies[0].max_hp > EnemyKind::Boss.base_hp() * 10.0);
        game.sandbox_clear();
        game.start_wave();
        assert_eq!(game.wave, 41);
        run(&mut game, 200.0);
        assert_eq!(game.lives, START_LIVES);
        assert_eq!(game.state, GameState::Playing);
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
