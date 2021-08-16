use std::rc::Rc;

use ash::{extensions::khr::RayTracingPipeline, vk};

use crate::assets::shader;

/// Loads a vertex and fragment shader from the filesystem and creates a [`vk::ShaderModule`] for each.
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

pub fn create_rtx_pipeline(
    layout: vk::PipelineLayout,
    ray_gen_shader: &str,
    ray_chit_shader: &str,
    device: &ash::Device,
    rtx_ext: Rc<RayTracingPipeline>,
) -> vk::Pipeline {
    let func_name = std::ffi::CString::new("main").unwrap();

    let mut rgen_code = Vec::new();
    let rgen_shader_module = unsafe {
        device
            .create_shader_module(
                &shader::load_single(ray_gen_shader.to_owned(), &mut rgen_code),
                None,
            )
            .unwrap()
    };

    let mut rchit_code = Vec::new();
    let rchit_shader_module = unsafe {
        device
            .create_shader_module(
                &shader::load_single(ray_chit_shader.to_owned(), &mut rchit_code),
                None,
            )
            .unwrap()
    };

    let stages = [
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::RAYGEN_KHR)
            .module(rgen_shader_module)
            .name(&func_name)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
            .module(rchit_shader_module)
            .name(&func_name)
            .build(),
    ];
    let groups = [
        vk::RayTracingShaderGroupCreateInfoKHR::builder()
            .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
            .general_shader(0)
            .closest_hit_shader(vk::SHADER_UNUSED_KHR)
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR)
            .build(),
        vk::RayTracingShaderGroupCreateInfoKHR::builder()
            .ty(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
            .general_shader(vk::SHADER_UNUSED_KHR)
            .closest_hit_shader(1)
            .any_hit_shader(vk::SHADER_UNUSED_KHR)
            .intersection_shader(vk::SHADER_UNUSED_KHR)
            .build(),
    ];
    let pipe = unsafe {
        rtx_ext.create_ray_tracing_pipelines(
            vk::DeferredOperationKHR::null(),
            vk::PipelineCache::null(),
            &[vk::RayTracingPipelineCreateInfoKHR::builder()
                .stages(&stages)
                .groups(&groups)
                .max_pipeline_ray_recursion_depth(1)
                .layout(layout)
                .build()],
            None,
        )
    }
    .unwrap()[0];

    unsafe {
        device.destroy_shader_module(rgen_shader_module, None);
        device.destroy_shader_module(rchit_shader_module, None);
    }

    pipe
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
