use crate::assets::prefab::PrefabManager;
use crate::assets::Handle;
use crate::core::noise::perlin::Perlin;
use crate::core::random::RandomGenerator;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::enemy::{spawn_enemy, EnemyType, Satellite};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::pickup::spawn_pickup;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use hecs::Entity;
use luminance_glfw::GlfwSurface;
use rand::seq::SliceRandom;
use std::time::Duration;

const NB_BLOCKS_X: u32 = 80;
const NB_BLOCKS_Y: u32 = 50;

pub mod random_events;

pub struct Wave {
    enemies: Vec<hecs::Entity>,
    init: bool,
}

impl Wave {
    pub fn new() -> Self {
        Self {
            enemies: vec![],
            init: false,
        }
    }

    pub fn init(&mut self, world: &mut hecs::World, resources: &Resources) {
        let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
        if let Some(prefab) = prefab_manager.get(&Handle("base_enemy".to_string())) {
            prefab.execute(|prefab| {
                self.enemies.push(prefab.spawn(world));
            });
        } else {
            error!("Prefab base_enemy should have been loaded in the loading scene");
        }

        self.init = true;
    }

    pub fn remove_enemy(&mut self, entity: Entity) {
        let maybe_index = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(idx, &e)| e == entity)
            .map(|(i, _)| i)
            .next();

        if let Some(idx) = maybe_index {
            self.enemies.remove(idx);
        }
    }

    pub fn is_finished(&self) -> bool {
        trace!(
            "is+finished = {} && {} = {}",
            self.init,
            self.enemies.is_empty(),
            self.init && self.enemies.is_empty()
        );
        self.init && self.enemies.is_empty()
    }
}

pub struct Stage {
    background: hecs::Entity,
    asteroids: Vec<hecs::Entity>,
    waves: Vec<Wave>,

    current_wave: Option<usize>,
    next_wave: Option<usize>,
    timer_between_waves: Timer,

    finished: bool,
}

impl Stage {
    pub fn new(world: &mut hecs::World, resources: &Resources) -> Self {
        // 1. CREATE THE BACKGROUND!
        // ----------------------------------
        let backgrounds = ["front.png", "left.png", "top.png", "right.png", "back.png"];

        let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();

        let background = backgrounds.choose(random.rng()).unwrap().to_string();

        // First choose a random background.
        let background = world.spawn((
            Transform {
                translation: glam::Vec2::new(0.0, 0.0),
                scale: glam::Vec2::new(1600.0, 960.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite { id: background },
        ));

        // 2. GENERATE ASTEROIDS!
        // -------------------------------
        let (asteroids, no_asteroids) = generate_terrain(world, &mut *random, 15);

        // 3. Stuff that the player can pick up for bonuses.
        // -------------------------------------------------
        spawn_pickups(world, &mut *random, &no_asteroids, 3);
        let waves = vec![Wave::new(), Wave::new(), Wave::new()];
        assert!(waves.len() > 0);

        Self {
            background,
            asteroids,
            waves,
            finished: false,

            current_wave: None,
            next_wave: Some(0),
            timer_between_waves: Timer::of_seconds(3.0),
        }
    }

    pub fn enemy_died(&mut self, entity: Entity) {
        if let Some(wave) = self.current_wave {
            if let Some(wave) = self.waves.get_mut(wave) {
                wave.remove_enemy(entity);
            }
        }
    }

    pub fn update(&mut self, world: &mut hecs::World, resources: &Resources, dt: Duration) {
        if self.finished {
            return;
        }

        match self.current_wave {
            None => {
                self.timer_between_waves.tick(dt);
                if self.timer_between_waves.finished() {
                    self.timer_between_waves.stop();
                    self.timer_between_waves.reset();

                    // NOW START next wave if there is any.
                    if let Some(next_wave) = self.next_wave {
                        let wave = self.waves.get_mut(next_wave).unwrap();
                        wave.init(world, resources);
                        self.current_wave = Some(next_wave);
                    } else {
                        // Bye bye :)
                        self.finished = true;
                    }
                }
            }
            Some(idx) => {
                let wave = self.waves.get(idx).unwrap();
                if wave.is_finished() {
                    println!("FINISHED");
                    self.current_wave = None;
                    self.next_wave = if self.waves.len() > idx + 1 {
                        Some(idx + 1)
                    } else {
                        None
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

                let world_value = glam::Vec2::new(x as f32 * 32.0, y as f32 * 32.0);
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
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 5.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: CollisionLayer::PLAYER
                    | CollisionLayer::ENEMY
                    | CollisionLayer::PLAYER_BULLET
                    | CollisionLayer::ENEMY_BULLET,
                half_extend: glam::vec2(16.0, 16.0),
            },
        )));
    }

    for p in asteroid_field2.drain(..).take(asteroid_per_field) {
        trace!("will spawn asteroid at {:?}", p);
        asteroids.push(world.spawn((
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
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 5.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: CollisionLayer::PLAYER
                    | CollisionLayer::ENEMY
                    | CollisionLayer::PLAYER_BULLET
                    | CollisionLayer::ENEMY_BULLET,
                half_extend: glam::vec2(16.0, 16.0),
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
) {
    let mut positions = pick_positions(random, no_asteroid, nb_pickup);
    positions.drain(..).for_each(|p| {
        spawn_pickup(world, p);
    })
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
