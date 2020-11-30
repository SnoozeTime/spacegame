use crate::assets::prefab::Prefab;
use crate::core::animation::AnimationController;
use crate::core::transform::Transform;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::enemy::Enemy;
use crate::gameplay::health::{Health, Shield};
use crate::gameplay::physics::DynamicBody;
use crate::gameplay::trail::Trail;
use crate::render::particle::ParticleEmitter;
use crate::render::sprite::Sprite;
use hecs::EntityBuilder;
use serde_derive::{Deserialize, Serialize};

pub const ENEMY_PREFABS: [&str; 12] = [
    "base_enemy",
    "base_enemy_2",
    "base_enemy_3",
    "satellite",
    "boss1",
    "last_boss",
    "mine_lander",
    "mine",
    "wanderer",
    "spammer",
    "carrier",
    "kamikaze",
];

pub const ENEMY_STR_1: [&str; 4] = ["base_enemy", "base_enemy", "wanderer", "kamikaze"];
pub const ENEMY_STR_2: [&str; 4] = ["mine_lander", "satellite", "base_enemy_2", "spammer"];
pub const ENEMY_STR_3: [&str; 4] = ["boss1", "base_enemy_3", "carrier", "last_boss"];

#[derive(Debug, Serialize, Deserialize)]
pub struct EnemyPrefab {
    pub dynamic_body: DynamicBody,
    pub transform: Transform,
    pub sprite: Sprite,
    pub bounding_box: BoundingBox,
    pub health: Option<Health>,
    pub shield: Option<Shield>,
    pub enemy: Enemy,
    pub trail: Option<ParticleEmitter>,
    pub animation: Option<AnimationController>,
}

#[typetag::serde]
impl Prefab for EnemyPrefab {
    fn spawn(&self, world: &mut hecs::World) -> hecs::Entity {
        let mut components = EntityBuilder::new();
        components.add(self.dynamic_body.clone());
        components.add(self.transform.clone());
        components.add(self.sprite.clone());
        components.add(self.bounding_box);
        if let Some(h) = self.health.clone() {
            components.add(h);
        }
        if let Some(s) = self.shield.clone() {
            components.add(s);
        }
        components.add(self.enemy.clone());
        if let Some(mut particles) = self.trail.clone() {
            particles.init_pool();
            components.add(particles);
            components.add(Trail {
                should_display: true,
                offset: 0.0,
            });
        }
        if let Some(animation) = self.animation.clone() {
            components.add(animation);
        }
        world.spawn(components.build())
    }
}

impl Default for EnemyPrefab {
    fn default() -> Self {
        Self {
            dynamic_body: DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 0.0,
                mass: 0.0,
                max_force: 500.0,
            },
            transform: Transform::default(),
            sprite: Sprite { id: String::new() },
            bounding_box: BoundingBox {
                half_extend: Default::default(),
                collision_layer: CollisionLayer::NOTHING,
                collision_mask: None,
            },
            health: None,
            shield: None,
            enemy: Enemy::default(),
            trail: None,
            animation: None,
        }
    }
}
