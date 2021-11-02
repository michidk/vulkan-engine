pub mod light;
pub mod material;
pub mod model;
pub mod transform;

use self::light::LightManager;
use self::model::Model;

pub struct Scene {
    pub light_manager: LightManager,
    pub models: Vec<Model>,
}

impl Scene {}

impl Scene {
    pub(crate) fn new() -> Self {
        Self {
            light_manager: LightManager::default(),
            models: Vec::new(),
        }
    }

    pub fn add(&mut self, model: Model) {
        self.models.push(model);
    }
}
