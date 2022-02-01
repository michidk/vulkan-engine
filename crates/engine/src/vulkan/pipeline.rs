use ash::vk;

use crate::assets::shader;

/// Loads a vertex and fragment shader from the filesystem and creates a [`vk::ShaderModule`] for each.
pub(crate) fn create_shader_modules(
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

/// Creates a [`vk::Pipeline`] with the given options.
///
/// Used to reduce code duplication.
///
/// # Parameters
/// - `uses_vertex_attribs`: set to true if the Pipeline expects vertex data to be passed into the vertex shader, false is useful for PP Effects.
/// - `attachment_count`: the number of color attachments the pipeline expects.
/// - `blend_func`: a single [`vk::PipelineColorBlendAttachmentState`] to be used for every color attachment.
/// - `depth_test`: true if the Pipeline should have depth testing and writing enabled, false otherwise.
/// - `stencil_func`: an optional [`vk::StencilOpState`] to be used for stencil testing.
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_pipeline(
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
    wireframe: bool,
) -> Result<vk::Pipeline, vk::Result> {
    let vert_func_name = std::ffi::CString::new("vert").unwrap();
    let frag_func_name = std::ffi::CString::new("frag").unwrap();

    let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vertexshader_module)
        .name(&vert_func_name);
    let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(fragmentshader_module)
        .name(&frag_func_name);
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
        .cull_mode(if wireframe {
            vk::CullModeFlags::NONE
        } else {
            vk::CullModeFlags::BACK
        })
        .polygon_mode(if wireframe {
            vk::PolygonMode::LINE
        } else {
            vk::PolygonMode::FILL
        });
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
        if wireframe {
            depth_stencil_info = depth_stencil_info.back(stencil_func);
        }
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
    Ok(graphicspipeline)
}

pub(crate) fn create_ui_pipeline(
    device: &ash::Device,
    sampler_linear: vk::Sampler,
) -> (
    vk::DescriptorSetLayout,
    vk::PipelineLayout,
    vk::RenderPass,
    vk::Pipeline,
    vk::Pipeline,
) {
    let desc_set_layout = {
        let samplers = [sampler_linear];
        let bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .immutable_samplers(&samplers)
            .build()];
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&bindings)
            .build();

        unsafe { device.create_descriptor_set_layout(&info, None) }.unwrap()
    };

    let ranges = [vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(64)
        .build()];

    let pipeline_layout = {
        let sets = [desc_set_layout];
        let info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&sets)
            .push_constant_ranges(&ranges)
            .build();
        unsafe { device.create_pipeline_layout(&info, None) }.unwrap()
    };

    let renderpass = {
        let attachments = [vk::AttachmentDescription::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&refs)
            .build()];
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .build();
        unsafe { device.create_render_pass(&info, None) }.unwrap()
    };

    let (pipeline, pipeline_wireframe) = {
        let vert_func_name = std::ffi::CString::new("vert").unwrap();
        let frag_func_name = std::ffi::CString::new("frag").unwrap();

        let mut spv_vert = Vec::new();
        let mut spv_frag = Vec::new();
        let (vert_mod, frag_mod) =
            create_shader_modules("ui", device, &mut spv_vert, &mut spv_frag).unwrap();

        let mut spv_vert_wireframe = Vec::new();
        let mut spv_frag_wireframe = Vec::new();
        let (vert_mod_wireframe, frag_mod_wireframe) = create_shader_modules(
            "ui_debug",
            device,
            &mut spv_vert_wireframe,
            &mut spv_frag_wireframe,
        )
        .unwrap();

        let stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_mod)
                .name(&vert_func_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_mod)
                .name(&frag_func_name)
                .build(),
        ];
        let vertex_bindings = [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(20)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()];
        let vertex_attribs = [
            vk::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(8)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(2)
                .binding(0)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(16)
                .build(),
        ];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attribs)
            .build();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0)
            .build();
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .build();
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build()];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments)
            .build();
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dynamic_states)
            .build();
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .subpass(0)
            .build();

        let stages_wireframe = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_mod_wireframe)
                .name(&vert_func_name)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_mod_wireframe)
                .name(&frag_func_name)
                .build(),
        ];
        let info_wireframe = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&stages_wireframe)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(renderpass)
            .subpass(0)
            .build();

        let res = unsafe {
            device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[info, info_wireframe],
                None,
            )
        }
        .unwrap();

        unsafe {
            device.destroy_shader_module(vert_mod, None);
            device.destroy_shader_module(frag_mod, None);

            device.destroy_shader_module(vert_mod_wireframe, None);
            device.destroy_shader_module(frag_mod_wireframe, None);
        }

        (res[0], res[1])
    };

    (
        desc_set_layout,
        pipeline_layout,
        renderpass,
        pipeline,
        pipeline_wireframe,
    )
}
