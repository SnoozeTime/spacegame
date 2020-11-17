#![allow(warnings)]

use spacegame::assets::prefab::Prefab;
use spacegame::core::timer::Timer;
use spacegame::core::transform::Transform;
use spacegame::gameplay::collision::{BoundingBox, CollisionLayer};
use spacegame::gameplay::enemy::{Boss1, Enemy, EnemyType, Satellite};
use spacegame::gameplay::health::Health;
use spacegame::gameplay::physics::DynamicBody;
use spacegame::gameplay::player::Player;
use spacegame::prefab::enemies::EnemyPrefab;
use spacegame::prefab::player::PlayerPrefab;
use spacegame::render::particle::ParticleEmitter;
use spacegame::render::sprite::Sprite;
use std::path::PathBuf;

fn gen_player() {
    let player = {
        let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
        let mut emitter: ParticleEmitter = serde_json::from_str(
            &std::fs::read_to_string(PathBuf::from(&base_path).join("particle/trail.json"))
                .unwrap(),
        )
        .unwrap();
        let scale = 50.0;
        let player_prefab = PlayerPrefab {
            dynamic_body: DynamicBody {
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 500.0,
                mass: 1.0,
            },
            transform: Transform {
                translation: glam::Vec2::new(100.0, 100.0),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: true,
            },
            sprite: Sprite {
                id: "P-blue-a.png".to_string(),
            },
            bounding_box: BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::PLAYER,
                collision_mask: None,
            },
            health: Health::new(3.0, Timer::of_seconds(3.0)),
            shield: None,
            trail: emitter,
        };

        let prefab = &player_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/player.json", player);
}

fn main() {
    gen_player();
    let satellite = {
        let scale = 40.0;
        let enemy_prefab = EnemyPrefab {
            dynamic_body: DynamicBody {
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 0.0,
                mass: 1.0,
            },
            transform: Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            },
            sprite: Sprite {
                id: "Proto-ship.png".to_string(),
            },
            bounding_box: BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            },
            health: Some(Health::new(3.0, Timer::of_seconds(1.0))),
            shield: None,
            enemy: Enemy {
                enemy_type: EnemyType::Satellite(Satellite {
                    shoot_timer: Timer::of_seconds(3.0),
                    shoot_distance: 500.0,
                }),
                speed: 0.0,
            },
            trail: None,
        };

        let prefab = &enemy_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/satellite.json", satellite);

    //
    let boss = {
        let scale = 100.0;
        let enemy_prefab = EnemyPrefab {
            dynamic_body: DynamicBody {
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 500.0,
                mass: 10.0,
            },
            transform: Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            },
            sprite: Sprite {
                id: "EnemyBoss.png".to_string(),
            },
            bounding_box: BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            },
            health: Some(Health::new(10.0, Timer::of_seconds(1.0))),
            shield: None,
            enemy: Enemy {
                enemy_type: EnemyType::Boss1(Boss1 {
                    shoot_timer: Timer::of_seconds(0.3),
                    nb_shot: 20,
                    current_shot: 0,
                    salve_timer: Timer::of_seconds(5.0),
                }),
                speed: 5.0,
            },
            trail: None,
        };

        let prefab = &enemy_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/boss1.json", boss);
}
