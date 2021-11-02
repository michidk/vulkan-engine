use gfx_maths::*;

#[repr(C)]
pub struct DirectionalLight {
    pub direction: Vec4,
    pub illuminance: Vec4, // in lx = lm / m^2
}

#[repr(C)]
pub struct PointLight {
    pub position: Vec4,
    pub luminous_flux: Vec4, // in lm
}

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

#[derive(Default)]
pub struct LightManager {
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
}

impl LightManager {
    pub fn add_light<T: Into<Light>>(&mut self, l: T) {
        use Light::*;
        match l.into() {
            Directional(dl) => {
                self.directional_lights.push(dl);
            }
            Point(pl) => {
                self.point_lights.push(pl);
            }
        }
    }
}
