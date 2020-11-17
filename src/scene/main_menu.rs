use crate::core::colors::RgbaColor;
use crate::core::scene::{Scene, SceneResult};
use crate::core::transform::Transform;
use crate::render::particle::ParticleEmitter;
use crate::render::ui::gui::{GuiContext, HorizontalAlign, VerticalAlign};
use crate::render::ui::{Button, Gui};
use crate::resources::Resources;
use crate::scene::loading::LoadingScene;
use bitflags::_core::time::Duration;
use hecs::World;
use std::path::PathBuf;

#[derive(Default)]
pub struct MainMenu {
    does_start: bool,
    emitter_entity: Option<hecs::Entity>,
}

impl Scene for MainMenu {
    fn on_create(&mut self, world: &mut hecs::World, _resources: &mut Resources) {
        //generate_terrain(world, resources);
        let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
        let emitter = ParticleEmitter::load_from_path(
            PathBuf::from(&base_path).join("particle").join("menu.json"),
        )
        .unwrap();

        self.emitter_entity = Some(world.spawn((emitter, Transform::default())));
    }

    fn update(&mut self, _dt: Duration, _world: &mut World, _resources: &Resources) -> SceneResult {
        if self.does_start {
            SceneResult::ReplaceScene(Box::new(LoadingScene::new(vec![
                "base_enemy".to_string(),
                "base_enemy_2".to_string(),
                "boss1".to_string(),
                "satellite".to_string(),
                "player".to_string(),
            ])))
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
            self.does_start = true;
        }

        // EXIT BUTTON
        if menu_button(
            "Quit to Desktop",
            anchor + 80.0 * glam::Vec2::unit_y(),
            &mut gui,
        ) {
            std::process::exit(0);
        }

        Some(gui)
    }

    fn on_destroy(&mut self, world: &mut hecs::World) {
        if let Some(e) = self.emitter_entity {
            if let Err(e) = world.despawn(e) {
                error!("Error despawning menu particle = {:?}", e);
            }
        }
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
