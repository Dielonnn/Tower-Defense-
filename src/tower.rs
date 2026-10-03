use crate::map::TILE;
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

pub const PATHS: usize = 3;
pub const TIERS: u8 = 4;
/// At most this many upgrade paths can be used on one tower...
pub const MAX_PATHS_USED: usize = 2;
/// ...and only one of them may go above this tier.
pub const MAX_SECONDARY_TIER: u8 = 2;
pub const SELL_RATIO: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TowerKind {
    Arrow,
    Mercenary,
    Cannon,
    Frost,
    Sniper,
    Farm,
}

pub struct Upgrade {
    pub name: &'static str,
    pub desc: &'static str,
    pub cost: u32,
}

const fn up(name: &'static str, desc: &'static str, cost: u32) -> Upgrade {
    Upgrade { name, desc, cost }
}

const ARROW_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Sharp Tips", "+4 damage", 70),
        up("Barbed Arrows", "+6 damage", 160),
        up("Steel Heads", "+14 damage", 380),
        up("Ballista", "+35 dmg, +2 tile pierce", 1100),
    ],
    [
        up("Quick Draw", "Shoots 20% faster", 90),
        up("Double Shot", "Fires 2 arrows", 220),
        up("Triple Shot", "3 arrows, 15% faster", 480),
        up("Arrow Storm", "5 arrows, 40% faster", 1400),
    ],
    [
        up("Long Bow", "+0.6 tile range", 60),
        up("Eagle Eye", "+0.6 range, +1 pierce", 150),
        up("Far Flight", "+2 pierce, +5 damage", 360),
        up("Hawk Eye", "+1.5 range, +10 damage", 900),
    ],
];

const MERCENARY_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Long Rifle", "+0.8 range, +6 damage", 90),
        up("Scoped Rifle", "+12 dmg, +0.6 range", 220),
        up("AP Rounds", "+25 dmg, pierces 2 enemies", 520),
        up("Deadeye", "+70 dmg, 2x vs bosses", 1400),
    ],
    [
        up("Sidearm", "Shoots 30% faster", 80),
        up("Dual Pistols", "Shoots 2 targets at once", 240),
        up("Quickdraw", "Shoots 40% faster", 520),
        up("Gunslinger", "4 targets, +10 damage", 1300),
    ],
    [
        up("Drum Mag", "Shoots 25% faster", 100),
        up("Light MG", "2.5x fire rate, -40% dmg", 300),
        up("Suppressing Fire", "Hits slow enemies 25%", 650),
        up("Heavy Gunner", "2x fire rate, +8 damage", 1500),
    ],
];

const CANNON_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Bigger Bombs", "+8 dmg, bigger blast", 110),
        up("Heavy Shells", "+20 damage", 240),
        up("Demolition", "+40 dmg, bigger blast", 550),
        up("Earthshaker", "+120 dmg, stuns 0.6s", 1500),
    ],
    [
        up("Faster Reload", "15% faster, +25% vs heavy", 120),
        up("Twin Barrels", "25% faster, +50% vs heavy", 260),
        up("Auto-Loader", "35% faster, 2x vs heavy", 650),
        up("Bombardier", "40% faster, 3x vs heavy", 1600),
    ],
    [
        up("Fire Bombs", "Burns 5/s for 3s", 120),
        up("Napalm", "Burns 12/s", 280),
        up("Hellfire", "Burns 28/s, bigger blast", 650),
        up("Inferno", "Burns 70/s for 5s", 1500),
    ],
];

const FROST_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Cold Snap", "Freezes 0.3s longer", 100),
        up("Permafrost", "Pulses 25% faster", 240),
        up("Glacier", "Freezes 0.5s longer", 550),
        up("Absolute Zero", "+0.5s freeze, 20% faster", 1400),
    ],
    [
        up("Frostbite", "Thawed enemies slowed 30%", 120),
        up("Chill Wind", "Slow 45%, +0.4 range", 260),
        up("Blizzard", "Slow 55%, lasts 3s", 550),
        up("Eternal Winter", "Slow 65%, +1 range", 1300),
    ],
    [
        up("Icicles", "+6 damage", 100),
        up("Brittle Ice", "Hit enemies take +15%", 260),
        up("Ice Shards", "+18 dmg, brittle 25%", 600),
        up("Shatter", "+50 dmg, brittle 40%", 1400),
    ],
];

