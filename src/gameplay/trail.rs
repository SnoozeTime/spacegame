use crate::core::colors::RgbColor;
use crate::core::transform::Transform;
use crate::gameplay::physics::DynamicBody;
use crate::render::particle::{EmitterSource, ParticleEmitter};
use glam::Vec2;
use hecs::World;

pub struct Trail;

pub fn update_trails(world: &mut World) {
    for (_, (_trail, transform, emitter, body)) in world
        .query::<(&Trail, &Transform, &mut ParticleEmitter, &DynamicBody)>()
        .iter()
    {
        let dir = glam::Mat2::from_angle(transform.rotation) * glam::Vec2::unit_y();

        emitter.source = EmitterSource::Point(transform.translation - transform.scale.x() * dir);
        emitter.angle_range = (-transform.rotation - 0.1, -transform.rotation + 0.1)
    }
}
