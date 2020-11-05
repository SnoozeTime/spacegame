use crate::core::camera::Camera;
use crate::core::colors::RgbaColor;
use crate::core::random::Seed;
use crate::core::scene::{Scene, SceneResult};
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::camera::update_camera;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::enemy::{spawn_enemy, EnemyType};
use crate::gameplay::health::{Health, HealthSystem};
use crate::gameplay::level::{LevelInstruction, LevelSystem};
use crate::gameplay::physics::{DynamicBody, PhysicConfig, PhysicSystem};
use crate::gameplay::player::{get_player, Player, Weapon};
use crate::gameplay::{bullet, collision, enemy, player};
use crate::render::sprite::Sprite;
use crate::render::ui::text::Text;
use crate::render::ui::Gui;
use crate::resources::Resources;
use hecs::World;
use log::info;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use shrev::EventChannel;
use std::time::Duration;

pub struct MainScene {
    level_system: LevelSystem,
    health_system: Option<HealthSystem>,
    physic_system: PhysicSystem,
}

fn load_level() -> LevelSystem {
    let instructions = vec![LevelInstruction::SpawnEnemy {
        health: 2,
        pos: glam::Vec2::new(200.0, 400.0),
        enemy_type: EnemyType::FollowPlayer(Timer::of_seconds(2.0)),
    }];
    LevelSystem::new(instructions)
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            level_system: load_level(),
            health_system: None,
            physic_system: PhysicSystem::new(PhysicConfig { damping: 0.99 }),
        }
    }
}

impl Scene for MainScene {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        info!("Create MainScene");
        self.health_system = Some(HealthSystem::new(resources));

        let backgrounds = ["front.png", "left.png", "top.png", "right.png", "back.png"];

        let mut rng = if let Some(seed) = resources.fetch::<Seed>() {
            StdRng::from_seed(seed.0)
        } else {
            StdRng::from_entropy()
        };

        // First choose a random background.
        world.spawn((
            Transform {
                translation: glam::Vec2::new(0.0, 0.0),
                scale: glam::Vec2::new(2048.0, 2048.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite {
                id: backgrounds.choose(&mut rng).unwrap().to_string(),
            },
        ));

        world.spawn((
            Transform {
                translation: glam::Vec2::new(0.0, 0.0),
                scale: glam::Vec2::new(32.0, 32.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite {
                id: "asteroid.png".to_string(),
            },
            DynamicBody {
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 5.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: CollisionLayer::PLAYER
                    | CollisionLayer::ENEMY
                    | CollisionLayer::PLAYER_BULLET
                    | CollisionLayer::ENEMY_BULLET,
                half_extend: glam::vec2(16.0, 16.0),
            },
        ));

        world.spawn((
            Transform {
                translation: glam::Vec2::new(-200.0, 0.0),
                scale: glam::Vec2::new(64.0, 64.0),
                rotation: 0.0,
                dirty: false,
            },
            Sprite {
                id: "asteroid.png".to_string(),
            },
            DynamicBody {
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 100.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::ASTEROID,
                collision_mask: CollisionLayer::PLAYER
                    | CollisionLayer::ENEMY
                    | CollisionLayer::PLAYER_BULLET
                    | CollisionLayer::ENEMY_BULLET,
                half_extend: glam::vec2(32.0, 32.0),
            },
        ));

        world.spawn((Camera::new(),));
        let player_components = (
            Transform {
                translation: glam::Vec2::new(100.0, 100.0),
                scale: glam::Vec2::new(50.0, 50.0),
                rotation: 0.0,
                dirty: true,
            },
            Sprite {
                id: "P-blue-a.png".to_string(),
            },
            Player {
                weapon: Weapon::Simple,
                direction: glam::vec2(0.0, 1.0),
            },
            DynamicBody {
                forces: vec![],
                velocity: Default::default(),
                max_velocity: 500.0,
                mass: 1.0,
            },
            BoundingBox {
                collision_layer: CollisionLayer::PLAYER,
                collision_mask: CollisionLayer::ENEMY_BULLET
                    | CollisionLayer::ENEMY
                    | CollisionLayer::ASTEROID,
                half_extend: glam::vec2(20.0, 20.0),
            },
            Health::new(5, Timer::of_seconds(1.0)),
        );
        world.spawn(player_components);

        spawn_enemy(
            world,
            2,
            glam::vec2(500.0, 400.0),
            EnemyType::FollowPlayer(Timer::of_seconds(2.0)),
        );
        spawn_enemy(
            world,
            2,
            glam::vec2(550.0, 400.0),
            EnemyType::FollowPlayer(Timer::of_seconds(2.0)),
        );
        spawn_enemy(
            world,
            2,
            glam::vec2(550.0, 450.0),
            EnemyType::FollowPlayer(Timer::of_seconds(2.0)),
        );
        spawn_enemy(
            world,
            2,
            glam::vec2(500.0, 700.0),
            EnemyType::FollowPlayer(Timer::of_seconds(2.0)),
        );

        {
            let mut channel = resources.fetch_mut::<EventChannel<GameEvent>>().unwrap();
            channel.single_write(GameEvent::TextUpdated);
        }
    }

    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        log::debug!("UPDATE SYSTEMS");
        self.level_system.update(world, dt);
        player::update_player(world, dt, resources);
        update_camera(world, resources);
        enemy::update_enemies(world, &resources, dt);

        self.physic_system.update(world, dt, resources);

        bullet::process_bullets(world, resources);
        let collisions = collision::find_collisions(world);
        collision::process_collisions(world, collisions, &resources);
        if let Some(hs) = self.health_system.as_mut() {
            hs.update(world, &resources, dt);
        }

        SceneResult::Noop
    }

    fn process_event(&mut self, ev: GameEvent) {
        if let GameEvent::GameOver = ev {
            std::process::exit(0); // TODO Replace that.
        }
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        world: &mut World,
        _resources: &Resources,
    ) -> Option<Gui> {
        let mut gui = Gui::new();
        gui.panel(
            glam::vec2(10.0, 10.0),
            glam::vec2(200.0, 500.0),
            RgbaColor::new(60, 60, 60, 150),
        );

        if let Some(player_health) = get_player(world) {
            let health = world.get::<Health>(player_health).unwrap();

            let text = format!(
                "Hull {:02}%",
                ((health.current as f32 / health.max as f32) * 100.0).round() as i32
            );
            gui.label(
                glam::vec2(15.0, 15.0),
                Text {
                    font_size: 16.0,
                    content: text,
                },
                RgbaColor::new(255, 0, 0, 255),
            );
        }

        Some(gui)
    }
}
