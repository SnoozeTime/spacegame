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
