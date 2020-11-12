use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::Bullet;
use crate::gameplay::health::Health;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::pickup::is_pickup;
use crate::resources::Resources;
use glam::Vec2;
use hecs::{Entity, World};
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::mem::swap;

#[derive(Debug)]
pub struct CollisionWorld {
    bodies: Vec<(glam::Vec2, BoundingBox, Entity)>,
}

impl Default for CollisionWorld {
    fn default() -> Self {
        Self { bodies: vec![] }
    }
}

impl CollisionWorld {
    /// most likely not super good to do... let's see if we have perf issues later.
    pub fn synchronize(&mut self, world: &World) {
        self.bodies = world
            .query::<(&Transform, &BoundingBox)>()
            .iter()
            .map(|(e, (t, b))| (t.translation, *b, e))
            .collect();
    }

    /// Find collisions with ray. will ignore the bounding boxes with the `ignore` layer.
    pub fn ray(&self, ray: Ray, ignore: CollisionLayer) -> Vec<(Entity, f32, Vec2)> {
        let mut intersections = vec![];
        for (t, bb, e) in self.bodies.iter() {
            if (bb.collision_layer & ignore).bits != 0 {
                continue;
            }

            if let Some((t, pos)) = bb.intersect_ray(*t, ray) {
                intersections.push((*e, t, pos));
            }
        }

        return intersections;
    }

    pub fn ray_with_offset(
        &self,
        ray: Ray,
        ignore: CollisionLayer,
        offset: f32,
    ) -> Vec<(Entity, f32, Vec2, Vec2)> {
        let mut intersections = vec![];
        for (transform, bb, e) in self.bodies.iter() {
            if (bb.collision_layer & ignore).bits != 0 {
                continue;
            }

            // enlarge the bounding box with our own.
            let enlarged = BoundingBox {
                collision_mask: CollisionLayer::NOTHING,
                collision_layer: CollisionLayer::NOTHING,
                half_extend: bb.half_extend + offset * glam::Vec2::one(),
            };
            if let Some((t, pos)) = enlarged.intersect_ray(*transform, ray) {
                intersections.push((*e, t, pos, *transform));
            }
        }

        return intersections;
    }
}

/// Bounding box to detect collisions.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BoundingBox {
    pub half_extend: Vec2,
    pub collision_layer: CollisionLayer,
    pub collision_mask: CollisionLayer,
}

const EPSILON: f32 = 0.0001;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    /// origin of ray
    pub c: Vec2,

    /// direction of ray
    pub d: Vec2,
}

