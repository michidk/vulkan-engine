use ash::{version::DeviceV1_0, vk};
use crystal::prelude::{Vec2, Vec3, Vec4};
use std::{cell::RefCell, collections::HashMap, mem::size_of, rc::Rc};

use crate::vulkan::{descriptor_manager::DescriptorData, lighting_pipeline::LightingPipeline, pipeline, texture::Texture2D};

mod material_compiler;

#[derive(thiserror::Error, Debug)]
pub enum MaterialError {
    #[error("Vulkan error {0}")]
    VulkanError(#[from] vk::Result),
    #[error("VkMem error {0}")]
    VkMemError(#[from] vk_mem::Error),
    #[error("Shader reflection Error: {0}")]
    ShaderReflectError(#[from] ve_shader_reflect::Error),
    #[error("Material Error: {0}")]
    MaterialError(String),
    #[error("Invalid Property: {0}")]
    InvalidProperty(String),
    #[error("Incompatible Property Type")]
    IncompatiblePropertyType,
}

enum MaterialProperty {
    Unsupported,
    Float { binding: u32, offset: u32 },
    Vec2 { binding: u32, offset: u32 },
    Vec3 { binding: u32, offset: u32 },
    Vec4 { binding: u32, offset: u32 },
    Sampler2D { binding: u32 },
}

pub struct MaterialPipeline {
    device: Rc<ash::Device>,
    allocator: Rc<vk_mem::Allocator>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    properties: HashMap<String, MaterialProperty>,
    resource_infos: Vec<DescriptorData>,
}

impl MaterialPipeline {
    pub fn new(
        device: Rc<ash::Device>,
        allocator: Rc<vk_mem::Allocator>,
        shader: &str,
        frame_data_layout: vk::DescriptorSetLayout,
        renderpass: vk::RenderPass,
        lighting_pipeline: &LightingPipeline,
    ) -> Result<Rc<MaterialPipeline>, MaterialError> {
        let mut vertexshader_code = Vec::new();
        let mut fragmentshader_code = Vec::new();
        let (vertex_shader, fragment_shader) = pipeline::create_shader_modules(
            shader,
            &device,
            &mut vertexshader_code,
            &mut fragmentshader_code,
        )?;

        let refl_vertex = ve_shader_reflect::reflect_shader(&vertexshader_code)?;
        let refl_fragment = ve_shader_reflect::reflect_shader(&fragmentshader_code)?;
        let refl = ve_shader_reflect::merge(refl_vertex, &refl_fragment, false).unwrap();

        let mut properties = HashMap::new();
        let mut resource_infos = Vec::new();
        for binding in &refl.set_bindings {
            if binding.set != 1 {
                continue;
            }

            let res_info = match &binding.data {
                ve_shader_reflect::SetBindingData::UniformBuffer { layout } => {
                    for property in &layout.members {
                        let prop = match property.kind {
                            ve_shader_reflect::BlockMemberType::Unsupported => {
                                MaterialProperty::Unsupported
                            }
                            ve_shader_reflect::BlockMemberType::Float => MaterialProperty::Float {
                                binding: binding.binding,
                                offset: property.offset,
                            },
                            ve_shader_reflect::BlockMemberType::FloatVector(2) => {
                                MaterialProperty::Vec2 {
                                    binding: binding.binding,
                                    offset: property.offset,
                                }
                            }
                            ve_shader_reflect::BlockMemberType::FloatVector(3) => {
                                MaterialProperty::Vec3 {
                                    binding: binding.binding,
                                    offset: property.offset,
                                }
                            }
                            ve_shader_reflect::BlockMemberType::FloatVector(4) => {
                                MaterialProperty::Vec4 {
                                    binding: binding.binding,
                                    offset: property.offset,
                                }
                            }
                            ve_shader_reflect::BlockMemberType::FloatMatrix(_) => {
                                MaterialProperty::Unsupported
                            }
                            _ => MaterialProperty::Unsupported,
                        };
                        properties.insert(property.name.clone(), prop);
                    }

                    DescriptorData::UniformBuffer {
                        buffer: vk::Buffer::null(),
                        offset: 0,
                        size: layout.total_size as u64,
                    }
                }
                ve_shader_reflect::SetBindingData::SampledImage { dim: ve_shader_reflect::ImageDimension::Two } => {
                    properties.insert(binding.var_name.clone(), MaterialProperty::Sampler2D {
                        binding: binding.binding,
                    });

                    DescriptorData::ImageSampler {
                        image: vk::ImageView::null(),
                        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                        sampler: vk::Sampler::null(),
                    }
                }
                _ => DescriptorData::None,
            };

            if binding.binding >= resource_infos.len() as u32 {
                resource_infos.resize(binding.binding as usize + 1, DescriptorData::None);
            }
            resource_infos[binding.binding as usize] = res_info;
        }

        let descriptor_set_layout =
            material_compiler::compile_descriptor_set_layout(device.as_ref(), &resource_infos)?;
        let pipeline_layout = material_compiler::compile_pipeline_layout(
            device.as_ref(),
            &[frame_data_layout, descriptor_set_layout],
        )?;

        let blend_func = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build();
        let stencil_func = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::REPLACE)
            .compare_op(vk::CompareOp::ALWAYS)
            .write_mask(0xFF)
            .compare_mask(0xFF)
            .reference(lighting_pipeline.stencil_id as u32)
            .build();
        let pipeline = pipeline::create_pipeline(
            pipeline_layout,
            renderpass,
            0,
            true,
            2,
            blend_func,
            true,
            Some(stencil_func),
            &device,
            vertex_shader,
            fragment_shader,
        )?;

        Ok(Rc::new(MaterialPipeline {
            device,
            allocator,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            properties,
            resource_infos,
        }))
    }

    pub fn create_material(self: &Rc<Self>) -> Result<Rc<Material>, MaterialError> {
        let (resources, allocations) =
            material_compiler::compile_resources(&self.resource_infos, self.allocator.as_ref())?;

        Ok(Rc::new(Material {
            pipeline: self.clone(),
            resources: RefCell::new(resources),
            allocations,
            textures: RefCell::new(HashMap::new()),
        }))
    }
}

impl Drop for MaterialPipeline {
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

pub struct Material {
    pipeline: Rc<MaterialPipeline>,
    resources: RefCell<Vec<DescriptorData>>,
    allocations: Vec<vk_mem::Allocation>,
    textures: RefCell<HashMap<String, Rc<Texture2D>>>,
}

impl Material {
    fn set_uniform_property(
        &self,
        alloc: &vk_mem::Allocation,
        offset: u64,
        size: u64,
        data: *const u8,
    ) -> Result<(), vk_mem::Error> {
        let map = self.pipeline.allocator.map_memory(alloc)?;
        unsafe {
            map.offset(offset as isize)
                .copy_from_nonoverlapping(data, size as usize);
        }
        self.pipeline.allocator.unmap_memory(alloc);

        Ok(())
    }

    pub fn set_float(&self, name: &str, val: f32) -> Result<(), MaterialError> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| MaterialError::InvalidProperty(String::from(name)))?;
        match prop {
            MaterialProperty::Float { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<f32>() as u64,
                    &val as *const f32 as *const u8,
                )?;
            }
            _ => return Err(MaterialError::IncompatiblePropertyType),
        }

        Ok(())
    }
    pub fn set_vec2(&self, name: &str, val: Vec2<f32>) -> Result<(), MaterialError> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| MaterialError::InvalidProperty(String::from(name)))?;
        match prop {
            MaterialProperty::Vec2 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec2<f32>>() as u64,
                    &val as *const Vec2<f32> as *const u8,
                )?;
            }
            _ => return Err(MaterialError::IncompatiblePropertyType),
        }

        Ok(())
    }
    pub fn set_vec3(&self, name: &str, val: Vec3<f32>) -> Result<(), MaterialError> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| MaterialError::InvalidProperty(String::from(name)))?;
        match prop {
            MaterialProperty::Vec3 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec3<f32>>() as u64,
                    &val as *const Vec3<f32> as *const u8,
                )?;
            }
            _ => return Err(MaterialError::IncompatiblePropertyType),
        }

        Ok(())
    }
    pub fn set_vec4(&self, name: &str, val: Vec4<f32>) -> Result<(), MaterialError> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| MaterialError::InvalidProperty(String::from(name)))?;
        match prop {
            MaterialProperty::Vec4 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec4<f32>>() as u64,
                    &val as *const Vec4<f32> as *const u8,
                )?;
            }
            _ => return Err(MaterialError::IncompatiblePropertyType),
        }

        Ok(())
    }

    pub fn set_texture(&self, name: &str, val: Rc<Texture2D>) -> Result<(), MaterialError> {
        let prop = self.pipeline.properties.get(name).ok_or_else(|| MaterialError::InvalidProperty(String::from(name)))?;
        match prop {
            &MaterialProperty::Sampler2D { binding } => {
                self.textures.borrow_mut().insert(String::from(name), val.clone());
                
                let res = &mut self.resources.borrow_mut()[binding as usize];
                match res {
                    DescriptorData::ImageSampler { image, layout, sampler } => {
                        *image = val.view;
                        *layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                        *sampler = val.sampler;
                    }
                    _ => return Err(MaterialError::IncompatiblePropertyType),
                }
            }
            _ => return Err(MaterialError::IncompatiblePropertyType),
        }

        Ok(())
    }

    pub fn get_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.pipeline_layout
    }
    pub fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline
    }
    pub fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.pipeline.descriptor_set_layout
    }
    pub fn get_descriptor_data(&self) -> Vec<DescriptorData> {
        self.resources.borrow().clone()
    }
}

impl Drop for Material {
    fn drop(&mut self) {
        unsafe {
            for r in &*self.resources.borrow() {
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
