use glam::{Mat3, Mat4, Quat, Vec2};
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Transform of an element to place it on the screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// Translation along x-y
    pub translation: Vec2,
    /// Scale along x-y
    pub scale: Vec2,
    /// rotation along z
    pub rotation: f32,

    #[serde(default = "default_dirty")]
    pub dirty: bool,
}

impl Transform {
    /// Get the model matrix for the transform
    pub fn to_model(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(0.0),
            Quat::from_rotation_z(self.rotation),
            self.translation.extend(0.0),
        )
    }

    pub fn to_mat(&self) -> glam::Mat3 {
        glam::Mat3::from_scale_angle_translation(self.scale, self.rotation, self.translation)
    }

    pub fn translate(&mut self, translation: Vec2) {
        self.translation += translation;
        self.dirty = true;
    }
}

/// Transform relative the the parent component.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalTransform {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32,

    #[serde(default = "default_dirty")]
    pub dirty: bool,
}

impl From<Transform> for LocalTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation,
            scale: t.scale,
            rotation: t.rotation,
            dirty: true,
        }
    }
}

fn default_dirty() -> bool {
    true
}
impl LocalTransform {
    pub fn new(translation: Vec2, rotation: f32, scale: Vec2) -> Self {
        Self {
            translation,
            scale,
            rotation,
            dirty: true,
        }
    }

    pub fn to_model(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale.extend(0.0),
            Quat::from_rotation_z(self.rotation),
            self.translation.extend(0.0),
        )
    }

    pub fn to_mat(&self) -> glam::Mat3 {
        glam::Mat3::from_scale_angle_translation(self.scale, self.rotation, self.translation)
    }
}

pub struct HasParent {
    pub entity: hecs::Entity,
}

pub struct HasChildren {
    pub children: Vec<hecs::Entity>,
}

pub fn update_transforms(world: &mut hecs::World) {
    let mut to_process = VecDeque::new();
    // first gather the entities to update.
    for (e, (transform, has_children)) in world.query::<(&mut Transform, &HasChildren)>().iter() {
        // Root entities.
        if let Ok(_) = world.get::<HasParent>(e) {
            continue;
        }

        debug!("Will process {:?}", e);

        // Process all parents even if their transform is not dirty. The reason is that children
        // can be moved independently, so we would need to update their children.
        for child in &has_children.children {
            to_process.push_back((transform.clone(), *child));
        }
        transform.dirty = false;
    }

    debug!("Local Transform to update = {:?}", to_process);
    // process in order of insertion.
    while let Some((t, child)) = to_process.pop_front() {
        let parent_matrix = t.to_mat();
        // First, calculate the new transform.
        let mut global_transform = world
            .get_mut::<Transform>(child)
            .expect("Child component should have a global transform");
        let mut local_transform = world
            .get_mut::<LocalTransform>(child)
            .expect("Child component should have a local transform");

        if local_transform.dirty || t.dirty {
            debug!("Will update transforms");
            // Need to recalculate the global transform.
            let local_matrix = local_transform.to_mat();
            debug!("Local Matrix = {:#?}", local_matrix);
            let new_global_matrix =
                Mat3::from_scale_angle_translation(Vec2::one(), t.rotation, t.translation)
                    * local_matrix;

            debug!("parent Matrix = {:#?}", parent_matrix);
            debug!("new global Matrix = {:#?}", new_global_matrix);

            let (rot, translation) = decompose_mat3(new_global_matrix);
            global_transform.rotation = rot;
            global_transform.translation = translation;
            global_transform.dirty = true;
        }

        if let Ok(children) = world.get::<HasChildren>(child) {
            for child_of_child in &children.children {
                to_process.push_back((*global_transform, *child_of_child));
            }
        }

        global_transform.dirty = false;
        local_transform.dirty = false;
    }
}

/// assume scale is always 1 to simplify. Only true in that specific case of course.
fn decompose_mat3(mat: Mat3) -> (f32, Vec2) {
    let translation = glam::vec2(mat.z_axis().x(), mat.z_axis().y());
    let angle = mat.x_axis().y().atan2(mat.x_axis().x());
    (angle, translation)
}
