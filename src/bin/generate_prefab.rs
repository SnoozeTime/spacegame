#![allow(warnings)]

use downcast_rs::__std::collections::HashMap;
use spacegame::core::animation::{Animation, AnimationController};
use spacegame::core::serialization::SerializedEntity;
use spacegame::core::timer::Timer;
use spacegame::core::transform::Transform;
use spacegame::gameplay::collision::{BoundingBox, CollisionLayer};
use spacegame::gameplay::enemy::{
    Boss1, Enemy, EnemyType, LastBoss, MovementBehavior, Satellite, Spammer,
};
use spacegame::gameplay::health::Health;
use spacegame::gameplay::physics::DynamicBody;
use spacegame::gameplay::player::{Player, Stats, Weapon};
use spacegame::gameplay::trail::Trail;
use spacegame::paths::get_assets_path;
use spacegame::render::particle::ParticleEmitter;
use spacegame::render::sprite::Sprite;

const TRAIL_PARTICLE: &[u8] = include_bytes!("../../assets/particle/trail.json");
const ENEMY_TRAIL_PARTICLE: &[u8] = include_bytes!("../../assets/particle/enemy_trail.json");

fn gen_player() {
    let player = {
        let mut emitter: ParticleEmitter = serde_json::from_slice(TRAIL_PARTICLE).unwrap();
        let scale = 24.0;

        let player_prefab = SerializedEntity {
            transform: Some(Transform {
                translation: glam::Vec2::new(100.0, 100.0),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: true,
            }),
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 600.0,
                mass: 1.0,
                max_force: 1500.0,
            }),
            sprite: Some(Sprite {
                id: "spaceships/blue_05.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: 20.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::PLAYER,
                collision_mask: None,
            }),
            health: Some(Health::new(10.0, Timer::of_seconds(0.5))),
            trail: Some(Trail {
                should_display: true,
                offset: 20.0,
            }),
            emitter: Some(emitter),
            player: Some(Player {
                weapon: Weapon::Simple,
                stats: Stats {
                    dmg: 1.0,
                    crit_percent: 50,
                    crit_multiplier: 1.5,
                    missile_percent: 0,
                    boost_timer: Timer::of_seconds(1.0),
                    boost_magnitude: 500.0,
                },
                direction: glam::vec2(0.0, 1.0),
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&player_prefab).unwrap()
    };

    std::fs::write("assets/serialized/player.json", player);
}

fn gen_mine() {
    let mine = {
        let scale = 20.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                forces: vec![],
                impulses: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 0.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/explosion-05.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::MINE,
                collision_mask: None,
            }),
            health: Some(Health::new(2.0, Timer::of_seconds(0.5))),
            shield: None,
            enemy: Some(Enemy {
                enemy_type: EnemyType::Mine {
                    trigger_distance: 200.0,
                    explosion_timer: {
                        let mut timer = Timer::of_seconds(2.0);
                        timer.stop();
                        timer
                    },
                },
                scrap_drop: (0, 0),
                pickup_drop_percent: 0,
                movement: MovementBehavior::Nothing,
            }),
            animation: Some({
                let mut animations = HashMap::new();
                animations.insert(
                    String::from("boum"),
                    Animation::new(vec![
                        (String::from("windshield_wiper/00.png"), 0),
                        (String::from("windshield_wiper/01.png"), 1),
                        (String::from("windshield_wiper/02.png"), 2),
                        (String::from("windshield_wiper/03.png"), 3),
                        (String::from("windshield_wiper/04.png"), 4),
                        (String::from("windshield_wiper/05.png"), 5),
                        (String::from("windshield_wiper/06.png"), 6),
                        (String::from("windshield_wiper/07.png"), 7),
                        (String::from("windshield_wiper/08.png"), 8),
                        (String::from("windshield_wiper/09.png"), 9),
                        (String::from("windshield_wiper/10.png"), 10),
                    ]),
                );

                let mut animation_controller = AnimationController {
                    animations,
                    current_animation: None,
                    delete_on_finished: false,
                };
                animation_controller
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/mine.json", mine);
}

fn gen_mine_lander() {
    let mine_lander = {
        let scale = 24.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                forces: vec![],
                impulses: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 200.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/red_03.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::MineLander(Timer::of_seconds(4.0)),
                scrap_drop: (10, 70),
                pickup_drop_percent: 2,
                movement: MovementBehavior::RandomPath(glam::Vec2::zero(), false),
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/mine_lander.json", mine_lander);
}

fn gen_wanderer() {
    let prefab = {
        let scale = 24.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                forces: vec![],
                impulses: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 200.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/red_04.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Wanderer(Timer::of_seconds(4.0)),
                scrap_drop: (10, 70),
                pickup_drop_percent: 2,
                movement: MovementBehavior::RandomPath(glam::Vec2::zero(), false),
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/wanderer.json", prefab);
}

fn gen_base_enemy() {
    let base_enemy = {
        let scale = 24.0;
        let mut emitter: ParticleEmitter = serde_json::from_slice(ENEMY_TRAIL_PARTICLE).unwrap();
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 100.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/darkgrey_02.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(0.5))),
            shield: None,
            enemy: Some(Enemy {
                enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(3.0)),
                scrap_drop: (10, 40),
                pickup_drop_percent: 2,
                movement: MovementBehavior::Follow,
            }),
            trail: Some(Trail {
                should_display: true,
                offset: 0.0,
            }),
            emitter: Some(emitter),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/base_enemy.json", base_enemy);
}

fn gen_carrier() {
    let base_enemy = {
        let scale = 128.0;
        let mut emitter: ParticleEmitter = serde_json::from_slice(ENEMY_TRAIL_PARTICLE).unwrap();

        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 50.0,
                mass: 10.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/large_red_01.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(15.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Carrier {
                    nb_of_spaceships: 4,
                    time_between_deploy: Timer::of_seconds(12.0),
                },
                scrap_drop: (10, 40),
                pickup_drop_percent: 70,
                movement: MovementBehavior::Follow,
            }),
            trail: Some(Trail {
                should_display: true,
                offset: 0.0,
            }),
            emitter: Some(emitter),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/carrier.json", base_enemy);
}

fn gen_kamikaze() {
    let base_enemy = {
        let scale = 20.0;
        let mut emitter: ParticleEmitter = serde_json::from_slice(ENEMY_TRAIL_PARTICLE).unwrap();

        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 300.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/metalic_06.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(2.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Kamikaze,
                scrap_drop: (10, 40),
                pickup_drop_percent: 2,
                movement: MovementBehavior::GoToPlayer,
            }),
            trail: Some(Trail {
                should_display: true,
                offset: 0.0,
            }),
            emitter: Some(emitter),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/kamikaze.json", base_enemy);
}

fn gen_base_enemy_2() {
    let base_enemy = {
        let scale = 24.0;
        let mut emitter: ParticleEmitter = serde_json::from_slice(ENEMY_TRAIL_PARTICLE).unwrap();

        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 300.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/metalic_06.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(1.0)),
                scrap_drop: (30, 60),
                pickup_drop_percent: 5,
                movement: MovementBehavior::Follow,
            }),
            trail: Some(Trail {
                should_display: true,
                offset: 0.0,
            }),
            emitter: Some(emitter),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/base_enemy_2.json", base_enemy);
}

