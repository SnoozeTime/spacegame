use glam::Vec2;
use hecs::World;

/// Camera to display stuff to the screen. If main is true, then it will be used for the rendering.
/// If multiple main camera, then the first one will be used.
#[derive(Debug)]
pub struct Camera {
    pub main: bool,
    pub position: glam::Vec2,
}

impl Camera {
    pub fn new() -> Camera {
        Self {
            main: true,
            position: Vec2::zero(),
        }
    }

    pub fn to_view(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(
            self.position.extend(1.0),
            self.position.extend(0.0),
            glam::Vec3::unit_y(),
        )
    }
}

pub fn get_view_matrix(world: &World) -> Option<glam::Mat4> {
    world
        .query::<&Camera>()
        .iter()
        .map(|(_, c)| c.to_view())
        .next()
}
