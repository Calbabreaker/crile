use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Color with RGBA values from 0 to 1 in SRGB color space
#[derive(PartialEq, Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
#[repr(C)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        wgpu::Color {
            r: value.0[0] as f64,
            g: value.0[1] as f64,
            b: value.0[2] as f64,
            a: value.0[3] as f64,
        }
    }
}

impl Color {
    pub const BLACK: Color = Color::from_hex(0x000000);
    pub const WHITE: Color = Color::from_hex(0xffffff);

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }

    pub const fn from_srgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([
            r as f32 / 255.,
            g as f32 / 255.,
            b as f32 / 255.,
            a as f32 / 255.,
        ])
    }

    /// Converts a hexidecimal number RGB or RGBA into the Color
    pub const fn from_hex(hex: u32) -> Self {
        if hex <= 0xffffff {
            Color::from_srgba(
                ((hex >> 16) & 0xff) as u8,
                ((hex >> 8) & 0xff) as u8,
                (hex & 0xff) as u8,
                255,
            )
        } else {
            Color::from_srgba(
                ((hex >> 24) & 0xff) as u8,
                ((hex >> 16) & 0xff) as u8,
                ((hex >> 8) & 0xff) as u8,
                (hex & 0xff) as u8,
            )
        }
    }

    pub const fn r(&self) -> f32 {
        self.0[0]
    }

    pub const fn g(&self) -> f32 {
        self.0[1]
    }

    pub const fn b(&self) -> f32 {
        self.0[2]
    }

    pub const fn a(&self) -> f32 {
        self.0[3]
    }
}
