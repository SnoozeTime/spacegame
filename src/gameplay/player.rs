use super::bullet;
use crate::config::PlayerConfig;
use crate::core::audio;
use crate::core::camera::screen_to_world;
use crate::core::input::{Axis, Input};
use crate::core::random::RandomGenerator;
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::gameplay::bullet::spawn_missile;
use crate::gameplay::collision::CollisionLayer;
use crate::gameplay::enemy::Enemy;
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
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Weapon {
    Simple,
}

/// Tag to tell the ECS that the entity is a player.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub weapon: Weapon,
    pub direction: glam::Vec2,
    pub stats: Stats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    /// dmg per bullet.
    pub dmg: f32,
    /// between 0 and 100.
    pub crit_percent: u32,
    /// dmg multiplier (>1)
    pub crit_multiplier: f32,
    /// % change to shoot a missile when shooting
    pub missile_percent: u32,
    /// Time between boosts
    pub boost_timer: Timer,
    /// force of the boost
    pub boost_magnitude: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            dmg: 1.0,
            crit_percent: 5,
            crit_multiplier: 1.2,
            missile_percent: 0,
            boost_timer: Timer::of_seconds(1.0),
            boost_magnitude: 500.0,
        }
    }
}

impl Stats {
    fn is_crit<R: Rng>(&self, rand: &mut R) -> bool {
        let pick: u32 = rand.gen_range(0, 101);
        pick <= self.crit_percent
    }

    fn dmg(&self, is_crit: bool) -> f32 {
        let multiplier = if is_crit { self.crit_multiplier } else { 1.0 };
        multiplier * self.dmg
    }

    fn should_shoot_missile<R: Rng>(&self, rand: &mut R) -> bool {
        let pick: u32 = rand.gen_range(0, 101);
        pick < self.missile_percent
    }
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

pub fn update_player(world: &mut World, dt: Duration, resources: &Resources) {
    // only one player for now.
    let input = resources.fetch::<Input<Action>>().unwrap();
    let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
    let player_controller_conf = resources.fetch::<PlayerConfig>().unwrap();

    let projection_matrix =
        glam::Mat4::orthographic_rh_gl(0.0, WIDTH as f32, 0.0, HEIGHT as f32, -1.0, 10.0);

    let mut bullets = vec![];
    let mut missiles = vec![];

    let enemies = world
        .query::<(&Transform, &Enemy)>()
        .iter()
        .map(|(e, (t, _))| (e, *t))
        .collect::<Vec<_>>();

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

        trace!("Seek {:?}", target);
        let steering_force = steering::seek(
            transform.translation,
            dynamic.velocity,
            target,
            dynamic.max_velocity * delta_y.max(0.0),
        );
        trace!("steering force {:?}", steering_force);

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

        // boost
        player.stats.boost_timer.tick(dt);
        if player.stats.boost_timer.finished() {
            if input.is_just_pressed(Action::Boost) {
                player.stats.boost_timer.reset();
                dynamic.add_impulse(dir * player.stats.boost_magnitude);
            }
        }

        // Shoot stuff
        if input.is_just_pressed(Action::Shoot) {
            // shoot from the top.
            let initial_pos = transform.translation
                + glam::Mat2::from_angle(transform.rotation)
                    * glam::vec2(0.0, transform.scale.y() / 2.0);

            // calculate damages.
            let is_crit = player.stats.is_crit(random.rng());
            let dmg = player.stats.dmg(is_crit);

            if player.stats.should_shoot_missile(random.rng()) {
                if let Some(enemy) = enemies.first() {
                    missiles.push((initial_pos, dir, enemy.0));
                }
            }

            audio::play_sound(resources, "sounds/scifi_kit/Laser/Laser_09.wav");
            bullets = vec![(
                initial_pos,
                dir,
                bullet::BulletType::Twin,
                HitDetails {
                    hit_points: dmg,
                    is_crit,
                },
            )];
        }
    }

    bullets.iter().for_each(|(p, d, b, details)| {
        bullet::spawn_player_bullet(world, *p, *d, *b, *details);
    });

    missiles.iter().for_each(|(p, d, e)| {
        spawn_missile(
            world,
            *p,
            *d,
            *e,
            CollisionLayer::ENEMY | CollisionLayer::ENEMY_BULLET,
        );
    });

    trace!("finished update_player");
}
