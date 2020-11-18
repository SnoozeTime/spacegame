#[allow(unused_imports)]
use log::info;
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;
use std::path::PathBuf;
use std::process::exit;

use spacegame::game::{Game, GameBuilder};

use spacegame::config::{load_config, GameEngineConfig, PlayerConfig};
use spacegame::gameplay::inventory::Inventory;
use spacegame::gameplay::Action;
use spacegame::scene::loading::LoadingScene;
#[allow(unused_imports)]
use spacegame::scene::main_menu::MainMenu;
#[allow(unused_imports)]
use spacegame::scene::particle_scene::ParticleScene;
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
    let player_config_path = PathBuf::from(base_path.clone()).join("config/player_controller.json");
    let player_config: PlayerConfig = load_config(&player_config_path).unwrap_or_else(|e| {
        log::info!("Will use default PlayerConfig because = {:?}", e);
        PlayerConfig::default()
    });
    let engine_config_path = PathBuf::from(base_path.clone()).join("config/engine.json");
    let engine_config: GameEngineConfig = load_config(&engine_config_path).unwrap_or_else(|e| {
        log::info!("Will use default PlayerConfig because = {:?}", e);
        GameEngineConfig::default()
    });

    let mut game: Game<Action> = GameBuilder::new(&mut surface)
        // .for_scene(Box::new(ParticleScene::new(
        //     PathBuf::from(base_path).join("particle/particle.json"),
        //     false,
        // )))
        // .for_scene(Box::new(MainMenu::default()))
        .for_scene(Box::new(LoadingScene::new(
            vec![],
            vec![
                "music/spacelifeNo14.ogg".to_string(),
                "music/Finding-Flora.wav".to_string(),
                "sounds/scifi_kit/Laser/Laser_09.wav".to_string(),
                "sounds/scifi_kit/Laser/Laser_01.wav".to_string(),
            ],
            MainMenu::default(),
        )))
        .with_resource(player_config)
        .with_resource(engine_config)
        .with_resource(Inventory::default())
        .build();

    game.run();
}
