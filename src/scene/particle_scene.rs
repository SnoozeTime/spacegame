//! Just a scene to experiment with particles.

use crate::core::colors::RgbColor;
use crate::core::scene::{Scene, SceneResult};
use crate::event::GameEvent;
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use crate::{HEIGHT, WIDTH};
use bitflags::_core::time::Duration;
use hecs::{Entity, World};
use std::path::PathBuf;

pub struct ParticleScene {
    entity: Option<Entity>,
    particle_emitter: ParticleEmitter,
    reload: bool,
    filename: PathBuf,
}

impl ParticleScene {
    pub fn new(filename: PathBuf) -> Self {
        let mut emitter: ParticleEmitter =
            serde_json::from_str(&std::fs::read_to_string(&filename).unwrap()).unwrap();
        emitter.init_pool();

        Self {
            entity: None,
            particle_emitter: emitter,
            reload: false,
            filename,
        }
    }
}

impl Scene for ParticleScene {
    fn on_create(&mut self, world: &mut World, _resources: &mut Resources) {
        self.entity = Some(world.spawn((self.particle_emitter.clone(),)));
    }

    fn update(&mut self, _dt: Duration, world: &mut World, _resources: &Resources) -> SceneResult {
        if self.reload {
            // remove entity, reload emitter from file and spawn the new emitter.
            world.despawn(self.entity.unwrap());

            let mut emitter: ParticleEmitter =
                serde_json::from_str(&std::fs::read_to_string(&self.filename).unwrap()).unwrap();
            emitter.init_pool();

            self.particle_emitter = emitter;
            self.entity = Some(world.spawn((self.particle_emitter.clone(),)));

            self.reload = false;
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
