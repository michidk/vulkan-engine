use ash::{version::DeviceV1_0, vk};
use std::{marker::PhantomData, rc::Rc};

pub use vulkan_engine_derive::MaterialBindingFragment;
pub use vulkan_engine_derive::MaterialBindingVertex;
pub use vulkan_engine_derive::MaterialData;

use crate::vulkan::{descriptor_manager::DescriptorData, lighting_pipeline::LightingPipeline};

mod material_compiler;

#[derive(thiserror::Error, Debug)]
pub enum MaterialError {
    #[error("Vulkan error {0}")]
    VulkanError(#[from] vk::Result),
    #[error("VkMem error {0}")]
    VkMemError(#[from] vk_mem::Error),
    #[error("Material Error: {0}")]
    MaterialError(String),
}

pub enum MaterialDataBindingStage {
    Vertex,
    Fragment,
}

pub enum MaterialDataBindingType {
    Uniform,
}

pub struct MaterialDataBinding {
    pub binding_type: MaterialDataBindingType,
    pub binding_stage: MaterialDataBindingStage,
}

pub enum MaterialResourceHelper<'a> {
    UniformBuffer(&'a [u8]),
}

pub struct MaterialDataLayout {
    pub bindings: Vec<MaterialDataBinding>,
}

pub trait MaterialBinding {
    fn get_material_binding() -> MaterialDataBinding;
    fn get_material_resource_helper(&self) -> MaterialResourceHelper;
}

pub trait MaterialData {
    fn get_material_layout() -> MaterialDataLayout;
    fn get_material_resource_helpers(&self) -> Vec<MaterialResourceHelper>;
}

pub struct MaterialPipeline<T: MaterialData> {
    device: Rc<ash::Device>,
    allocator: Rc<vk_mem::Allocator>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    phantom: PhantomData<T>,
}

impl<T: MaterialData> MaterialPipeline<T> {
    pub fn new(
        device: Rc<ash::Device>,
        allocator: Rc<vk_mem::Allocator>,
        shader: &str,
        frame_data_layout: vk::DescriptorSetLayout,
        renderpass: vk::RenderPass,
        lighting_pipeline: &LightingPipeline
    ) -> Result<Rc<MaterialPipeline<T>>, MaterialError> {
        let descriptor_set_layout = material_compiler::compile_descriptor_set_layout(
            device.as_ref(),
            &T::get_material_layout(),
        )?;
        let pipeline_layout = material_compiler::compile_pipeline_layout(
            device.as_ref(),
            &[frame_data_layout, descriptor_set_layout],
        )?;
        let pipeline = material_compiler::compile_pipeline(
            device.as_ref(),
            pipeline_layout,
            shader,
            renderpass,
            lighting_pipeline.stencil_id
        )?;

        Ok(Rc::new(MaterialPipeline {
            device,
            allocator,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            phantom: PhantomData
        }))
    }

    pub fn create_material(self: &Rc<Self>, data: T) -> Result<Rc<Material<T>>, MaterialError> {
        let (resources, allocations) =
            material_compiler::compile_resources(&data, self.allocator.as_ref())?;

        Ok(Rc::new(Material {
            pipeline: self.clone(),
            resources,
            allocations,
        }))
    }
}

impl<T: MaterialData> Drop for MaterialPipeline<T> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

pub struct Material<T: MaterialData> {
    pipeline: Rc<MaterialPipeline<T>>,
    resources: Vec<DescriptorData>,
    allocations: Vec<vk_mem::Allocation>,
}

impl<T: MaterialData> Drop for Material<T> {
    fn drop(&mut self) {
        unsafe {
            for r in &self.resources {
                if let DescriptorData::UniformBuffer { buffer, .. } = r {
                    self.pipeline.device.destroy_buffer(*buffer, None);
                }
            }
            for a in &self.allocations {
                self.pipeline.allocator.free_memory(a);
            }
        }
    }
}

pub trait MaterialInterface {
    fn get_pipeline_layout(&self) -> vk::PipelineLayout;
    fn get_pipeline(&self) -> vk::Pipeline;
    fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout;
    fn get_descriptor_data(&self) -> &Vec<DescriptorData>;
}

impl<T: MaterialData> MaterialInterface for Material<T> {
    fn get_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.pipeline_layout
    }
    fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline
    }
    fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.pipeline.descriptor_set_layout
    }
    fn get_descriptor_data(&self) -> &Vec<DescriptorData> {
        &self.resources
    }
}
