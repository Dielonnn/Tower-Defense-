//! Sound effects, synthesized at startup so the game needs no audio files.

use std::collections::HashMap;
use std::f32::consts::TAU;

use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};
use serde::{Deserialize, Serialize};

pub const DEFAULT_VOLUME: f32 = 0.25;
const RATE: u32 = 22_050;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sfx {
    Arrow,
    Cannon,
    Explosion,
    Frost,
    Sniper,
    Gunshot,
    EnemyDeath,
    BaseHit,
    Coin,
    Upgrade,
    Place,
    Sell,
    WaveStart,
    WaveClear,
}

impl Sfx {
    const ALL: [Sfx; 14] = [
        Sfx::Arrow,
        Sfx::Cannon,
        Sfx::Explosion,
        Sfx::Frost,
        Sfx::Sniper,
        Sfx::Gunshot,
        Sfx::EnemyDeath,
        Sfx::BaseHit,
        Sfx::Coin,
        Sfx::Upgrade,
        Sfx::Place,
        Sfx::Sell,
        Sfx::WaveStart,
        Sfx::WaveClear,
    ];

    /// Tower sounds follow the tower volume slider, the rest the game slider.
    pub fn is_tower(self) -> bool {
        matches!(
            self,
            Sfx::Arrow | Sfx::Cannon | Sfx::Explosion | Sfx::Frost | Sfx::Sniper | Sfx::Gunshot
        )
    }

    /// Minimum seconds between two plays, so a dozen towers firing at once
    /// doesn't turn into noise.
    fn min_gap(self) -> f64 {
        match self {
            Sfx::Arrow | Sfx::EnemyDeath | Sfx::Gunshot => 0.06,
            Sfx::Cannon | Sfx::Explosion | Sfx::Sniper | Sfx::Frost => 0.08,
            _ => 0.03,
        }
    }

    /// Relative loudness of each sound.
    fn gain(self) -> f32 {
        match self {
            Sfx::Arrow => 0.5,
            Sfx::EnemyDeath => 0.4,
            Sfx::Explosion | Sfx::Cannon => 0.8,
            _ => 0.7,
        }
    }
}

pub struct Audio {
    sounds: HashMap<Sfx, Sound>,
    last_played: HashMap<Sfx, f64>,
}

impl Audio {
    pub async fn load() -> Self {
        let mut sounds = HashMap::new();
        for sfx in Sfx::ALL {
            if let Ok(sound) = load_sound_from_bytes(&wav(&synth(sfx))).await {
                sounds.insert(sfx, sound);
            }
        }
        Self {
            sounds,
            last_played: HashMap::new(),
        }
    }

    pub fn play(&mut self, sfx: Sfx, tower_volume: f32, game_volume: f32, now: f64) {
        let volume = if sfx.is_tower() {
            tower_volume
        } else {
            game_volume
        } * sfx.gain();
        if volume <= 0.001 {
            return;
        }
        let last = self.last_played.get(&sfx).copied().unwrap_or(f64::MIN);
        if now - last < sfx.min_gap() {
            return;
        }
        if let Some(sound) = self.sounds.get(&sfx) {
            self.last_played.insert(sfx, now);
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume,
                },
            );
        }
    }
}

/// Deterministic white noise in -1..1.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

