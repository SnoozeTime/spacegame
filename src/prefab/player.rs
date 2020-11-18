use crate::assets::prefab::Prefab;
use crate::core::transform::Transform;
use crate::gameplay::collision::BoundingBox;
use crate::gameplay::health::{Health, Shield};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::player::{Player, Stats, Weapon};
use crate::gameplay::trail::Trail;
use crate::render::particle::ParticleEmitter;
use crate::render::sprite::Sprite;
use hecs::{Entity, EntityBuilder, World};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PlayerPrefab {
    pub dynamic_body: DynamicBody,
    pub transform: Transform,
    pub sprite: Sprite,
    pub bounding_box: BoundingBox,
    pub health: Health,
    pub shield: Option<Shield>,
    pub trail: ParticleEmitter,
    pub stats: Stats,
}

#[typetag::serde]
impl Prefab for PlayerPrefab {
    fn spawn(&self, world: &mut World) -> Entity {
        let mut components = EntityBuilder::new();
        components.add(self.dynamic_body.clone());
        components.add(self.transform.clone());
        components.add(self.sprite.clone());
        components.add(self.bounding_box);
        components.add(self.health.clone());
        if let Some(s) = self.shield.clone() {
            components.add(s);
        }
        let mut particles = self.trail.clone();
        particles.init_pool();
        components.add(particles);
        components.add(Trail {
            should_display: true,
        });
        components.add(Player {
            weapon: Weapon::Simple,
            stats: self.stats.clone(),
            direction: glam::vec2(0.0, 1.0),
        });

        world.spawn(components.build())
    }
}
