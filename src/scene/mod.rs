pub mod camera;
pub mod light;
pub mod model;
pub mod transform;
pub mod material;

use self::model::Model;
use self::{light::LightManager};

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
