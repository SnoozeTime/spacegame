use crate::assets::prefab::PrefabManager;
use crate::assets::AssetManager;
use crate::core::colors::RgbaColor;
use crate::core::random::RandomGenerator;
use crate::core::scene::{Scene, SceneResult};
use crate::core::timer::Timer;
use crate::core::transform::Transform;
use crate::event::GameEvent;
use crate::gameplay::camera::update_camera;
use crate::gameplay::collision::{BoundingBox, CollisionLayer};
use crate::gameplay::enemy::{spawn_enemy, EnemyType, Satellite};
use crate::gameplay::health::{Health, HealthSystem, Shield};
use crate::gameplay::inventory::Inventory;
use crate::gameplay::level::{generate_terrain, Stage};
use crate::gameplay::physics::{DynamicBody, PhysicConfig, PhysicSystem};
use crate::gameplay::pickup::{process_pickups, spawn_pickup};
use crate::gameplay::player::{get_player, Player, Weapon};
use crate::gameplay::trail::{update_trails, Trail};
use crate::gameplay::{bullet, collision, enemy, player};
use crate::render::particle::ParticleEmitter;
use crate::render::sprite::Sprite;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use hecs::World;
use log::info;
use luminance_glfw::GlfwSurface;
use rand::seq::SliceRandom;
use shrev::EventChannel;
use std::path::PathBuf;
use std::time::Duration;

pub mod loading;
pub mod main_menu;
pub mod particle_scene;

pub struct MainScene {
    stage: Option<Stage>,
    health_system: Option<HealthSystem>,
    physic_system: PhysicSystem,
}

impl Default for MainScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            stage: None,
            health_system: None,
            physic_system: PhysicSystem::new(PhysicConfig { damping: 0.99 }),
        }
    }
}

impl Scene for MainScene {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        info!("Create MainScene");
        self.health_system = Some(HealthSystem::new(resources));

        let stage = Stage::new(world, resources);
        self.stage = Some(stage);

        //generate_terrain(world, resources);
        let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
        let mut emitter: ParticleEmitter = serde_json::from_str(
            &std::fs::read_to_string(PathBuf::from(&base_path).join("particle/trail.json"))
                .unwrap(),
        )
        .unwrap();
        emitter.init_pool();

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
                    | CollisionLayer::ASTEROID
                    | CollisionLayer::MISSILE,
                half_extend: glam::vec2(20.0, 20.0),
            },
            Health::new(5, Timer::of_seconds(1.0)),
            Shield::new(1.0, 5.0, 0.1),
            Trail {
                should_display: true,
            },
            emitter, // ParticleEmitter::new(EmitterSource::Point(glam::Vec2::new(100.0, 100.0)),
                     // )
        );
        world.spawn(player_components);
    }

    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        log::debug!("UPDATE SYSTEMS");
        player::update_player(world, dt, resources);
        update_camera(world, resources);
        enemy::update_enemies(world, &resources, dt);
        update_trails(world);
        self.physic_system.update(world, dt, resources);

        bullet::process_bullets(world, resources);
        bullet::process_missiles(world, resources);
        process_pickups(world, resources);

        let collisions = collision::find_collisions(world);
        collision::process_collisions(world, collisions, &resources);
        if let Some(hs) = self.health_system.as_mut() {
            hs.update(world, &resources, dt);
        }
        if let Some(ref mut stage) = self.stage {
            stage.update(world, resources, dt);
        }

        SceneResult::Noop
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        world: &mut World,
        resources: &Resources,
    ) -> Option<Gui> {
        let gui_context = resources.fetch::<GuiContext>().unwrap();

        let mut gui = gui_context.new_frame();
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
            gui.colored_label(glam::vec2(15.0, 15.0), text, RgbaColor::new(255, 0, 0, 255));

            let shield = world.get::<Shield>(player_health);
            let shield_text = if let Ok(shield) = shield {
                format!(
                    "Hull {:02}%",
                    ((shield.current as f32 / shield.max as f32) * 100.0).round() as i32
                )
            } else {
                "Shield: 0%".to_string()
            };
            gui.colored_label(
                glam::vec2(15.0, 40.0),
                shield_text,
                RgbaColor::new(0, 0, 255, 255),
            );

            if let Some(inv) = resources.fetch::<Inventory>() {
                gui.label(
                    glam::vec2(15.0, 75.0),
                    format!("Scratch: {}", inv.scratch()),
                )
            }
        }

        // information about stage and waves.
        if let Some(stage_text) = self.stage.as_ref().and_then(|s| s.display()) {
            let center = gui_context.window_dim.to_vec2() / 2.0 - glam::Vec2::unit_y() * 100.0;
            gui.colored_label(center, stage_text, RgbaColor::new(255, 255, 255, 255))
        }

        Some(gui)
    }

    fn process_event(&mut self, ev: GameEvent, resources: &Resources) {
        if let GameEvent::GameOver = ev {
            std::process::exit(0); // TODO Replace that.
        } else if let GameEvent::EnemyDied(e) = ev {
            if let Some(ref mut inv) = resources.fetch_mut::<Inventory>() {
                inv.add_scratch(50);
            }
            if let Some(stage) = self.stage.as_mut() {
                stage.enemy_died(e)
            }
        }
    }
}