impl Ray {
    pub fn new(c: Vec2, d: Vec2) -> Self {
        Self { c, d }
    }
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
            || (self.collision_mask & other.collision_layer).bits != 0
    }

    /// Check if ray intersects the AABB. If yes, it will return the time of intersection and point
    /// of intersection;
    ///
    /// # Algorithm
    /// Check the intersection of ray with each slabs of the AABB (x-axis slab, y-axis, z-axis).
    /// If the intersections overlap, then the ray intersects with the AABB (recall, a point is
    /// in the AABB if it is in the three slabs).
    ///
    /// For each slab, compute the time of entry and the time of exit. Then, take the max of time
    /// of entry, take the min of time of exit. If t_entry < t_exit, the slabs overlap.
    ///
    /// Ray equation: R(t) = P + t.d where P is origin of ray and d its direction.
    /// Equation of planes: X.ni = di.
    /// Substitute X by R to get the intersection.
    /// (P + t.d) . ni = di
    /// t = (di - P.ni)/(d.ni)
    ///
    /// For the AABB planes, n is along the axis. The expression can be simplified: for example
    /// t = (d - px)/dx where d is the position of the plane along the x axis.
    pub fn intersect_ray(&self, pos: glam::Vec2, ray: Ray) -> Option<(f32, Vec2)> {
        let mut tmin = 0.0f32; // set to -FLT_MAX to get first hit on the line.
        let mut tmax = std::f32::MAX; // max distance the ray can travel.

        let min = pos - self.half_extend;
        let max = pos + self.half_extend;

        for i in 0..2 {
            if ray.d[i].abs() < EPSILON {
                // ray is parallel to the slab so we only need to test whether the origin is within
                // the slab.
                if ray.c[i] < min[i] || ray.c[i] > max[i] {
                    return None;
                }
            } else {
                let ood = 1.0 / ray.d[i];
                let mut t1 = (min[i] - ray.c[i]) * ood;
                let mut t2 = (max[i] - ray.c[i]) * ood;

                // make t1 intersection with the near plane.
                if t2 < t1 {
                    swap(&mut t2, &mut t1);
                }

                // compute intersection of slabs intersection intervals.
                // farthest of all entries.
                if t1 > tmin {
                    tmin = t1;
                }
                // nearest of all exits
                if t2 < tmax {
                    tmax = t2;
                }

                if tmin > tmax {
                    return None;
                }
            }
        }

        let q = ray.c + tmin * ray.d;

        Some((tmin, q))
    }
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct CollisionLayer: u32 {
        const NOTHING = 0b10000000;
        const PLAYER = 0b00000001;
        const ENEMY = 0b00000010;
        const PLAYER_BULLET = 0b00000100;
        const ENEMY_BULLET = 0b00001000;
        const ASTEROID = 0b00010000;
        const MISSILE = 0b00100000;
        const PICKUP = 0b01000000;
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

pub fn aabb_intersection(
    transform1: &Transform,
    bb1: &BoundingBox,
    transform2: &Transform,
    bb2: &BoundingBox,
) -> bool {
    transform1.translation.x() - bb1.half_extend.x()
        < transform2.translation.x() + bb2.half_extend.x()
        && transform1.translation.x() + bb1.half_extend.x()
            > transform2.translation.x() - bb2.half_extend.x()
        && transform1.translation.y() - bb1.half_extend.y()
            < transform2.translation.y() + bb2.half_extend.y()
        && transform1.translation.y() + bb1.half_extend.y()
            > transform2.translation.y() - bb2.half_extend.y()
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
        if let Ok(mut b) = world.get_mut::<Bullet>(e1) {
            // if bullet is not alive, let's not process the rest.
            if !b.alive {
                continue;
            }
            b.alive = false;

            events.push(GameEvent::Delete(e1));
        }
        if let Ok(mut b) = world.get_mut::<Bullet>(e2) {
            // if bullet is not alive, let's not process the rest.
            if !b.alive {
                continue;
            }
            b.alive = false;
            events.push(GameEvent::Delete(e2));
        }

        // If an entity has health, let's register a hit
        if world.get::<Health>(e1).is_ok() {
            events.push(GameEvent::Hit(e1));
        }
        if world.get::<Health>(e2).is_ok() {
            events.push(GameEvent::Hit(e2));
        }

        // // process pickups.
        // // ---------------------------------------
        // {
        //     if let Some((pickup_entity, _player)) =
        //         match (is_pickup(world, e1), is_pickup(world, e2)) {
        //             (true, _) => Some((e1, e2)),
        //             (_, true) => Some((e2, e1)),
        //             (_, _) => None,
        //         }
        //     {
        //         info!("PLAYER HAS PICKED UP SOMETHING");
        //         events.push(GameEvent::Delete(pickup_entity));
        //     }
        // }

        // Apply forces for dynamic bodies.
        // --------------------------------------------
        let e1_query = world.query_one::<(&Transform, &mut DynamicBody)>(e1);
        let e2_query = world.query_one::<(&Transform, &mut DynamicBody)>(e2);

        match (e1_query, e2_query) {
            (Ok(mut e1_query), Ok(mut e2_query)) => match (e1_query.get(), e2_query.get()) {
                (Some((t1, b1)), Some((t2, b2))) => {
                    let dir = (t1.translation - t2.translation).normalize();
                    // BIM!
                    b2.add_force(dir * 10.0 * -b2.max_velocity);
                    b1.add_force(dir * 10.0 * b1.max_velocity);
                }
                _ => (),
            },
            _ => (),
        }
    }

    if !events.is_empty() {
        debug!("Will publish {:?}", events);
        ev_channel.drain_vec_write(&mut events);
    }

    trace!("Finished process_collisions");
}
