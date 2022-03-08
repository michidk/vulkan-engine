use std::rc::Rc;

use ash::vk;

use crate::graphics::{context::Context, error::GraphicsResult, desc_layouts};

use super::Renderer;


pub(crate) struct DeferredRenderer {
    context: Rc<Context>,

    // DescriptorSetLayouts
    desc_layout_frame_data: vk::DescriptorSetLayout,
}

impl Renderer for DeferredRenderer {
    fn create(context: Rc<Context>) -> GraphicsResult<Self> where Self: Sized {
        let desc_layout_frame_data = desc_layouts::deferred_frame_data(&context.device)?;

        Ok(Self {
            context,
            
            desc_layout_frame_data,
        })
    }
}

impl Drop for DeferredRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.destroy_descriptor_set_layout(self.desc_layout_frame_data, None);
        }
    }
}
