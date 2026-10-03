use std::collections::VecDeque;

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::audio::{DEFAULT_VOLUME, Sfx};
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
pub const SANDBOX_MAX_WAVE: u32 = 999;
/// Gap between parallel arrows from multi-shot upgrades.
const ARROW_SPACING: f32 = 7.0;

/// Full-screen menus. They pause single-player games while open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Settings,
    /// Tower guide, showing the tower at this index of `TowerKind::ALL`.
    Guide(usize),
}

/// Things the UI asks the main loop to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    MainMenu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Playing,
    GameOver,
    Victory,
}

/// Everything a player can do that changes the shared game. In multiplayer,
/// clients send these to the host, which applies them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Place { kind: TowerKind, pos: Vec2 },
    Upgrade { tower: u32, path: usize },
    Sell { tower: u32 },
    CycleTarget { tower: u32 },
    StartWave,
    ToggleAuto,
    SetSpeed(u32),
    SetPaused(bool),
    SandboxSpawn { kind: EnemyKind, count: u32 },
    SandboxSetWave(u32),
    SandboxClear,
    Restart,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Projectile {
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub stats: Stats,
    pub shot: Shot,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EffectShape {
    Ring(f32),
    Line(Vec2),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Effect {
    pub pos: Vec2,
    pub shape: EffectShape,
    #[serde(with = "color_serde")]
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
#[derive(Clone, Serialize, Deserialize)]
pub struct Floater {
    pub pos: Vec2,
    pub text: String,
    #[serde(with = "color_serde")]
    pub color: Color,
    pub life: f32,
}

/// Serializes macroquad colors as plain arrays.
mod color_serde {
    use macroquad::color::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(c: &Color, s: S) -> Result<S::Ok, S::Error> {
        [c.r, c.g, c.b, c.a].serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
        let [r, g, b, a] = <[f32; 4]>::deserialize(d)?;
        Ok(Color::new(r, g, b, a))
    }
}

/// The game. Fields marked `serde(skip)` are local to this player (UI state,
/// settings) and are never sent over the network.
#[derive(Serialize, Deserialize)]
pub struct Game {
    #[serde(skip)]
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
    pub time: f32,
    pub message: Option<(String, f32)>,
    /// Sandbox: everything is free, lives never drop, waves never end the game.
    pub sandbox: bool,
    /// Time since an enemy last reached the castle, for the hit flash.
    pub since_base_hit: f32,
    next_id: u32,
    next_tower_id: u32,

    // Host-only simulation state.
    #[serde(skip)]
    spawn_queue: VecDeque<Spawn>,
    #[serde(skip)]
    spawn_timer: f32,

    // Local to this player.
    #[serde(skip)]
    pub build_choice: Option<TowerKind>,
    /// Id of the selected tower.
    #[serde(skip)]
    pub selected: Option<u32>,
    #[serde(skip)]
    pub night: bool,
    #[serde(skip)]
    pub menu: Option<Menu>,
    #[serde(skip)]
    pub tower_volume: f32,
    #[serde(skip)]
    pub game_volume: f32,
    /// Which settings slider is being dragged.
    #[serde(skip)]
    pub dragging: Option<usize>,
    /// Sounds produced since the main loop last played them.
    #[serde(skip)]
    pub sounds: Vec<Sfx>,
    /// Actions waiting to be applied locally or sent to the host.
    #[serde(skip)]
    pub outbox: Vec<Action>,
    #[serde(skip)]
    pub request: Option<Request>,
    /// Part of a multiplayer party (menus don't pause the game).
    #[serde(skip)]
    pub online: bool,
    /// Connected as a client: the host owns the game.
    #[serde(skip)]
    pub is_client: bool,
    /// Party status line shown on screen, e.g. "Hosting on ...".
    #[serde(skip)]
    pub party: Option<String>,
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
            time: 0.0,
            message: None,
            sandbox: false,
            since_base_hit: 10.0,
            next_id: 0,
            next_tower_id: 0,
            spawn_queue: VecDeque::new(),
            spawn_timer: 0.0,
            build_choice: None,
            selected: None,
            night: false,
            menu: None,
            tower_volume: DEFAULT_VOLUME,
            game_volume: DEFAULT_VOLUME,
            dragging: None,
            sounds: Vec::new(),
            outbox: Vec::new(),
            request: None,
            online: false,
            is_client: false,
            party: None,
        }
    }

    /// Starts a fresh game, keeping the player's settings.
    pub fn restart(&mut self, map_index: usize, sandbox: bool) {
        let mut fresh = Self::with_map(map_index);
        fresh.speed = self.speed;
        fresh.auto_play = self.auto_play;
        fresh.sandbox = sandbox;
        fresh.keep_local(self);
        fresh.build_choice = None;
        fresh.selected = None;
        fresh.menu = None;
        *self = fresh;
        if sandbox {
            self.flash("Sandbox: everything is free");
        }
    }

    /// Copies this player's local settings and UI state from `old`.
    fn keep_local(&mut self, old: &mut Game) {
        self.build_choice = old.build_choice;
        self.night = old.night;
        self.menu = old.menu;
        self.tower_volume = old.tower_volume;
        self.game_volume = old.game_volume;
        self.dragging = old.dragging;
        self.outbox = std::mem::take(&mut old.outbox);
        self.request = old.request.take();
        self.online = old.online;
        self.is_client = old.is_client;
        self.party = old.party.take();
        // Keep the selection only if that tower still exists.
        self.selected = old
            .selected
            .filter(|id| self.towers.iter().any(|t| t.id == *id));
    }

    /// Replaces the shared game with a snapshot from the host, keeping local
    /// state. Used by multiplayer clients.
    pub fn adopt(&mut self, mut snapshot: Game) {
        snapshot.map = if snapshot.map_index == self.map_index && !self.map.waypoints.is_empty() {
            std::mem::take(&mut self.map)
        } else {
            Map::new(snapshot.map_index)
        };
        snapshot.keep_local(self);
        *self = snapshot;
    }

    pub fn max_wave(&self) -> u32 {
        if self.sandbox {
            SANDBOX_MAX_WAVE
        } else {
            MAX_WAVES
        }
    }

    pub fn tower_index(&self, id: u32) -> Option<usize> {
        self.towers.iter().position(|t| t.id == id)
    }

    pub fn selected_tower(&self) -> Option<&Tower> {
        self.selected
            .and_then(|id| self.tower_index(id))
            .map(|i| &self.towers[i])
    }

    pub fn tower_at(&self, pos: Vec2) -> Option<&Tower> {
        self.towers
            .iter()
            .find(|t| t.pos.distance(pos) <= TOWER_RADIUS + 2.0)
    }

    pub fn can_place(&self, pos: Vec2) -> bool {
        in_map(pos, TOWER_RADIUS)
            && self.map.distance_to_path(pos) >= PATH_WIDTH / 2.0 + TOWER_RADIUS - 2.0
            && !self.map.blocked(pos, TOWER_RADIUS)
            && self
                .towers
                .iter()
                .all(|t| t.pos.distance(pos) >= TOWER_RADIUS * 2.0)
    }

    pub fn can_afford(&self, cost: u32) -> bool {
        self.sandbox || self.gold >= cost
    }

    /// Queues an action. The main loop applies it, or sends it to the host.
    pub fn act(&mut self, action: Action) {
        self.outbox.push(action);
    }

    /// Applies an action from any player.
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Place { kind, pos } => {
                self.place(kind, pos);
            }
            Action::Upgrade { tower, path } => self.upgrade(tower, path),
            Action::Sell { tower } => self.sell(tower),
            Action::CycleTarget { tower } => {
                if let Some(t) = self.towers.iter_mut().find(|t| t.id == tower)
                    && t.kind.has_targeting()
                {
                    t.targeting = t.targeting.next();
                }
            }
            Action::StartWave => self.start_wave(),
            Action::ToggleAuto => {
                self.auto_play = !self.auto_play;
                self.auto_timer = 0.0;
            }
            Action::SetSpeed(s) => self.speed = s.clamp(1, 3),
            Action::SetPaused(p) => self.paused = p,
            Action::SandboxSpawn { kind, count } => {
                if self.sandbox {
                    for _ in 0..count.min(50) {
                        self.spawn(kind);
                    }
                }
            }
            Action::SandboxSetWave(w) => {
                if self.sandbox && !self.wave_active {
                    self.wave = w.min(SANDBOX_MAX_WAVE - 1);
                }
            }
            Action::SandboxClear => {
                if self.sandbox {
                    self.enemies.clear();
                    self.projectiles.clear();
                    self.spawn_queue.clear();
                    self.wave_active = false;
                }
            }
            Action::Restart => {
                let (map, sandbox) = (self.map_index, self.sandbox);
                self.restart(map, sandbox);
            }
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

    pub fn place(&mut self, kind: TowerKind, pos: Vec2) -> bool {
        if !self.can_place(pos) || !self.charge(kind.cost()) {
            return false;
        }
        self.towers.push(Tower::new(self.next_tower_id, kind, pos));
        self.next_tower_id += 1;
        self.sounds.push(Sfx::Place);
        true
    }

    pub fn upgrade(&mut self, id: u32, path: usize) {
        let Some(i) = self.tower_index(id) else {
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
        self.sounds.push(Sfx::Upgrade);
    }

    pub fn sell(&mut self, id: u32) {
        let Some(i) = self.tower_index(id) else {
            return;
        };
        let tower = self.towers.remove(i);
        let value = tower.sell_value();
        self.gold += value;
        self.sounds.push(Sfx::Sell);
        self.floaters.push(Floater {
            pos: tower.pos,
            text: format!("+{value}"),
            color: GOLD,
            life: 1.0,
        });
    }

    pub fn can_start_wave(&self) -> bool {
        self.state == GameState::Playing && !self.wave_active && self.wave < self.max_wave()
    }

    pub fn start_wave(&mut self) {
        if !self.can_start_wave() {
            return;
        }
        self.wave += 1;
        self.wave_active = true;
        self.spawn_queue = wave::generate(self.wave).into();
        self.spawn_timer = self.spawn_queue.front().map_or(0.0, |s| s.delay);
        self.sounds.push(Sfx::WaveStart);
        self.flash(&format!("Wave {}", self.wave));
    }

    fn flash(&mut self, text: &str) {
        self.message = Some((text.to_string(), 1.5));
    }

    /// Shows a short message, e.g. "A player joined".
    pub fn notify(&mut self, text: &str) {
        self.flash(text);
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

    /// Stats for tower `i` including buffs from nearby support farms.
    pub fn effective_stats(&self, i: usize) -> Stats {
        effective_stats(&self.towers, i)
    }

    /// Whether tower `i` is currently boosted by a support farm.
    pub fn is_buffed(&self, i: usize) -> bool {
        buff_for(&self.towers, i).is_some()
    }

    pub fn update(&mut self, dt: f32) {
        self.tick_message(dt);
        let menu_pauses = self.menu.is_some() && !self.online;
        if self.paused || menu_pauses || self.state != GameState::Playing {
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

    /// Multiplayer clients: move things along between host snapshots so motion
    /// looks smooth. Nothing is decided here; the next snapshot overrides it.
    pub fn client_tick(&mut self, dt: f32) {
        self.tick_message(dt);
        if self.paused || self.state != GameState::Playing {
            return;
        }
        self.time += dt;
        self.since_base_hit += dt;
        for e in &mut self.enemies {
            e.advance(dt, &self.map.waypoints);
        }
        for p in &mut self.projectiles {
            p.prev_pos = p.pos;
            let step = p.stats.projectile_speed * dt;
            match &p.shot {
                Shot::Arrow { dir, .. } => p.pos += *dir * step,
                Shot::Shell { target_pos, .. } => {
                    let to = *target_pos - p.pos;
                    if to.length() > step {
                        p.pos += to.normalize() * step;
                    }
                }
            }
        }
        for t in &mut self.towers {
            t.since_shot += dt;
        }
        self.update_effects(dt);
    }

    fn tick_message(&mut self, dt: f32) {
        if let Some((_, t)) = &mut self.message {
            *t -= dt;
            if *t <= 0.0 {
                self.message = None;
            }
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
            if earned > 0 {
                self.floaters.push(Floater {
                    pos: tower.pos - vec2(0.0, 20.0),
                    text: format!("+{earned}g"),
                    color: GOLD,
                    life: 1.5,
                });
            }
        }
        self.gold += bonus + income;
        self.sounds.push(Sfx::WaveClear);
        if income > 0 {
            self.sounds.push(Sfx::Coin);
        }
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
        self.sounds.push(Sfx::BaseHit);
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
        let all_stats: Vec<Stats> = (0..self.towers.len())
            .map(|i| effective_stats(&self.towers, i))
            .collect();
        let Self {
            towers,
            enemies,
            projectiles,
            effects,
            sounds,
            map,
            ..
        } = self;

        for (tower, &stats) in towers.iter_mut().zip(&all_stats) {
            tower.cooldown = (tower.cooldown - dt).max(0.0);
            tower.since_shot += dt;
            let in_range = |e: &Enemy| {
                e.alive() && in_map(e.pos, 0.0) && e.pos.distance(tower.pos) <= stats.range
            };

            match tower.kind {
                TowerKind::Farm => {}

                TowerKind::Frost => {
                    // Debuff tower: only pulse when something in range can be
                    // frozen, then hit everything in range.
                    let fresh = enemies.iter().any(|e| in_range(e) && e.can_freeze());
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
                    sounds.push(Sfx::Frost);
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
                    sounds.push(Sfx::Sniper);
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
                    let dir = to_aim.normalize_or_zero();

                    if tower.kind == TowerKind::Cannon {
                        sounds.push(Sfx::Cannon);
                        let muzzle = tower.pos + dir * 18.0;
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

                    // Multi-shot arrows fly side by side in a tight volley.
                    sounds.push(Sfx::Arrow);
                    let n = stats.arrows.max(1);
                    let side = vec2(-dir.y, dir.x);
                    for k in 0..n {
                        let offset = (k as f32 - (n - 1) as f32 / 2.0) * ARROW_SPACING;
                        let muzzle = tower.pos + dir * 16.0 + side * offset;
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
        let sounds = &mut self.sounds;
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
                    sounds.push(Sfx::Explosion);
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
            self.sounds.push(Sfx::EnemyDeath);
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

/// The strongest buffs any support farm gives tower `i`: speed, range, damage.
fn buff_for(towers: &[Tower], i: usize) -> Option<(f32, f32, f32)> {
    let target = &towers[i];
    if target.kind == TowerKind::Farm {
        return None;
    }
    let mut best: Option<(f32, f32, f32)> = None;
    for (j, farm) in towers.iter().enumerate() {
        if j == i || farm.kind != TowerKind::Farm {
            continue;
        }
        let s = farm.stats();
        if s.buff_radius <= 0.0 || farm.pos.distance(target.pos) > s.buff_radius {
            continue;
        }
        let b = best.get_or_insert((0.0, 0.0, 0.0));
        b.0 = b.0.max(s.buff_speed);
        b.1 = b.1.max(s.buff_range);
        b.2 = b.2.max(s.buff_damage);
    }
    best
}

fn effective_stats(towers: &[Tower], i: usize) -> Stats {
    let mut s = towers[i].stats();
    if let Some((speed, range, damage)) = buff_for(towers, i) {
        s.cooldown /= 1.0 + speed;
        if s.range.is_finite() {
            s.range *= 1.0 + range;
        }
        s.damage *= 1.0 + damage;
    }
    s
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
pub mod tests {
    use super::*;

    pub fn run(game: &mut Game, seconds: f32) {
        let dt = 1.0 / 60.0;
        for _ in 0..(seconds / dt) as usize {
            game.update(dt);
        }
    }

    /// An open grass spot on the default map near (560, 300).
    pub fn grass(game: &Game) -> Vec2 {
        let want = vec2(560.0, 300.0);
        let mut best = want;
        let mut best_d = f32::MAX;
        for i in -20..=20 {
            for j in -20..=20 {
                let p = want + vec2(i as f32, j as f32) * 4.0;
                if game.can_place(p) && p.distance(want) < best_d {
                    best = p;
                    best_d = p.distance(want);
                }
            }
        }
        best
    }

    fn add_tower(game: &mut Game, kind: TowerKind, pos: Vec2) -> u32 {
        let id = game.next_tower_id;
        game.next_tower_id += 1;
        game.towers.push(Tower::new(id, kind, pos));
        id
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
        let spot = grass(&game);
        let on_path = game.map.waypoints[1];
        assert!(!game.place(TowerKind::Arrow, on_path));
        assert!(game.place(TowerKind::Arrow, spot));
        assert!(
            !game.place(TowerKind::Arrow, spot + vec2(10.0, 0.0)),
            "overlap"
        );
        assert_eq!(game.gold, START_GOLD - TowerKind::Arrow.cost());
        let tree = game.map.props[0].pos;
        assert!(!game.can_place(tree), "scenery blocks placement");
    }

    #[test]
    fn cannot_place_without_gold() {
        let mut game = Game::new();
        game.gold = 10;
        assert!(!game.place(TowerKind::Sniper, grass(&game)));
        assert!(game.towers.is_empty());
    }

    #[test]
    fn upgrade_and_sell_by_id() {
        let mut game = Game::new();
        game.gold = 10_000;
        game.place(TowerKind::Arrow, grass(&game));
        let id = game.towers[0].id;
        game.apply(Action::Upgrade { tower: id, path: 0 });
        game.apply(Action::Upgrade { tower: id, path: 1 });
        game.apply(Action::Upgrade { tower: id, path: 2 }); // blocked: only two paths
        assert_eq!(game.towers[0].tiers, [1, 1, 0]);
        let invested = game.towers[0].invested;
        let before = game.gold;
        game.apply(Action::Sell { tower: id });
        assert!(game.towers.is_empty());
        assert_eq!(game.gold - before, (invested as f32 * 0.7) as u32);
    }

    #[test]
    fn arrows_pierce_two_tiles_behind() {
        let mut game = Game::new();
        let origin = vec2(500.0, 300.0);
        add_tower(&mut game, TowerKind::Arrow, origin);
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
        game.update_towers(0.0);
        assert_eq!(game.projectiles.len(), 1);
        for _ in 0..60 {
            game.update_projectiles(1.0 / 60.0);
        }
        let get = |id| game.enemies.iter().find(|e| e.id == id).unwrap();
        assert!(get(near).hp < get(near).max_hp);
        assert!(get(behind).hp < get(behind).max_hp);
        assert_eq!(get(far).hp, get(far).max_hp);
    }

    #[test]
    fn multi_shot_arrows_fly_parallel() {
        let mut game = Game::new();
        let origin = vec2(500.0, 300.0);
        let id = add_tower(&mut game, TowerKind::Arrow, origin);
        game.towers[0].tiers = [0, 4, 0];
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(80.0, 30.0));
        game.update_towers(0.0);
        assert_eq!(game.projectiles.len(), 5);
        let dirs: Vec<Vec2> = game
            .projectiles
            .iter()
            .map(|p| match p.shot {
                Shot::Arrow { dir, .. } => dir,
                _ => unreachable!(),
            })
            .collect();
        assert!(dirs.iter().all(|d| d.distance(dirs[0]) < 1e-5));
        // Side by side, ARROW_SPACING apart.
        let a = game.projectiles[0].pos;
        let b = game.projectiles[1].pos;
        assert!((a.distance(b) - ARROW_SPACING).abs() < 0.01);
        let _ = id;
    }

    #[test]
    fn frost_freezes_everything_in_range_but_skips_when_all_frozen() {
        let mut game = Game::new();
        let origin = grass(&game);
        add_tower(&mut game, TowerKind::Frost, origin);
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(40.0, 0.0));
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(-40.0, 20.0));
        game.update_towers(0.0);
        assert!(game.enemies.iter().all(|e| e.frozen() && e.hp < e.max_hp));

        let hp: Vec<f32> = game.enemies.iter().map(|e| e.hp).collect();
        game.towers[0].cooldown = 0.0;
        game.update_towers(0.0);
        let after: Vec<f32> = game.enemies.iter().map(|e| e.hp).collect();
        assert_eq!(hp, after);
        assert_eq!(game.towers[0].cooldown, 0.0);
    }

    #[test]
    fn thawed_enemies_cannot_be_refrozen_right_away() {
        let mut game = Game::new();
        enemy_at(&mut game, EnemyKind::Tank, vec2(500.0, 300.0));
        let wps = game.map.waypoints.clone();
        let e = &mut game.enemies[0];
        assert!(e.apply_freeze(0.6));
        e.advance(0.7, &wps);
        assert!(!e.frozen());
        assert!(e.freeze_immune());
        assert!(!e.apply_freeze(0.6), "still immune");
        e.advance(crate::enemy::FREEZE_IMMUNITY, &wps);
        assert!(e.apply_freeze(0.6));
    }

    #[test]
    fn frostbite_slows_after_thawing() {
        let mut game = Game::new();
        let origin = grass(&game);
        add_tower(&mut game, TowerKind::Frost, origin);
        game.towers[0].tiers = [0, 1, 0];
        enemy_at(&mut game, EnemyKind::Tank, origin + vec2(40.0, 0.0));
        game.update_towers(0.0);
        let wps = game.map.waypoints.clone();
        let e = &mut game.enemies[0];
        assert!(e.frozen() && e.slowed());
        e.advance(0.7, &wps);
        assert!(!e.frozen() && e.slowed());
    }

    #[test]
    fn lone_frost_cannot_lock_a_runner_wave() {
        let mut game = Game::new();
        game.lives = 1000;
        let spot = game.map.waypoints[1] + vec2(-60.0, -45.0);
        add_tower(&mut game, TowerKind::Frost, spot);
        let mut clear = Game::new();
        clear.lives = 1000;
        for _ in 0..12 {
            game.spawn(EnemyKind::Runner);
            clear.spawn(EnemyKind::Runner);
            run(&mut game, 0.2);
            run(&mut clear, 0.2);
        }
        let (mut frost_time, mut clear_time) = (0.0, 0.0);
        while !game.enemies.is_empty() && frost_time < 120.0 {
            run(&mut game, 0.5);
            frost_time += 0.5;
        }
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
    fn support_farm_buffs_nearby_towers() {
        let mut game = Game::new();
        let origin = vec2(500.0, 300.0);
        add_tower(&mut game, TowerKind::Arrow, origin);
        add_tower(&mut game, TowerKind::Arrow, origin + vec2(400.0, 0.0));
        add_tower(&mut game, TowerKind::Farm, origin + vec2(60.0, 0.0));
        let base = game.towers[0].stats();
        assert!(!game.is_buffed(0));
        game.towers[2].tiers = [0, 0, 3];
        assert!(game.is_buffed(0));
        assert!(!game.is_buffed(1), "too far away");
        assert!(!game.is_buffed(2), "farms don't buff themselves");
        let buffed = game.effective_stats(0);
        assert!(buffed.cooldown < base.cooldown);
        assert!(buffed.range > base.range);
        assert!((buffed.damage - base.damage * 1.2).abs() < 1e-4);
    }

    #[test]
    fn farm_pays_each_wave() {
        let mut game = Game::new();
        game.place(TowerKind::Farm, grass(&game));
        game.lives = 100;
        let gold = game.gold;
        game.start_wave();
        run(&mut game, 120.0);
        assert!(!game.wave_active);
        assert_eq!(game.gold, gold + wave::clear_bonus(1) + 40);
    }

    #[test]
    fn farm_bank_pays_capped_interest() {
        let mut game = Game::new();
        game.lives = 100;
        let spot = grass(&game);
        add_tower(&mut game, TowerKind::Farm, spot);
        game.towers[0].tiers = [0, 1, 0];
        game.gold = 10_000;
        game.start_wave();
        run(&mut game, 120.0);
        assert_eq!(game.gold, 10_000 + wave::clear_bonus(1) + 40 + 40);
    }

    #[test]
    fn auto_play_starts_next_wave() {
        let mut game = Game::new();
        game.lives = 1000;
        game.apply(Action::ToggleAuto);
        run(&mut game, 0.1);
        assert_eq!(game.wave, 1);
        run(&mut game, 120.0);
        assert!(game.wave >= 2);
    }

    #[test]
    fn targeting_modes() {
        let mut game = Game::new();
        let origin = vec2(500.0, 300.0);
        let near = enemy_at(&mut game, EnemyKind::Grunt, origin + vec2(30.0, 0.0));
        let tank = enemy_at(&mut game, EnemyKind::Tank, origin + vec2(90.0, 0.0));
        let boss = enemy_at(&mut game, EnemyKind::Boss, origin + vec2(120.0, 0.0));
        let ahead = enemy_at(&mut game, EnemyKind::Runner, origin + vec2(0.0, 100.0));
        for (id, d) in [(near, 10.0), (tank, 20.0), (boss, 5.0), (ahead, 50.0)] {
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
        let end = game.map.waypoints.len() - 1;
        let e = game.enemies.last_mut().unwrap();
        e.pos = game.map.waypoints[end] - vec2(0.0, 5.0);
        e.next_waypoint = end;
        run(&mut game, 0.5);
        assert!(game.enemies.is_empty());
        assert_eq!(game.lives, START_LIVES - 2);
        assert!(game.sounds.contains(&Sfx::BaseHit));
    }

    #[test]
    fn sandbox_is_free_and_endless() {
        let mut game = Game::new();
        game.restart(0, true);
        game.gold = 0;
        assert!(game.place(TowerKind::Sniper, grass(&game)));
        let id = game.towers[0].id;
        game.apply(Action::Upgrade { tower: id, path: 0 });
        assert_eq!(game.towers[0].tiers[0], 1);
        game.apply(Action::SandboxSetWave(140));
        game.apply(Action::SandboxSpawn {
            kind: EnemyKind::Boss,
            count: 1,
        });
        assert!(game.enemies[0].max_hp > EnemyKind::Boss.base_hp() * 100.0);
        game.apply(Action::SandboxClear);
        game.start_wave();
        assert_eq!(game.wave, 141, "no 30 or 99 wave cap");
        run(&mut game, 200.0);
        assert_eq!(game.lives, START_LIVES);
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn snapshots_round_trip_and_keep_local_state() {
        let mut host = Game::new();
        host.gold = 999;
        host.place(TowerKind::Cannon, grass(&host));
        host.start_wave();
        run(&mut host, 3.0);
        let bytes = bincode::serialize(&host).unwrap();

        let mut client = Game::new();
        client.night = true;
        client.tower_volume = 0.6;
        client.selected = Some(host.towers[0].id);
        client.adopt(bincode::deserialize(&bytes).unwrap());
        assert_eq!(client.gold, host.gold);
        assert_eq!(client.enemies.len(), host.enemies.len());
        assert_eq!(client.towers[0].kind, TowerKind::Cannon);
        assert!(client.night);
        assert_eq!(client.tower_volume, 0.6);
        assert_eq!(client.selected, Some(host.towers[0].id));
        assert!(!client.map.waypoints.is_empty());
    }
}
