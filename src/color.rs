use crate::math::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    color: Vec4,
}

impl Color {
    pub const RED: Color = Color { color: Vec4::new(1.0, 0.0, 0.0, 1.0) };
    pub const GREEN: Color = Color { color: Vec4::new(0.0, 1.0, 0.0, 1.0) };
    pub const BLUE: Color = Color { color: Vec4::new(0.0, 0.0, 1.0, 1.0) };

    const CLAMP_MAX: f32 = u8::max_value() as f32;

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: Vec4 {
                x: (f32::from(r) / Color::CLAMP_MAX),
                y: (f32::from(g) / Color::CLAMP_MAX),
                z: (f32::from(b) / Color::CLAMP_MAX),
                w: (f32::from(a) / Color::CLAMP_MAX),
            },
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, u8::max_value())
    }
}
