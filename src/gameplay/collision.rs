use crate::core::colors::RgbaColor;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::{Bullet, Missile};
use crate::gameplay::explosion::ExplosionDetails;
use crate::gameplay::health::{Health, HitDetails};
use crate::gameplay::physics::DynamicBody;
use crate::render::path::debug;
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
                collision_mask: None,
                collision_layer: CollisionLayer::NOTHING,
                half_extend: bb.half_extend + offset * glam::Vec2::one(),
            };
            if let Some((t, pos)) = enlarged.intersect_ray(*transform, ray) {
                intersections.push((*e, t, pos, *transform));
            }
        }

        return intersections;
    }

    pub fn circle_query(&self, center: glam::Vec2, radius: f32) -> Vec<Entity> {
        let mut intersections = vec![];
        let circle = Circle { center, radius };
        for (transform, bb, e) in self.bodies.iter() {
            if bb.collide_with_circle(*transform, circle) {
                intersections.push(*e);
            }
        }

        intersections
    }
}

/// Bounding box to detect collisions.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BoundingBox {
    pub half_extend: Vec2,
    pub collision_layer: CollisionLayer,
    pub collision_mask: Option<CollisionLayer>,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            half_extend: Default::default(),
            collision_layer: CollisionLayer::NOTHING,
            collision_mask: None,
        }
    }
}

const EPSILON: f32 = 0.0001;

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    pub center: glam::Vec2,
    pub radius: f32,
}

impl Circle {
    fn is_point_inside(&self, point: glam::Vec2) -> bool {
        (self.center - point).length() <= self.radius
    }
}

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
        match (
            self.collision_layer,
            other.collision_mask,
            other.collision_layer,
            self.collision_mask,
        ) {
            (layer, Some(mask), _, _) => (layer & mask).bits != 0,
            (_, _, layer, Some(mask)) => (layer & mask).bits != 0,
            _ => false,
        }
    }

    fn is_point_inside(&self, position: glam::Vec2, point: glam::Vec2) -> bool {
        let min = position - self.half_extend;
        let max = position + self.half_extend;
        point.x() >= min.x() && point.x() <= max.x() && point.y() >= min.y() && point.y() <= max.y()
    }

    fn collide_with_circle(&self, position: glam::Vec2, circle: Circle) -> bool {
        // edge case is circle center inside AABB
        if self.is_point_inside(position, circle.center) {
            return true;
        }

        // If any edge in circle, that's a collision.
        let edge1 = position - self.half_extend;
        let edge2 = position - self.half_extend.x() * glam::Vec2::unit_x()
            + self.half_extend.y() * glam::Vec2::unit_y();
        let edge3 = position + self.half_extend.x() * glam::Vec2::unit_x()
            - self.half_extend.y() * glam::Vec2::unit_y();
        let edge4 = position + self.half_extend;

        circle.is_point_inside(edge1)
            || circle.is_point_inside(edge2)
            || circle.is_point_inside(edge3)
            || circle.is_point_inside(edge4)
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
        const NOTHING = 0b1000000000000;
        const PLAYER = 0b00000001;
        const ENEMY = 0b00000010;
        const PLAYER_BULLET = 0b00000100;
        const ENEMY_BULLET = 0b00001000;
        const ASTEROID = 0b00010000;
        const MISSILE = 0b00100000;
        const PICKUP = 0b01000000;
        const MINE = 0b010000000;
    }
}

