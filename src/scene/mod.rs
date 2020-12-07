use crate::assets::prefab::PrefabManager;
use crate::assets::Handle;
use crate::core::animation::AnimationSystem;
use crate::core::audio;
use crate::core::colors::RgbaColor;
use crate::core::input::ser::{InputEvent, VirtualAction, VirtualKey};
use crate::core::random::RandomGenerator;
use crate::core::scene::{Scene, SceneResult};
use crate::core::timer::Timer;
use crate::core::transform::{HasChildren, HasParent, LocalTransform, Transform};
use crate::event::GameEvent;
use crate::gameplay::bullet::{Bullet, Missile};
use crate::gameplay::camera::update_camera;
use crate::gameplay::explosion::ExplosionSystem;
use crate::gameplay::health::{Health, HealthSystem, Shield};
use crate::gameplay::inventory::Inventory;
use crate::gameplay::level::{Stage, StageDescription};
use crate::gameplay::physics::{PhysicConfig, PhysicSystem};
use crate::gameplay::pickup::{process_pickups, spawn_pickup, Pickup};
use crate::gameplay::player::get_player;
use crate::gameplay::trail::update_trails;
use crate::gameplay::{bullet, collision, enemy, player};
use crate::paths::get_assets_path;
use crate::render::mesh::{Material, MeshRender};
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::{Button, Gui, HorizontalAlign, VerticalAlign};
use crate::resources::Resources;
use crate::save::{get_wave_record, save_new_wave_record, save_unlocked};
use crate::scene::main_menu::MainMenu;
use crate::scene::pause::PauseScene;
use crate::scene::story::StoryScene;
use crate::ui::draw_cursor;
use hecs::World;
use log::info;
use rand::Rng;
use std::time::Duration;

pub mod loading;
pub mod main_menu;
pub mod particle_scene;
pub mod pause;
pub mod story;
pub mod wave_selection;

enum MainSceneState {
    Running,
    GameOver,
    GameWon,
    Paused,
}

pub struct MainScene {
    stage: Option<Stage>,
    health_system: Option<HealthSystem>,
    physic_system: PhysicSystem,
    animation_system: AnimationSystem,
    explosion_system: Option<ExplosionSystem>,

    state: MainSceneState,
    return_to_menu: bool,
    restart: bool,
    player: Option<hecs::Entity>,

    info_text: Option<String>,
    info_text_timer: Timer,

    is_infinite: bool,
    starting_wave_nb: usize,
}

impl Default for MainScene {
    fn default() -> Self {
        Self::new(false, 1)
    }
}

impl MainScene {
    pub fn new(is_infinite: bool, starting_wave_nb: usize) -> Self {
        Self {
            is_infinite,
            starting_wave_nb,
            player: None,
            info_text: None,
            restart: false,
            state: MainSceneState::Running,
            return_to_menu: false,
            animation_system: AnimationSystem,
            stage: None,
            health_system: None,
            explosion_system: None,
            physic_system: PhysicSystem::new(PhysicConfig { damping: 0.99 }),
            info_text_timer: Timer::of_seconds(3.0),
        }
    }
}

impl Scene for MainScene {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        info!("Create MainScene");
        self.health_system = Some(HealthSystem::new(resources));
        self.explosion_system = Some(ExplosionSystem::new(resources));

        //generate_terrain(world, resources);
        let base_path = get_assets_path();
        let mut emitter: ParticleEmitter = serde_json::from_str(
            &std::fs::read_to_string(base_path.join("particle/trail.json")).unwrap(),
        )
        .unwrap();
        emitter.init_pool();

        let stage_desc: StageDescription = if self.is_infinite {
            StageDescription::infinite()
        } else {
            let p = base_path.join("stages/stage1.json");
            let content = std::fs::read_to_string(p).unwrap();
            serde_json::from_str(&content).unwrap()
        };
        let stage = Stage::new(world, resources, stage_desc, self.starting_wave_nb);
        self.stage = Some(stage);

        self.player = Some({
            let prefab_manager = resources.fetch_mut::<PrefabManager>().unwrap();
            let asset = prefab_manager
                .get(&Handle("player".to_string()))
                .expect("Player asset should have been loaded");
            assert!(asset.is_loaded());
            asset
                .execute(|prefab| prefab.spawn(world))
                .expect("Should be able to spawn player")
        });

        let player_scale = { world.get::<Transform>(self.player.unwrap()).unwrap().scale * 2.0 };

        // add the shield to the player...
        let shield_entity = world.spawn((
            Transform {
                translation: Default::default(),
                scale: player_scale,
                rotation: 0.0,
                dirty: true,
            },
            LocalTransform {
                translation: Default::default(),
                scale: Default::default(),
                rotation: 0.0,
                dirty: true,
            },
            HasParent {
                entity: self.player.unwrap(),
            },
            MeshRender {
                enabled: true,
                material: Material::Shader {
                    vertex_shader_id: "simple-vs.glsl".to_string(),
                    fragment_shader_id: "simple-fs.glsl".to_string(),
                },
            },
        ));

