use std::{marker::PhantomData, rc::Rc};
use ash::vk;

pub use vulkan_engine_derive::MaterialBindingVertex;
pub use vulkan_engine_derive::MaterialBindingFragment;
pub use vulkan_engine_derive::MaterialData;

use crate::vulkan::descriptor_manager::DescriptorData;

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
    Fragment
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

pub struct MaterialDataLayout{
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
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    phantom: PhantomData<T>,
}

impl<T: MaterialData> MaterialPipeline<T> {
    pub fn new(device: &ash::Device, shader: &str, frame_data_layout: vk::DescriptorSetLayout, renderpass: vk::RenderPass, width: u32, height: u32) -> Result<Rc<MaterialPipeline<T>>, MaterialError> {
        let descriptor_set_layout = material_compiler::compile_descriptor_set_layout(device, &T::get_material_layout())?;
        let pipeline_layout = material_compiler::compile_pipeline_layout(device, &[frame_data_layout, descriptor_set_layout])?;
        let pipeline = material_compiler::compile_pipeline(device, pipeline_layout, shader, renderpass, width, height)?;

        Ok(Rc::new(MaterialPipeline {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            phantom: PhantomData
        }))
    }

    pub fn create_material(self: &Rc<Self>, data: T, allocator: &vk_mem::Allocator) -> Result<Rc<Material<T>>, MaterialError> {
        let (resources, allocations) = material_compiler::compile_resources(&data, allocator)?;

        Ok(Rc::new(Material {
            pipeline: self.clone(),
            resources,
            allocations,
        }))
    }
}

pub struct Material<T: MaterialData> {
    pipeline: Rc<MaterialPipeline<T>>,
    resources: Vec<DescriptorData>,
    allocations: Vec<vk_mem::Allocation>,
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
