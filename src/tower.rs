use crate::map::TILE;
use macroquad::prelude::*;

pub const PATHS: usize = 3;
pub const TIERS: u8 = 4;
/// At most this many upgrade paths can be used on one tower...
pub const MAX_PATHS_USED: usize = 2;
/// ...and only one of them may go above this tier.
pub const MAX_SECONDARY_TIER: u8 = 2;
pub const SELL_RATIO: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerKind {
    Arrow,
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

const CANNON_UPGRADES: [[Upgrade; 4]; PATHS] = [
    [
        up("Bigger Bombs", "+8 dmg, bigger blast", 110),
        up("Heavy Shells", "+20 damage", 240),
        up("Demolition", "+40 dmg, bigger blast", 550),
        up("Earthshaker", "+120 dmg, stuns 0.6s", 1500),
    ],
    [
        up("Faster Reload", "Shoots 15% faster", 120),
        up("Twin Barrels", "Shoots 25% faster", 260),
        up("Auto-Loader", "Shoots 35% faster", 650),
        up("Bombardier", "40% faster, +1 range", 1600),
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
        up("Cold Snap", "Slows by 50%", 90),
        up("Permafrost", "Slow lasts 1s longer", 200),
        up("Arctic Wind", "Slows by 60%", 450),
        up("Absolute Zero", "Freezes enemies 1s", 1300),
    ],
    [
        up("Wider Chill", "+0.4 tile range", 90),
        up("Snowstorm", "+0.3 range, 20% faster", 230),
        up("Blizzard", "+0.6 tile range", 520),
        up("Eternal Winter", "+1 range, 30% faster", 1200),
    ],
    [
        up("Icicles", "+5 damage", 100),
        up("Brittle Ice", "Slowed take +15% dmg", 260),
        up("Ice Shards", "+15 dmg, brittle 25%", 600),
        up("Shatter", "+40 dmg, brittle 40%", 1400),
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
        up("More Seeds", "+15 gold per wave", 150),
        up("Orchard", "+30 gold per wave", 320),
        up("Plantation", "+70 gold per wave", 750),
        up("Agri-Corp", "+180 gold per wave", 1900),
    ],
    [
        up("Tax Collector", "+1 gold/kill nearby", 200),
        up("Marketplace", "+1 gold/kill, range", 380),
        up("Trade Hub", "+2 gold/kill nearby", 800),
        up("Merchant Guild", "+4 gold/kill, range", 2000),
    ],
    [
        up("Herb Garden", "+1 life per wave", 160),
        up("Clinic", "+1 life per wave", 380),
        up("Hospital", "+2 lives per wave", 800),
        up("Sanctuary", "+4 lives per wave", 1800),
    ],
];

impl TowerKind {
    pub const ALL: [TowerKind; 5] = [
        Self::Arrow,
        Self::Cannon,
        Self::Frost,
        Self::Sniper,
        Self::Farm,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Arrow => "Arrow",
            Self::Cannon => "Cannon",
            Self::Frost => "Frost",
            Self::Sniper => "Sniper",
            Self::Farm => "Farm",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Arrow => "Pierces 2 tiles behind",
            Self::Cannon => "Slow, splash damage",
            Self::Frost => "Slows all in range",
            Self::Sniper => "Global range, big hits",
            Self::Farm => "Earns gold every wave",
        }
    }

    pub fn cost(self) -> u32 {
        match self {
            Self::Arrow => 50,
            Self::Cannon => 90,
            Self::Frost => 70,
            Self::Sniper => 120,
            Self::Farm => 150,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Arrow => Color::from_rgba(230, 190, 70, 255),
            Self::Cannon => Color::from_rgba(200, 90, 60, 255),
            Self::Frost => Color::from_rgba(110, 200, 240, 255),
            Self::Sniper => Color::from_rgba(170, 120, 220, 255),
            Self::Farm => Color::from_rgba(120, 190, 80, 255),
        }
    }

    pub fn path_names(self) -> [&'static str; PATHS] {
        match self {
            Self::Arrow => ["Sharp Arrows", "Rapid Fire", "Long Shot"],
            Self::Cannon => ["Big Bombs", "Rapid Reload", "Incendiary"],
            Self::Frost => ["Deep Freeze", "Blizzard", "Shatter"],
            Self::Sniper => ["Full Metal", "Fast Firing", "Ricochet"],
            Self::Farm => ["Crops", "Market", "Clinic"],
        }
    }

    pub fn upgrades(self) -> &'static [[Upgrade; 4]; PATHS] {
        match self {
            Self::Arrow => &ARROW_UPGRADES,
            Self::Cannon => &CANNON_UPGRADES,
            Self::Frost => &FROST_UPGRADES,
            Self::Sniper => &SNIPER_UPGRADES,
            Self::Farm => &FARM_UPGRADES,
        }
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
                cooldown: 0.5,
                projectile_speed: 600.0,
                pierce: 2.0 * TILE,
                ..Stats::default()
            },
            Self::Cannon => Stats {
                damage: 22.0,
                range: 2.5 * TILE,
                cooldown: 1.3,
                projectile_speed: 320.0,
                splash: 0.9 * TILE,
                ..Stats::default()
            },
            Self::Frost => Stats {
                damage: 3.0,
                range: 1.9 * TILE,
                cooldown: 1.0,
                slow: 0.6,
                slow_time: 1.5,
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
                income: 30,
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
        s
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
        (Cannon, 1, 0) => s.cooldown *= 0.85,
        (Cannon, 1, 1) => s.cooldown *= 0.75,
        (Cannon, 1, 2) => s.cooldown *= 0.65,
        (Cannon, 1, 3) => {
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

        (Frost, 0, 0) => s.slow = 0.5,
        (Frost, 0, 1) => s.slow_time += 1.0,
        (Frost, 0, 2) => s.slow = 0.4,
        (Frost, 0, 3) => s.freeze_time = 1.0,
        (Frost, 1, 0) => s.range += 0.4 * TILE,
        (Frost, 1, 1) => {
            s.range += 0.3 * TILE;
            s.cooldown *= 0.8;
        }
        (Frost, 1, 2) => s.range += 0.6 * TILE,
        (Frost, 1, 3) => {
            s.range += TILE;
            s.cooldown *= 0.7;
        }
        (Frost, 2, 0) => s.damage += 5.0,
        (Frost, 2, 1) => s.brittle = 0.15,
        (Frost, 2, 2) => {
            s.damage += 15.0;
            s.brittle = 0.25;
        }
        (Frost, 2, 3) => {
            s.damage += 40.0;
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

        (Farm, 0, 0) => s.income += 15,
        (Farm, 0, 1) => s.income += 30,
        (Farm, 0, 2) => s.income += 70,
        (Farm, 0, 3) => s.income += 180,
        (Farm, 1, 0) => {
            s.kill_bounty += 1;
            s.bounty_range = 3.0 * TILE;
        }
        (Farm, 1, 1) => {
            s.kill_bounty += 1;
            s.bounty_range += 0.5 * TILE;
        }
        (Farm, 1, 2) => s.kill_bounty += 2,
        (Farm, 1, 3) => {
            s.kill_bounty += 4;
            s.bounty_range += TILE;
        }
        (Farm, 2, 0) => s.lives_per_wave += 1,
        (Farm, 2, 1) => s.lives_per_wave += 1,
        (Farm, 2, 2) => s.lives_per_wave += 2,
        (Farm, 2, 3) => s.lives_per_wave += 4,

        _ => {}
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub damage: f32,
    /// Range in pixels; infinite for global towers.
    pub range: f32,
    /// Seconds between shots.
    pub cooldown: f32,
    pub projectile_speed: f32,
    /// Splash radius in pixels; 0 for single target.
    pub splash: f32,
    /// Speed multiplier applied to enemies that are hit; 1 for none.
    pub slow: f32,
    pub slow_time: f32,
    pub freeze_time: f32,
    /// Extra damage fraction slowed enemies take from everything.
    pub brittle: f32,
    /// How far arrows keep flying past their target, in pixels.
    pub pierce: f32,
    pub arrows: u32,
    pub burn_dps: f32,
    pub burn_time: f32,
    pub stun_time: f32,
    /// Extra enemies a sniper shot bounces to.
    pub bounces: u32,
    pub boss_mult: f32,
    /// Gold paid when a wave is cleared.
    pub income: u32,
    pub lives_per_wave: u32,
    /// Extra gold for each enemy killed within `bounty_range`.
    pub kill_bounty: u32,
    pub bounty_range: f32,
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
            boss_mult: 1.0,
            income: 0,
            lives_per_wave: 0,
            kill_bounty: 0,
            bounty_range: 0.0,
        }
    }
}

pub struct Tower {
    pub kind: TowerKind,
    pub pos: Vec2,
    pub tiers: [u8; PATHS],
    pub cooldown: f32,
    pub angle: f32,
    /// Gold spent on this tower so far, used for the sell price.
    pub invested: u32,
    /// Time since the last shot, used for recoil and muzzle flash.
    pub since_shot: f32,
}

impl Tower {
    pub fn new(kind: TowerKind, pos: Vec2) -> Self {
        Self {
            kind,
            pos,
            tiers: [0; PATHS],
            cooldown: 0.0,
            angle: -std::f32::consts::FRAC_PI_2,
            invested: kind.cost(),
            since_shot: 10.0,
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

    pub fn sell_value(&self) -> u32 {
        (self.invested as f32 * SELL_RATIO) as u32
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
        let mut t = Tower::new(TowerKind::Arrow, Vec2::ZERO);
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
        let upgraded = TowerKind::Farm.stats([2, 0, 1]);
        assert_eq!(upgraded.income, base.income + 45);
        assert_eq!(upgraded.lives_per_wave, 1);
    }
}
