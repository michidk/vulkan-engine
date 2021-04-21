use ash::{version::DeviceV1_0, vk};
use crystal::prelude::{Vec2, Vec3, Vec4};
use std::{cell::RefCell, collections::HashMap, mem::size_of, rc::Rc};

use crate::vulkan::{
    descriptor_manager::DescriptorData,
    error::{GraphicsError, GraphicsResult},
    lighting_pipeline::LightingPipeline,
    pipeline,
    texture::Texture2D,
};

mod material_compiler;

/// Description of a single named property in a shader
enum MaterialProperty {
    Unsupported,
    Float { binding: u32, offset: u32 },
    Vec2 { binding: u32, offset: u32 },
    Vec3 { binding: u32, offset: u32 },
    Vec4 { binding: u32, offset: u32 },
    Sampler2D { binding: u32 },
}

/// A MaterialPipeline represents a GPass shader and its corresponding material properties.
///
/// Each MaterialPipeline can be used to create multiple [`Materials`](Material).
///
/// # MaterialProperties
/// Material properties are directly reflected from the shaders SPIRV code,
/// meaning Debug information has to be enabled when compiling the GLSL files.
///
/// Standalone properties are named after their variable name, e.g.
/// `uniform sampler2D u_AlbedoTex` will be named "u_AlbedoTex".
///
/// Properties inside uniform blocks are named after their inner names (i.e. the name of the surrounding block is ignored).
/// ```glsl
/// uniform MaterialData {
///     vec3 albedo;
/// } u_MaterialData;
/// ```
/// In the above code, a MaterialProperty named "albedo" will be exposed.
/// This means, multiple variables with the same name in different uniform blocks will clash in the material properties.
pub struct MaterialPipeline {
    device: Rc<ash::Device>,
    allocator: Rc<vk_mem::Allocator>,
    pipeline: vk::Pipeline,
    pipeline_wireframe: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    properties: HashMap<String, MaterialProperty>,
    resource_infos: Vec<DescriptorData>,
}

impl MaterialPipeline {
    /// Create a new MaterialPipeline from a given shader and [`LightingPipeline`].
    ///
    /// This function creates a new MaterialPipeline by loading the shader with the given name and reflecting its properties.
    ///
    /// # Parameters
    /// - `device`: Handle to the Vulkan Device
    /// - `allocator`: Handle to the Vulkan Allocator
    /// - `shader`: Name of the shader to use for this pipeline (minus the .glsl extension)
    /// - `frame_data_layout`: A DescriptorSetLayout describing the layout of descriptor set 0 of the pipeline (currently used for Camera matrices)
    /// - `renderpass`: The Deferred RenderPass (of which subpass 0 will be used for this pipeline)
    /// - `lighing_pipeline`: The [LightingPipeline] which will be used in the Deferred Resolve Pass for Materials created with this MaterialPipeline
    pub fn new(
        device: Rc<ash::Device>,
        allocator: Rc<vk_mem::Allocator>,
        shader: &str,
        frame_data_layout: vk::DescriptorSetLayout,
        renderpass: vk::RenderPass,
        lighting_pipeline: &LightingPipeline,
    ) -> GraphicsResult<Rc<MaterialPipeline>> {
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
        let refl = ve_shader_reflect::merge(refl_vertex, &refl_fragment, false)?;

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
                ve_shader_reflect::SetBindingData::SampledImage {
                    dim: ve_shader_reflect::ImageDimension::Two,
                } => {
                    properties.insert(
                        binding.var_name.clone(),
                        MaterialProperty::Sampler2D {
                            binding: binding.binding,
                        },
                    );

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
            false,
        )?;
        let pipeline_wireframe = pipeline::create_pipeline(
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
            true,
        )?;

        unsafe {
            device.destroy_shader_module(vertex_shader, None);
            device.destroy_shader_module(fragment_shader, None);
        }

        Ok(Rc::new(MaterialPipeline {
            device,
            allocator,
            pipeline,
            pipeline_wireframe,
            pipeline_layout,
            descriptor_set_layout,
            properties,
            resource_infos,
        }))
    }

    /// Creates a new [`Material`] from the given MaterialPipeline.
    pub fn create_material(self: &Rc<Self>) -> GraphicsResult<Rc<Material>> {
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
            self.device.destroy_pipeline(self.pipeline_wireframe, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

/// A Material is an instance of a [`MaterialPipeline`].
///
/// While a [`MaterialPipeline`] stores information about which properties a Material exposes, a Material stores the values of each exposed Property.
/// Thus a [`MaterialPipeline`] can be viewed as a Material Template, while a Material is an instantiation of such a template.
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

    /// Sets a MaterialProperty of type float
    ///
    /// For naming scheme, see [`MaterialPipeline`]
    ///
    /// # Errors
    /// - [`GraphicsError::InvalidMaterialProperty`] when no property with name `name` exists
    /// - [`GraphicsError::InvalidMaterialPropertyType`] when property `name` does not have type `float`
    /// - [`GraphicsError::VkMem`]
    pub fn set_float(&self, name: &str, val: f32) -> GraphicsResult<()> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| GraphicsError::InvalidMaterialProperty(name.to_owned()))?;
        match prop {
            MaterialProperty::Float { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<f32>() as u64,
                    &val as *const f32 as *const u8,
                )?;
            }
            _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
        }

