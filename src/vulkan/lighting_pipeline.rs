use std::rc::Rc;

use ash::{version::DeviceV1_0, vk};

use super::pipeline;

/// This struct describes a Deferred Resolve shader and its associated state.
/// 
/// A LightingPipeline can be thought of as a specific lighting equation to be applied to a specific set of [`Materials`](crate::scene::material::Material).
pub struct LightingPipeline {
    /// The [`vk::Pipeline`] to be used for rendering point lights
    pub point_pipeline: Option<vk::Pipeline>,
    /// The [`vk::Pipeline`] to be used for rendering directional lights
    pub directional_pipeline: Option<vk::Pipeline>,
    /// The [`vk::Pipeline`] to be used for rendering ambient lighting (or unlit materials).
    /// Will be rendered exactly once each frame.
    pub ambient_pipeline: Option<vk::Pipeline>,
    /// The stencil value that identifies this LightingPipeline in the GPass attachments.
    pub stencil_id: u8,
    device: Rc<ash::Device>,
}

impl LightingPipeline {
    /// Creates a new [`LightingPipeline`].
    /// 
    /// # Parameters
    /// - `point_shader`, `directional_shader`, `ambient_shader`: names of the shaders to be used for rendering
    ///   point lights, directional lights and ambient light respectively. All three are optional.
    /// - `pipe_layout_resolve`: The [`vk::PipelineLayout`] of the deferred resolve SubPass.
    /// - `renderpass`: The [`vk::RenderPass`] that describes the deferred RenderPass. SubPass 1 will be used for the [`LightingPipeline`].
    /// - `stencil_id`: The stencil value used to identify this [`LightingPipeline`]. This value has to be unique among all [`LightingPipelines`](LightingPipeline).
    pub fn new(
        point_shader: Option<&str>,
        directional_shader: Option<&str>,
        ambient_shader: Option<&str>,
        pipe_layout_resolve: vk::PipelineLayout,
        renderpass: vk::RenderPass,
        device: Rc<ash::Device>,
        stencil_id: u8,
    ) -> Result<Rc<LightingPipeline>, vk::Result> {
        let stencil_func = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::EQUAL)
            .write_mask(0x00)
            .compare_mask(0xFF)
            .reference(stencil_id as u32)
            .build();
        let blend_func = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::DST_ALPHA) // this ensures that the clear color is not added to the final color
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
            .build();

        let point_pipeline = if let Some(point_shader) = point_shader {
            let mut vertexshader_code = Vec::new();
            let mut fragmentshader_code = Vec::new();
            let (vertex_shader, fragment_shader) = pipeline::create_shader_modules(
                point_shader,
                &device,
                &mut vertexshader_code,
                &mut fragmentshader_code,
            )?;

            Some(pipeline::create_pipeline(
                pipe_layout_resolve,
                renderpass,
                1,
                false,
                1,
                blend_func,
                false,
                Some(stencil_func),
                &device,
                vertex_shader,
                fragment_shader,
            )?)
        } else {
            None
        };

        let directional_pipeline = if let Some(directional_shader) = directional_shader {
            let mut vertexshader_code = Vec::new();
            let mut fragmentshader_code = Vec::new();
            let (vertex_shader, fragment_shader) = pipeline::create_shader_modules(
                directional_shader,
                &device,
                &mut vertexshader_code,
                &mut fragmentshader_code,
            )?;

            Some(pipeline::create_pipeline(
                pipe_layout_resolve,
                renderpass,
                1,
                false,
                1,
                blend_func,
                false,
                Some(stencil_func),
                &device,
                vertex_shader,
                fragment_shader,
            )?)
        } else {
            None
        };

        let ambient_pipeline = if let Some(ambient_shader) = ambient_shader {
            let mut vertexshader_code = Vec::new();
            let mut fragmentshader_code = Vec::new();
            let (vertex_shader, fragment_shader) = pipeline::create_shader_modules(
                ambient_shader,
                &device,
                &mut vertexshader_code,
                &mut fragmentshader_code,
            )?;

            Some(pipeline::create_pipeline(
                pipe_layout_resolve,
                renderpass,
                1,
                false,
                1,
                blend_func,
                false,
                Some(stencil_func),
                &device,
                vertex_shader,
                fragment_shader,
            )?)
        } else {
            None
        };

        Ok(Rc::new(LightingPipeline {
            point_pipeline,
            directional_pipeline,
            ambient_pipeline,
            stencil_id,
            device,
        }))
    }
}

impl Drop for LightingPipeline {
    fn drop(&mut self) {
        unsafe {
            if let Some(pp) = self.point_pipeline {
                self.device.destroy_pipeline(pp, None);
            }
            if let Some(dp) = self.directional_pipeline {
                self.device.destroy_pipeline(dp, None);
            }
            if let Some(ap) = self.ambient_pipeline {
                self.device.destroy_pipeline(ap, None);
            }
        }
    }
}
