use crate::assets::prefab::PrefabManager;
use crate::assets::Handle;
use crate::core::colors;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::bullet::{spawn_enemy_bullet, spawn_missile, BulletType};
use crate::gameplay::collision::{CollisionLayer, CollisionWorld};
use crate::gameplay::health::HitDetails;
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::{get_player, Player};
use crate::gameplay::steering::behavior::{avoid_obstacles, follow_player, follow_random_path};
use crate::render::path::debug;
use crate::resources::Resources;
use hecs::World;
use log::{debug, trace};
use luminance_glfw::GlfwSurface;
use serde_derive::{Deserialize, Serialize};
use shrev::EventChannel;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub speed: f32,
    pub scratch_drop: (u32, u32),
    pub movement: MovementBehavior,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(4.0)),
            speed: 10.0,
            scratch_drop: (10, 50),
            movement: MovementBehavior::Follow,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MovementBehavior {
    /// Move toward the player. Avoid basic obstacles
    Follow,
    /// Follow a path. Each point will be randomly generated.
    RandomPath(glam::Vec2, bool),
    /// Do not move.
    Nothing,
}

impl Default for MovementBehavior {
    fn default() -> Self {
        Self::Follow
    }
}

impl MovementBehavior {
    pub fn apply(
        &mut self,
        e: hecs::Entity,
        t: &mut Transform,
        body: &mut DynamicBody,
        maybe_player: Option<glam::Vec2>,
        resources: &Resources,
    ) {
        match self {
            Self::Nothing => (),
            Self::Follow => {
                follow_player(t, body, maybe_player, resources);
                avoid_obstacles(e, t, body, resources);
            }
            Self::RandomPath(ref mut target, ref mut is_init) => {
                follow_random_path(target, is_init, t, body, resources);
                avoid_obstacles(e, t, body, resources);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnemyType {
    FollowPlayer(Timer),
    Satellite(Satellite),
    Boss1(Boss1),
    /// Drop some mines like an asshole.
    MineLander(Timer),
    /// Will explode when player comes near,
    Mine {
        /// Distance from the player below which the mine will be triggered
        trigger_distance: f32,
        /// Time from when the mine is triggered until the mine explode.
        explosion_timer: Timer,
    },
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
pub struct Route {
    pub path: Vec<glam::Vec2>,

    #[serde(default)]
    pub current: usize,
}

impl Route {
    pub fn get_current(&self) -> Option<glam::Vec2> {
        self.path.get(self.current).map(|v| *v)
    }

    pub fn go_to_next(&mut self) {
        self.current = (self.current + 1) % self.path.len()
    }

    pub fn debug_draw(&self, resources: &Resources) {
        if self.path.len() == 0 {
            return;
        }
        //
        for i in 0..self.path.len() - 1 {
            let p1 = self.path[i];
            let p2 = self.path[i + 1];

            debug::stroke_line(resources, p1, p2, colors::GREEN);
        }
    }
}

pub fn update_enemies(world: &mut World, resources: &Resources, dt: Duration) {
    trace!("update_enemies");
    let maybe_player = get_player(world);
    if let None = maybe_player {
        return;
    }
    let player = maybe_player.unwrap();

    let mut ev_channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();

    // prefabs to spawn.
    let mut to_spawn: Vec<(String, glam::Vec2)> = vec![];
    let mut bullets = vec![];
    let mut to_remove = vec![];
    let mut missiles = vec![];
    let mut explosions = vec![];
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
        enemy.movement.apply(e, t, body, maybe_player, resources);
        //follow_player(t, body, maybe_player, resources);
        //avoid_obstacles(e, t, body, resources);
        // Basic movement.
        if let Some(player_position) = maybe_player {
            let dir = player_position - t.translation;
            match enemy.enemy_type {
                EnemyType::Boss1(ref mut boss1) => {
                    if boss1.should_shoot() {
                        boss1.shoot_timer.tick(dt);
                        if boss1.shoot_timer.finished() {
                            boss1.shoot_timer.reset();

                            // shoot.
                            let to_spawn = (t.translation, dir.normalize(), BulletType::Round2);
                            bullets.push(to_spawn);
                            ev_channel.single_write(GameEvent::PlaySound(
                                "sounds/scifi_kit/Laser/Laser_01.wav".to_string(),
                            ));

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
                            ev_channel.single_write(GameEvent::PlaySound(
                                "sounds/scifi_kit/Laser/Laser_01.wav".to_string(),
                            ));
                            bullets.push(to_spawn);
                        }
                    }

                    // Draw stuff to the screen.
                    debug::stroke_circle(resources, t.translation, 1500.0, colors::RED);
                }
                EnemyType::MineLander(ref mut timer) => {
                    timer.tick(dt);
                    if timer.finished() {
                        timer.reset();
                        to_spawn.push(("mine".to_string(), t.translation));
                    }
                }
                EnemyType::Mine {
                    trigger_distance,
                    ref mut explosion_timer,
                } => {
                    if explosion_timer.enabled {
                        explosion_timer.tick(dt);
                        if explosion_timer.finished() {
                            // badaboum
                            explosions.push((e, t.translation, trigger_distance));
                            to_remove.push(GameEvent::Delete(e));
                        }
                        debug::stroke_circle(
                            resources,
                            t.translation,
                            trigger_distance,
                            colors::GREEN,
                        );
                    } else {
                        if (player_position - t.translation).length() < trigger_distance {
                            // BOOM
                            explosion_timer.reset();
                            explosion_timer.start();
                        }
                        debug::stroke_circle(
                            resources,
                            t.translation,
                            trigger_distance,
                            colors::RED,
                        );
                    }
                }
            }
        }
    }

    {
        let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
        for (prefab, pos) in to_spawn {
            if let Some(prefab) = prefab_manager.get(&Handle(prefab)) {
                prefab.execute(|prefab| {
                    prefab.spawn_at_pos(world, pos);
                });
            }
        }
    }

    for (pos, dir, bullet) in bullets {
        debug!("Will spawn bullet at position ={:?}", pos);
        spawn_enemy_bullet(
            world,
            pos,
            dir,
            bullet,
            HitDetails {
                hit_points: 1.0,
                is_crit: false,
            },
        );
    }

    for (pos, dir, entity) in missiles {
        spawn_missile(
            world,
            pos,
            dir,
            entity,
            CollisionLayer::PLAYER | CollisionLayer::PLAYER_BULLET,
        );
    }

    // process explosions.
    //
    {
        let collision_world = resources.fetch::<CollisionWorld>().unwrap();
        for (mine, pos, radius) in explosions {
            info!("EXPLOSION HAPPENED AT {:?}", (pos, radius));
            let entity_touched = collision_world.circle_query(pos, radius);
            info!("Entities in explosion = {:?}", entity_touched);

            for e in entity_touched {
                if e == mine {
                    continue;
                }

                // Apply force from center to position.
                let mut query = world
                    .query_one::<(&Transform, &mut DynamicBody)>(e)
                    .unwrap();

                if let Some((t, body)) = query.get() {
                    let force = (t.translation - pos).normalize() * 500.0;
                    debug::stroke_line(
                        resources,
                        t.translation,
                        t.translation + force,
                        colors::RED,
                    );
                    body.add_impulse(force);
                } else {
                    info!("No transform and body for entity");
                }

                if world.get::<Player>(e).is_ok() {
                    info!("Hit the player");
                }
            }
        }
    }

    ev_channel.drain_vec_write(&mut to_remove);
    trace!("Finished update_enemies")
}
