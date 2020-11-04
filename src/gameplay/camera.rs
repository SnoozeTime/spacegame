use crate::core::camera::Camera;
use crate::core::transform::Transform;
use crate::gameplay::player::Player;
use crate::{HEIGHT, WIDTH};
use hecs::World;

pub fn update_camera(world: &mut World) {
    let pos = world
        .query::<(&Transform, &Player)>()
        .iter()
        .map(|(_, (t, _p))| t.translation)
        .next();
    if let Some(pos) = pos {
        if let Some((_, t)) = world.query::<&mut Camera>().iter().next() {
            t.position = pos - glam::vec2(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
        }
    }
}
