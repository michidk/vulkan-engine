pub mod camera;
pub mod light;
pub mod model;

use self::light::LightManager;

pub struct Scene {
    pub light_manager: LightManager,
}

impl Scene {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            light_manager: LightManager::default(),
        })
    }
}