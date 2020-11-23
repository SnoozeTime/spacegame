use crate::render::sprite::Sprite;
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// One animation (in one spreadsheet).
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Animation {
    /// Keyframes element are sprite_index and number of frames to elapse for the current
    /// keyframe.
    pub keyframes: Vec<(String, usize)>,

    /// in frames
    pub current_index: usize,
    // in seconds
    pub elapsed_frame: usize,
}

impl Animation {
    pub fn new(keyframes: Vec<(String, usize)>) -> Self {
        Self {
            keyframes,
            current_index: 0,
            elapsed_frame: 0,
        }
    }
}

/// All Animations for an entity
/// Control what entity is active with current_animation
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AnimationController {
    /// Animation will cycle through the sprites on its spritesheet
    pub animations: HashMap<String, Animation>,

    /// if set to something, will play the corresponding animation
    pub current_animation: Option<String>,
}

pub struct AnimationSystem;

impl AnimationSystem {
    pub fn animate(&mut self, world: &mut hecs::World) {
        for (_, (controller, sprite)) in world
            .query::<(&mut AnimationController, &mut Sprite)>()
            .iter()
        {
            if let Some(ref animation_name) = controller.current_animation {
                if let Some(ref mut animation) = controller.animations.get_mut(animation_name) {
                    sprite.id = animation.keyframes[animation.current_index].0.clone();

                    animation.elapsed_frame += 1;
                    if animation.elapsed_frame > animation.keyframes[animation.current_index].1 {
                        animation.elapsed_frame = 0;
                        animation.current_index =
                            (animation.current_index + 1) % animation.keyframes.len();
                    }
                } else {
                    error!("Cannot find animation with name = {}", animation_name);
                }
            }
        }
    }
}
