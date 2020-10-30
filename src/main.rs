use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;
use std::process::exit;

#[allow(unused_imports)]
use log::info;

use spacegame::game::{Game, GameBuilder};

use spacegame::gameplay::Action;
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

    let mut game: Game<Action> = GameBuilder::new(&mut surface)
        .for_scene(Box::new(MainScene::new()))
        .build();

    game.run();
}
