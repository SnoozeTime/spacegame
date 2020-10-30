use crate::core::transform::Transform;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::render::sprite::Sprite;
use hecs::World;
use log::trace;

#[derive(Debug, Copy, Clone)]
pub enum BulletType {
    Small,
    Fast,
    Round1,
    Round2,
    BigAss,
}

impl BulletType {
    /// Get the name of the sprite that this bullet is representing
    fn get_sprite_name(&self) -> String {
        match *self {
            BulletType::Small => "small_bullet",
            BulletType::Fast => "fast_bullet",
            BulletType::Round1 => "round_bullet",
            BulletType::Round2 => "round_bullet_2",
            BulletType::BigAss => "big_ass_bullet",
        }
        .to_string()
    }
}

/// Tag to indicate the entity is a bullet.
pub struct Bullet {
    pub direction: glam::Vec2,
    pub speed: f32,
}

/// Every frame, will move the bullet in the given direction at the given speed.
pub fn process_bullets(world: &World) {
    trace!("process_bullets");

    for (_, (b, t)) in world.query::<(&Bullet, &mut Transform)>().iter() {
        t.translation += b.direction * b.speed;
    }
    trace!("finished process_bullets");
}

pub fn spawn_player_bullet(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    bullet_type: BulletType,
) -> hecs::Entity {
    world.spawn((
        Bullet {
            direction,
            speed: 5.0,
        },
        Sprite {
            id: bullet_type.get_sprite_name(),
        },
        Transform {
            translation: initial_position,
            rotation: 0.0,
            scale: glam::vec2(7.0, 7.0),
        },
        BoundingBox {
            half_extend: glam::vec2(3.5, 3.5),
            collision_layer: CollisionLayer::PLAYER_BULLET,
            collision_mask: CollisionLayer::ENEMY,
        },
    ))
}

pub fn spawn_enemy_bullet(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    bullet_type: BulletType,
) -> hecs::Entity {
    world.spawn((
        Bullet {
            direction,
            speed: 5.0,
        },
        Sprite {
            id: bullet_type.get_sprite_name(),
        },
        Transform {
            translation: initial_position,
            rotation: 0.0,
            scale: glam::vec2(7.0, 7.0),
        },
        BoundingBox {
            half_extend: glam::vec2(3.5, 3.5),
            collision_layer: CollisionLayer::ENEMY_BULLET,
            collision_mask: CollisionLayer::PLAYER,
        },
    ))
}
