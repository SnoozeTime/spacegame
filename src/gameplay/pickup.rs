use crate::core::input::Input;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::collision::{aabb_intersection, BoundingBox, CollisionLayer};
use crate::gameplay::inventory::Inventory;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::Player;
use crate::gameplay::Action;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;

pub struct Pickup {
    pub item: Items,
}

pub fn spawn_pickup(world: &mut hecs::World, pos: glam::Vec2) -> hecs::Entity {
    world.spawn((
        Transform {
            translation: pos,
            scale: glam::vec2(20.0, 20.0),
            rotation: 0.0,
            dirty: false,
        },
        Sprite {
            id: "capsule.png".to_string(),
        },
        Pickup {
            item: Items::SpeedBonus,
        },
        BoundingBox {
            half_extend: glam::vec2(10.0, 10.0),
            collision_layer: CollisionLayer::PICKUP,
            collision_mask: None,
        },
    ))
}

pub fn is_pickup(world: &hecs::World, entity: hecs::Entity) -> bool {
    world.get::<Pickup>(entity).is_ok()
}

pub fn process_pickups(world: &mut hecs::World, resources: &Resources) {
    let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
    let input = resources.fetch::<Input<Action>>().unwrap();
    let mut inventory = resources.fetch_mut::<Inventory>().unwrap();
    let mut to_delete = vec![];

    let maybe_player = {
        let mut query = world.query::<(&Transform, &BoundingBox, &Player)>();
        let maybe = query.iter().next().map(|(e, (t, bb, _p))| (e, *t, *bb));
        maybe
    };
    if let Some((player_entity, pos, bounding_box)) = maybe_player {
        let mut picked_up = vec![];
        for (e, (pickup, t, bb)) in world.query::<(&Pickup, &Transform, &BoundingBox)>().iter() {
            if input.is_just_pressed(Action::Pickup)
                && aabb_intersection(&pos, &bounding_box, t, bb)
            {
                if let Ok(()) = inventory.remove_scratch(50) {
                    to_delete.push(GameEvent::Delete(e));
                    picked_up.push(pickup.item);
                }
            }
        }

        for item in picked_up {
            item.process(world, player_entity);
        }
    }

    channel.drain_vec_write(&mut to_delete);
}

pub fn aabb_intersection2(
    transform1: &Transform,
    bb1: &BoundingBox,
    transform2: &Transform,
    bb2: &BoundingBox,
) -> bool {
    dbg!(transform1.translation - bb1.half_extend);
    dbg!(transform1.translation + bb1.half_extend);
    dbg!(transform2.translation - bb2.half_extend);
    dbg!(transform2.translation + bb2.half_extend);
    transform1.translation.x() - bb1.half_extend.x()
        < transform2.translation.x() + bb2.half_extend.x()
        && transform1.translation.x() + bb1.half_extend.x()
            > transform2.translation.x() - bb2.half_extend.x()
        && transform1.translation.y() - bb1.half_extend.y()
            < transform2.translation.y() + bb2.half_extend.y()
        && transform1.translation.y() + bb1.half_extend.y()
            > transform2.translation.y() - bb2.half_extend.y()
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Items {
    // Speed +30%
    SpeedBonus,
}

impl Items {
    pub fn process(&self, world: &mut hecs::World, player: hecs::Entity) {
        match self {
            Items::SpeedBonus => {
                let mut dynamic_body = world.get_mut::<DynamicBody>(player).unwrap();
                dynamic_body.velocity *= 1.3;
            }
        }
    }
}