const SNIPER_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Full Metal Jacket", "+40 damage", 160),
        up("Large Calibre", "+80 damage", 380),
        up("Deadly Precision", "+200 damage", 950),
        up("Executioner", "+300 dmg, 3x vs bosses", 2400),
    ],
    [
        up("Faster Firing", "Shoots 20% faster", 200),
        up("Even Faster", "Shoots 25% faster", 420),
        up("Semi-Auto", "Shoots twice as fast", 1100),
        up("Full Auto", "Twice as fast again", 2800),
    ],
    [
        up("Ricochet", "Bounces to 1 more enemy", 220),
        up("Double Ricochet", "Bounces to 1 more", 450),
        up("Supply Drop", "+100 gold per wave", 1300),
        up("Elite Supply", "+150 gold, +2 bounces", 3200),
    ],
];

const FARM_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("More Seeds", "+20 gold per wave", 150),
        up("Orchard", "+40 gold per wave", 320),
        up("Plantation", "+90 gold per wave", 750),
        up("Agri-Corp", "+220 gold per wave", 1900),
    ],
    [
        up("Savings", "3% interest, max 40g", 180),
        up("Bank", "5% interest, max 100g", 400),
        up("Investments", "8% interest, max 200g", 850),
        up("Wall Street", "12% interest, max 500g", 2100),
    ],
    [
        up("War Drums", "Nearby towers +10% speed", 220),
        up("Watchtower", "Nearby +10% range, wider", 450),
        up("Armory", "Nearby +20% damage, wider", 900),
        up("Command Post", "+15% on all buffs, wider", 2200),
    ],
];