        world
            .insert_one(
                self.player.unwrap(),
                HasChildren {
                    children: vec![shield_entity],
                },
            )
            .expect("Should be able to add shield");

        // "music/Finding-Flora.wav"
        audio::play_background_music(resources, "music/Finding-Flora.wav");
    }

    fn on_destroy(&mut self, world: &mut hecs::World) {
        // remove the player.
        if let Some(p) = self.player {
            let mut to_despawn = vec![];
            if let Ok(children) = world.get::<HasChildren>(p) {
                for c in &children.children {
                    to_despawn.push(*c);
                }
            }

            to_despawn.iter().for_each(|&c| {
                if let Err(e) = world.despawn(c) {
                    error!("Cannot despawn player's children = {:?}", e);
                }
            });

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
        let pickups: Vec<_> = world.query::<&Pickup>().iter().map(|(e, _)| e).collect();
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
        pickups.iter().for_each(|&e| {
            if let Err(e) = world.despawn(e) {
                error!("Error while despawn pickups = {:?}", e);
            }
        });
    }

    fn update(&mut self, dt: Duration, world: &mut World, resources: &Resources) -> SceneResult {
        log::debug!("UPDATE SYSTEMS");
        self.info_text_timer.tick(dt);
        if self.info_text_timer.finished() {
            self.info_text = None;
        }

        if let MainSceneState::Running = self.state {
            player::update_player(world, dt, resources);
            update_camera(world, resources);
            enemy::update_enemies(world, &resources, dt);
            self.animation_system.animate(world, resources);
            update_trails(world);
            self.physic_system.update(world, dt, resources);

            bullet::process_bullets(world, resources);
            bullet::process_missiles(world, resources);
            process_pickups(world, resources);

            let collisions = collision::find_collisions(world, resources);
            collision::process_collisions(world, collisions, &resources);
            if let Some(hs) = self.health_system.as_mut() {
                hs.update(world, &resources, dt);
            }
            if let Some(system) = self.explosion_system.as_mut() {
                system.update(world, &resources);
            }
            if let Some(ref mut stage) = self.stage {
                stage.update(world, resources, dt);
            }
        }

        if let MainSceneState::Paused = self.state {
            self.state = MainSceneState::Running;
            SceneResult::Push(Box::new(PauseScene::default()))
        } else if let MainSceneState::GameWon = self.state {
            SceneResult::ReplaceScene(Box::new(StoryScene::new(
                vec![
                    "You reach the moon, with all its riches".to_string(),
                    "Now, the whole space is waiting for you...".to_string(),
                ],
                MainMenu::default(),
            )))
        } else if self.return_to_menu {
            SceneResult::ReplaceScene(Box::new(MainMenu::default()))
        } else if self.restart {
            SceneResult::ReplaceScene(Box::new(MainScene::new(
                self.is_infinite,
                self.starting_wave_nb,
            )))
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
        draw_cursor(&mut gui);

        match self.state {
            MainSceneState::Paused => (),
            MainSceneState::Running => {
                if let Some(player_health) = get_player(world) {
                    let health = world.get::<Health>(player_health).unwrap();

                    let health_percent = health.current / health.max;
                    let bar_width = 100.0;

                    // health bar
                    gui.panel(
                        glam::vec2(15.0, 15.0),
                        glam::vec2(bar_width, 10.0),
                        RgbaColor::new(0, 0, 0, 255),
                    );
                    gui.panel(
                        glam::vec2(15.0, 15.0),
                        glam::vec2(bar_width * health_percent, 10.0),
                        RgbaColor::new(255, 0, 0, 255),
                    );

                    let shield = world.get::<Shield>(player_health);
                    if let Ok(shield) = shield {
                        // Shield bar
                        let shield_percent = shield.current / shield.max;

                        gui.panel(
                            glam::vec2(15.0, 30.0),
                            glam::vec2(bar_width, 10.0),
                            RgbaColor::new(0, 0, 0, 255),
                        );
                        gui.panel(
                            glam::vec2(15.0, 30.0),
                            glam::vec2(bar_width * shield_percent, 10.0),
                            RgbaColor::new(0, 0, 255, 255),
                        );
                    }

                    if let Some(inv) = resources.fetch::<Inventory>() {
                        gui.colored_label(
                            glam::vec2(15.0, 50.0),
                            format!("Scrap: {}", inv.scratch()),
                            RgbaColor::new(255, 255, 255, 255),
                        )
                    }
                }

                // information about stage and waves.
                if let Some(stage_text) = self.stage.as_ref().and_then(|s| s.display()) {
                    let center =
                        gui_context.window_dim.to_vec2() / 2.0 - glam::Vec2::unit_y() * 100.0;
                    gui.centered_label(center, stage_text)
                }

                // extra info (pick ups...)
                if let Some(ref info) = self.info_text {
                    if !self.info_text_timer.finished() {
                        gui.colored_label(
                            glam::vec2(10.0, gui.window_dim.height as f32 - 40.0),
                            info.to_string(),
                            RgbaColor::new(255, 255, 255, 255),
                        )
                    }
                }

                // information about infinite wave.
                if let Some(ref stage) = self.stage {
                    if stage.is_infinite {
                        gui.colored_label(
                            glam::vec2(
                                gui.window_dim.width as f32 - 100.0,
                                gui.window_dim.height as f32 - 40.0,
                            ),
                            format!("Wave {}", stage.wave_number),
                            RgbaColor::new(255, 255, 255, 255),
                        );

                        gui.colored_label(
                            glam::vec2(gui.window_dim.width as f32 - 200.0, 10.0),
                            format!("Wave Record {}", get_wave_record(resources)),
                            RgbaColor::new(255, 255, 255, 255),
                        )
                    }
                }
            }
            MainSceneState::GameOver => {
                let center = gui_context.window_dim.to_vec2() / 2.0 - glam::Vec2::unit_y() * 100.0;
                if let Some(ref stage) = self.stage {
                    if stage.is_infinite {
                        gui.colored_label(
                            center,
                            format!("You died at wave {}", stage.wave_number),
                            RgbaColor::new(255, 255, 255, 255),
                        )
                    } else {
                        // In case of game over, let's just show the message and buttons to return back home.
                        gui.colored_label(
                            center,
                            "Game Over...".to_string(),
                            RgbaColor::new(255, 255, 255, 255),
                        );
                    }
                }

                if game_button("Restart", center + glam::Vec2::unit_y() * 50.0, &mut gui) {
                    self.restart = true;
                }
                if game_button(
                    "Return to menu",
                    center + glam::Vec2::unit_y() * 100.0,
                    &mut gui,
                ) {
                    self.return_to_menu = true;
                }
            }
            MainSceneState::GameWon => {}
        }
        Some(gui)
    }

    fn process_event(&mut self, world: &mut World, ev: GameEvent, resources: &Resources) {
        let mut drain_scratch = false;
        match ev {
            GameEvent::GameOver => {
                self.state = MainSceneState::GameOver;

                // if infinite, let's set new wave record if it's more than current.
                if self.is_infinite {
                    if let Err(e) = save_new_wave_record(
                        resources,
                        self.stage
                            .as_ref()
                            .expect("Should have a stage...")
                            .wave_number,
                    ) {
                        error!("could not save data = {:?}", e);
                    }
                }

                drain_scratch = true;
            }
            GameEvent::YouWin => {
                drain_scratch = true;
                if let Err(e) = save_unlocked(resources) {
                    error!("could not save data = {:?}", e);
                }
                self.state = MainSceneState::GameWon
            }
            GameEvent::EnemyDied(e, pos, (low_scrap, high_scrap), pickup_drop) => {
                let mut random = resources
                    .fetch_mut::<RandomGenerator>()
                    .expect("Should have a random generator");

                if high_scrap > low_scrap {
                    if let Some(ref mut inv) = resources.fetch_mut::<Inventory>() {
                        let scratch_to_add = random.rng().gen_range(low_scrap, high_scrap);
                        inv.add_scratch(scratch_to_add);
                    }
                }

                // drop some pickups :)
                let pick: u8 = random.rng().gen_range(0, 101);
                if pick <= pickup_drop {
                    spawn_pickup(world, pos, &mut *random);
                }

                if let Some(stage) = self.stage.as_mut() {
                    stage.enemy_died(e)
                }
            }
            GameEvent::InfoText(info) => {
                self.info_text_timer.reset();
                self.info_text_timer.start();
                self.info_text = Some(info);
            }
            GameEvent::NextStage(stage_name) => {
                let base_path = get_assets_path();
                let stage_desc: StageDescription = {
                    let p = base_path.join("stages").join(stage_name);
                    let content = std::fs::read_to_string(p).unwrap();
                    serde_json::from_str(&content).unwrap()
                };
                if let Some(stage) = self.stage.as_mut() {
                    stage.clean(world);
                }
                let stage = Stage::new(world, resources, stage_desc, 0);
                self.stage = Some(stage);

                drain_scratch = true;
            }
            _ => (),
        }

        if drain_scratch {
            // Remove all scratch :) You need to spend that money.
            if let Some(ref mut inv) = resources.fetch_mut::<Inventory>() {
                inv.drain_scratch();
            }
        }
    }

    fn process_input(&mut self, _world: &mut World, input: InputEvent, _resources: &Resources) {
        if let InputEvent::KeyEvent(VirtualKey::Escape, VirtualAction::Pressed) = input {
            self.state = MainSceneState::Paused;
        }
    }
}

fn game_button(text: &str, position: glam::Vec2, ui: &mut Gui) -> bool {
    Button::new(text.to_string(), position)
        .set_bg_color(RgbaColor::new(0, 0, 0, 0), RgbaColor::new(0, 0, 0, 0))
        .set_text_color(
            RgbaColor::from_hex("FFFFFFFF").unwrap(),
            RgbaColor::from_hex("01FFFFFF").unwrap(),
        )
        .set_font_size(32.0)
        .set_text_align(HorizontalAlign::Left, VerticalAlign::Top)
        .set_padding(0.0)
        .build(ui)
}
