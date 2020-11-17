use super::bullet;
use crate::config::PlayerConfig;
use crate::core::camera::screen_to_world;
use crate::core::input::{Axis, Input};
use crate::core::transform::Transform;
use crate::gameplay::bullet::BulletType;
use crate::gameplay::health::HitDetails;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::trail::Trail;
use crate::gameplay::{steering, Action};
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use bitflags::_core::time::Duration;
use hecs::{Entity, World};
#[allow(unused_imports)]
use log::{info, trace};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Weapon {
    Simple,
    Multiple,
}

impl Weapon {
    pub fn shoot(
        &self,
        initial_pos: glam::Vec2,
        direction: glam::Vec2,
    ) -> Vec<(glam::Vec2, glam::Vec2, BulletType)> {
        match *self {
            Weapon::Simple => vec![(initial_pos, direction, bullet::BulletType::Twin)],
            Weapon::Multiple => vec![
                (initial_pos, direction, bullet::BulletType::Fast),
                (
                    initial_pos,
                    glam::Mat2::from_angle(3.14 / 4.0) * direction,
                    bullet::BulletType::Fast,
                ),
                (
                    initial_pos,
                    glam::Mat2::from_angle(-3.14 / 4.0) * direction,
                    bullet::BulletType::Fast,
                ),
            ],
        }
    }
}

/// Tag to tell the ECS that the entity is a player.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub weapon: Weapon,
    pub direction: glam::Vec2,
}

/// Will return the player entity if it exists.
pub fn get_player(world: &World) -> Option<Entity> {
    world.query::<&Player>().iter().map(|(e, _)| e).next()
}

fn x_axis() -> Axis<Action> {
    Axis {
        left: Action::MoveLeft,
        right: Action::MoveRight,
    }
}

fn y_axis() -> Axis<Action> {
    Axis {
        left: Action::MoveDown,
        right: Action::MoveUp,
    }
}

fn z_axis() -> Axis<Action> {
    Axis {
        left: Action::RotateLeft,
        right: Action::RotateRight,
    }
}

pub fn update_player(world: &mut World, _dt: Duration, resources: &Resources) {
    // only one player for now.
    let input = resources.fetch::<Input<Action>>().unwrap();
    let player_controller_conf = resources.fetch::<PlayerConfig>().unwrap();

    let projection_matrix =
        glam::Mat4::orthographic_rh_gl(0.0, WIDTH as f32, 0.0, HEIGHT as f32, -1.0, 10.0);

    let mut bullets = vec![];

    for (_e, (transform, player, dynamic, trail)) in world
        .query::<(&mut Transform, &mut Player, &mut DynamicBody, &mut Trail)>()
        .iter()
    {
        let (delta_x, delta_y, delta_z) = (
            input.get_axis(x_axis()),
            input.get_axis(y_axis()),
            input.get_axis(z_axis()),
        );
        let dir = glam::Mat2::from_angle(transform.rotation) * glam::Vec2::unit_y();
        trail.should_display = delta_y.max(0.0) > 0.0;

        // DESIRED VELOCITY IF FORWARD TO THE MOUSE CURSOR
        let target = screen_to_world(input.mouse_position(), projection_matrix, world);

        let steering_force = steering::seek(
            transform.translation,
            dynamic.velocity,
            target,
            dynamic.max_velocity * delta_y.max(0.0),
        );

        // lateral force from the side-thrusters.
        let lateral_force = delta_x
            * player_controller_conf.lateral_thrust
            * glam::Mat2::from_angle(transform.rotation)
            * glam::Vec2::unit_x();

        dynamic.add_force(steering_force);
        dynamic.add_force(lateral_force);

        // rotation from player angle to desired direction.
        if dynamic.forces.len() > 0 {
            let angle_to_perform = (target - transform.translation).angle_between(dir);
            transform.rotation -= player_controller_conf.rotation_delta * angle_to_perform;
        }
        transform.rotation -= player_controller_conf.rotation_delta * delta_z;

        // Shoot stuff
        if input.is_just_pressed(Action::Shoot) {
            // shoot from the top.
            let initial_pos = transform.translation
                + glam::Mat2::from_angle(transform.rotation)
                    * glam::vec2(0.0, transform.scale.y() / 2.0);
            bullets = player.weapon.shoot(initial_pos, dir);
        }
    }

    bullets.iter().for_each(|(p, d, b)| {
        bullet::spawn_player_bullet(world, *p, *d, *b, HitDetails { hit_points: 1.0 });
    });

    trace!("finished update_player");
}