impl TowerKind {
    pub const ALL: [TowerKind; 6] = [
        Self::Arrow,
        Self::Mercenary,
        Self::Cannon,
        Self::Frost,
        Self::Sniper,
        Self::Farm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Arrow => "Arrow",
            Self::Mercenary => "Mercenary",
            Self::Cannon => "Cannon",
            Self::Frost => "Frost",
            Self::Sniper => "Sniper",
            Self::Farm => "Farm",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Arrow => "Capy archer, piercing bolts",
            Self::Mercenary => "Capy gun for hire",
            Self::Cannon => "Capy cannon, big splash",
            Self::Frost => "Spa capy, freezes all",
            Self::Sniper => "Capy sniper, global range",
            Self::Farm => "Capy farmer, earns gold",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Self::Arrow => 50,
            Self::Mercenary => 60,
            Self::Cannon => 90,
            Self::Frost => 70,
            Self::Sniper => 120,
            Self::Farm => 150,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Arrow => Color::from_rgba(230, 190, 70, 255),
            Self::Mercenary => Color::from_rgba(105, 120, 65, 255),
            Self::Cannon => Color::from_rgba(200, 90, 60, 255),
            Self::Frost => Color::from_rgba(110, 200, 240, 255),
            Self::Sniper => Color::from_rgba(170, 120, 220, 255),
            Self::Farm => Color::from_rgba(120, 190, 80, 255),
        }
    }

    pub fn path_names(self) -> [&'static str; PATHS] {
        match self {
            Self::Arrow => ["Sharp Arrows", "Rapid Fire", "Long Shot"],
            Self::Mercenary => ["Rifle", "Pistols", "Gunner"],
            Self::Cannon => ["Big Bombs", "Rapid Reload", "Incendiary"],
            Self::Frost => ["Deep Freeze", "Frostbite", "Shatter"],
            Self::Sniper => ["Full Metal", "Fast Firing", "Ricochet"],
            Self::Farm => ["Crops", "Bank", "Support"],
        }
    }

    pub fn upgrades(self) -> &'static [[Upgrade; 4]; PATHS] {
        match self {
            Self::Arrow => &ARROW_UPGRADES,
            Self::Mercenary => &MERCENARY_UPGRADES,
            Self::Cannon => &CANNON_UPGRADES,
            Self::Frost => &FROST_UPGRADES,
            Self::Sniper => &SNIPER_UPGRADES,
            Self::Farm => &FARM_UPGRADES,
        }
    }

    /// Position in `ALL`.
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&k| k == self).unwrap_or(0)
    }

    /// Whether the tower picks a single target (and so has a targeting mode).
    pub fn has_targeting(self) -> bool {
        matches!(
            self,
            Self::Arrow | Self::Mercenary | Self::Cannon | Self::Sniper
        )
    }

    /// Cheap, reliable towers to open a game with.
    pub fn is_starter(self) -> bool {
        matches!(self, Self::Arrow | Self::Mercenary)
    }

    /// Debuff towers only fire when an enemy in range lacks their debuff.
    pub fn is_debuff(self) -> bool {
        self == Self::Frost
    }

    fn base_stats(self) -> Stats {
        match self {
            Self::Arrow => Stats {
                damage: 9.0,
                range: 2.8 * TILE,
                cooldown: 0.7,
                projectile_speed: 600.0,
                pierce: 2.0 * TILE,
                ..Stats::default()
            },
            Self::Mercenary => Stats {
                damage: 12.0,
                range: 2.8 * TILE,
                cooldown: 0.85,
                ..Stats::default()
            },
            Self::Cannon => Stats {
                damage: 22.0,
                range: 2.5 * TILE,
                cooldown: 1.3,
                projectile_speed: 320.0,
                splash: 1.2 * TILE,
                ..Stats::default()
            },
            Self::Frost => Stats {
                damage: 2.0,
                range: 1.9 * TILE,
                // Slow enough that one frost can't freeze-lock a whole wave.
                cooldown: 2.2,
                freeze_time: 0.6,
                ..Stats::default()
            },
            Self::Sniper => Stats {
                damage: 60.0,
                range: f32::INFINITY,
                // Global range comes at the cost of firing 15% slower.
                cooldown: 1.8 / 0.85,
                ..Stats::default()
            },
            Self::Farm => Stats {
                income: 40,
                ..Stats::default()
            },
        }
    }

    pub fn stats(self, tiers: [u8; PATHS]) -> Stats {
        let mut s = self.base_stats();
        for (path, &tier) in tiers.iter().enumerate() {
            for t in 0..tier {
                apply_upgrade(self, path, t, &mut s);
            }
        }
        let scale = self.damage_scale();
        s.damage *= scale;
        s.burn_dps *= scale;
        s
    }

    /// Overall damage tuning per tower, on top of the numbers above. The
    /// starter towers and cannon are boosted so they can carry a game.
    fn damage_scale(self) -> f32 {
        match self {
            // Tuned with a bot: mixes of these three now clear wave 30 on
            // Sketch in most builds it tries.
            Self::Arrow | Self::Mercenary | Self::Cannon => 1.5,
            _ => 1.0,
        }
    }
}

