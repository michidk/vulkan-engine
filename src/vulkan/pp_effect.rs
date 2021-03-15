use std::rc::Rc;

use ash::{version::DeviceV1_0, vk};

use super::pipeline;

pub struct PPEffect {
    pub pipeline: vk::Pipeline,
    device: Rc<ash::Device>,
}

impl PPEffect {
    pub fn new(shader: &str, pipe_layout: vk::PipelineLayout, renderpass: vk::RenderPass, device: Rc<ash::Device>) -> Result<Rc<PPEffect>, vk::Result> {
        let blend_func = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .build();

        let pipeline = pipeline::create_pipeline(
            shader,
            pipe_layout,
            renderpass,
            0,
            false,
            1,
            blend_func,
            false,
            None,
            &device
        )?;

        Ok(Rc::new(Self {
            pipeline,
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
