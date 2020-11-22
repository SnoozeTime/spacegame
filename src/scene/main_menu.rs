use crate::core::audio;
use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::core::transform::Transform;
use crate::prefab::enemies::ENEMY_PREFABS;
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::{GuiContext, HorizontalAlign, VerticalAlign};
use crate::render::ui::{Button, Gui};
use crate::resources::Resources;
use crate::scene::loading::LoadingScene;
use crate::scene::MainScene;
use bitflags::_core::time::Duration;
use hecs::World;
use std::path::PathBuf;

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

impl Scene for MainMenu {
    fn on_create(&mut self, world: &mut hecs::World, resources: &mut Resources) {
        //generate_terrain(world, resources);
        let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
        let emitter = ParticleEmitter::load_from_path(
            PathBuf::from(&base_path).join("particle").join("menu.json"),
        )
        .unwrap();

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

    fn update(&mut self, _dt: Duration, _world: &mut World, _resources: &Resources) -> SceneResult {
        let mut prefabs: Vec<String> = ENEMY_PREFABS.iter().map(|e| e.to_string()).collect();

        prefabs.push("player".to_string());
        if let Some(GameMode::Normal) = self.game_mode {
            SceneResult::ReplaceScene(Box::new(LoadingScene::new(
                prefabs,
                vec![],
                MainScene::new(false),
            )))
        } else if let Some(GameMode::Infinite) = self.game_mode {
            SceneResult::ReplaceScene(Box::new(LoadingScene::new(
                prefabs,
                vec![],
                MainScene::new(true),
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
        //let gui_context = resources.fetch::<GuiContext>().unwrap();

        let w = gui_context.window_dim.width as f32;
        let h = gui_context.window_dim.height as f32;
        // panel should take up 20% of the window width and 80% of the window height
        let panel_width = w * 0.2;
        let panel_height = h * 0.6;

        let anchor = glam::vec2(w / 2.0 - panel_width / 2.0, h / 2.0 - panel_height / 2.0);

        let mut gui = gui_context.new_frame();

        // START BUTTON
        if menu_button("Start", anchor, &mut gui) {
            self.game_mode = Some(GameMode::Normal);
        }

        if menu_button(
            "Infinite Mode",
            anchor + 80.0 * glam::Vec2::unit_y(),
            &mut gui,
        ) {
            self.game_mode = Some(GameMode::Infinite);
        }

        // EXIT BUTTON
        if menu_button(
            "Quit to Desktop",
            anchor + 160.0 * glam::Vec2::unit_y(),
            &mut gui,
        ) {
            std::process::exit(0);
        }

        Some(gui)
    }
}

fn menu_button(text: &str, position: glam::Vec2, ui: &mut Gui) -> bool {
    Button::new(text.to_string(), position)
        .set_bg_color(RgbaColor::new(0, 0, 0, 0), RgbaColor::new(0, 0, 0, 0))
        .set_text_color(
            RgbaColor::from_hex("FFFFFFFF").unwrap(),
            RgbaColor::from_hex("01FFFFFF").unwrap(),
        )
        .set_font_size(48.0)
        .set_text_align(HorizontalAlign::Left, VerticalAlign::Top)
        .set_padding(0.0)
        .build(ui)
}
