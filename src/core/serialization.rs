//! Provide a macro to create SerializableEntity that can be saved, sent over network and so on...
use serde_derive::{Deserialize, Serialize};

fn get_component<T>(world: &hecs::World, e: hecs::Entity) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    world.get::<T>(e).ok().map(|c| (*c).clone())
}

macro_rules! serialize {
    ($(($name:ident, $component:ty)),+) => {


        #[derive(Debug, Clone, Serialize, Deserialize, Default)]
        pub struct SerializedEntity {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(default)]
                pub $name: Option<$component>
            ),+

        }

        impl SerializedEntity {

            pub fn spawn(&self, world: &mut hecs::World) -> hecs::Entity {
                let mut builder = hecs::EntityBuilder::new();

                $(
                    if let Some(ref c) = self.$name {
                        builder.add(c.clone());
                    }
                )+

                world.spawn(builder.build())

            }

            pub fn spawn_at_pos(&self, world: &mut hecs::World, pos: glam::Vec2) -> hecs::Entity {
                let e = self.spawn(world);

                if let Ok(mut t) = world.get_mut::<Transform>(e) {
                    t.translation = pos;
                }

                e
            }
        }
    };
}
use super::transform::Transform;
use crate::core::animation::{Animation, AnimationController};
use crate::gameplay::collision::BoundingBox;
use crate::gameplay::enemy::Enemy;
use crate::gameplay::health::{Health, Shield};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::Player;
use crate::gameplay::trail::Trail;
use crate::render::particle::ParticleEmitter;
use crate::render::sprite::Sprite;
/**

            sprite: Sprite {
                id: "spaceships/blue_05.png".to_string(),
            },
            bounding_box: BoundingBox {
                half_extend: 20.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::PLAYER,
                collision_mask: None,
            },
            health: Health::new(10.0, Timer::of_seconds(0.5)),
            shield: None,
            trail: emitter,
            stats: Stats {
                dmg: 1.0,
                crit_percent: 50,
                crit_multiplier: 1.5,
                missile_percent: 0,
                boost_timer: Timer::of_seconds(1.0),
                boost_magnitude: 500.0,
            },
**/

serialize! {
    (transform, Transform),
    (dynamic_body, DynamicBody),
    (sprite, Sprite),
    (bounding_box, BoundingBox),
    (health, Health),
    (shield, Shield),
    (trail, Trail),
    (emitter, ParticleEmitter),
    (player, Player),
    (enemy, Enemy),
    (animation, AnimationController)
}
