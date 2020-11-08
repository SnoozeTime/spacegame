use crate::gameplay::collision::{CollisionLayer, CollisionWorld, Ray};
use hecs::Entity;
use rand::Rng;

/// Will compute the force to move towards a target without slowing down.
///
/// # Returns
/// the force to apply to the entity
pub fn seek(
    position: glam::Vec2,
    velocity: glam::Vec2,
    target: glam::Vec2,
    max_speed: f32,
) -> glam::Vec2 {
    (max_speed * (target - position).normalize()) - velocity
}

/// Random movements that do not look too weird
pub fn wander(velocity: glam::Vec2, wander_strength: f32) -> glam::Vec2 {
    let circle_center = velocity.normalize() * 20.0; // TODO Circle distance somewhere else.
    let mut rng = rand::thread_rng();
    let displacement = glam::Mat2::from_angle(rng.gen_range(0.0, 2.0 * std::f32::consts::PI))
        * glam::Vec2::unit_x()
        * wander_strength;

    circle_center + displacement
}

/// STOP
pub fn halt(velocity: glam::Vec2) -> glam::Vec2 {
    -velocity
}

pub fn avoid(
    myself: Entity,
    pos: glam::Vec2,
    velocity: glam::Vec2,
    look_ahead: f32,
    collision_world: &CollisionWorld,
    avoidance_strength: f32,
) -> Option<glam::Vec2> {
    let ray = Ray {
        c: pos,
        d: velocity.normalize(),
    };

    let collisions = collision_world.ray(ray, CollisionLayer::NOTHING);

    let mut current_t = std::f32::INFINITY;
    let mut collision_pos = None;
    for (e, t, pos) in collisions {
        if e == myself {
            continue;
        }
        info!("Collide with entity = {:?} in {} at {:?}", e, t, pos);

        if t < look_ahead && t < current_t {
            current_t = t;
            // there will be a collisions.
            collision_pos = Some(pos);
        }
    }

    if let Some(obstacle_pos) = collision_pos {
        let d = (obstacle_pos - pos).length();
        let avoidance_force = glam::vec2(velocity.y(), -velocity.x());
        Some(avoidance_force.normalize() * avoidance_strength * look_ahead / d)
    } else {
        None
    }
}
