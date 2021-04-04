use ash::{version::DeviceV1_0, vk};

use crate::assets::shader;

pub fn create_shader_modules(
    shader: &str,
    device: &ash::Device,
    out_spv_vert: &mut Vec<u32>,
    out_spv_frag: &mut Vec<u32>,
) -> Result<(vk::ShaderModule, vk::ShaderModule), vk::Result> {
    let vertexshader_createinfo = shader::load(shader, shader::ShaderKind::Vertex, out_spv_vert);
    let vertexshader_module =
        unsafe { device.create_shader_module(&vertexshader_createinfo, None)? };

    let fragmentshader_createinfo =
        shader::load(shader, shader::ShaderKind::Fragment, out_spv_frag);
    let fragmentshader_module =
        unsafe { device.create_shader_module(&fragmentshader_createinfo, None)? };

    Ok((vertexshader_module, fragmentshader_module))
}

#[allow(clippy::too_many_arguments)]
pub fn create_pipeline(
    layout: vk::PipelineLayout,
    renderpass: vk::RenderPass,
    subpass: u32,
    uses_vertex_attribs: bool,
    attachment_count: usize,
    blend_func: vk::PipelineColorBlendAttachmentState,
    depth_test: bool,
    stencil_func: Option<vk::StencilOpState>,
    device: &ash::Device,
    vertexshader_module: vk::ShaderModule,
    fragmentshader_module: vk::ShaderModule,
) -> Result<vk::Pipeline, vk::Result> {
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
        },
    ];
    let vertex_binding_descs = [vk::VertexInputBindingDescription {
        binding: 0,
        stride: 44,
        input_rate: vk::VertexInputRate::VERTEX,
    }];
    let mut vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();
    if uses_vertex_attribs {
        vertex_input_info = vertex_input_info
            .vertex_attribute_descriptions(&vertex_attrib_descs)
            .vertex_binding_descriptions(&vertex_binding_descs);
    }
    let vertex_input_info = vertex_input_info.build();

    let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let scissors = [vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: i32::MAX as u32,
            height: i32::MAX as u32,
        },
    }];
    let viewports = [vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        min_depth: 0.0,
        max_depth: 1.0,
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

    let colourblend_attachments = vec![blend_func; attachment_count];
    let colourblend_info =
        vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);

    let mut depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(depth_test)
        .depth_write_enable(depth_test)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .stencil_test_enable(stencil_func.is_some());
    if let Some(stencil_func) = stencil_func {
        depth_stencil_info = depth_stencil_info.front(stencil_func);
    }
    let depth_stencil_info = depth_stencil_info.build();

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
        .subpass(subpass);
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
