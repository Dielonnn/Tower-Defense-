//! The player's profile outside of games: XP, level and the skill tree.
//! It is saved to a small text file so progress carries between sessions.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::map::Difficulty;

/// Skill points needed per tier of the tree.
pub const TIER_COSTS: [u32; 2] = [1, 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skill {
    pub id: usize,
    pub name: &'static str,
    pub desc: &'static str,
    /// 0 for tier 1, 1 for tier 2.
    pub tier: usize,
    /// The tier 1 skill this one hangs off.
    pub parent: Option<usize>,
}

const fn skill(id: usize, name: &'static str, desc: &'static str, parent: Option<usize>) -> Skill {
    Skill {
        id,
        name,
        desc,
        tier: if parent.is_some() { 1 } else { 0 },
        parent,
    }
}

/// Branch names, in the order the tree is drawn.
pub const BRANCHES: [&str; 5] = ["Economy", "Defense", "Offense", "Utility", "Builder"];

/// Ids 0..5 are the tier 1 roots (one per branch); each root has two
/// tier 2 children.
pub const SKILLS: [Skill; 15] = [
    skill(0, "Savings", "Start with +50 gold", None),
    skill(1, "Thick Walls", "Start with +10 lives", None),
    skill(2, "Sharp Teeth", "Towers deal +5% damage", None),
    skill(3, "Lookout", "Towers get +5% range", None),
    skill(4, "Bargain", "Towers cost 5% less", None),
    skill(5, "Investor", "Start with +100 more gold", Some(0)),
    skill(6, "Bounty Hunter", "+15% gold from cats", Some(0)),
    skill(7, "Fortress", "Start with +20 more lives", Some(1)),
    skill(8, "Second Wind", "+1 life per wave cleared", Some(1)),
    skill(9, "Fury", "Towers deal +10% more damage", Some(2)),
    skill(10, "Quick Paws", "Towers attack 8% faster", Some(2)),
    skill(11, "Eagle Eyes", "Towers get +10% more range", Some(3)),
    skill(12, "Deep Chill", "Freezes last 20% longer", Some(3)),
    skill(13, "Haggler", "Upgrades cost 10% less", Some(4)),
    skill(14, "Fair Trade", "Sell towers for 85%", Some(4)),
];

impl Skill {
    pub fn cost(&self) -> u32 {
        TIER_COSTS[self.tier]
    }

    pub fn children(&self) -> impl Iterator<Item = &'static Skill> + '_ {
        SKILLS.iter().filter(move |s| s.parent == Some(self.id))
    }
}

/// In-game bonuses from the skill tree. Part of the shared game state, so in
/// multiplayer the host's skills apply to everyone.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Perks {
    pub start_gold: u32,
    pub start_lives: u32,
    pub kill_gold: f32,
    pub lives_per_wave: u32,
    pub damage: f32,
    pub attack_speed: f32,
    pub range: f32,
    pub freeze: f32,
    pub build_discount: f32,
    pub upgrade_discount: f32,
    pub sell_ratio: f32,
}

