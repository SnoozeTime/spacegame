use crate::core::noise::perlin::Perlin;
use crate::core::random::RandomGenerator;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::pickup::spawn_pickup;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use hecs::Entity;
use rand::seq::SliceRandom;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

const NB_BLOCKS_X: u32 = 80;
const NB_BLOCKS_Y: u32 = 50;

pub mod difficulty;
pub mod wave;
use crate::event::GameEvent;
use crate::gameplay::health::Invulnerable;
use crate::gameplay::level::difficulty::{DifficultyConfig, WaveDifficulty};
use shrev::EventChannel;
use wave::{Wave, WaveDescription};

#[derive(Debug, Serialize, Deserialize)]
pub struct StageDescription {
    pub waves: Vec<WaveDescription>,
    pub nb_pickups: usize,

    #[serde(default)]
    pub is_infinite: bool,
    #[serde(default)]
    pub next_stage: Option<String>,

    pub backgrounds: Vec<String>,
}

impl StageDescription {
    pub fn infinite() -> Self {
        Self {
            waves: vec![],
            nb_pickups: 7,
            is_infinite: true,
            next_stage: None,
            backgrounds: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stage {
    background: Option<hecs::Entity>,
    /// Asteroid entities.
    asteroids: Vec<hecs::Entity>,
    pickups: Vec<hecs::Entity>,
    /// area without asteroids
    no_asteroids: Vec<glam::Vec2>,
    waves: Vec<Wave>,

    current_wave: Option<usize>,
    next_wave: Option<usize>,
    timer_between_waves: Timer,
    timer_between_stages: Timer,

    finished: bool,
    next_stage: Option<String>,

    pub is_infinite: bool,
    pub wave_number: usize,
    pub wave_difficulty: Option<WaveDifficulty>,
}

impl Stage {
    pub fn new(
        world: &mut hecs::World,
        resources: &Resources,
        mut stage_desc: StageDescription,
    ) -> Self {
        // 1. CREATE THE BACKGROUND!
        // ----------------------------------
        let backgrounds = ["front.png", "left.png", "top.png", "right.png", "back.png"];

        let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();

        let background = stage_desc
            .backgrounds
            .choose(random.rng())
            .cloned()
            .map(|background| {
                world.spawn((
                    Transform {
                        translation: glam::Vec2::new(0.0, 0.0),
                        scale: glam::Vec2::new(1600.0, 960.0),
                        rotation: 0.0,
                        dirty: false,
                    },
                    Sprite { id: background },
                ))
            });

        // 2. GENERATE ASTEROIDS!
        // -------------------------------
        let (asteroids, no_asteroids) = generate_terrain(world, &mut *random, 15);

        // 3. Stuff that the player can pick up for bonuses.
        // -------------------------------------------------
        let pickups = spawn_pickups(world, &mut *random, &no_asteroids, stage_desc.nb_pickups);
        let (waves, wave_difficulty): (Vec<Wave>, Option<_>) = if stage_desc.is_infinite {
            // Generate the wave difficulty.
            let wave_difficulty = WaveDifficulty::default();
            let difficulty_config = resources.fetch::<DifficultyConfig>().unwrap();
            let generated = Self::gen_waves(wave_difficulty, &mut *random, &*difficulty_config);
            (generated.0, Some(generated.1))
        } else {
            (stage_desc.waves.drain(..).map(|w| w.into()).collect(), None)
        };
        assert!(waves.len() > 0);

        Self {
            background,
            asteroids,
            pickups,
            wave_number: 1,
            waves,
            finished: false,
            no_asteroids,
            current_wave: None,
            next_wave: Some(0),
            timer_between_waves: Timer::of_seconds(5.0),
            timer_between_stages: Timer::of_seconds(10.0),
            wave_difficulty,
            next_stage: stage_desc.next_stage,
            is_infinite: stage_desc.is_infinite,
        }
    }

    fn gen_waves(
        mut wave_difficulty: WaveDifficulty,
        random: &mut RandomGenerator,
        difficulty_config: &DifficultyConfig,
    ) -> (Vec<Wave>, WaveDifficulty) {
        info!("Generate waves");
        (
            (0..2)
                .map(|_| {
                    let wave: Wave = WaveDescription {
                        to_instantiate: wave_difficulty.pick_prefabs(&mut *random),
                    }
                    .into();
                    wave_difficulty = difficulty_config.next_difficulty(&wave_difficulty);
                    info!("New wave difficulty = {:#?}", wave_difficulty);
                    wave
                })
                .collect(),
            wave_difficulty,
        )
    }

    pub fn enemy_died(&mut self, entity: Entity) {
        if let Some(wave) = self.current_wave {
            if let Some(wave) = self.waves.get_mut(wave) {
                wave.remove_enemy(entity);
            }
        }
    }

    pub fn update(&mut self, world: &mut hecs::World, resources: &Resources, dt: Duration) {
        match (self.current_wave, self.next_wave) {
            (None, None) => {
                let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
                if let Some(next_stage) = self.next_stage.as_ref() {
                    // In this case, the stage is over !
                    self.finished = true;
                    self.timer_between_stages.tick(dt);
                    if self.timer_between_stages.finished() {
                        channel.single_write(GameEvent::NextStage(next_stage.clone()));
                    }
                } else {
                    // no more stages, the game is finished !
                    channel.single_write(GameEvent::YouWin);
                }
            }
            (None, Some(next_wave)) => {
                // Tick the timer between waves.
                self.timer_between_waves.tick(dt);
                if self.timer_between_waves.finished() {
                    self.wave_number += 1;
                    self.timer_between_waves.stop();
                    self.timer_between_waves.reset();

                    // NOW START next wave if there is any.
                    let wave = self.waves.get_mut(next_wave).unwrap();
                    wave.init(world, resources, &self.no_asteroids);
                    self.current_wave = Some(next_wave);
                }
            }
            (Some(idx), _) => {
                // just check if the current wave is over. If yes, then prepare for next wave or finish the stage
                let wave = self.waves.get(idx).unwrap();
                if wave.is_finished() {
                    self.current_wave = None;
                    self.next_wave = if self.waves.len() > idx + 1 {
                        Some(idx + 1)
                    } else {
                        // if we are in infinite mode, then we will generate more waves :D never
                        // stop!
                        if self.is_infinite {
                            let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
                            let wave_difficulty = self.wave_difficulty.unwrap();
                            let difficulty_config = resources.fetch::<DifficultyConfig>().unwrap();
                            let generated =
                                Self::gen_waves(wave_difficulty, &mut *random, &*difficulty_config);
                            self.waves = generated.0;
                            self.wave_difficulty = Some(generated.1);
                            Some(0)
                        } else {
                            None
                        }
                    };
                }
            }
        }
    }

    pub fn display(&self) -> Option<String> {
        if self.finished {
            return Some("Brace for next stage".to_string());
        }
        if let None = self.current_wave {
            Some(format!(
                "Next wave will start in {:02}",
                self.timer_between_waves.remaining().floor()
            ))
        } else {
            None
        }
    }

    /// Remove all the entities :)
    pub fn clean(&mut self, world: &mut hecs::World) {
        for w in self.waves.iter() {
            w.enemies.iter().for_each(|&e| {
                if let Err(e) = world.despawn(e) {
                    error!("Error while despawning wave = {:?}", e);
                }
            });
        }

        if let Some(bg) = self.background {
            if let Err(e) = world.despawn(bg) {
                error!("Error while despawning background = {:?}", e);
            }
        }

        self.asteroids.iter().for_each(|&e| {
            if let Err(e) = world.despawn(e) {
                error!("Error while despawning asteroids = {:?}", e);
            }
        });

        self.pickups.iter().for_each(|&e| {
            if let Err(e) = world.despawn(e) {
                error!("Error while despawning pickups = {:?}", e);
            }
        });
    }
}

//
pub fn generate_terrain(
    world: &mut hecs::World,
    random: &mut RandomGenerator,
    asteroid_per_field: usize,
) -> (Vec<hecs::Entity>, Vec<glam::Vec2>) {
    let perlin = Perlin::new(random.rng());

    let mut asteroids: Vec<hecs::Entity> = Vec::with_capacity(asteroid_per_field * 2);

    let mut no_asteroid = vec![];
    let (mut asteroids_field1, mut asteroid_field2) = {
        let w = NB_BLOCKS_X;
        let h = NB_BLOCKS_Y;
        let mut values = vec![];
        let mut values2 = vec![];
        for x in 0..w {
            for y in 0..h {
                let xf = x as f32 / w as f32;
                let yf = y as f32 / h as f32;

                let perlin = perlin.octave_perlin(xf, yf, 4, 0.9);

                let x = x as i32 - NB_BLOCKS_X as i32 / 2;
                let y = y as i32 - NB_BLOCKS_Y as i32 / 2;

                let world_value = glam::Vec2::new(x as f32 * 48.0, y as f32 * 48.0);
                if perlin >= 0.0 && perlin <= 0.10 {
                    values.push(world_value);
                } else if perlin >= 0.8 {
                    values2.push(world_value);
                } else {
                    no_asteroid.push(world_value);
                }
            }
        }
        (values, values2)
    };

    debug!(
        "{} possible locations for asteroids field 1",
        asteroids_field1.len()
    );
    debug!(
        "{} possible locations for asteroids field 2",
        asteroid_field2.len()
    );
    asteroids_field1.shuffle(random.rng());
    asteroid_field2.shuffle(random.rng());

    for p in asteroids_field1.drain(..).take(asteroid_per_field) {
        trace!("will spawn asteroid at {:?}", p);
        asteroids.push(world.spawn((
            Invulnerable,
            Transform {
                translation: p,
                scale: glam::Vec2::new(32.0, 32.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite {
                id: "asteroid.png".to_string(),
            },
            DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 5.0,
                max_force: 500.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: Some(
                    CollisionLayer::PLAYER
                        | CollisionLayer::ENEMY
                        | CollisionLayer::PLAYER_BULLET
                        | CollisionLayer::ENEMY_BULLET,
                ),
                half_extend: glam::vec2(32.0, 32.0),
            },
        )));
    }

    for p in asteroid_field2.drain(..).take(asteroid_per_field) {
        trace!("will spawn asteroid at {:?}", p);
        asteroids.push(world.spawn((
            Invulnerable,
            Transform {
                translation: p,
                scale: glam::Vec2::new(32.0, 32.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite {
                id: "asteroid.png".to_string(),
            },
            DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 5.0,
                max_force: 500.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: Some(
                    CollisionLayer::PLAYER
                        | CollisionLayer::ENEMY
                        | CollisionLayer::PLAYER_BULLET
                        | CollisionLayer::ENEMY_BULLET,
                ),
                half_extend: glam::vec2(32.0, 32.0),
            },
        )));
    }

    (asteroids, no_asteroid)
}

fn spawn_pickups(
    world: &mut hecs::World,
    random: &mut RandomGenerator,
    no_asteroid: &Vec<glam::Vec2>,
    nb_pickup: usize,
) -> Vec<hecs::Entity> {
    let positions = pick_positions(random, no_asteroid, nb_pickup);
    positions
        .iter()
        .map(|p| spawn_pickup(world, *p, random))
        .collect()
}

fn pick_positions(
    random: &mut RandomGenerator,
    no_asteroid: &Vec<glam::Vec2>,
    amount: usize,
) -> Vec<glam::Vec2> {
    no_asteroid
        .choose_multiple(random.rng(), amount)
        .map(|p| *p)
        .collect()
}
