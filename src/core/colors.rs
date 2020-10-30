#[allow(unused)]
pub const PASTEL_RED: RgbColor = RgbColor::new(212, 80, 121);
#[allow(unused)]
pub const PASTEL_PURPLE: RgbColor = RgbColor::new(110, 87, 115);
#[allow(unused)]
pub const PASTEL_ORANGE: RgbColor = RgbColor::new(234, 144, 133);
#[allow(unused)]
pub const PASTEL_BEIGE: RgbColor = RgbColor::new(233, 225, 204);
#[allow(unused)]
pub const RED: RgbColor = RgbColor::new(255, 0, 0);
#[allow(unused)]
pub const BLUE: RgbColor = RgbColor::new(0, 0, 255);
#[allow(unused)]
pub const GREEN: RgbColor = RgbColor::new(0, 255, 0);

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_normalized(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    pub fn to_rgba_normalized(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            1.0,
        ]
    }
}

impl From<[f32; 4]> for RgbColor {
    fn from(c: [f32; 4]) -> Self {
        let r = (c[0] * 255.0).round().min(255.0).max(0.0) as u8;
        let g = (c[1] * 255.0).round().min(255.0).max(0.0) as u8;
        let b = (c[2] * 255.0).round().min(255.0).max(0.0) as u8;
        Self { r, g, b }
    }
}
