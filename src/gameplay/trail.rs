use crate::core::colors::RgbColor;
use crate::core::transform::Transform;
use crate::gameplay::physics::DynamicBody;
use crate::render::particle::{EmitterSource, ParticleEmitter};
use glam::Vec2;
use hecs::World;

pub struct Trail {
    pub should_display: bool,
}

pub fn update_trails(world: &mut World) {
    for (_, (trail, transform, emitter, _body)) in world
        .query::<(&Trail, &Transform, &mut ParticleEmitter, &DynamicBody)>()
        .iter()
    {
        if trail.should_display {
            emitter.angle_range = (
                std::f32::consts::FRAC_PI_2 - transform.rotation - 0.1,
                std::f32::consts::FRAC_PI_2 - transform.rotation + 0.1,
            );

            let dir = glam::Mat2::from_angle(transform.rotation) * glam::Vec2::unit_y();
            emitter.position_offset = -dir * transform.scale.y() / 2.0;

            emitter.enable();
        } else {
            emitter.disable();
        }
    }
}
