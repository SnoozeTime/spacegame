use super::bullet;
use crate::core::input::{Axis, Input};
use crate::core::transform::Transform;
use crate::gameplay::bullet::BulletType;
use crate::gameplay::Action;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use hecs::{Entity, World};
use log::trace;

pub enum Weapon {
    Simple,
    Multiple,
}

impl Weapon {
    pub fn shoot(&self, initial_pos: glam::Vec2) -> Vec<(glam::Vec2, glam::Vec2, BulletType)> {
        let direction = glam::Vec2::unit_y();

        match *self {
            Weapon::Simple => vec![(initial_pos, direction, bullet::BulletType::Round1)],
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
pub struct Player {
    pub weapon: Weapon,
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

pub fn update_player(world: &mut World, resources: &Resources) {
    // only one player for now.
    let player_entity = get_player(world);
    if let Some(player_entity) = player_entity {
        trace!("update_player");
        let input = resources.fetch::<Input<Action>>().unwrap();
        // Move the player
        let player_pos = {
            let mut player_pos = world.get_mut::<Transform>(player_entity).unwrap();
            let (delta_x, delta_y) = (input.get_axis(x_axis()), input.get_axis(y_axis()));
            if delta_x != 0.0 || delta_y != 0.0 {
                let delta_pos = 5.0 * glam::Vec2::new(delta_x, delta_y).normalize();
                player_pos.translation += delta_pos;

                // constraint position to the screen.
                let min_allowed_x = 10.0;
                let min_allowed_y = 10.0;
                let max_allowed_y = HEIGHT as f32 / 2.0 - 10.0;
                let max_allowed_x = WIDTH as f32 - 10.0;
                if player_pos.translation.x() < min_allowed_x {
                    player_pos.translation.set_x(min_allowed_x);
                } else if player_pos.translation.x() > max_allowed_x {
                    player_pos.translation.set_x(max_allowed_x);
                } else if player_pos.translation.y() < min_allowed_y {
                    player_pos.translation.set_y(min_allowed_y);
                } else if player_pos.translation.y() > max_allowed_y {
                    player_pos.translation.set_y(max_allowed_y);
                }
            }

            *player_pos
        };

        // check if we should shoot.
        if input.is_just_pressed(Action::Shoot) {
            // shoot from the top.
            let initial_pos = player_pos.translation + glam::vec2(0.0, player_pos.scale.y() / 2.0);
            let bullets = {
                let player = world.get::<Player>(player_entity).unwrap();
                player.weapon.shoot(initial_pos)
            };

            bullets.iter().for_each(|(p, d, b)| {
                bullet::spawn_player_bullet(world, *p, *d, *b);
            });
        }
    }

    trace!("finished update_player");
}
