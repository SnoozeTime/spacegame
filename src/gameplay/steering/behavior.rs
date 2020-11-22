//! Different behavior for enemies.
//!
use super::{avoid, go_to_path_point, halt, seek};
use crate::core::colors;
use crate::core::random::RandomGenerator;
use crate::core::transform::Transform;
use crate::gameplay::collision::CollisionWorld;
use crate::gameplay::physics::DynamicBody;
use crate::render::path::debug;
use crate::resources::Resources;
use rand::Rng;

/// Follow the player. Also rotate to face the player.
pub fn follow_player(
    t: &mut Transform,
    body: &mut DynamicBody,
    player_pos: Option<glam::Vec2>,
    resources: &Resources,
) {
    if let Some(player_position) = player_pos {
        let steering = if (t.translation - player_position).length() > 200.0 {
            seek(
                t.translation,
                body.velocity,
                player_position,
                body.max_velocity,
            )
        } else {
            halt(body.velocity)
        };

        body.add_force(steering);
        debug::stroke_line(
            resources,
            t.translation,
            t.translation + steering,
            colors::RED,
        );

        // rotate toward the player
        {
            let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
            let angle_to_perform = (player_position - t.translation).angle_between(dir);
            t.rotation -= 0.05 * angle_to_perform;
        }
    }
}

fn rotate_towards(t: &mut Transform, target: glam::Vec2) {
    let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
    let angle_to_perform = (target - t.translation).angle_between(dir);
    t.rotation -= 0.05 * angle_to_perform;
}

///
/// Apply some steering force if there is some obstacle on the way,
pub fn avoid_obstacles(
    e: hecs::Entity,
    t: &mut Transform,
    body: &mut DynamicBody,
    resources: &Resources,
) {
    {
        let collision_world = resources.fetch::<CollisionWorld>().unwrap();
        if body.velocity.length() > 0.0 {
            if let Some(f) = avoid(e, t, body.velocity, 300.0, &*collision_world, 300.0) {
                body.add_force(f);
                debug::stroke_line(resources, t.translation, t.translation + f, colors::BLUE);
            }
        }
    }
}

pub fn follow_random_path(
    target: &mut glam::Vec2,
    is_initialized: &mut bool,
    t: &mut Transform,
    body: &mut DynamicBody,
    resources: &Resources,
) {
    if !(*is_initialized) {
        let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
        *target = pick_point(&mut *random);
        *is_initialized = true;
    }

    let steering = go_to_path_point(*target, t.translation, body.velocity, body.max_velocity);
    debug::stroke_line(resources, t.translation, *target, colors::BLUE);

    if let Some(steering) = steering {
        body.add_force(steering);
        debug::stroke_line(
            resources,
            t.translation,
            t.translation + steering,
            colors::RED,
        );
    } else {
        // Need to generate new target.
        //
        let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
        *target = pick_point(&mut *random);
    }

    rotate_towards(t, *target);
}

fn pick_point(random: &mut RandomGenerator) -> glam::Vec2 {
    let x_range = (-1500.0, 1500.0f32);
    let y_range = (-850.0, 850.0f32);

    let rng = random.rng();
    let (x, y) = (
        rng.gen_range(x_range.0, x_range.1),
        rng.gen_range(y_range.0, y_range.1),
    );

    glam::vec2(x, y)
}
