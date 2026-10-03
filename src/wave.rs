use crate::enemy::EnemyKind;

pub const MAX_WAVES: u32 = 30;

pub struct Spawn {
    /// Seconds to wait after the previous spawn.
    pub delay: f32,
    pub kind: EnemyKind,
}

pub fn hp_multiplier(wave: u32) -> f32 {
    let w = wave.saturating_sub(1) as f32;
    1.0 + 0.3 * w + 0.04 * w * w
}

pub fn clear_bonus(wave: u32) -> u32 {
    20 + 5 * wave
}

/// Builds the spawn list for a wave (1-based). Waves grow in size and mix in
/// faster and tougher enemies as they go, with bosses every fifth wave.
pub fn generate(wave: u32) -> Vec<Spawn> {
    let gap = (0.9 - wave as f32 * 0.02).max(0.4);
    let grunts = 5 + wave * 2;
    let runners = if wave >= 3 {
        ((wave - 2) * 2).min(24)
    } else {
        0
    };
    let tanks = if wave >= 5 {
        ((wave - 3) / 2).min(12)
    } else {
        0
    };
    let bosses = if wave.is_multiple_of(5) { wave / 5 } else { 0 };

    let mut spawns = Vec::new();
    let mut push = |kind, delay| spawns.push(Spawn { delay, kind });

    for i in 0..grunts {
        push(EnemyKind::Grunt, if i == 0 { 0.5 } else { gap });
        // Break the grunt line up with a burst of runners midway through.
        if i == grunts / 2 {
            for _ in 0..runners {
                push(EnemyKind::Runner, gap * 0.4);
            }
        }
    }
    for _ in 0..tanks {
        push(EnemyKind::Tank, gap * 1.6);
    }
    for _ in 0..bosses {
        push(EnemyKind::Boss, 2.5);
    }
    spawns
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(wave: u32, kind: EnemyKind) -> usize {
        generate(wave).iter().filter(|s| s.kind == kind).count()
    }

    #[test]
    fn early_waves_are_grunts_only() {
        assert_eq!(count(1, EnemyKind::Grunt), 7);
        assert_eq!(count(1, EnemyKind::Runner), 0);
        assert_eq!(count(1, EnemyKind::Tank), 0);
    }

    #[test]
    fn bosses_every_fifth_wave() {
        assert_eq!(count(4, EnemyKind::Boss), 0);
        assert_eq!(count(5, EnemyKind::Boss), 1);
        assert_eq!(count(30, EnemyKind::Boss), 6);
    }

    #[test]
    fn waves_get_harder() {
        assert!(hp_multiplier(10) > hp_multiplier(5));
        assert!(generate(10).len() > generate(5).len());
    }
}
