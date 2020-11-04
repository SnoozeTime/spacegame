#[macro_use]
extern crate bitflags;

use luminance_windowing::WindowDim;

pub mod assets;
pub mod config;
pub mod core;
pub mod event;
pub mod game;
pub mod gameplay;
pub mod render;
pub mod resources;
pub mod scene;

pub const WIDTH: u32 = 800;
pub const HEIGHT: u32 = 800;
pub const DIMENSIONS: WindowDim = WindowDim::Windowed {
    width: WIDTH,
    height: HEIGHT,
};
