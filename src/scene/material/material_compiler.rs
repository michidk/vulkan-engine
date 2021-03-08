use ash::{version::DeviceV1_0, vk};

use super::{MaterialData, MaterialDataLayout};
use crate::{assets::shader, vulkan::descriptor_manager::DescriptorData};

pub fn compile_descriptor_set_layout(
    device: &ash::Device,
    layout: &MaterialDataLayout,
) -> Result<vk::DescriptorSetLayout, vk::Result> {
    let mut bindings = Vec::with_capacity(layout.bindings.len());
    for (i, b) in layout.bindings.iter().enumerate() {
        let stage;
        match &b.binding_stage {
            super::MaterialDataBindingStage::Vertex => {
                stage = vk::ShaderStageFlags::VERTEX;
            }
            super::MaterialDataBindingStage::Fragment => {
                stage = vk::ShaderStageFlags::FRAGMENT;
            }
        }

        let vk_type;
        match &b.binding_type {
            super::MaterialDataBindingType::Uniform => {
                vk_type = vk::DescriptorType::UNIFORM_BUFFER;
            }
        }

        bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(i as u32)
                .descriptor_count(1)
                .descriptor_type(vk_type)
                .stage_flags(stage)
                .build(),
        );
    }

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
    Ok(set_layout)
}

pub fn compile_pipeline_layout(
    device: &ash::Device,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<vk::PipelineLayout, vk::Result> {
    // all devices must at least support 128 bytes of push constants, so this is safe
    let push_constants = [
        vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(128)
            .build()
    ];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(layouts)
        .push_constant_ranges(&push_constants)
        .build();
    unsafe { device.create_pipeline_layout(&layout_info, None) }
}

pub fn compile_pipeline(
    device: &ash::Device,
    layout: vk::PipelineLayout,
    shader: &str,
    renderpass: vk::RenderPass,
    stencil_id: u8
) -> Result<vk::Pipeline, vk::Result> {
    let (mut vertexshader_code, mut fragmentshader_code) = (Vec::new(), Vec::new());
    let vertexshader_createinfo =
        shader::load(shader, shader::ShaderKind::Vertex, &mut vertexshader_code);
    let vertexshader_module =
        unsafe { device.create_shader_module(&vertexshader_createinfo, None)? };
    let fragmentshader_createinfo = shader::load(
        shader,
        shader::ShaderKind::Fragment,
        &mut fragmentshader_code,
    );
    let fragmentshader_module =
        unsafe { device.create_shader_module(&fragmentshader_createinfo, None)? };
    drop(vertexshader_code);
    drop(fragmentshader_code);
    let mainfunctionname = std::ffi::CString::new("main").unwrap();

    let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertexshader_module)
        .name(&mainfunctionname);
    let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragmentshader_module)
        .name(&mainfunctionname);
    let shader_stages = [vertexshader_stage.build(), fragmentshader_stage.build()];

    let vertex_attrib_descs = [
        // position
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            offset: 0,
            format: vk::Format::R32G32B32_SFLOAT,
        },
        // color
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            offset: 12,
            format: vk::Format::R32G32B32_SFLOAT,
        },
        // normal
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            offset: 24,
            format: vk::Format::R32G32B32_SFLOAT,
        },
        // uv
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 3,
            offset: 36,
            format: vk::Format::R32G32_SFLOAT,
        }
    ];
    let vertex_binding_descs = [
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: 44,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    ];
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_attribute_descriptions(&vertex_attrib_descs)
        .vertex_binding_descriptions(&vertex_binding_descs);

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewports = [
        vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    ];
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: i32::MAX as u32,
            height: i32::MAX as u32,
        },
    }];

    let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&scissors)
        .viewports(&viewports);
    let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .line_width(1.0)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .cull_mode(vk::CullModeFlags::BACK)
        .polygon_mode(vk::PolygonMode::FILL);
    let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let colourblend_attachments = [
        vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build(),
        vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()
    ];
    let colourblend_info =
        vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
    
    let stencil_front = vk::StencilOpState::builder()
        .fail_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::REPLACE)
        .compare_op(vk::CompareOp::ALWAYS)
        .write_mask(0xFF)
        .compare_mask(0xFF)
        .reference(stencil_id as u32)
        .build();
    let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .stencil_test_enable(true)
        .front(stencil_front)
        .build();

    let dynamic_states = [vk::DynamicState::VIEWPORT];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
        .dynamic_states(&dynamic_states)
        .build();

    let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input_info)
        .input_assembly_state(&input_assembly_info)
        .viewport_state(&viewport_info)
        .rasterization_state(&rasterizer_info)
        .multisample_state(&multisampler_info)
        .depth_stencil_state(&depth_stencil_info)
        .color_blend_state(&colourblend_info)
        .layout(layout)
        .render_pass(renderpass)
        .dynamic_state(&dynamic_state)
        .subpass(0);
    let graphicspipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info.build()], None)
            .expect("A problem with the pipeline creation")
    }[0];
    unsafe {
        device.destroy_shader_module(fragmentshader_module, None);
        device.destroy_shader_module(vertexshader_module, None);
    }
    Ok(graphicspipeline)
}

pub fn compile_resources<T: MaterialData>(
    resources: &T,
    allocator: &vk_mem::Allocator,
) -> Result<(Vec<DescriptorData>, Vec<vk_mem::Allocation>), vk_mem::Error> {
    let helpers = resources.get_material_resource_helpers();

    let mut resources = Vec::with_capacity(helpers.len());
    let mut allocations = Vec::with_capacity(helpers.len());

    for res in helpers {
        match res {
            super::MaterialResourceHelper::UniformBuffer(data) => {
                let buffer_info = vk::BufferCreateInfo::builder()
                    .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                    .size(data.len() as u64)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build();
                let alloc_info = vk_mem::AllocationCreateInfo {
                    usage: vk_mem::MemoryUsage::CpuToGpu,
                    ..Default::default()
                };
                let (buffer, alloc, _) = allocator.create_buffer(&buffer_info, &alloc_info)?;

                let map = allocator.map_memory(&alloc)?;
                unsafe {
                    map.copy_from_nonoverlapping(data.as_ptr(), data.len());
                }
                allocator.unmap_memory(&alloc);

                allocations.push(alloc);
                resources.push(DescriptorData::UniformBuffer {
                    buffer,
                    offset: 0,
                    size: data.len() as u64,
                });
            }
        }
    }

    Ok((resources, allocations))
}
