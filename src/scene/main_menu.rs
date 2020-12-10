use crate::core::audio;
use crate::core::scene::{Scene, SceneResult};
use crate::core::transform::Transform;
use crate::paths::get_assets_path;
use crate::prefab::enemies::ENEMY_PREFABS;
use crate::render::particle::ParticleEmitter;
use crate::render::sprite::Sprite;
use crate::render::ui::gui::GuiContext;
use crate::render::ui::Gui;
use crate::resources::Resources;
use crate::save::is_infinite_unlocked;
use crate::scene::loading::LoadingScene;
use crate::scene::story::StoryScene;
use crate::scene::wave_selection::WaveSelectionScene;
use crate::scene::MainScene;
use crate::ui::{disabled_menu_button, draw_cursor, menu_button};
use core::time::Duration;
use hecs::World;

#[derive(Debug, Clone)]
enum GameMode {
    Normal,
    Infinite,
}

#[derive(Default, Clone)]
pub struct MainMenu {
    does_start: bool,
    game_mode: Option<GameMode>,
    emitter_entity: Option<hecs::Entity>,
}

const EMITTER_BYTES: &[u8] = include_bytes!("../../assets/particle/menu.json");

impl Scene for MainMenu {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        //generate_terrain(world, resources);
        let mut emitter: ParticleEmitter = serde_json::from_slice(EMITTER_BYTES).unwrap();
        emitter.init_pool();
        self.emitter_entity = Some(world.spawn((emitter, Transform::default())));

        audio::play_background_music(resources, "music/spacelifeNo14.ogg");
    }

    fn on_destroy(&mut self, world: &mut hecs::World) {
        if let Some(e) = self.emitter_entity {
            if let Err(e) = world.despawn(e) {
                error!("Error despawning menu particle = {:?}", e);
            }
        }
    }

    fn update(&mut self, _dt: Duration, _world: &mut World, resources: &Resources) -> SceneResult {
        let mut prefabs: Vec<String> = ENEMY_PREFABS.iter().map(|e| e.to_string()).collect();

        // let mut prefabs = vec!["base_enemy".to_string()];
        prefabs.push("player".to_string());
        if let Some(GameMode::Normal) = self.game_mode {
            SceneResult::ReplaceScene(Box::new(LoadingScene::new(
                prefabs,
                vec![
                    "music/spacelifeNo14.ogg".to_string(),
                    "music/Finding-Flora.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_09.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_01.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_02.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_03.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_04.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_05.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_06.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_07.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_08.wav".to_string(),
                    "sounds/explosion.wav".to_string(),
                    // "sounds/powerUp2.mp3".to_string(),
                ],
                vec![
                    "capsule.png".to_string(),
                    "asteroid.png".to_string(),
                    "back.png".to_string(),
                    "left.png".to_string(),
                    "right.png".to_string(),
                    "top.png".to_string(),
                    "spaceships/blue_05.png".to_string(),
                    "spaceships/darkgrey_02.png".to_string(),
                ],
                StoryScene::new(
                    vec![
                        "Humans discovered an alien artefact deep inside the moon.".to_string(),
                        "It should be ours...".to_string(),
                    ],
                    MainScene::new(false, 0),
                ),
            )))
        } else if let Some(GameMode::Infinite) = self.game_mode {
            SceneResult::ReplaceScene(Box::new(LoadingScene::new(
                prefabs,
                vec![
                    "music/spacelifeNo14.ogg".to_string(),
                    "music/Finding-Flora.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_09.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_01.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_02.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_03.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_04.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_05.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_06.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_07.wav".to_string(),
                    "sounds/scifi_kit/Laser/Laser_08.wav".to_string(),
                    "sounds/explosion.wav".to_string(),
                ],
                vec![],
                WaveSelectionScene::new(resources),
            )))
        } else {
            SceneResult::Noop
        }
    }

    fn prepare_gui(
        &mut self,
        _dt: Duration,
        _world: &mut World,
        _resources: &Resources,
        gui_context: &GuiContext,
    ) -> Option<Gui> {
        let w = gui_context.window_dim.width as f32;
        let h = gui_context.window_dim.height as f32;
        // panel should take up 20% of the window width and 80% of the window height
        let panel_width = w * 0.2;
        let panel_height = h * 0.6;

        let anchor = glam::vec2(w / 2.0 - panel_width / 2.0, h / 2.0 - panel_height / 2.0);

        let mut gui = gui_context.new_frame();
        draw_cursor(&mut gui);

        // START BUTTON
        if menu_button("Start", anchor, 48.0, &mut gui) {
            self.game_mode = Some(GameMode::Normal);
        }

        if is_infinite_unlocked(_resources) {
            if menu_button(
                "Infinite Mode",
                anchor + 80.0 * glam::Vec2::unit_y(),
                48.0,
                &mut gui,
            ) {
                self.game_mode = Some(GameMode::Infinite);
            }
        } else {
            disabled_menu_button(
                "Infinite Mode",
                anchor + 80.0 * glam::Vec2::unit_y(),
                48.0,
                &mut gui,
            );
        }

        if cfg!(not(target_arch = "wasm32")) {
            // EXIT BUTTON
            if menu_button(
                "Quit to Desktop",
                anchor + 160.0 * glam::Vec2::unit_y(),
                48.0,
                &mut gui,
            ) {
                std::process::exit(0);
            }
        }

        Some(gui)
    }
}
