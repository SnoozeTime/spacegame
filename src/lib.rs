#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate log;

use luminance_windowing::WindowDim;

pub mod assets;
pub mod config;
pub mod core;
pub mod event;
pub mod game;
pub mod gameplay;
pub mod paths;
pub mod prefab;
pub mod render;
pub mod resources;
pub mod save;
pub mod scene;
pub mod ui;
pub const WIDTH: u32 = 1600;
pub const HEIGHT: u32 = 960;
pub const DIMENSIONS: WindowDim = WindowDim::Windowed {
    width: WIDTH,
    height: HEIGHT,
};
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(target_arch = "wasm32")]
mod web {
    use crate::gameplay::inventory::Inventory;
    use std::panic;

    use crate::config::GameEngineConfig;
    use crate::config::PlayerConfig;
    use crate::core::input::ser::{InputEvent, VirtualAction, VirtualButton, VirtualKey};
    use crate::core::input::{Input, InputAction};
    use crate::game::{Game, GameBuilder};
    use crate::gameplay::level::difficulty::DifficultyConfig;
    use crate::gameplay::Action;
    use crate::save::read_saved_data;
    use crate::scene::main_menu::MainMenu;
    use log::Level;
    use luminance::pipeline::PipelineState;
    use luminance_web_sys::WebSysWebGL2Surface;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct WasmGame {
        surface: WebSysWebGL2Surface,
        game: Game<Action>,
    }

    #[wasm_bindgen]
    pub fn handle_event() {}

    #[wasm_bindgen]
    pub fn create_audio_system(game: &mut WasmGame) {
        let WasmGame { ref mut game, .. } = game;

        game.create_audio_system();
    }

    #[wasm_bindgen]
    pub fn get_game(canvas_name: &str) -> WasmGame {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::new(Level::Info));

        let saved_data = read_saved_data();

        info!("Get surface from canvas");
        // First thing first: we create a new surface to render to and get events from.
        let mut surface = WebSysWebGL2Surface::new(canvas_name).expect("web-sys surface");

        info!("Build the game");
        let conf = GameEngineConfig { show_gizmos: false };
        let game = GameBuilder::new()
            .for_scene(Box::new(MainMenu::default()))
            .with_resource(conf)
            .with_resource(PlayerConfig::default())
            .with_resource(DifficultyConfig::default())
            .with_resource(Inventory::default())
            .with_resource(saved_data)
            .build(&mut surface);

        info!("Ok !");
        WasmGame { surface, game }
    }

    #[wasm_bindgen]
    pub fn render_scene(game: &mut WasmGame) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        if let Ok(mut back_buffer) = surface.back_buffer() {
            let dt = std::time::Duration::from_millis(16);

            game.run_frame(surface, &mut back_buffer, dt);
        } else {
            error!("Cannot get backbuffer");
        }
    }

    #[wasm_bindgen]
    pub fn prepare_input(game: &mut WasmGame) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        game.prepare_input();
    }

    #[wasm_bindgen]
    pub fn process_key(game: &mut WasmGame, key: VirtualKey) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        game.process_input(InputEvent::KeyEvent(key, VirtualAction::Pressed));
    }

    #[wasm_bindgen]
    pub fn release_key(game: &mut WasmGame, key: VirtualKey) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        game.process_input(InputEvent::KeyEvent(key, VirtualAction::Release));
    }

    #[wasm_bindgen]
    pub fn process_mouse_move(game: &mut WasmGame, x: f64, y: f64) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        game.process_input(InputEvent::CursorPos(x, y));
    }

    #[wasm_bindgen]
    pub fn process_mouse_click(game: &mut WasmGame) {
        let WasmGame {
            ref mut surface,
            ref mut game,
        } = game;
        game.process_input(InputEvent::MouseEvent(
            VirtualButton::Button1,
            VirtualAction::Pressed,
        ));
    }
}
