use gfx_maths::*;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DirectionalLight {
    pub direction: Vec4,
    pub illuminance: Vec4, // in lx = lm / m^2
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    pub position: Vec4,
    pub luminous_flux: Vec4, // in lm
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
}

impl From<PointLight> for Light {
    fn from(value: PointLight) -> Self {
        Light::Point(value)
    }
}

impl From<DirectionalLight> for Light {
    fn from(value: DirectionalLight) -> Self {
        Light::Directional(value)
    }
}
