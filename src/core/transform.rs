use glam::Quat;

/// Transform of an element to place it on the screen
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Translation along x-y
    pub translation: glam::Vec2,
    /// Scale along x-y
    pub scale: glam::Vec2,
    /// rotation along z
    pub rotation: f32,
}

impl Transform {
    /// Get the model matrix for the transform
    pub fn to_model(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale.extend(0.0),
            Quat::from_rotation_z(self.rotation),
            self.translation.extend(0.0),
        )
    }
}
