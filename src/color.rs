use math::prelude::*;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    color: Vec4<f32>,
}

impl Color {
    pub const BLACK: Color = Color {
        color: Vec4::new(0.0, 0.0, 0.0, 1.0),
    };
    pub const WHITE: Color = Color {
        color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    };
    pub const RED: Color = Color {
        color: Vec4::new(1.0, 0.0, 0.0, 1.0),
    };
    pub const GREEN: Color = Color {
        color: Vec4::new(0.0, 1.0, 0.0, 1.0),
    };
    pub const BLUE: Color = Color {
        color: Vec4::new(0.0, 0.0, 1.0, 1.0),
    };

    const CLAMP_MAX: f32 = u8::max_value() as f32;

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            color: Vec4::new(
                f32::from(r) / Color::CLAMP_MAX,
                f32::from(g) / Color::CLAMP_MAX,
                f32::from(b) / Color::CLAMP_MAX,
                f32::from(a) / Color::CLAMP_MAX,
            ),
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, u8::max_value())
    }

    pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        assert!(r <= 1.0, "Red value of color > 1.0");
        assert!(g <= 1.0, "Green value of color > 1.0");
        assert!(b <= 1.0, "Blue value of color > 1.0");
        assert!(a <= 1.0, "Alpha value of color > 1.0");

        Self {
            color: Vec4::new(r, g, b, a),
        }
    }

    pub fn rgb_f32(r: f32, g: f32, b: f32) -> Self {
        Self::rgba_f32(r, g, b, 1.0)
    }

    pub fn from_u32(value: u32) -> Self {
        let r = (value >> 24) & 0xFF;
        let g = (value >> 16) & 0xFF;
        let b = (value >> 8) & 0xFF;
        let a = value & 0xFF;
        Self::rgba(r as u8, g as u8, b as u8, a as u8)
    }
}

impl From<Vec3<f32>> for Color {
    fn from(value: Vec3<f32>) -> Self {
        Self::rgb_f32(*value.x(), *value.y(), *value.z())
    }
}

impl From<Vec4<f32>> for Color {
    fn from(value: Vec4<f32>) -> Self {
        Self::rgba_f32(*value.x(), *value.y(), *value.z(), *value.w())
    }
}

impl From<Vec3<u8>> for Color {
    fn from(value: Vec3<u8>) -> Self {
        Self::rgb(*value.x(), *value.y(), *value.z())
    }
}

impl From<Vec4<u8>> for Color {
    fn from(value: Vec4<u8>) -> Self {
        Self::rgba(*value.x(), *value.y(), *value.z(), *value.w())
    }
}

#[cfg(test)]
mod test {
    use super::Color;

    #[test]
    fn test_from_hex() {
        assert_eq!(Color::from_u32(0xFFFFFFFF), Color::WHITE);
        assert_eq!(Color::from_u32(0x000000FF), Color::BLACK);
        assert_eq!(Color::from_u32(0xFF0000FF), Color::RED);
    }
}
