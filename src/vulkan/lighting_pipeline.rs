use std::rc::Rc;

use ash::{version::DeviceV1_0, vk};

use crate::assets::shader;

pub struct LightingPipeline {
    pub point_pipeline: vk::Pipeline,
    pub directional_pipeline: vk::Pipeline,
    pub stencil_id: u8,
    device: Rc<ash::Device>,
}

impl LightingPipeline {
    pub fn new(point_shader: &str, directional_shader: &str, pipe_layout_resolve: vk::PipelineLayout, renderpass: vk::RenderPass, device: Rc<ash::Device>, stencil_id: u8) -> Result<Rc<LightingPipeline>, vk::Result> {
        let point_pipeline = Self::compile_pipeline(point_shader, pipe_layout_resolve, renderpass, device.as_ref(), stencil_id)?;
        let directional_pipeline = Self::compile_pipeline(directional_shader, pipe_layout_resolve, renderpass, device.as_ref(), stencil_id)?;

        Ok(Rc::new(LightingPipeline {
            point_pipeline,
            directional_pipeline,
            stencil_id,
            device
        }))
    }

    fn compile_pipeline(shader: &str, layout: vk::PipelineLayout, renderpass: vk::RenderPass, device: &ash::Device, stencil_id: u8) -> Result<vk::Pipeline, vk::Result> {
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
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: i32::MAX as u32,
                height: i32::MAX as u32,
            },
        }];
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
        let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
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
        let colourblend_info =
            vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
        
        let stencil_front = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::EQUAL)
            .write_mask(0x00)
            .compare_mask(0xFF)
            .reference(stencil_id as u32)
            .build();
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
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
            .subpass(1);
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

}

impl Drop for LightingPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.point_pipeline, None);
            self.device.destroy_pipeline(self.directional_pipeline, None);
        }
    }
}
