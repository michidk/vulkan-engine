use std::rc::Rc;

use ash::vk;

use super::pipeline;

/// This struct holds the necessary information about a single post processing effect.
pub struct PPEffect {
    /// The [`vk::Pipeline`] used by this post processing effect.
    pub pipeline: vk::Pipeline,
    device: Rc<ash::Device>,
}

impl PPEffect {
    /// Creates a new [`PPEffect`].
    ///
    /// # Parameters
    /// - `pipe_layout`: The [`vk::PipelineLayout`] that describes the post processing pipeline layout.
    /// - `renderpass`: The [`vk::RenderPass`] in which this [`PPEffect`] will be used. SubPass 0 will be used.
    pub fn new(
        shader: &str,
        pipe_layout: vk::PipelineLayout,
        renderpass: vk::RenderPass,
        device: Rc<ash::Device>,
    ) -> Result<Rc<PPEffect>, vk::Result> {
        let blend_func = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build();

        let mut vertexshader_code = Vec::new();
        let mut fragmentshader_code = Vec::new();
        let (vertex_shader, fragment_shader) = pipeline::create_shader_modules(
            shader,
            &device,
            &mut vertexshader_code,
            &mut fragmentshader_code,
        )?;

        let pipeline = pipeline::create_pipeline(
            pipe_layout,
            renderpass,
            0,
            false,
            1,
            blend_func,
            false,
            None,
            &device,
            vertex_shader,
            fragment_shader,
            false,
        )?;

        unsafe {
            device.destroy_shader_module(vertex_shader, None);
            device.destroy_shader_module(fragment_shader, None);
        }

        Ok(Rc::new(Self { pipeline, device }))
    }
}

impl Drop for PPEffect {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
        }
    }
}
