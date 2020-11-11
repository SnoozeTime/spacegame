use crate::core::colors;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::{spawn_enemy_bullet, spawn_missile, BulletType};
use crate::gameplay::collision::{BoundingBox, CollisionLayer, CollisionWorld};
use crate::gameplay::health::Health;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::{get_player, Player};
use crate::gameplay::steering::{avoid, halt, seek};
use crate::gameplay::trail::Trail;
use crate::render::particle::ParticleEmitter;
use crate::render::path::debug;
use crate::render::sprite::Sprite;
use crate::resources::Resources;
use hecs::World;
use log::{debug, trace};
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    enemy_type: EnemyType,
    speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyType {
    FollowPlayer(Timer),
    Satellite(Satellite),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satellite {
    /// Time between missiles
    shoot_timer: Timer,
    /// Detection distance for player
    shoot_distance: f32,
}

impl Default for Satellite {
    fn default() -> Self {
        Self {
            shoot_distance: 500.0,
            shoot_timer: Timer::of_seconds(5.0),
        }
    }
}

impl EnemyType {
    fn get_sprite(&self) -> String {
        match *self {
            EnemyType::Satellite(_) => "sat.png",
            EnemyType::FollowPlayer(_) => "Enemy2.png",
        }
        .to_string()
    }

    fn get_scale(&self) -> glam::Vec2 {
        match *self {
            EnemyType::Satellite(_) => glam::vec2(20.0, 20.0),
            EnemyType::FollowPlayer(_) => glam::vec2(50.0, 50.0),
        }
    }

    fn get_speed(&self) -> f32 {
        match *self {
            EnemyType::Satellite(_) => 0.0,
            EnemyType::FollowPlayer(_) => 2.0,
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
    let maybe_player = get_player(world);
    if let None = maybe_player {
        return;
    }
    let player = maybe_player.unwrap();

    let mut ev_channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

    let mut bullets = vec![];
    let mut to_remove = vec![];
    let mut missiles = vec![];

    let maybe_player = world
        .query::<(&Player, &Transform)>()
        .iter()
        .map(|(_, (_, t))| t.translation)
        .next();
    for (e, (t, enemy, body)) in world
        .query::<(&mut Transform, &mut Enemy, &mut DynamicBody)>()
        .iter()
    {
        match enemy.enemy_type {
            EnemyType::Satellite(ref mut sat) => {
                // If the player is really close, will shoot some missiles.
                if let Some(player_position) = maybe_player {
                    sat.shoot_timer.tick(dt);
                    let mut dir = player_position - t.translation;
                    if dir.length() < sat.shoot_distance {
                        if sat.shoot_timer.finished() {
                            sat.shoot_timer.reset();
                            dir = dir.normalize();
                            missiles.push((
                                t.translation + dir * t.scale.x() * 2.0, // TODO better spawn points
                                dir,
                                player,
                            ));
                        }
                    }

                    // face player.
                    let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
                    let angle_to_perform = (player_position - t.translation).angle_between(dir);
                    t.rotation -= 0.05 * angle_to_perform;
                }
            }
            EnemyType::FollowPlayer(ref mut shoot_timer) => {
                if let Some(player_position) = maybe_player {
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

                    {
                        let collision_world = resources.fetch::<CollisionWorld>().unwrap();
                        if body.velocity.length() > 0.0 {
                            if let Some(f) =
                                avoid(e, t, body.velocity, 300.0, &*collision_world, 300.0)
                            {
                                body.add_force(f);
                                debug::stroke_line(
                                    resources,
                                    t.translation,
                                    t.translation + f,
                                    colors::BLUE,
                                );
                            }
                        }
                    }

                    body.add_force(steering);
                    debug::stroke_line(
                        resources,
                        t.translation,
                        t.translation + steering,
                        colors::RED,
                    );

                    // rotate toward the player
                    let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
                    let angle_to_perform = (player_position - t.translation).angle_between(dir);
                    t.rotation -= 0.05 * angle_to_perform;

                    // shoot if it's time.
                    shoot_timer.tick(dt);
                    if shoot_timer.finished() {
                        shoot_timer.reset();
                        let to_spawn = (t.translation, dir.normalize(), BulletType::Round2);
                        bullets.push(to_spawn);
                    }

                    // Draw stuff to the screen.
                    debug::stroke_circle(resources, t.translation, 300.0, colors::RED);
                }
            }
        }
    }

    for (pos, dir, bullet) in bullets {
        debug!("Will spawn bullet at position ={:?}", pos);
        spawn_enemy_bullet(world, pos, dir, bullet);
    }

    for (pos, dir, entity) in missiles {
        spawn_missile(world, pos, dir, entity);
    }

    ev_channel.drain_vec_write(&mut to_remove);
    trace!("Finished update_enemies")
}

pub fn spawn_enemy(world: &mut World, health: u32, position: glam::Vec2, enemy_type: EnemyType) {
    let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());

    let mut enemy_emitter: ParticleEmitter = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(base_path).join("particle/enemy_trail.json"))
            .unwrap(),
    )
    .unwrap();
    enemy_emitter.init_pool();

    world.spawn((
        DynamicBody {
            forces: vec![],
            velocity: Default::default(),
            max_velocity: 100.0,
            mass: 1.0,
        },
        Transform {
            translation: position,
            rotation: 3.14,
            scale: enemy_type.get_scale(),
            dirty: false,
        },
        Sprite {
            id: enemy_type.get_sprite(),
        },
        BoundingBox {
            half_extend: enemy_type.get_scale() / 2.0,
            collision_layer: CollisionLayer::ENEMY,
            collision_mask: CollisionLayer::PLAYER_BULLET
                | CollisionLayer::PLAYER
                | CollisionLayer::ASTEROID
                | CollisionLayer::MISSILE,
        },
        Health::new(health, Timer::of_seconds(0.5)),
        Enemy {
            speed: enemy_type.get_speed(),
            enemy_type,
        },
        Trail {
            should_display: true,
        },
        enemy_emitter,
    ));
}
