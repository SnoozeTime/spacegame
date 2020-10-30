use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::Bullet;
use crate::gameplay::health::Health;
use crate::resources::Resources;
use hecs::{Entity, World};
use log::{debug, trace};
use shrev::EventChannel;

/// Bounding box to detect collisions.
#[derive(Debug)]
pub struct BoundingBox {
    pub half_extend: glam::Vec2,
    pub collision_layer: CollisionLayer,
    pub collision_mask: CollisionLayer,
}

impl BoundingBox {
    fn can_collide(&self, other: &BoundingBox) -> bool {
        trace!(
            "My collision layer {:?} & other collision mask {:?} = {:?}",
            self.collision_layer,
            other.collision_mask,
            self.collision_layer & other.collision_mask
        );
        trace!(
            "(self.collision_layer & other.collision_layer).bits = {:?}",
            (self.collision_layer & other.collision_mask).bits
        );
        (self.collision_layer & other.collision_mask).bits != 0
    }
}

bitflags! {
    pub struct CollisionLayer: u32 {
        const PLAYER = 0b00000001;
        const ENEMY = 0b00000010;
        const PLAYER_BULLET = 0b00000100;
        const ENEMY_BULLET = 0b00001000;
    }
}

pub fn find_collisions(world: &World) -> Vec<(Entity, Entity)> {
    trace!("find_collisions");
    let mut query = world.query::<(&Transform, &BoundingBox)>();
    let candidates: Vec<(Entity, (&Transform, &BoundingBox))> = query.iter().collect();
    if candidates.is_empty() {
        trace!("No candidate for collision");
        return vec![];
    }

    let mut collision_pairs = vec![];
    trace!("Candidates for collision = {:?}", candidates);
    for i in 0..candidates.len() - 1 {
        for j in (i + 1)..candidates.len() {
            trace!("Will process {} and {}", i, j);
            trace!("Fetch first entity");
            let (e1, (transform1, bb1)) = candidates[i];
            trace!("Fetch second entity");
            let (e2, (transform2, bb2)) = candidates[j];
            trace!("Entities are {:?} and {:?}", e1, e2);

            if !bb1.can_collide(bb2) {
                continue;
            }

            if transform1.translation.x() - bb1.half_extend.x()
                < transform2.translation.x() + bb2.half_extend.x()
                && transform1.translation.x() + bb1.half_extend.x()
                    > transform2.translation.x() - bb2.half_extend.x()
                && transform1.translation.y() - bb1.half_extend.y()
                    < transform2.translation.y() + bb2.half_extend.y()
                && transform1.translation.y() + bb1.half_extend.y()
                    > transform2.translation.y() - bb2.half_extend.y()
            {
                collision_pairs.push((e1, e2));
            }
        }
    }

    trace!("Finished find_collisions, OUT = {:?}", collision_pairs);

    collision_pairs
}

pub fn process_collisions(
    world: &mut World,
    collision_pairs: Vec<(Entity, Entity)>,
    resources: &Resources,
) {
    trace!("process_collisions, IN = {:?}", collision_pairs);

    let mut ev_channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
    let mut events = vec![];
    for (e1, e2) in collision_pairs {
        debug!("Will process collision between {:?} and {:?}", e1, e2);
        // If an entity is a bullet, let's destroy it.
        if world.get::<Bullet>(e1).is_ok() {
            events.push(GameEvent::Delete(e1));
        }
        if world.get::<Bullet>(e2).is_ok() {
            events.push(GameEvent::Delete(e2));
        }

        // If an entity has health, let's register a hit
        if world.get::<Health>(e1).is_ok() {
            events.push(GameEvent::Hit(e1));
        }
        if world.get::<Health>(e2).is_ok() {
            events.push(GameEvent::Hit(e2));
        }
    }

    if !events.is_empty() {
        debug!("Will publish {:?}", events);
        ev_channel.drain_vec_write(&mut events);
    }

    trace!("Finished process_collisions");
}