impl Default for Perks {
    fn default() -> Self {
        Self {
            start_gold: 0,
            start_lives: 0,
            kill_gold: 1.0,
            lives_per_wave: 0,
            damage: 1.0,
            attack_speed: 1.0,
            range: 1.0,
            freeze: 1.0,
            build_discount: 0.0,
            upgrade_discount: 0.0,
            sell_ratio: crate::tower::SELL_RATIO,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Profile {
    pub xp: u32,
    pub unlocked: Vec<usize>,
    path: Option<PathBuf>,
}

/// XP needed to go from `level` to `level + 1`.
pub fn xp_to_next(level: u32) -> u32 {
    100 + 40 * (level - 1)
}

/// XP for clearing a wave, scaled by map difficulty.
pub fn wave_xp(wave: u32, difficulty: Difficulty) -> u32 {
    ((10 + 2 * wave) as f32 * difficulty_mult(difficulty)).round() as u32
}

/// Bonus XP for winning a whole game.
pub fn victory_xp(difficulty: Difficulty) -> u32 {
    (250.0 * difficulty_mult(difficulty)).round() as u32
}

fn difficulty_mult(d: Difficulty) -> f32 {
    match d {
        Difficulty::Easy => 1.0,
        Difficulty::Medium => 1.3,
        Difficulty::Hard => 1.6,
    }
}

impl Profile {
    /// Loads the profile from the usual save location, or starts fresh.
    pub fn load() -> Self {
        let path = save_path();
        let mut p = std::fs::read_to_string(&path)
            .map(|text| Self::parse(&text))
            .unwrap_or_default();
        p.path = Some(path);
        p
    }

    fn parse(text: &str) -> Self {
        let mut p = Self::default();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "xp" => p.xp = value.trim().parse().unwrap_or(0),
                "skills" => {
                    p.unlocked = value
                        .split(',')
                        .filter_map(|v| v.trim().parse().ok())
                        .filter(|&id: &usize| id < SKILLS.len())
                        .collect();
                }
                _ => {}
            }
        }
        p
    }

    fn serialize(&self) -> String {
        let skills: Vec<String> = self.unlocked.iter().map(|id| id.to_string()).collect();
        format!(
            "# Rusty Tower Defense profile\nxp={}\nskills={}\n",
            self.xp,
            skills.join(",")
        )
    }

    pub fn save(&self) {
        if let Some(path) = &self.path {
            save_to(path, &self.serialize());
        }
    }

    /// Current level and XP progress within it.
    pub fn level_progress(&self) -> (u32, u32, u32) {
        let mut level = 1;
        let mut left = self.xp;
        while left >= xp_to_next(level) {
            left -= xp_to_next(level);
            level += 1;
        }
        (level, left, xp_to_next(level))
    }

    pub fn level(&self) -> u32 {
        self.level_progress().0
    }

    /// One skill point per level gained.
    pub fn points_total(&self) -> u32 {
        self.level() - 1
    }

    pub fn points_spent(&self) -> u32 {
        self.unlocked.iter().map(|&id| SKILLS[id].cost()).sum()
    }

    pub fn points_free(&self) -> u32 {
        self.points_total().saturating_sub(self.points_spent())
    }

    pub fn has(&self, id: usize) -> bool {
        self.unlocked.contains(&id)
    }

    pub fn can_unlock(&self, id: usize) -> bool {
        let s = &SKILLS[id];
        !self.has(id) && s.parent.is_none_or(|p| self.has(p)) && self.points_free() >= s.cost()
    }

    pub fn unlock(&mut self, id: usize) -> bool {
        if !self.can_unlock(id) {
            return false;
        }
        self.unlocked.push(id);
        self.save();
        true
    }

    /// Refunds every skill point.
    pub fn reset_skills(&mut self) {
        self.unlocked.clear();
        self.save();
    }

    /// Adds XP and returns how many levels were gained.
    pub fn add_xp(&mut self, xp: u32) -> u32 {
        let before = self.level();
        self.xp = self.xp.saturating_add(xp);
        self.save();
        self.level() - before
    }

    pub fn perks(&self) -> Perks {
        let mut p = Perks::default();
        for &id in &self.unlocked {
            match id {
                0 => p.start_gold += 50,
                1 => p.start_lives += 10,
                2 => p.damage += 0.05,
                3 => p.range += 0.05,
                4 => p.build_discount += 0.05,
                5 => p.start_gold += 100,
                6 => p.kill_gold += 0.15,
                7 => p.start_lives += 20,
                8 => p.lives_per_wave += 1,
                9 => p.damage += 0.10,
                10 => p.attack_speed += 0.08,
                11 => p.range += 0.10,
                12 => p.freeze += 0.20,
                13 => p.upgrade_discount += 0.10,
                14 => p.sell_ratio = 0.85,
                _ => {}
            }
        }
        p
    }
}

fn save_to(path: &Path, text: &str) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

/// %APPDATA%\RustyTowerDefense on Windows, ~/.local/share elsewhere.
fn save_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("RustyTowerDefense").join("profile.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_shape() {
        let roots: Vec<_> = SKILLS.iter().filter(|s| s.tier == 0).collect();
        assert_eq!(roots.len(), 5);
        assert!(roots.iter().all(|s| s.cost() == 1));
        let tier2: Vec<_> = SKILLS.iter().filter(|s| s.tier == 1).collect();
        assert_eq!(tier2.len(), 10);
        assert!(tier2.iter().all(|s| s.cost() == 2));
        assert!(roots.iter().all(|r| r.children().count() == 2));
    }

    #[test]
    fn levels_and_points() {
        let mut p = Profile::default();
        assert_eq!(p.level(), 1);
        assert_eq!(p.points_free(), 0);
        let gained = p.add_xp(100 + 140);
        assert_eq!(gained, 2);
        assert_eq!(p.level(), 3);
        assert_eq!(p.points_free(), 2);
    }

    #[test]
    fn tier_two_needs_its_parent_and_two_points() {
        let mut p = Profile {
            xp: 100 + 140 + 180,
            ..Default::default()
        }; // level 4: 3 points
        assert!(!p.can_unlock(5), "parent not unlocked");
        assert!(p.unlock(0));
        assert!(p.unlock(5));
        assert_eq!(p.points_free(), 0);
        assert!(!p.unlock(1), "out of points");
        assert_eq!(p.perks().start_gold, 150);
        p.reset_skills();
        assert_eq!(p.points_free(), 3);
    }

    #[test]
    fn save_and_load_round_trip() {
        let p = Profile {
            xp: 777,
            unlocked: vec![0, 5, 2],
            path: None,
        };
        let loaded = Profile::parse(&p.serialize());
        assert_eq!(loaded.xp, 777);
        assert_eq!(loaded.unlocked, vec![0, 5, 2]);
        assert_eq!(Profile::parse("garbage\nxp=abc").xp, 0);
    }

    #[test]
    fn harder_maps_give_more_xp() {
        assert!(wave_xp(10, Difficulty::Hard) > wave_xp(10, Difficulty::Easy));
        assert!(victory_xp(Difficulty::Medium) > victory_xp(Difficulty::Easy));
    }
}
