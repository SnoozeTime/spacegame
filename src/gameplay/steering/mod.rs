use crate::core::transform::Transform;
use crate::gameplay::collision::{CollisionLayer, CollisionWorld, Ray};
use hecs::Entity;
use rand::Rng;

const CLOSE_ENOUGH: f32 = 50.0;

pub mod behavior;

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
    transform: &Transform,
    velocity: glam::Vec2,
    look_ahead: f32,
    collision_world: &CollisionWorld,
    avoidance_strength: f32,
) -> Option<glam::Vec2> {
    let ray = Ray {
        c: transform.translation,
        d: velocity.normalize(),
    };

    let collisions = collision_world.ray_with_offset(
        ray,
        CollisionLayer::ENEMY_BULLET
            | CollisionLayer::PLAYER_BULLET
            | CollisionLayer::PICKUP
            | CollisionLayer::MINE,
        transform.scale.x() / 2.0,
    );

    let mut current_t = std::f32::INFINITY;
    let mut collision_data = None;
    for (e, t, pos, center) in collisions {
        if e == myself {
            continue;
        }
        debug!("Collide with entity = {:?} in {} at {:?}", e, t, pos);

        if t < look_ahead && t < current_t {
            current_t = t;
            // there will be a collisions.
            collision_data = Some((pos, center));
        }
    }

    if let Some((_obstacle_pos, obstacle_center)) = collision_data {
        let d = (obstacle_center - transform.translation).length();
        let avoidance_force = glam::vec2(velocity.y(), -velocity.x());
        Some(avoidance_force.normalize() * avoidance_strength * look_ahead / d)
    } else {
        None
    }
}

/// Follow the target. If it;s close enough, then it will return None
pub fn go_to_path_point(
    target: glam::Vec2,
    position: glam::Vec2,
    velocity: glam::Vec2,
    max_speed: f32,
) -> Option<glam::Vec2> {
    // Compute distance from position to current. If it's close enough, move to next point.
    if (target - position).length() > CLOSE_ENOUGH {
        // go to next.
        Some(seek(position, velocity, target, max_speed))
    } else {
        None
    }
}
