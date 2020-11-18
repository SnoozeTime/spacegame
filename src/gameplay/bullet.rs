use crate::core::colors::RgbaColor;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::core::window::WindowDim;
use crate::event::GameEvent;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::health::{Health, HitDetails};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::steering::seek;
use crate::render::sprite::{Sprite, Tint};
use crate::resources::Resources;
use hecs::{Entity, World};
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
    pub details: HitDetails,
}

/// Missile is a physical bullet
pub struct Missile {
    /// If some, will then follow the entity :D
    pub home_to_entity: Option<Entity>,
}

pub fn process_missiles(world: &World, resources: &Resources) {
    let mut to_despawn = vec![];
    let window_dim = resources.fetch::<WindowDim>().unwrap();

    let max_width = 2.0 * window_dim.width as f32;
    let max_height = 2.0 * window_dim.height as f32;
    for (e, (t, missile, body)) in world
        .query::<(&mut Transform, &mut Missile, &mut DynamicBody)>()
        .iter()
    {
        // If should follow an entity, then apply some steering
        if let Some(to_home) = missile.home_to_entity {
            let target_pos = world.get::<Transform>(to_home);
            if let Ok(target_pos) = target_pos {
                let steering = seek(
                    t.translation,
                    body.velocity,
                    target_pos.translation,
                    body.max_velocity,
                );

                body.add_force(steering);

                // rotate toward the player
                let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
                let angle_to_perform = (target_pos.translation - t.translation).angle_between(dir);
                t.rotation -= 0.05 * angle_to_perform;
            } else {
                // no more entity, will just continue straight.
                body.add_force(body.velocity.normalize() * body.max_velocity);
            }
        } else {
            // no entity,  just go straight.
            body.add_force(body.velocity.normalize() * body.max_velocity);
        }

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
    hit_details: HitDetails,
) -> hecs::Entity {
    let angle = -direction.angle_between(glam::Vec2::unit_y());

    let e = world.spawn((
        Bullet {
            direction,
            speed: 20.0,
            alive: true,
            details: hit_details,
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
            collision_mask: Some(CollisionLayer::ENEMY | CollisionLayer::ASTEROID),
        },
    ));

    if hit_details.is_crit {
        world
            .insert_one(
                e,
                Tint {
                    color: RgbaColor::new(255, 0, 0, 255),
                },
            )
            .expect("cannot insert blink to bullet...")
    }

    e
}

pub fn spawn_enemy_bullet(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    bullet_type: BulletType,
    hit_details: HitDetails,
) -> hecs::Entity {
    let angle = -direction.angle_between(glam::Vec2::unit_y());
    world.spawn((
        Bullet {
            direction,
            speed: 10.0,
            alive: true,
            details: hit_details,
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
            collision_mask: Some(CollisionLayer::PLAYER | CollisionLayer::ASTEROID),
        },
    ))
}

pub fn spawn_missile(
    world: &mut World,
    initial_position: glam::Vec2,
    direction: glam::Vec2,
    target: Entity,
    mask: CollisionLayer,
) -> hecs::Entity {
    let angle = -direction.angle_between(glam::Vec2::unit_y());
    world.spawn((
        Missile {
            home_to_entity: Some(target),
        },
        Sprite {
            id: "fast_bullet.png".to_string(),
            // id: "missile.png".to_string(),
        },
        Transform {
            translation: initial_position,
            rotation: angle,
            scale: glam::vec2(7.0, 7.0),
            dirty: false,
        },
        DynamicBody {
            forces: vec![],
            velocity: direction * 80.0,
            max_velocity: 500.0,
            mass: 0.5,
            max_force: 200.0,
        },
        Health::new(1.0, Timer::of_seconds(1.0)),
        BoundingBox {
            half_extend: glam::vec2(7.0, 7.0),
            collision_layer: CollisionLayer::MISSILE,
            collision_mask: Some(mask | CollisionLayer::ASTEROID | CollisionLayer::MISSILE),
        },
    ))
}