/// Builds the samples for one sound effect.
fn synth(sfx: Sfx) -> Vec<f32> {
    let mut noise = Noise(0x1234_5678 ^ sfx as u32);
    let mut out = Vec::new();
    // `f(t, dur)` returns the sample at time t.
    let mut render = |dur: f32, f: &mut dyn FnMut(f32) -> f32| {
        let n = (dur * RATE as f32) as usize;
        for i in 0..n {
            out.push(f(i as f32 / RATE as f32));
        }
    };
    let sine = |freq: f32, t: f32| (TAU * freq * t).sin();

    match sfx {
        Sfx::Arrow => {
            // Bowstring twang: falling pitch with a breathy edge.
            render(0.14, &mut |t| {
                let f = 700.0 - 2500.0 * t;
                let env = (-t * 30.0).exp();
                (sine(f, t) * 0.7 + noise.next() * 0.3) * env
            });
        }
        Sfx::Cannon => {
            let mut low = 0.0;
            render(0.35, &mut |t| {
                low += (noise.next() - low) * 0.08;
                let env = (-t * 9.0).exp();
                (low * 2.5 + sine(70.0 - 40.0 * t, t) * 0.6) * env
            });
        }
        Sfx::Explosion => {
            let mut low = 0.0;
            render(0.5, &mut |t| {
                low += (noise.next() - low) * 0.15;
                let env = (-t * 6.0).exp();
                (low * 2.2 + sine(50.0, t) * 0.4) * env
            });
        }
        Sfx::Frost => {
            // Glassy chime.
            render(0.5, &mut |t| {
                let env = (-t * 7.0).exp();
                let shimmer = 1.0 + 0.3 * sine(12.0, t);
                (sine(1320.0, t) * 0.5 + sine(1980.0, t) * 0.3 + sine(2640.0, t) * 0.2)
                    * env
                    * shimmer
            });
        }
        Sfx::Sniper => {
            render(0.25, &mut |t| {
                let crack = noise.next() * (-t * 60.0).exp();
                let thump = sine(140.0, t) * (-t * 18.0).exp();
                crack * 0.9 + thump * 0.6
            });
        }
        Sfx::Gunshot => {
            // Short sharp pop with a little body.
            let mut low = 0.0;
            render(0.16, &mut |t| {
                low += (noise.next() - low) * 0.3;
                let crack = noise.next() * (-t * 80.0).exp();
                let body = low * 1.6 * (-t * 25.0).exp();
                crack * 0.6 + body + sine(220.0, t) * 0.3 * (-t * 30.0).exp()
            });
        }
        Sfx::EnemyDeath => {
            render(0.09, &mut |t| {
                let f = 500.0 + 6000.0 * t;
                sine(f, t) * (-t * 35.0).exp()
            });
        }
        Sfx::BaseHit => {
            let mut low = 0.0;
            render(0.4, &mut |t| {
                low += (noise.next() - low) * 0.05;
                let env = (-t * 8.0).exp();
                (sine(90.0, t) * 0.8 + low * 2.0) * env
            });
        }
        Sfx::Coin => {
            render(0.28, &mut |t| {
                let f = if t < 0.07 { 988.0 } else { 1319.0 };
                let square = sine(f, t).signum() * 0.4 + sine(f, t) * 0.4;
                square * (-(t - 0.07).max(0.0) * 12.0).exp()
            });
        }
        Sfx::Upgrade => {
            render(0.4, &mut |t| {
                let f = match (t / 0.1) as usize {
                    0 => 523.0,
                    1 => 659.0,
                    _ => 784.0,
                };
                sine(f, t) * (-(t % 0.1) * 8.0).exp() * 0.8
            });
        }
        Sfx::Place => {
            render(0.14, &mut |t| {
                (sine(220.0 - 400.0 * t, t) * 0.8 + noise.next() * 0.2) * (-t * 28.0).exp()
            });
        }
        Sfx::Sell => {
            render(0.25, &mut |t| {
                let f = if t < 0.1 { 784.0 } else { 523.0 };
                sine(f, t) * (-(t % 0.1) * 10.0).exp() * 0.7
            });
        }
        Sfx::WaveStart => {
            // Horn: a few harmonics with a slow attack.
            render(0.7, &mut |t| {
                let env = (t * 12.0).min(1.0) * (-(t - 0.1).max(0.0) * 3.5).exp();
                let f = 220.0;
                (sine(f, t) * 0.5 + sine(2.0 * f, t) * 0.3 + sine(3.0 * f, t) * 0.15) * env
            });
        }
        Sfx::WaveClear => {
            render(0.6, &mut |t| {
                let f = match (t / 0.12) as usize {
                    0 => 523.0,
                    1 => 659.0,
                    2 => 784.0,
                    _ => 1047.0,
                };
                sine(f, t) * (-(t % 0.12) * 6.0).exp() * 0.7
            });
        }
    }
    out
}

/// Wraps mono samples in a 16-bit PCM WAV file.
fn wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_synthesizes_to_a_valid_wav() {
        for sfx in Sfx::ALL {
            let samples = synth(sfx);
            assert!(!samples.is_empty());
            assert!(samples.iter().all(|s| s.is_finite()));
            let bytes = wav(&samples);
            assert_eq!(&bytes[..4], b"RIFF");
            assert_eq!(bytes.len(), 44 + samples.len() * 2);
        }
    }
}
