use crate::core::transform::Transform;
use crate::core::window::WindowDim;
use crate::event::GameEvent;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use hecs::World;
use log::trace;
use shrev::EventChannel;

#[derive(Debug, Copy, Clone)]
pub enum BulletType {
    Small,
    Fast,
    Round1,
    Round2,
    Twin,
    BigAss,
}

impl BulletType {
    /// Get the name of the sprite that this bullet is representing
    fn get_sprite_name(&self) -> String {
        match *self {
            BulletType::Small => "small_bullet.png",
            BulletType::Fast => "fast_bullet.png",
            BulletType::Twin => "twin_bullets.png",
            BulletType::Round1 => "round_bullet.png",
            BulletType::Round2 => "round_bullet_2.png",
            BulletType::BigAss => "big_ass_bullet.png",
        }
        .to_string()
    }
}

/// Tag to indicate the entity is a bullet.
pub struct Bullet {
    pub direction: glam::Vec2,
    pub speed: f32,
    pub alive: bool,
}

/// Every frame, will move the bullet in the given direction at the given speed.
pub fn process_bullets(world: &World, resources: &Resources) {
    trace!("process_bullets");

    let window_dim = resources.fetch::<WindowDim>().unwrap();

    let max_width = 2.0 * window_dim.width as f32;
    let max_height = 2.0 * window_dim.height as f32;

    let mut to_despawn = vec![];
    for (e, (b, t)) in world.query::<(&Bullet, &mut Transform)>().iter() {
        t.translation += b.direction * b.speed;

        if t.translation.x() > max_width
            || t.translation.x() < -max_width
            || t.translation.y() > max_height
            || t.translation.y() < -max_height
        {
            to_despawn.push(GameEvent::Delete(e));
        }
    }

    {
        let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
        channel.drain_vec_write(&mut to_despawn)
    }
    trace!("finished process_bullets");
}

pub fn spawn_player_bullet(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    bullet_type: BulletType,
) -> hecs::Entity {
    let angle = -direction.angle_between(glam::Vec2::unit_y());

    world.spawn((
        Bullet {
            direction,
            speed: 20.0,
            alive: true,
        },
        Sprite {
            id: bullet_type.get_sprite_name(),
        },
        Transform {
            translation: initial_position,
            rotation: angle,
            scale: glam::vec2(7.0, 7.0),
            dirty: false,
        },
        BoundingBox {
            half_extend: glam::vec2(3.5, 3.5),
            collision_layer: CollisionLayer::PLAYER_BULLET,
            collision_mask: CollisionLayer::ENEMY | CollisionLayer::ASTEROID,
        },
    ))
}

pub fn spawn_enemy_bullet(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    bullet_type: BulletType,
) -> hecs::Entity {
    let angle = -direction.angle_between(glam::Vec2::unit_y());
    world.spawn((
        Bullet {
            direction,
            speed: 15.0,
            alive: true,
        },
        Sprite {
            id: bullet_type.get_sprite_name(),
        },
        Transform {
            translation: initial_position,
            rotation: angle,
            scale: glam::vec2(7.0, 7.0),
            dirty: false,
        },
        BoundingBox {
            half_extend: glam::vec2(3.5, 3.5),
            collision_layer: CollisionLayer::ENEMY_BULLET,
            collision_mask: CollisionLayer::PLAYER | CollisionLayer::ASTEROID,
        },
    ))
}
