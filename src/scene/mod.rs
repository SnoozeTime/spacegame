use crate::assets::prefab::PrefabManager;
use crate::assets::Handle;
use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::event::GameEvent;
use crate::gameplay::bullet::{Bullet, Missile};
use crate::gameplay::camera::update_camera;
use crate::gameplay::health::{Health, HealthSystem, Shield};
use crate::gameplay::inventory::Inventory;
use crate::gameplay::level::{Stage, StageDescription};
use crate::gameplay::physics::{PhysicConfig, PhysicSystem};
use crate::gameplay::pickup::process_pickups;
use crate::gameplay::player::get_player;
use crate::gameplay::trail::update_trails;
use crate::gameplay::{bullet, collision, enemy, player};
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use crate::scene::main_menu::MainMenu;
use hecs::World;
use log::info;
use luminance_glfw::GlfwSurface;
use std::path::PathBuf;
use std::time::Duration;

pub mod loading;
pub mod main_menu;
pub mod particle_scene;

pub struct MainScene {
    stage: Option<Stage>,
    health_system: Option<HealthSystem>,
    physic_system: PhysicSystem,
    game_over: bool,
    return_to_menu: bool,
    player: Option<hecs::Entity>,
}

impl Default for MainScene {
    fn default() -> Self {
        Self::new()
    }
}

impl MainScene {
    pub fn new() -> Self {
        Self {
            player: None,
            game_over: false,
            return_to_menu: false,
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

        //generate_terrain(world, resources);
        let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
        let mut emitter: ParticleEmitter = serde_json::from_str(
            &std::fs::read_to_string(PathBuf::from(&base_path).join("particle/trail.json"))
                .unwrap(),
        )
        .unwrap();
        emitter.init_pool();

        let stage_desc: StageDescription = {
            let p = PathBuf::from(&base_path).join("stages/stage1.json");
            let content = std::fs::read_to_string(p).unwrap();
            serde_json::from_str(&content).unwrap()
        };
        let stage = Stage::new(world, resources, stage_desc);
        self.stage = Some(stage);

        self.player = Some({
            let prefab_manager = resources.fetch_mut::<PrefabManager<GlfwSurface>>().unwrap();
            let asset = prefab_manager
                .get(&Handle("player".to_string()))
                .expect("Player asset should have been loaded");
            assert!(asset.is_loaded());
            asset
                .execute(|prefab| prefab.spawn(world))
                .expect("Should be able to spawn player")
        });
    }

    fn on_destroy(&mut self, world: &mut hecs::World) {
        // remove the player.
        if let Some(p) = self.player {
            if let Err(e) = world.despawn(p) {
                error!("Error while despawn player = {:?}", e);
            }
        }

        // Clean the stage.
        if let Some(stage) = self.stage.as_mut() {
            stage.clean(world);
        }

        // remove all the bullets :)
        let bullets: Vec<_> = world.query::<&Bullet>().iter().map(|(e, _)| e).collect();
        let missiles: Vec<_> = world.query::<&Missile>().iter().map(|(e, _)| e).collect();
        bullets.iter().for_each(|&e| {
            if let Err(e) = world.despawn(e) {
                error!("Error while despawn bullet = {:?}", e);
            }
        });
        missiles.iter().for_each(|&e| {
            if let Err(e) = world.despawn(e) {
                error!("Error while despawn missiles = {:?}", e);
            }
        });
    }

    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        log::debug!("UPDATE SYSTEMS");

        if !self.game_over {
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
        }

        if self.return_to_menu {
            SceneResult::ReplaceScene(Box::new(MainMenu::default()))
        } else {
            SceneResult::Noop
        }
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        world: &mut World,
        resources: &Resources,
        gui_context: &GuiContext,
    ) -> Option<Gui> {
        let mut gui = gui_context.new_frame();

        if !self.game_over {
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
        } else {
            // In case of game over, let's just show the message and buttons to return back home.
            let center = gui_context.window_dim.to_vec2() / 2.0 - glam::Vec2::unit_y() * 100.0;
            gui.colored_label(
                center,
                "Game Over...".to_string(),
                RgbaColor::new(255, 255, 255, 255),
            );
            if gui.button(
                center - glam::Vec2::unit_y() * 50.0,
                None,
                "Return to menu".to_string(),
            ) {
                self.return_to_menu = true;
            }
        }

        Some(gui)
    }

    fn process_event(&mut self, ev: GameEvent, resources: &Resources) {
        if let GameEvent::GameOver = ev {
            self.game_over = true;
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
