use crystal::prelude::Vec4;

#[repr(C)]
pub struct DirectionalLight {
    pub direction: Vec4<f32>,
    pub illuminance: Vec4<f32>, // in lx = lm / m^2
}

#[repr(C)]
pub struct PointLight {
    pub position: Vec4<f32>,
    pub luminous_flux: Vec4<f32>, // in lm
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

impl Default for LightManager {
    fn default() -> Self {
        LightManager {
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
        }
    }
}
