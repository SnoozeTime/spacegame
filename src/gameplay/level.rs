use crate::gameplay::enemy::{spawn_enemy, EnemyType};
use hecs::World;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LevelInstruction {
    /// Spawn an enemy
    SpawnEnemy {
        health: u32,
        pos: glam::Vec2,
        enemy_type: EnemyType,
    },

    /// Spawn multiple enemies with same stereotype and some delay.
    SpawnMultiple {
        health: u32,
        nb: usize,
        #[serde(default)]
        spawned: usize,
        time_between: f32,
        #[serde(default)]
        elapsed: f32,
        pos: glam::Vec2,
        enemy_type: EnemyType,
    },

    /// Wait seconds
    Wait {
        to_wait: f32,
        #[serde(default)]
        elapsed: f32,
    },
}

impl LevelInstruction {
    /// Return true if can move to the next instruction.
    pub fn move_next(&self) -> bool {
        match *self {
            LevelInstruction::SpawnEnemy { .. } => true,
            LevelInstruction::Wait { to_wait, elapsed } => elapsed >= to_wait,
            LevelInstruction::SpawnMultiple { nb, spawned, .. } => spawned >= nb,
        }
    }

    pub fn init(&mut self, world: &mut World) {
        match *self {
            LevelInstruction::SpawnEnemy {
                health,
                ref enemy_type,
                pos,
            } => {
                spawn_enemy(world, health, pos, enemy_type.clone());
            }
            LevelInstruction::SpawnMultiple {
                ref mut spawned,
                ref enemy_type,
                pos,
                health,
                ..
            } => {
                spawn_enemy(world, health, pos, enemy_type.clone());
                *spawned += 1;
            }
            _ => (),
        }
    }

    pub fn update(&mut self, world: &mut World, dt: Duration) {
        match *self {
            LevelInstruction::Wait {
                ref mut elapsed, ..
            } => *elapsed += dt.as_secs_f32(),
            LevelInstruction::SpawnMultiple {
                ref mut spawned,
                ref enemy_type,
                pos,
                time_between,
                health,
                ref mut elapsed,
                ..
            } => {
                *elapsed += dt.as_secs_f32();
                if *elapsed >= time_between {
                    spawn_enemy(world, health, pos, enemy_type.clone());
                    *spawned += 1;
                    *elapsed = 0.0;
                }
            }
            _ => (),
        }
    }
}

pub struct LevelSystem {
    instructions: Vec<LevelInstruction>,
    current_instruction: usize,
    init_next: bool,
}

impl LevelSystem {
    pub fn new(instructions: Vec<LevelInstruction>) -> Self {
        Self {
            instructions,
            current_instruction: 0,
            init_next: true,
        }
    }

    pub fn update(&mut self, world: &mut World, dt: Duration) {
        if let Some(current_instruction) = self.instructions.get_mut(self.current_instruction) {
            if self.init_next {
                current_instruction.init(world);
                self.init_next = false;
            }

            current_instruction.update(world, dt);

            if current_instruction.move_next() {
                self.current_instruction += 1;
                self.init_next = true;
            }
        }
    }
}
