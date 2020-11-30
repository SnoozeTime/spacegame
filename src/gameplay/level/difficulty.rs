use crate::core::random::RandomGenerator;
use crate::prefab::enemies::{ENEMY_STR_1, ENEMY_STR_2, ENEMY_STR_3};
use rand::seq::SliceRandom;
use serde_derive::{Deserialize, Serialize};

/// What kind of enemy should we spawn in infinite wave.
#[derive(Debug, Copy, Clone)]
pub struct WaveDifficulty {
    level_1_enemies: f32,
    level_2_enemies: f32,
    level_3_enemies: f32,
}

impl Default for WaveDifficulty {
    fn default() -> Self {
        Self {
            level_1_enemies: 1.0,
            level_2_enemies: -1.0,
            level_3_enemies: -5.0,
        }
    }
}

impl WaveDifficulty {
    pub fn pick_prefabs(&self, random: &mut RandomGenerator) -> Vec<String> {
        let lvl1 = self.level_1_enemies.max(0.0).floor() as usize;
        let lvl2 = self.level_2_enemies.max(0.0).floor() as usize;
        let lvl3 = self.level_3_enemies.max(0.0).floor() as usize;
        let mut prefabs = Vec::with_capacity(lvl1 + lvl2 + lvl3);

        for _ in 0..lvl1 {
            prefabs.push(
                ENEMY_STR_1
                    .choose(random.rng())
                    .map(|p| p.to_string())
                    .unwrap(),
            );
        }
        for _ in 0..lvl2 {
            prefabs.push(
                ENEMY_STR_2
                    .choose(random.rng())
                    .map(|p| p.to_string())
                    .unwrap(),
            );
        }
        for _ in 0..lvl3 {
            prefabs.push(
                ENEMY_STR_3
                    .choose(random.rng())
                    .map(|p| p.to_string())
                    .unwrap(),
            );
        }
        prefabs
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DifficultyCurve {
    Linear(f32, f32),
    Constant(f32),
}

impl DifficultyCurve {
    pub fn y(&self, x: f32) -> f32 {
        match self {
            Self::Linear(origin, slope) => origin + slope * x,
            Self::Constant(v) => *v,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DifficultyConfig {
    level_1_curve: DifficultyCurve,
    level_2_curve: DifficultyCurve,
    level_3_curve: DifficultyCurve,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            level_1_curve: DifficultyCurve::Linear(0.0, 1.0),
            level_2_curve: DifficultyCurve::Linear(-2.0, 0.5),
            level_3_curve: DifficultyCurve::Linear(-5.0, 0.2),
        }
    }
}

impl DifficultyConfig {
    pub fn difficulty(&self, wave_nb: usize) -> WaveDifficulty {
        let next_level_1 = self.level_1_curve.y(wave_nb as f32);
        let next_level_2 = self.level_2_curve.y(wave_nb as f32);
        let next_level_3 = self.level_3_curve.y(wave_nb as f32);
        WaveDifficulty {
            level_1_enemies: next_level_1,
            level_2_enemies: next_level_2,
            level_3_enemies: next_level_3,
        }
    }
}
