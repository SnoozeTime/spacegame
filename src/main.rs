#[allow(unused_imports)]
use log::info;
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;
use std::path::PathBuf;
use std::process::exit;

use spacegame::game::{Game, GameBuilder};

use spacegame::config::PlayerConfig;
use spacegame::gameplay::Action;
use spacegame::scene::main_menu::MainMenu;
use spacegame::scene::MainScene;
use spacegame::DIMENSIONS;

fn main() {
    let surface = GlfwSurface::new_gl33("Hello Window", WindowOpt::default().set_dim(DIMENSIONS));

    match surface {
        Ok(surface) => main_loop(surface),
        Err(e) => {
            eprintln!("Error = {}", e);
            exit(1);
        }
    }
}

fn main_loop(mut surface: GlfwSurface) {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let base_path = std::env::var("ASSET_PATH").unwrap_or("assets/".to_string());
    let player_config_path = PathBuf::from(base_path).join("config/player_controller.json");
    let player_config = PlayerConfig::load(player_config_path).unwrap_or_else(|e| {
        log::info!("Will use default PlayerConfig because = {:?}", e);
        PlayerConfig::default()
    });

    let mut game: Game<Action> = GameBuilder::new(&mut surface)
        .for_scene(Box::new(MainMenu::default()))
        .with_resource(player_config)
        .build();

    game.run();
}
