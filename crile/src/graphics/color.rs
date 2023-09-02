#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        wgpu::Color {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}

impl Color {
    pub fn from_rgb(r: u32, g: u32, b: u32) -> Self {
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: 1.,
        }
    }

    /// Converts a hexidecimal number rgb or rgba to a color struct.
    pub fn from_hex(hex: u32) -> Self {
        let mut color = Self::default();
        if hex <= 0xffffff {
            color.r = ((hex >> 16) & 0xff) as f32 / 255.;
            color.g = ((hex >> 8) & 0xff) as f32 / 255.;
            color.b = (hex & 0xff) as f32 / 255.;
            color.a = 1.;
        } else {
            color.r = ((hex >> 24) & 0xff) as f32 / 255.;
            color.g = ((hex >> 16) & 0xff) as f32 / 255.;
            color.b = ((hex >> 8) & 0xff) as f32 / 255.;
            color.a = (hex & 0xff) as f32 / 255.;
        }
        color
    }
}
