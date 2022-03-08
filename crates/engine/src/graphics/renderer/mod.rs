use std::rc::Rc;

use super::{context::Context, error::GraphicsResult};

pub(crate) mod deferred;

pub(crate) trait Renderer {
    fn create(context: Rc<Context>) -> GraphicsResult<Self> where Self: Sized;
}
