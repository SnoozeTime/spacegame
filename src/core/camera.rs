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
        glam::Mat4::look_at_rh(self.eye(), self.position.extend(0.0), glam::Vec3::unit_y())
    }

    pub fn eye(&self) -> glam::Vec3 {
        self.position.extend(1.0)
    }
}

pub fn get_view_matrix(world: &World) -> Option<glam::Mat4> {
    world
        .query::<&Camera>()
        .iter()
        .map(|(_, c)| c.to_view())
        .next()
}

pub fn screen_to_world(
    screen_coords: glam::Vec2,
    projection_matrix: glam::Mat4,
    world: &World,
) -> glam::Vec2 {
    let view = get_view_matrix(world).unwrap();
    let pv = projection_matrix * view;
    let inv = pv.inverse();
    let mouse_pos_world = inv * screen_coords.extend(0.0).extend(1.0);
    glam::vec2(mouse_pos_world.x(), mouse_pos_world.y())
}

#[derive(Copy, Clone, Debug)]
pub struct ProjectionMatrix(pub(crate) glam::Mat4);

impl ProjectionMatrix {
    pub fn new(w: f32, h: f32) -> Self {
        Self(glam::Mat4::orthographic_rh_gl(0.0, w, 0.0, h, -1.0, 10.0))
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.0 = glam::Mat4::orthographic_rh_gl(0.0, w, 0.0, h, -1.0, 10.0);
    }
}
