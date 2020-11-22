use crate::core::colors;
use crate::core::transform::Transform;
use crate::render::path::debug;
use crate::resources::Resources;
use hecs::World;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicBody {
    /// current forces applied to the body. These should be reset every frame and recomputed.
    #[serde(default)]
    pub forces: Vec<glam::Vec2>,

    #[serde(default)]
    pub impulses: Vec<glam::Vec2>,

    /// Current velocity of the body.
    pub velocity: glam::Vec2,

    /// Maximum velocity of the body.
    pub max_velocity: f32,

    /// mass of the body. is used to compute velocity from forces (ma = F)
    pub mass: f32,

    /// maximum amount of force to apply to the body (e.g 500)
    pub max_force: f32,
}

impl Default for DynamicBody {
    fn default() -> Self {
        Self {
            forces: vec![],
            impulses: vec![],
            velocity: Default::default(),
            max_velocity: 0.0,
            mass: 0.0,
            max_force: 500.0,
        }
    }
}

impl DynamicBody {
    pub fn add_force(&mut self, force: glam::Vec2) {
        if force.length_squared() > 0.0 {
            self.forces.push(force);
        }
    }

    pub fn add_impulse(&mut self, impulse: glam::Vec2) {
        self.impulses.push(impulse);
    }
}

#[derive(Debug, Default, Clone)]
pub struct PhysicConfig {
    pub damping: f32,
}

#[derive(Debug, Default, Clone)]
pub struct PhysicSystem {
    config: PhysicConfig,
}

impl PhysicSystem {
    pub fn new(config: PhysicConfig) -> Self {
        Self { config }
    }

    pub fn update(&self, world: &mut World, dt: Duration, resources: &Resources) {
        for (_e, (transform, body)) in world.query::<(&mut Transform, &mut DynamicBody)>().iter() {
            // acceleration is sum of all forces divided by the mass
            let mut sum_force = body
                .forces
                .drain(..)
                .rfold(glam::Vec2::zero(), |a, b| a + b);

            if sum_force.length() > body.max_force {
                sum_force = sum_force.normalize() * body.max_force;
            }
            let acc = sum_force / body.mass;
            let sum_impulses = body
                .impulses
                .drain(..)
                .rfold(glam::Vec2::zero(), |a, b| a + b);

            // integrate.
            body.velocity += dt.as_secs_f32() * acc + sum_impulses;
            if body.velocity.length() > body.max_velocity {
                body.velocity = body.velocity.normalize() * body.max_velocity;
            }
            if acc.length_squared() == 0.0 {
                body.velocity *= self.config.damping;
            }
            transform.translate(body.velocity * dt.as_secs_f32());

            debug::stroke_line(
                resources,
                transform.translation,
                transform.translation + body.velocity,
                colors::GREEN,
            );
        }
    }
}
