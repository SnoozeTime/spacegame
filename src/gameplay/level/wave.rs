use crate::assets::prefab::{Prefab, PrefabManager};
use crate::assets::Handle;
use crate::core::random::RandomGenerator;
use crate::prefab::enemies::EnemyPrefab;
use crate::resources::Resources;
use hecs::Entity;
use luminance_glfw::GlfwSurface;
use rand::seq::SliceRandom;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WaveDescription {
    /// List of stuff to instantiate in the wave
    to_instantiate: Vec<String>,
}

impl From<WaveDescription> for Wave {
    fn from(wave_desc: WaveDescription) -> Self {
        Self {
            wave_desc,
            enemies: vec![],
            init: false,
        }
    }
}

pub struct Wave {
    wave_desc: WaveDescription,
    pub enemies: Vec<hecs::Entity>,
    init: bool,
}

impl Wave {
    pub fn new() -> Self {
        let wave_desc = WaveDescription {
            to_instantiate: vec!["base_enemy".to_string()],
        };
        Self {
            wave_desc,
            enemies: vec![],
            init: false,
        }
    }

    pub fn init(
        &mut self,
        world: &mut hecs::World,
        resources: &Resources,
        no_asteroids: &Vec<glam::Vec2>,
    ) {
        let mut random = resources.fetch_mut::<RandomGenerator>().unwrap();
        let to_instantiate = &self.wave_desc.to_instantiate;
        let enemies = &mut self.enemies;
        let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
        for prefab_name in to_instantiate {
            let pos = no_asteroids.choose(random.rng());
            if let Some(prefab) = prefab_manager.get(&Handle(prefab_name.clone())) {
                prefab.execute(|prefab| {
                    info!("Will spawn = {:?}", prefab);

                    enemies.push(prefab.spawn_at_pos(world, *pos.unwrap()));
                });
            } else {
                error!(
                    "Prefab {} should have been loaded in the loading scene",
                    prefab_name
                );
            }
        }

        self.init = true;
    }

    pub fn remove_enemy(&mut self, entity: Entity) {
        let maybe_index = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(idx, &e)| e == entity)
            .map(|(i, _)| i)
            .next();

        if let Some(idx) = maybe_index {
            self.enemies.remove(idx);
        }
    }

    pub fn is_finished(&self) -> bool {
        trace!(
            "is+finished = {} && {} = {}",
            self.init,
            self.enemies.is_empty(),
            self.init && self.enemies.is_empty()
        );
        self.init && self.enemies.is_empty()
    }
}