/// Applies the effect of one upgrade (path and tier are 0-based).
fn apply_upgrade(kind: TowerKind, path: usize, tier: u8, s: &mut Stats) {
    use TowerKind::*;
    match (kind, path, tier) {
        (Arrow, 0, 0) => s.damage += 4.0,
        (Arrow, 0, 1) => s.damage += 6.0,
        (Arrow, 0, 2) => s.damage += 14.0,
        (Arrow, 0, 3) => {
            s.damage += 35.0;
            s.pierce += 2.0 * TILE;
            s.projectile_speed += 200.0;
        }
        (Arrow, 1, 0) => s.cooldown *= 0.8,
        (Arrow, 1, 1) => s.arrows = 2,
        (Arrow, 1, 2) => {
            s.arrows = 3;
            s.cooldown *= 0.85;
        }
        (Arrow, 1, 3) => {
            s.arrows = 5;
            s.cooldown *= 0.6;
        }
        (Arrow, 2, 0) => s.range += 0.6 * TILE,
        (Arrow, 2, 1) => {
            s.range += 0.6 * TILE;
            s.pierce += TILE;
        }
        (Arrow, 2, 2) => {
            s.pierce += 2.0 * TILE;
            s.damage += 5.0;
        }
        (Arrow, 2, 3) => {
            s.range += 1.5 * TILE;
            s.damage += 10.0;
        }

        (Cannon, 0, 0) => {
            s.damage += 8.0;
            s.splash += 0.3 * TILE;
        }
        (Cannon, 0, 1) => s.damage += 20.0,
        (Cannon, 0, 2) => {
            s.damage += 40.0;
            s.splash += 0.4 * TILE;
        }
        (Cannon, 0, 3) => {
            s.damage += 120.0;
            s.stun_time = 0.6;
        }
        // Rapid Reload also hits tanks and bosses ("heavy" enemies) harder.
        (Cannon, 1, 0) => {
            s.cooldown *= 0.85;
            s.heavy_mult = 1.25;
        }
        (Cannon, 1, 1) => {
            s.cooldown *= 0.75;
            s.heavy_mult = 1.5;
        }
        (Cannon, 1, 2) => {
            s.cooldown *= 0.65;
            s.heavy_mult = 2.0;
        }
        (Cannon, 1, 3) => {
            s.heavy_mult = 3.0;
            s.cooldown *= 0.6;
            s.range += TILE;
        }
        (Cannon, 2, 0) => {
            s.burn_dps = 5.0;
            s.burn_time = 3.0;
        }
        (Cannon, 2, 1) => s.burn_dps = 12.0,
        (Cannon, 2, 2) => {
            s.burn_dps = 28.0;
            s.splash += 0.3 * TILE;
        }
        (Cannon, 2, 3) => {
            s.burn_dps = 70.0;
            s.burn_time = 5.0;
        }

        (Frost, 0, 0) => s.freeze_time += 0.3,
        (Frost, 0, 1) => s.cooldown *= 0.75,
        (Frost, 0, 2) => s.freeze_time += 0.5,
        (Frost, 0, 3) => {
            s.freeze_time += 0.5;
            s.cooldown *= 0.8;
        }
        (Frost, 1, 0) => {
            s.slow = 0.7;
            s.slow_time = 2.0;
        }
        (Frost, 1, 1) => {
            s.slow = 0.55;
            s.range += 0.4 * TILE;
        }
        (Frost, 1, 2) => {
            s.slow = 0.45;
            s.slow_time = 3.0;
        }
        (Frost, 1, 3) => {
            s.slow = 0.35;
            s.range += TILE;
        }
        (Frost, 2, 0) => s.damage += 6.0,
        (Frost, 2, 1) => s.brittle = 0.15,
        (Frost, 2, 2) => {
            s.damage += 18.0;
            s.brittle = 0.25;
        }
        (Frost, 2, 3) => {
            s.damage += 50.0;
            s.brittle = 0.4;
        }

        (Sniper, 0, 0) => s.damage += 40.0,
        (Sniper, 0, 1) => s.damage += 80.0,
        (Sniper, 0, 2) => s.damage += 200.0,
        (Sniper, 0, 3) => {
            s.damage += 300.0;
            s.boss_mult = 3.0;
        }
        (Sniper, 1, 0) => s.cooldown *= 0.8,
        (Sniper, 1, 1) => s.cooldown *= 0.75,
        (Sniper, 1, 2) => s.cooldown *= 0.5,
        (Sniper, 1, 3) => s.cooldown *= 0.5,
        (Sniper, 2, 0) => s.bounces += 1,
        (Sniper, 2, 1) => s.bounces += 1,
        (Sniper, 2, 2) => s.income += 100,
        (Sniper, 2, 3) => {
            s.income += 150;
            s.bounces += 2;
        }

        (Farm, 0, 0) => s.income += 20,
        (Farm, 0, 1) => s.income += 40,
        (Farm, 0, 2) => s.income += 90,
        (Farm, 0, 3) => s.income += 220,
        (Farm, 1, 0) => (s.interest, s.interest_cap) = (0.03, 40),
        (Farm, 1, 1) => (s.interest, s.interest_cap) = (0.05, 100),
        (Farm, 1, 2) => (s.interest, s.interest_cap) = (0.08, 200),
        (Farm, 1, 3) => (s.interest, s.interest_cap) = (0.12, 500),
        (Farm, 2, 0) => {
            // Same reach as a base frost tower; every tier widens it a little.
            s.buff_radius = TowerKind::Frost.base_stats().range;
            s.buff_speed = 0.10;
        }
        (Farm, 2, 1) => {
            s.buff_range = 0.10;
            s.buff_radius += 0.25 * TILE;
        }
        (Farm, 2, 2) => {
            s.buff_damage = 0.20;
            s.buff_radius += 0.25 * TILE;
        }
        (Farm, 2, 3) => {
            s.buff_speed += 0.15;
            s.buff_range += 0.15;
            s.buff_damage += 0.15;
            s.buff_radius += 0.4 * TILE;
        }

        (Mercenary, 0, 0) => {
            s.range += 0.8 * TILE;
            s.damage += 6.0;
        }
        (Mercenary, 0, 1) => {
            s.damage += 12.0;
            s.range += 0.6 * TILE;
        }
        (Mercenary, 0, 2) => {
            s.damage += 25.0;
            s.penetrate = 2;
        }
        (Mercenary, 0, 3) => {
            s.damage += 70.0;
            s.boss_mult = 2.0;
        }
        (Mercenary, 1, 0) => s.cooldown /= 1.3,
        (Mercenary, 1, 1) => s.targets = 2,
        (Mercenary, 1, 2) => s.cooldown /= 1.4,
        (Mercenary, 1, 3) => {
            s.targets = 4;
            s.damage += 10.0;
        }
        (Mercenary, 2, 0) => s.cooldown /= 1.25,
        (Mercenary, 2, 1) => {
            s.cooldown /= 2.5;
            s.damage *= 0.6;
        }
        (Mercenary, 2, 2) => {
            s.slow = 0.75;
            s.slow_time = 0.8;
        }
        (Mercenary, 2, 3) => {
            s.cooldown /= 2.0;
            s.damage += 8.0;
        }

        _ => {}
    }
}

