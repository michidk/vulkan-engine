use std::rc::Rc;

use ash::vk;

use super::{context::Context, error::GraphicsResult};

pub(crate) mod deferred;

pub(crate) trait Renderer {
    fn create(context: Rc<Context>) -> GraphicsResult<Self> where Self: Sized;

    fn render_frame(&mut self, command_buffer: vk::CommandBuffer) -> GraphicsResult<vk::Image>;

    fn set_size(&mut self, size: (u32, u32)) -> GraphicsResult<()>;
    fn get_size(&self) -> (u32, u32);
}
