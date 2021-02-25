use ash::{version::DeviceV1_0, vk};
use crystal::prelude::Vec3;

use crate::vulkan::buffer::BufferWrapper;
pub struct DirectionalLight {
    pub direction: Vec3<f32>,
    pub illuminance: Vec3<f32>, // in lx = lm / m^2
}

pub struct PointLight {
    pub position: Vec3<f32>,      // in m
    pub luminous_flux: Vec3<f32>, // in lm
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
    directional_lights: Vec<DirectionalLight>,
    point_lights: Vec<PointLight>,
    is_dirty: bool,
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
        self.is_dirty = true;
    }

    pub fn update_buffer(
        &self,
        logical_device: &ash::Device,
        allocator: &vk_mem::Allocator,
        buffer: &mut BufferWrapper,
        descriptor_sets_light: &mut [vk::DescriptorSet],
    ) -> Result<(), vk_mem::error::Error> {
        if !self.is_dirty {
            return Ok(());
        }

        // push padding float as vulkan vecs are always 4 * T
        let mut data: Vec<f32> = vec![
            self.directional_lights.len() as f32,
            self.point_lights.len() as f32,
            0.0,
            0.0,
        ];
        for dl in &self.directional_lights {
            data.push(*dl.direction.x());
            data.push(*dl.direction.y());
            data.push(*dl.direction.z());
            // push padding float as vulkan vecs are always 4 * T
            data.push(0.0);
            data.push(*dl.illuminance.x());
            data.push(*dl.illuminance.y());
            data.push(*dl.illuminance.z());
            // push padding float as vulkan vecs are always 4 * T
            data.push(0.0);
        }
        for pl in &self.point_lights {
            data.push(*pl.position.x());
            data.push(*pl.position.y());
            data.push(*pl.position.z());
            // push padding float as vulkan vecs are always 4 * T
            data.push(0.0);
            data.push(*pl.luminous_flux.x());
            data.push(*pl.luminous_flux.y());
            data.push(*pl.luminous_flux.z());
            // push padding float as vulkan vecs are always 4 * T
            data.push(0.0);
        }
        buffer.fill(allocator, &data)?;
        for descset in descriptor_sets_light {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: buffer.buffer,
                offset: 0,
                range: 4 * data.len() as u64,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe {
                logical_device.update_descriptor_sets(&desc_sets_write, &[]);
            }
        }
        Ok(())
    }
}

impl Default for LightManager {
    fn default() -> Self {
        LightManager {
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            is_dirty: false,
        }
    }
}
