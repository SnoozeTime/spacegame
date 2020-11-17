// #[allow(unused)]
// pub const PASTEL_RED: RgbColor = RgbColor::new(212, 80, 121);
// #[allow(unused)]
// pub const PASTEL_PURPLE: RgbColor = RgbColor::new(110, 87, 115);
// #[allow(unused)]
// pub const PASTEL_ORANGE: RgbColor = RgbColor::new(234, 144, 133);
// #[allow(unused)]
// pub const PASTEL_BEIGE: RgbColor = RgbColor::new(233, 225, 204);
use anyhow::anyhow;
#[allow(unused)]
pub const RED: RgbaColor = RgbaColor {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};
#[allow(unused)]
pub const BLUE: RgbaColor = RgbaColor {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};
#[allow(unused)]
pub const GREEN: RgbaColor = RgbaColor {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

use crate::core::curve::CurveNode;
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

impl From<[f32; 3]> for RgbColor {
    fn from(c: [f32; 3]) -> Self {
        let r = (c[0] * 255.0).round().min(255.0).max(0.0) as u8;
        let g = (c[1] * 255.0).round().min(255.0).max(0.0) as u8;
        let b = (c[2] * 255.0).round().min(255.0).max(0.0) as u8;
        Self { r, g, b }
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaColor {
    pub fn from_hex(hex_code: &str) -> Result<Self, anyhow::Error> {
        if hex_code.len() != 8 {
            return Err(anyhow!("Hex code for RGBA should have a length of 8."));
        }
        let rhex = &hex_code[0..2];
        let ghex = &hex_code[2..4];
        let bhex = &hex_code[4..6];
        let ahex = &hex_code[6..8];

        let r = u8::from_str_radix(rhex, 16)?;
        let g = u8::from_str_radix(ghex, 16)?;
        let b = u8::from_str_radix(bhex, 16)?;
        let a = u8::from_str_radix(ahex, 16)?;

        Ok(Self::new(r, g, b, a))
    }

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32) / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn to_normalized(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl CurveNode for RgbaColor {}

impl std::ops::Mul<f32> for RgbaColor {
    type Output = RgbaColor;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl std::ops::Add for RgbaColor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl std::ops::Sub for RgbaColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
        }
    }
}
