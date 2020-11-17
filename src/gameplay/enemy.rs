use crate::assets::prefab::Prefab;
use crate::core::colors;
use crate::core::random::RandomGenerator;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::{spawn_enemy_bullet, spawn_missile, BulletType};
use crate::gameplay::collision::{BoundingBox, CollisionLayer, CollisionWorld};
use crate::gameplay::health::{Health, HitDetails};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::{get_player, Player};
use crate::gameplay::steering::{avoid, halt, seek};
use crate::gameplay::trail::Trail;
use crate::prefab::enemies::EnemyPrefab;
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
    pub enemy_type: EnemyType,
    pub speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(4.0)),
            speed: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyType {
    FollowPlayer(Timer),
    Satellite(Satellite),
    Boss1(Boss1),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boss1 {
    /// Time between shots
    pub shoot_timer: Timer,
    /// nb of time to shoot during a salve
    pub nb_shot: usize,
    /// nb of shots during current salve.
    pub current_shot: usize,
    /// timeout between salves.
    pub salve_timer: Timer,
}

impl Boss1 {
    fn should_shoot(&mut self) -> bool {
        self.nb_shot != self.current_shot
    }

    fn prepare_to_shoot(&mut self) {
        self.shoot_timer.reset();
        self.shoot_timer.start();
        self.salve_timer.reset();
        self.salve_timer.stop();
        self.current_shot = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Satellite {
    /// Time between missiles
    pub shoot_timer: Timer,
    /// Detection distance for player
    pub shoot_distance: f32,
}

impl Default for Satellite {
    fn default() -> Self {
        Self {
            shoot_distance: 500.0,
            shoot_timer: Timer::of_seconds(5.0),
        }
    }
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
    //let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();

    let maybe_player = world
        .query::<(&Player, &Transform)>()
        .iter()
        .map(|(_, (_, t))| t.translation)
        .next();
    for (e, (t, enemy, body)) in world
        .query::<(&mut Transform, &mut Enemy, &mut DynamicBody)>()
        .iter()
    {
        // Basic movement.
        if let Some(player_position) = maybe_player {
            let mut dir = player_position - t.translation;

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
                    if let Some(f) = avoid(e, t, body.velocity, 300.0, &*collision_world, 300.0) {
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
            {
                let dir = glam::Mat2::from_angle(t.rotation) * glam::Vec2::unit_y();
                let angle_to_perform = (player_position - t.translation).angle_between(dir);
                t.rotation -= 0.05 * angle_to_perform;
            }

            match enemy.enemy_type {
                EnemyType::Boss1(ref mut boss1) => {
                    if let Some(player_position) = maybe_player {
                        if boss1.should_shoot() {
                            boss1.shoot_timer.tick(dt);
                            if boss1.shoot_timer.finished() {
                                boss1.shoot_timer.reset();

                                // shoot.
                                let mut dir = player_position - t.translation;
                                let to_spawn = (t.translation, dir.normalize(), BulletType::Round2);
                                bullets.push(to_spawn);

                                boss1.current_shot += 1;
                            }
                        } else {
                            // if here, boss1 needs to wait before it is able to shoot again.
                            boss1.salve_timer.tick(dt);
                            if boss1.salve_timer.finished() {
                                boss1.prepare_to_shoot();
                            }
                        }
                    }
                }
                EnemyType::Satellite(ref mut sat) => {
                    // If the player is really close, will shoot some missiles.
                    sat.shoot_timer.tick(dt);
                    if dir.length() < sat.shoot_distance {
                        if sat.shoot_timer.finished() {
                            sat.shoot_timer.reset();
                            let norm_dir = dir.normalize();
                            missiles.push((
                                t.translation + norm_dir * t.scale.x() * 2.0, // TODO better spawn points
                                norm_dir,
                                player,
                            ));
                        }
                    }
                }
                EnemyType::FollowPlayer(ref mut shoot_timer) => {
                    // shoot if it's time.
                    shoot_timer.tick(dt);
                    if (player_position - t.translation).length() < 1000.0 {
                        if shoot_timer.finished() {
                            shoot_timer.reset();
                            let to_spawn = (t.translation, dir.normalize(), BulletType::Round2);
                            bullets.push(to_spawn);
                        }
                    }

                    // Draw stuff to the screen.
                    debug::stroke_circle(resources, t.translation, 1500.0, colors::RED);
                }
            }
        }
    }

    for (pos, dir, bullet) in bullets {
        debug!("Will spawn bullet at position ={:?}", pos);
        spawn_enemy_bullet(world, pos, dir, bullet, HitDetails { hit_points: 1.0 });
    }

    for (pos, dir, entity) in missiles {
        spawn_missile(world, pos, dir, entity);
    }

    ev_channel.drain_vec_write(&mut to_remove);
    trace!("Finished update_enemies")
}
