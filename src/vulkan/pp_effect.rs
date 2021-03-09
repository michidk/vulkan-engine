use std::rc::Rc;

use ash::{version::DeviceV1_0, vk};

use crate::assets::shader;


pub struct PPEffect {
    pub pipeline: vk::Pipeline,
    device: Rc<ash::Device>,
}

impl PPEffect {
    pub fn new(shader: &str, pipe_layout: vk::PipelineLayout, renderpass: vk::RenderPass, device: Rc<ash::Device>) -> Result<Rc<PPEffect>, vk::Result> {
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

        let vertex_attrib_descs = [];
        let vertex_binding_descs = [];
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
                .build()
        ];
        let colourblend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .stencil_test_enable(false)
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
            .layout(pipe_layout)
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
        Ok(Rc::new(Self {
            pipeline: graphicspipeline,
            device
        }))
    }
}

impl Drop for PPEffect {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
