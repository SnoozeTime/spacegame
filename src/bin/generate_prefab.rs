#![allow(warnings)]

use spacegame::assets::prefab::Prefab;
use spacegame::core::timer::Timer;
use spacegame::core::transform::Transform;
use spacegame::gameplay::collision::{BoundingBox, CollisionLayer};
use spacegame::gameplay::enemy::{Boss1, Enemy, EnemyType, MovementBehavior, Satellite};
use spacegame::gameplay::health::Health;
use spacegame::gameplay::physics::DynamicBody;
use spacegame::gameplay::player::{Player, Stats};
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
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 500.0,
                mass: 1.0,
                max_force: 500.0,
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
            stats: Stats {
                dmg: 1.0,
                crit_percent: 50,
                crit_multiplier: 1.5,
                missile_percent: 0,
            },
        };

        let prefab = &player_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/player.json", player);
}

fn gen_mine() {
    let mine = {
        let scale = 40.0;
        let enemy_prefab = EnemyPrefab {
            dynamic_body: DynamicBody {
                forces: vec![],
                impulses: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 0.0,
                mass: 1.0,
                max_force: 500.0,
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
                collision_layer: CollisionLayer::MINE,
                collision_mask: None,
            },
            health: Some(Health::new(2.0, Timer::of_seconds(1.0))),
            shield: None,
            enemy: Enemy {
                enemy_type: EnemyType::Mine {
                    trigger_distance: 100.0,
                    explosion_timer: {
                        let mut timer = Timer::of_seconds(1.0);
                        timer.stop();
                        timer
                    },
                },
                speed: 0.0,
                scratch_drop: (0, 40),
                movement: MovementBehavior::Nothing,
            },
            trail: None,
        };

        let prefab = &enemy_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/mine.json", mine);
}

fn gen_mine_lander() {
    let mine_lander = {
        let scale = 40.0;
        let enemy_prefab = EnemyPrefab {
            dynamic_body: DynamicBody {
                forces: vec![],
                impulses: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 200.0,
                mass: 1.0,
                max_force: 500.0,
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
                enemy_type: EnemyType::MineLander(Timer::of_seconds(4.0)),
                speed: 5.0,
                scratch_drop: (0, 40),
                movement: MovementBehavior::RandomPath(glam::Vec2::zero(), false),
            },
            trail: None,
        };

        let prefab = &enemy_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/mine_lander.json", mine_lander);
}

fn main() {
    gen_player();
    gen_mine_lander();
    gen_mine();
    let satellite = {
        let scale = 40.0;
        let enemy_prefab = EnemyPrefab {
            dynamic_body: DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 0.0,
                mass: 1.0,
                max_force: 500.0,
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
                    shoot_distance: 800.0,
                }),
                speed: 0.0,
                scratch_drop: (0, 40),
                movement: MovementBehavior::Follow,
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
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 500.0,
                mass: 10.0,
                max_force: 500.0,
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
                scratch_drop: (0, 100),
                movement: MovementBehavior::Follow,
            },
            trail: None,
        };

        let prefab = &enemy_prefab as &dyn Prefab;
        serde_json::to_string_pretty(prefab).unwrap()
    };

    std::fs::write("assets/prefab/boss1.json", boss);
}