        Ok(())
    }

    /// Sets a MaterialProperty of type vec2
    ///
    /// For naming scheme, see [`MaterialPipeline`]
    ///
    /// # Errors
    /// - [`GraphicsError::InvalidMaterialProperty`] when no property with name `name` exists
    /// - [`GraphicsError::InvalidMaterialPropertyType`] when property `name` does not have type `vec2`
    /// - [`GraphicsError::VkMem`]
    pub fn set_vec2(&self, name: &str, val: Vec2<f32>) -> GraphicsResult<()> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| GraphicsError::InvalidMaterialProperty(name.to_owned()))?;
        match prop {
            MaterialProperty::Vec2 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec2<f32>>() as u64,
                    &val as *const Vec2<f32> as *const u8,
                )?;
            }
            _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
        }

        Ok(())
    }

    /// Sets a MaterialProperty of type vec3
    ///
    /// For naming scheme, see [`MaterialPipeline`]
    ///
    /// # Errors
    /// - [`GraphicsError::InvalidMaterialProperty`] when no property with name `name` exists
    /// - [`GraphicsError::InvalidMaterialPropertyType`] when property `name` does not have type `vec3`
    /// - [`GraphicsError::VkMem`]
    pub fn set_vec3(&self, name: &str, val: Vec3<f32>) -> GraphicsResult<()> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| GraphicsError::InvalidMaterialProperty(name.to_owned()))?;
        match prop {
            MaterialProperty::Vec3 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec3<f32>>() as u64,
                    &val as *const Vec3<f32> as *const u8,
                )?;
            }
            _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
        }

        Ok(())
    }

    /// Sets a MaterialProperty of type vec4
    ///
    /// For naming scheme, see [`MaterialPipeline`]
    ///
    /// # Errors
    /// - [`GraphicsError::InvalidMaterialProperty`] when no property with name `name` exists
    /// - [`GraphicsError::InvalidMaterialPropertyType`] when property `name` does not have type `vec4`
    /// - [`GraphicsError::VkMem`]
    pub fn set_vec4(&self, name: &str, val: Vec4<f32>) -> GraphicsResult<()> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| GraphicsError::InvalidMaterialProperty(name.to_owned()))?;
        match prop {
            MaterialProperty::Vec4 { binding, offset } => {
                self.set_uniform_property(
                    &self.allocations[*binding as usize],
                    *offset as u64,
                    size_of::<Vec4<f32>>() as u64,
                    &val as *const Vec4<f32> as *const u8,
                )?;
            }
            _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
        }

        Ok(())
    }

    /// Sets a MaterialProperty of type sampler2D
    ///
    /// For naming scheme, see [`MaterialPipeline`]
    ///
    /// # Errors
    /// - [`GraphicsError::InvalidMaterialProperty`] when no property with name `name` exists
    /// - [`GraphicsError::InvalidMaterialPropertyType`] when property `name` does not have type `sampler2D`
    pub fn set_texture(&self, name: &str, val: Rc<Texture2D>) -> GraphicsResult<()> {
        let prop = self
            .pipeline
            .properties
            .get(name)
            .ok_or_else(|| GraphicsError::InvalidMaterialProperty(name.to_owned()))?;
        match prop {
            MaterialProperty::Sampler2D { binding } => {
                self.textures
                    .borrow_mut()
                    .insert(name.to_owned(), val.clone());

                let res = &mut self.resources.borrow_mut()[*binding as usize];
                match res {
                    DescriptorData::ImageSampler {
                        image,
                        layout,
                        sampler,
                    } => {
                        *image = val.view;
                        *layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                        *sampler = val.sampler;
                    }
                    _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
                }
            }
            _ => return Err(GraphicsError::InvalidMaterialPropertyType(name.to_owned())),
        }

        Ok(())
    }

    /// Returns the vk::PipelineLayout associated with this Material
    pub fn get_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.pipeline_layout
    }

    /// Returns the vk::Pipeline that has to be used with this Material
    pub fn get_pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline
    }

    pub fn get_wireframe_pipeline(&self) -> vk::Pipeline {
        self.pipeline.pipeline_wireframe
    }

    /// Returns the vk::DescriptorSetLayout of set #1 of this Material's Pipeline.
    ///
    /// Set #1 should contain all MaterialProperties
    pub fn get_descriptor_set_layout(&self) -> vk::DescriptorSetLayout {
        self.pipeline.descriptor_set_layout
    }

    /// Returns the DescriptorData entries that can be used to get a valid DescriptorSet
    /// from the [DescriptorManager](crate::vulkan::descriptor_manager::DescriptorManager).
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
