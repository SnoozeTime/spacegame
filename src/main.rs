#[allow(unused_imports)]
use log::info;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{CursorMode, WindowOpt};
use std::path::PathBuf;
use std::process::exit;

use spacegame::game::{Game, GameBuilder};

use spacegame::config::{load_config, AudioConfig, GameEngineConfig, InputConfig, PlayerConfig};
use spacegame::gameplay::inventory::Inventory;
use spacegame::gameplay::level::difficulty::DifficultyConfig;
use spacegame::gameplay::Action;
use spacegame::save::read_saved_data;
use spacegame::scene::loading::LoadingScene;
#[allow(unused_imports)]
use spacegame::scene::main_menu::MainMenu;
#[allow(unused_imports)]
use spacegame::scene::particle_scene::ParticleScene;
use spacegame::DIMENSIONS;

fn main() {
    let surface = GlfwSurface::new_gl33(
        "EverFight",
        WindowOpt::default()
            .set_cursor_mode(CursorMode::Invisible)
            .set_dim(DIMENSIONS),
    );

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

    let base_path = PathBuf::from(std::env::var("ASSET_PATH").unwrap_or("assets/".to_string()));
    let player_config_path = base_path.join("config/player_controller.json");
    let player_config: PlayerConfig = load_config(&player_config_path).unwrap_or_else(|e| {
        log::info!("Will use default PlayerConfig because = {:?}", e);
        PlayerConfig::default()
    });
    let engine_config_path = base_path.join("config/engine.json");
    let engine_config: GameEngineConfig = load_config(&engine_config_path).unwrap_or_else(|e| {
        log::info!("Will use default GameEngineConfig because = {:?}", e);
        GameEngineConfig::default()
    });

    let difficulty_config_path = base_path.join("config/difficulty.json");
    let difficulty_config: DifficultyConfig =
        load_config(&difficulty_config_path).unwrap_or_else(|e| {
            log::info!("Will use default Difficulty because = {:?}", e);
            DifficultyConfig::default()
        });

    let input_config_path = base_path.join("config/input.json");
    let input_config: Result<InputConfig, _> = load_config(&input_config_path);

    let audio_config_path = base_path.join("config/audio.json");
    let audio_config: Result<AudioConfig, _> = load_config(&audio_config_path);

    let saved_data = read_saved_data();

    let mut builder: GameBuilder<Action> = GameBuilder::new(&mut surface)
        .for_scene(Box::new(LoadingScene::new(
            vec![],
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
                "sounds/powerUp2.mp3".to_string(),
            ],
            MainMenu::default(),
        )))
        .with_resource(saved_data)
        .with_resource(player_config)
        .with_resource(engine_config)
        .with_resource(difficulty_config)
        .with_resource(Inventory::default());

    if let Ok(input_config) = input_config {
        let (km, mm) = input_config.input_maps();
        builder = builder.with_input_config(km, mm);
    }

    if let Ok(audio_config) = audio_config {
        builder = builder.with_audio_config(audio_config);
    }

    let mut game: Game<Action> = builder.build();
    game.run();
}
