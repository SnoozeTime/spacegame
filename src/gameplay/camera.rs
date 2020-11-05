use crate::core::camera::Camera;
use crate::core::transform::Transform;
use crate::core::window::WindowDim;
use crate::gameplay::player::Player;
use crate::resources::Resources;
use hecs::World;

pub fn update_camera(world: &mut World, resources: &Resources) {
    let dim = resources.fetch::<WindowDim>().unwrap();
    let pos = world
        .query::<(&Transform, &Player)>()
        .iter()
        .map(|(_, (t, _p))| t.translation)
        .next();
    if let Some(pos) = pos {
        if let Some((_, t)) = world.query::<&mut Camera>().iter().next() {
            t.position = pos; // - glam::vec2(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
            t.position
                .set_x((t.position.x().max(-800.0)).min(800.0) - dim.width as f32 / 2.0);
            t.position
                .set_y(t.position.y().max(-450.0).min(450.0) - dim.height as f32 / 2.0);
        }
    }
}