/// Which enemy in range a tower shoots at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Targeting {
    /// Furthest along the path.
    First,
    /// Least far along the path.
    Last,
    /// Most health remaining.
    Strongest,
    /// Nearest to the tower.
    Closest,
    /// Bosses, then tanks, then the rest; furthest along within each.
    Boss,
}

impl Targeting {
    pub const ALL: [Targeting; 5] = [
        Self::First,
        Self::Last,
        Self::Strongest,
        Self::Closest,
        Self::Boss,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::First => "First",
            Self::Last => "Last",
            Self::Strongest => "Strongest",
            Self::Closest => "Closest",
            Self::Boss => "Boss",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub damage: f32,
    /// Range in pixels; infinite for global towers.
    pub range: f32,
    /// Seconds between shots.
    pub cooldown: f32,
    pub projectile_speed: f32,
    /// Splash radius in pixels; 0 for single target.
    pub splash: f32,
    /// Speed multiplier applied to enemies after they thaw; 1 for none.
    pub slow: f32,
    pub slow_time: f32,
    pub freeze_time: f32,
    /// Extra damage fraction enemies take from everything after being hit.
    pub brittle: f32,
    /// How far arrows keep flying past their target, in pixels.
    pub pierce: f32,
    pub arrows: u32,
    pub burn_dps: f32,
    pub burn_time: f32,
    pub stun_time: f32,
    /// Extra enemies a sniper shot bounces to.
    pub bounces: u32,
    /// Separate enemies a Mercenary shoots per volley.
    pub targets: u32,
    /// Extra enemies a Mercenary bullet passes through.
    pub penetrate: u32,
    pub boss_mult: f32,
    /// Damage multiplier against tanks and bosses.
    pub heavy_mult: f32,
    /// Gold paid when a wave is cleared.
    pub income: u32,
    /// Support farms boost towers within `buff_radius` by these fractions.
    pub buff_radius: f32,
    pub buff_speed: f32,
    pub buff_range: f32,
    pub buff_damage: f32,
    /// Fraction of your gold paid as interest each wave, up to `interest_cap`.
    pub interest: f32,
    pub interest_cap: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            damage: 0.0,
            range: 0.0,
            cooldown: 1.0,
            projectile_speed: 400.0,
            splash: 0.0,
            slow: 1.0,
            slow_time: 0.0,
            freeze_time: 0.0,
            brittle: 0.0,
            pierce: 0.0,
            arrows: 1,
            burn_dps: 0.0,
            burn_time: 0.0,
            stun_time: 0.0,
            bounces: 0,
            targets: 1,
            penetrate: 0,
            boss_mult: 1.0,
            heavy_mult: 1.0,
            income: 0,
            buff_radius: 0.0,
            buff_speed: 0.0,
            buff_range: 0.0,
            buff_damage: 0.0,
            interest: 0.0,
            interest_cap: 0,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tower {
    /// Stable id so players can refer to the same tower over the network.
    pub id: u32,
    pub kind: TowerKind,
    pub pos: Vec2,
    pub tiers: [u8; PATHS],
    pub cooldown: f32,
    pub angle: f32,
    /// Gold spent on this tower so far, used for the sell price.
    pub invested: u32,
    /// Time since the last shot, used for recoil and muzzle flash.
    pub since_shot: f32,
    pub targeting: Targeting,
    /// Total damage this tower has dealt.
    #[serde(default)]
    pub damage_dealt: f32,
}

impl Tower {
    pub fn new(id: u32, kind: TowerKind, pos: Vec2) -> Self {
        Self {
            id,
            kind,
            pos,
            tiers: [0; PATHS],
            cooldown: 0.0,
            angle: -std::f32::consts::FRAC_PI_2,
            invested: kind.cost(),
            since_shot: 10.0,
            targeting: Targeting::First,
            damage_dealt: 0.0,
        }
    }

    pub fn stats(&self) -> Stats {
        self.kind.stats(self.tiers)
    }

    pub fn next_upgrade(&self, path: usize) -> Option<&'static Upgrade> {
        self.kind.upgrades()[path].get(self.tiers[path] as usize)
    }

    /// Whether the upgrade rules allow buying the next tier on `path`.
    pub fn path_open(&self, path: usize) -> bool {
        let tier = self.tiers[path];
        if tier >= TIERS {
            return false;
        }
        let others = || (0..PATHS).filter(move |&p| p != path);
        let used = others().filter(|&p| self.tiers[p] > 0).count();
        if tier == 0 && used >= MAX_PATHS_USED {
            return false;
        }
        let other_high = others().any(|p| self.tiers[p] > MAX_SECONDARY_TIER);
        !(tier + 1 > MAX_SECONDARY_TIER && other_high)
    }

    pub fn upgrade_cost(&self, path: usize) -> Option<u32> {
        if !self.path_open(path) {
            return None;
        }
        self.next_upgrade(path).map(|u| u.cost)
    }

    /// Upgrade summary such as "2-0-1".
    pub fn tier_label(&self) -> String {
        let [a, b, c] = self.tiers;
        format!("{a}-{b}-{c}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upgrade_path_rules() {
        let mut t = Tower::new(0, TowerKind::Arrow, Vec2::ZERO);
        t.tiers = [1, 1, 0];
        assert!(!t.path_open(2), "a third path is not allowed");
        assert!(t.path_open(0));
        t.tiers = [3, 2, 0];
        assert!(!t.path_open(1), "only one path may pass tier 2");
        assert!(t.path_open(0));
        t.tiers = [4, 2, 0];
        assert!(!t.path_open(0), "tier 4 is the max");
        assert_eq!(t.upgrade_cost(0), None);
    }

    #[test]
    fn sniper_is_global_and_slower() {
        let s = TowerKind::Sniper.stats([0; PATHS]);
        assert!(s.range.is_infinite());
        assert!((s.cooldown - 1.8 / 0.85).abs() < 1e-5);
    }

    #[test]
    fn upgrades_stack() {
        let base = TowerKind::Farm.stats([0; PATHS]);
        let upgraded = TowerKind::Farm.stats([2, 0, 4]);
        assert_eq!(upgraded.income, base.income + 60);
        assert!((upgraded.buff_speed - 0.25).abs() < 1e-5);
        // Each support tier widens the farm's reach beyond the frost-sized base.
        let frost = TowerKind::Frost.stats([0; PATHS]).range;
        assert_eq!(TowerKind::Farm.stats([0, 0, 1]).buff_radius, frost);
        assert!(TowerKind::Farm.stats([0, 0, 2]).buff_radius > frost);
        assert!((upgraded.buff_radius - (frost + 0.9 * TILE)).abs() < 1e-3);
    }
}
