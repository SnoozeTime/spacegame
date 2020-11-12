//! Just a scene to experiment with particles.

use crate::core::camera::{screen_to_world, ProjectionMatrix};
use crate::core::input::Input;
use crate::core::scene::{Scene, SceneResult};
use crate::core::transform::Transform;
use crate::gameplay::Action;
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use bitflags::_core::time::Duration;
use glam::Vec2;
use hecs::{Entity, World};
use std::path::PathBuf;

pub struct ParticleScene {
    entity: Option<Entity>,
    particle_emitter: ParticleEmitter,
    reload: bool,
    filename: PathBuf,
    should_follow: bool,
}

impl ParticleScene {
    pub fn new(filename: PathBuf, should_follow: bool) -> Self {
        let mut emitter: ParticleEmitter =
            serde_json::from_str(&std::fs::read_to_string(&filename).unwrap()).unwrap();
        emitter.init_pool();

        Self {
            entity: None,
            particle_emitter: emitter,
            reload: false,
            filename,
            should_follow,
        }
    }
}

impl Scene for ParticleScene {
    fn on_create(&mut self, world: &mut World, _resources: &mut Resources) {
        let t = Transform {
            translation: Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
            rotation: 0.0,
            scale: Vec2::one(),
            dirty: false,
        };
        self.entity = Some(world.spawn((self.particle_emitter.clone(), t)));
    }

    fn update(&mut self, _dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        if self.reload {
            // remove entity, reload emitter from file and spawn the new emitter.
            let _ = world.despawn(self.entity.unwrap());

            let mut emitter: ParticleEmitter =
                serde_json::from_str(&std::fs::read_to_string(&self.filename).unwrap()).unwrap();
            emitter.init_pool();

            self.particle_emitter = emitter;
            let t = Transform {
                translation: Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
                rotation: 0.0,
                scale: Vec2::one(),
                dirty: false,
            };
            self.entity = Some(world.spawn((self.particle_emitter.clone(), t)));

            self.reload = false;
        }

        {
            let input = resources.fetch::<Input<Action>>().unwrap();
            let proj = resources.fetch::<ProjectionMatrix>().unwrap();
            if input.is_just_pressed(Action::Shoot) || self.should_follow {
                let new_pos = screen_to_world(input.mouse_position(), proj.0, world);
                let mut transform = world.get_mut::<Transform>(self.entity.unwrap()).unwrap();
                transform.translation = new_pos;
            }
        }

        SceneResult::Noop
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        resources: &Resources,
    ) -> Option<Gui> {
        let gui_context = resources.fetch::<GuiContext>().unwrap();
        let mut gui = gui_context.new_frame();
        if gui.button(glam::vec2(10.0, 10.0), None, "Reload".to_string()) {
            self.reload = true;
        }

        Some(gui)
    }
}