pub fn find_collisions(world: &World, resources: &Resources) -> Vec<(Entity, Entity)> {
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
                // if collision, let's draw the quads :)
                debug::stroke_quad(
                    resources,
                    transform1.translation - bb1.half_extend,
                    bb1.half_extend * 2.0,
                    RgbaColor::new(255, 0, 0, 255),
                );
                debug::stroke_quad(
                    resources,
                    transform2.translation - bb2.half_extend,
                    bb2.half_extend * 2.0,
                    RgbaColor::new(0, 255, 0, 255),
                );

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
        if e1 == e2 {
            continue;
        }
        info!("Will process collision between {:?} and {:?}", e1, e2);

        // bullet to health.
        // -------------------
        {
            let e1_health = world.get::<Health>(e1).is_ok();
            let e2_health = world.get::<Health>(e2).is_ok();
            let e1_bullet = world.get::<Bullet>(e1).is_ok();
            let e2_bullet = world.get::<Bullet>(e2).is_ok();
            match (e1_health, e2_bullet, e2_health, e1_bullet) {
                (true, true, _, _) => events.append(&mut process_bullet_collision(world, e1, e2)),
                (_, _, true, true) => events.append(&mut process_bullet_collision(world, e2, e1)),
                (false, true, _, _) => {
                    if let Some(ev) = delete_bullet(world, e2) {
                        events.push(ev)
                    }
                }
                (_, _, _, true) => {
                    if let Some(ev) = delete_bullet(world, e1) {
                        events.push(ev)
                    }
                }
                _ => (),
            }
        }

        // Missile to health
        // -------------------
        let has_missile;
        {
            let e1_missile = world.get::<Missile>(e1).is_ok();
            let e2_missile = world.get::<Missile>(e2).is_ok();
            has_missile = e1_missile || e2_missile;
            match (e2_missile, e1_missile) {
                (true, _) => {
                    events.push(GameEvent::Delete(e2));
                    let e2_transform = world
                        .get::<Transform>(e2)
                        .expect("Missile should have a transform");
                    events.push(GameEvent::Explosion(
                        e2,
                        ExplosionDetails { radius: 100.0 },
                        e2_transform.translation,
                    ));
                }
                (_, true) => {
                    events.push(GameEvent::Delete(e1));
                    let e1_transform = world
                        .get::<Transform>(e1)
                        .expect("Missile should have a transform");
                    events.push(GameEvent::Explosion(
                        e1,
                        ExplosionDetails { radius: 100.0 },
                        e1_transform.translation,
                    ));
                }
                _ => (),
            }
        }

        if has_missile {
            continue;
        }

        // Apply forces for dynamic bodies.
        // --------------------------------------------
        let mut e1_query = world
            .query_one::<(&Transform, &mut DynamicBody)>(e1)
            .expect("Entity should exist");
        let mut e2_query = world
            .query_one::<(&Transform, &mut DynamicBody)>(e2)
            .expect("Entity should exist");

        match (e1_query.get(), e2_query.get()) {
            (Some((t1, b1)), Some((t2, b2))) => {
                let dir = (t1.translation - t2.translation).normalize();

                let restitution =
                    (b1.velocity - b2.velocity).dot(dir) / (1.0 / b1.mass + 1.0 / b2.mass);
                // BIM!
                if restitution < 0.0 {
                    b2.add_impulse(dir * restitution / b2.mass);
                    b1.add_impulse(-dir * restitution / b1.mass);
                }
            }
            _ => (),
        }
    }

    if !events.is_empty() {
        debug!("Will publish {:?}", events);
        ev_channel.drain_vec_write(&mut events);
    }

    trace!("Finished process_collisions");
}

fn process_bullet_collision(
    world: &mut World,
    health_entity: hecs::Entity,
    bullet_entity: hecs::Entity,
) -> Vec<GameEvent> {
    let mut events = vec![];

    if let Ok(mut b) = world.get_mut::<Bullet>(bullet_entity) {
        // if bullet is not alive, let's not process the rest.
        if b.alive {
            b.alive = false;
            events.push(GameEvent::Delete(bullet_entity));
            events.push(GameEvent::Hit(health_entity, b.details));
        }
    }

    events
}

fn process_missile_collision(
    world: &mut World,
    health_entity: hecs::Entity,
    missile_entity: hecs::Entity,
) -> Vec<GameEvent> {
    let mut events = vec![];

    if let Ok(mut _b) = world.get_mut::<Missile>(missile_entity) {
        events.push(GameEvent::Delete(missile_entity));
        events.push(GameEvent::Hit(
            health_entity,
            HitDetails {
                hit_points: 1.0,
                is_crit: false,
            },
        ));
    }

    events
}

fn delete_bullet(world: &mut World, bullet_entity: hecs::Entity) -> Option<GameEvent> {
    let mut b = world.get_mut::<Bullet>(bullet_entity).unwrap();
    // if bullet is not alive, let's not process the rest.
    if !b.alive {
        None
    } else {
        b.alive = false;
        Some(GameEvent::Delete(bullet_entity))
    }
}
