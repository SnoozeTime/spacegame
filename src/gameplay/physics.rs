use crate::core::transform::Transform;
use crate::resources::Resources;
use hecs::World;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DynamicBody {
    /// current forces applied to the body. These should be reset every frame and recomputed.
    pub forces: Vec<glam::Vec2>,

    /// Current velocity of the body.
    pub velocity: glam::Vec2,

    /// Maximum velocity of the body.
    pub max_velocity: f32,

    /// mass of the body. is used to compute velocity from forces (ma = F)
    pub mass: f32,
    //TODO Max force
}

impl DynamicBody {
    pub fn add_force(&mut self, force: glam::Vec2) {
        if force.length_squared() > 0.0 {
            self.forces.push(force);
        }
    }
}

#[derive(Debug, Default)]
pub struct PhysicConfig {
    pub damping: f32,
}

#[derive(Debug, Default)]
pub struct PhysicSystem {
    config: PhysicConfig,
}

impl PhysicSystem {
    pub fn new(config: PhysicConfig) -> Self {
        Self { config }
    }

    pub fn update(&self, world: &mut World, dt: Duration, _resources: &Resources) {
        for (_, (transform, body)) in world.query::<(&mut Transform, &mut DynamicBody)>().iter() {
            // acceleration is sum of all forces divided by the mass
            let acc = body
                .forces
                .drain(..)
                .rfold(glam::Vec2::zero(), |a, b| a + b)
                / body.mass;

            // integrate.
            body.velocity += dt.as_secs_f32() * acc;
            if body.velocity.length() > body.max_velocity {
                body.velocity = body.velocity.normalize() * body.max_velocity;
            }
            if acc.length_squared() == 0.0 {
                body.velocity *= self.config.damping;
            }
            transform.translate(body.velocity * dt.as_secs_f32());
        }
    }
}
