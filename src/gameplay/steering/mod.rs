use rand::Rng;

/// Will compute the force to move towards a target without slowing down.
///
/// # Returns
/// the force to apply to the entity
pub fn seek(
    position: glam::Vec2,
    velocity: glam::Vec2,
    target: glam::Vec2,
    max_speed: f32,
) -> glam::Vec2 {
    (max_speed * (target - position).normalize()) - velocity
}

pub fn wander(velocity: glam::Vec2, wander_strength: f32) -> glam::Vec2 {
    let circle_center = velocity.normalize() * 20.0; // TODO Circle distance somewhere else.
    let mut rng = rand::thread_rng();
    let displacement = glam::Mat2::from_angle(rng.gen_range(0.0, 2.0 * std::f32::consts::PI))
        * glam::Vec2::unit_x()
        * wander_strength;

    circle_center + displacement
}

pub fn halt(velocity: glam::Vec2) -> glam::Vec2 {
    -velocity
}
