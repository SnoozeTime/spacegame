//! EXPLODE STUFF !

use crate::core::animation::{Animation, AnimationController};
use crate::core::colors;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::collision::CollisionWorld;
use crate::gameplay::health::HitDetails;
use crate::gameplay::physics::DynamicBody;
use crate::render::path::debug;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use serde_derive::{Deserialize, Serialize};
use shrev::{EventChannel, ReaderId};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explosive;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExplosionDetails {
    pub radius: f32,
}

pub struct ExplosionSystem {
    rdr_id: ReaderId<GameEvent>,
}

impl ExplosionSystem {
    pub fn new(resources: &mut Resources) -> Self {
        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

        Self {
            rdr_id: channel.register_reader(),
        }
    }

    pub fn update(&mut self, world: &mut hecs::World, resources: &Resources) {
        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        let collision_world = resources.fetch::<CollisionWorld>().unwrap();

        let mut explosions = vec![];
        for ev in channel.read(&mut self.rdr_id) {
            if let GameEvent::Explosion(e, explosion, pos) = ev {
                explosions.push((*e, *explosion, *pos));
            }
        }

        let mut events = vec![];
        for (entity, explosion, pos) in explosions {
            // play the sound, show the animation, then query who is hit by this explosion.
            events.push(GameEvent::PlaySound("sounds/explosion.wav".to_string()));
            spawn_explosion(world, pos, explosion.radius * glam::Vec2::one());

            let entity_touched = collision_world.circle_query(pos, explosion.radius);
            info!("Entities in explosion = {:?}", entity_touched);

            for e in entity_touched {
                if e == entity {
                    continue;
                }

                //if world.get::<Pickup>

                // Apply force from center to position.
                let mut query = world
                    .query_one::<(&Transform, &mut DynamicBody)>(e)
                    .unwrap();

                if let Some((t, body)) = query.get() {
                    let force = (t.translation - pos).normalize() * 500.0;
                    debug::stroke_line(
                        resources,
                        t.translation,
                        t.translation + force,
                        colors::RED,
                    );
                    body.add_impulse(force);

                    events.push(GameEvent::Hit(
                        e,
                        HitDetails {
                            hit_points: 2.0,
                            is_crit: false,
                        },
                    ));
                } else {
                    info!("No transform and body for entity");
                }
            }
        }

        channel.drain_vec_write(&mut events);

        //ev_channel.single_write(GameEvent::PlaySound(
        //                                 "sounds/explosion.wav".to_string(),
        //                             ));
    }
}

pub fn spawn_explosion(world: &mut hecs::World, position: glam::Vec2, scale: glam::Vec2) {
    let mut builder = hecs::EntityBuilder::new();

    builder.add(Transform {
        translation: position,
        scale,
        rotation: 0.0,
        dirty: false,
    });

    let mut animations = HashMap::new();
    animations.insert(
        String::from("boum"),
        Animation::new(vec![
            (String::from("explosion4/k2_0001.png"), 1),
            (String::from("explosion4/k2_0002.png"), 2),
            (String::from("explosion4/k2_0003.png"), 3),
            (String::from("explosion4/k2_0004.png"), 4),
            (String::from("explosion4/k2_0005.png"), 5),
            (String::from("explosion4/k2_0006.png"), 6),
            (String::from("explosion4/k2_0007.png"), 7),
            (String::from("explosion4/k2_0008.png"), 8),
            (String::from("explosion4/k2_0009.png"), 9),
            (String::from("explosion4/k2_0010.png"), 10),
            (String::from("explosion4/k2_0012.png"), 10),
            (String::from("explosion4/k2_0012.png"), 10),
            (String::from("explosion4/k2_0013.png"), 10),
            (String::from("explosion4/k2_0014.png"), 10),
            (String::from("explosion4/k2_0015.png"), 10),
        ]),
    );

    let animation_controller = AnimationController {
        animations,
        current_animation: Some("boum".to_string()),
        delete_on_finished: true,
    };

    builder.add(animation_controller);
    builder.add(Sprite {
        id: String::from("explosion4/k2_0001.png"),
    });

    world.spawn(builder.build());
}
