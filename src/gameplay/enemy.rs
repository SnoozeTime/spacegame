use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::{spawn_enemy_bullet, BulletType};
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::health::Health;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use hecs::World;
use log::{debug, trace};
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    enemy_type: EnemyType,
    speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyType {
    /// Move in a straight line.
    Straight,
    ProtoShip(ProtoShip),

    /// follow a path and crash in the player.
    KamikazeRandom(Path),

    /// moves slowly, shoots a ton of bullets.
    Spammer {
        path: Path,
        shoot_timer: Timer,
        bullet_timeout: Timer,
        shooting: bool,
    },
}

impl EnemyType {
    fn get_sprite(&self) -> String {
        match *self {
            EnemyType::Straight => "Enemy2.png",
            EnemyType::ProtoShip(_) => "Proto-ship.png",
            EnemyType::KamikazeRandom(_) => "Enemy3.png",
            EnemyType::Spammer { .. } => "EnemyBoss2.png",
        }
        .to_string()
    }

    fn get_scale(&self) -> glam::Vec2 {
        match *self {
            EnemyType::Straight => glam::vec2(50.0, 50.0),
            EnemyType::ProtoShip(_) => glam::vec2(50.0, 50.0),
            EnemyType::KamikazeRandom(_) => glam::vec2(25.0, 25.0),
            EnemyType::Spammer { .. } => glam::vec2(75.0, 75.0),
        }
    }

    fn get_speed(&self) -> f32 {
        match *self {
            EnemyType::Straight => 2.0,
            EnemyType::ProtoShip(_) => 2.0,
            EnemyType::KamikazeRandom(_) => 4.0,
            EnemyType::Spammer { .. } => 0.5,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ProtoShip {
    pub current_target: glam::Vec2,
    pub speed: f32,
    pub wait_time: f32,
    pub current_timed_waited: f32,
    pub elapsed_from_beginning: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub path: Vec<glam::Vec2>,

    #[serde(default)]
    pub current: usize,
}

pub fn update_enemies(world: &mut World, resources: &Resources, dt: Duration) {
    trace!("update_enemies");

    let mut ev_channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

    let mut bullets = vec![];
    let mut to_remove = vec![];
    for (e, (t, enemy)) in world.query::<(&mut Transform, &mut Enemy)>().iter() {
        match enemy.enemy_type {
            EnemyType::Straight => t.translation -= glam::Vec2::unit_y() * enemy.speed,
            EnemyType::ProtoShip(ref mut proto) => {
                // If not close to target, let's move the ship.
                if (proto.current_target - t.translation).length_squared() > 25.0 {
                    let d = (proto.current_target - t.translation).normalize();
                    t.translation += d * enemy.speed
                } else {
                    // find another target.
                    let mut rng = rand::thread_rng();
                    let x_next = rng.gen_range(10, WIDTH - 10) as f32;
                    let y_next = rng.gen_range(200, HEIGHT - 50) as f32;
                    proto.current_target = glam::vec2(x_next, y_next);
                }

                // update timers to decide when to shoot.
                proto.current_timed_waited += dt.as_secs_f32();
                proto.elapsed_from_beginning += dt.as_secs_f32();
                debug!("Time elapsed for proto = {:?}", proto.current_timed_waited);
                if proto.current_timed_waited > proto.wait_time {
                    proto.current_timed_waited = 0.0;
                    bullets.push((t.translation, -glam::Vec2::unit_y(), BulletType::Round2));
                }

                if proto.elapsed_from_beginning > 10.0 {
                    enemy.enemy_type = EnemyType::Straight;
                }
            }
            EnemyType::KamikazeRandom(ref mut p) => {
                if let Some(current_target) = p.path.get_mut(p.current) {
                    if (*current_target - t.translation).length_squared() > 25.0 {
                        let d = (*current_target - t.translation).normalize();
                        t.translation += d * enemy.speed
                    } else {
                        // find another target.
                        p.current += 1;
                    }
                } else {
                    // if no more target, just go straight !
                    enemy.enemy_type = EnemyType::Straight;
                }
            }
            EnemyType::Spammer {
                path: ref mut p,
                ref mut shooting,
                ref mut shoot_timer,
                ref mut bullet_timeout,
            } => {
                if !*shooting {
                    if let Some(current_target) = p.path.get_mut(p.current) {
                        if (*current_target - t.translation).length_squared() > 25.0 {
                            let d = (*current_target - t.translation).normalize();
                            t.translation += d * enemy.speed
                        } else {
                            // WAIT AND SHOOT !!!!.
                            *shooting = true;
                            p.current += 1;
                            shoot_timer.reset();
                            bullet_timeout.reset();
                            shoot_timer.start();
                            bullet_timeout.start();
                        }
                    } else {
                        // if no more target, just go straight !
                        enemy.enemy_type = EnemyType::Straight;
                    }
                } else {
                    // spawn some bullets.
                    shoot_timer.tick(dt);
                    bullet_timeout.tick(dt);

                    if bullet_timeout.finished() {
                        for i in 0..16 {
                            bullets.push((
                                t.translation,
                                glam::Mat2::from_angle(i as f32 * std::f32::consts::PI / 8.0)
                                    * glam::Vec2::unit_y(),
                                BulletType::Round2,
                            ));
                        }
                        bullet_timeout.reset();
                    }

                    if shoot_timer.finished() {
                        *shooting = false;
                    }
                }
            }
        }

        if t.translation.y() < 0.0 {
            to_remove.push(GameEvent::Delete(e));
        }
    }

    for (pos, dir, bullet) in bullets {
        debug!("Will spawn bullet at position ={:?}", pos);
        spawn_enemy_bullet(world, pos, dir, bullet);
    }

    ev_channel.drain_vec_write(&mut to_remove);
    trace!("Finished update_enemies")
}

pub fn spawn_enemy(world: &mut World, health: u32, position: glam::Vec2, enemy_type: EnemyType) {
    world.spawn((
        Transform {
            translation: position,
            rotation: 3.14,
            scale: enemy_type.get_scale(),
        },
        Sprite {
            id: enemy_type.get_sprite(),
        },
        BoundingBox {
            half_extend: enemy_type.get_scale() / 2.0,
            collision_layer: CollisionLayer::ENEMY,
            collision_mask: CollisionLayer::PLAYER_BULLET | CollisionLayer::PLAYER,
        },
        Health::new(health, Timer::of_seconds(0.5)),
        Enemy {
            speed: enemy_type.get_speed(),
            enemy_type,
        },
    ));
}
