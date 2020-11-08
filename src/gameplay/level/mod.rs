use crate::core::noise::perlin::Perlin;
use crate::core::random::RandomGenerator;
use crate::core::transform::Transform;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::physics::DynamicBody;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use rand::seq::SliceRandom;

const NB_BLOCKS_X: u32 = 80;
const NB_BLOCKS_Y: u32 = 50;

pub mod random_events;
//
pub fn generate_terrain(world: &mut hecs::World, resources: &Resources) {
    let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
    let perlin = Perlin::new(random.rng());

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

    for p in asteroids_field1.drain(..).take(5) {
        trace!("will spawn asteroid at {:?}", p);
        world.spawn((
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
        ));
    }

    for p in asteroid_field2.drain(..).take(15) {
        trace!("will spawn asteroid at {:?}", p);
        world.spawn((
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
        ));
    }
}