fn gen_base_enemy_3() {
    let base_enemy = {
        let scale = 24.0;
        let mut emitter: ParticleEmitter = serde_json::from_slice(ENEMY_TRAIL_PARTICLE).unwrap();

        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 400.0,
                mass: 1.0,
                max_force: 1000.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/darkgrey_04.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(1.0)),
                scrap_drop: (50, 90),
                pickup_drop_percent: 10,
                movement: MovementBehavior::Follow,
            }),
            trail: Some(Trail {
                should_display: true,
                offset: 0.0,
            }),
            emitter: Some(emitter),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/base_enemy_3.json", base_enemy);
}

fn gen_spammer() {
    let spammer = {
        let scale = 32.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 100.0,
                mass: 5.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/green_04.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(1.0))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Spammer(Spammer {
                    shoot_timer: Timer::of_seconds(1.0),
                    nb_shot: 3,
                    current_shot: 0,
                    salve_timer: Timer::of_seconds(6.0),
                }),
                scrap_drop: (20, 70),
                pickup_drop_percent: 10,
                movement: MovementBehavior::Follow,
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/spammer.json", spammer);
}

fn gen_last_boss() {
    let boss = {
        let scale = 64.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 1500.0,
                mass: 10.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/large_purple_01.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(20.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::LastBoss(LastBoss {
                    shoot_timer: Timer::of_seconds(0.3),
                    nb_shot: 10,
                    current_shot: 0,
                    salve_timer: Timer::of_seconds(5.0),
                }),
                scrap_drop: (20, 100),
                pickup_drop_percent: 100,
                movement: MovementBehavior::RandomPath(glam::Vec2::zero(), false),
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/last_boss.json", boss);
}

fn main() {
    gen_wanderer();
    gen_player();
    gen_mine_lander();
    gen_mine();
    gen_base_enemy();
    gen_base_enemy_2();
    gen_base_enemy_3();
    gen_spammer();
    gen_kamikaze();
    gen_carrier();
    gen_last_boss();
    let satellite = {
        let scale = 40.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 0.0,
                mass: 1.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "sat.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(3.0, Timer::of_seconds(1.0))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Satellite(Satellite {
                    shoot_timer: Timer::of_seconds(3.0),
                    shoot_distance: 800.0,
                }),
                scrap_drop: (10, 40),
                pickup_drop_percent: 2,
                movement: MovementBehavior::Follow,
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/satellite.json", satellite);

    //
    let boss = {
        let scale = 64.0;
        let enemy_prefab = SerializedEntity {
            dynamic_body: Some(DynamicBody {
                impulses: vec![],
                forces: vec![],
                velocity: glam::Vec2::zero(),
                max_velocity: 500.0,
                mass: 10.0,
                max_force: 500.0,
            }),
            transform: Some(Transform {
                translation: Default::default(),
                scale: scale * glam::Vec2::one(),
                rotation: 0.0,
                dirty: false,
            }),
            sprite: Some(Sprite {
                id: "spaceships/large_grey_02.png".to_string(),
            }),
            bounding_box: Some(BoundingBox {
                half_extend: scale / 2.0 * glam::Vec2::one(),
                collision_layer: CollisionLayer::ENEMY,
                collision_mask: None,
            }),
            health: Some(Health::new(10.0, Timer::of_seconds(0.5))),
            enemy: Some(Enemy {
                enemy_type: EnemyType::Boss1(Boss1 {
                    shoot_timer: Timer::of_seconds(0.3),
                    nb_shot: 20,
                    current_shot: 0,
                    salve_timer: Timer::of_seconds(5.0),
                }),
                scrap_drop: (20, 100),
                pickup_drop_percent: 100,
                movement: MovementBehavior::Follow,
            }),
            ..Default::default()
        };

        serde_json::to_string_pretty(&enemy_prefab).unwrap()
    };

    std::fs::write("assets/serialized/boss1.json", boss);
}
