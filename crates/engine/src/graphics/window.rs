use std::rc::Rc;

use ash::vk;

use super::{context::Context, error::{GraphicsResult, GraphicsError}};


pub(crate) struct Window {
    context: Rc<Context>,
    winit_window: winit::window::Window,
    surface: vk::SurfaceKHR,
    pub(crate) swapchain: Swapchain,

    pub(crate) acquire_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_semaphores: Vec<vk::Semaphore>,
}

pub(crate) struct Swapchain {
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) views: Vec<vk::ImageView>,
    pub(crate) format: vk::Format,
    pub(crate) size: vk::Extent2D,
}

impl Window {
    pub(crate) fn new<T>(width: u32, height: u32, title: &str, visible: bool, decorated: bool, context: Rc<Context>, event_loop: &winit::event_loop::EventLoopWindowTarget<T>) -> GraphicsResult<Self> {
        let winit_window = winit::window::WindowBuilder::new()
            .with_decorations(decorated)
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_resizable(true)
            .with_title(title)
            .with_visible(visible)
            .build(event_loop)
            .map_err(|_| GraphicsError::WindowCreationFailed)?;

        let surface = unsafe{ash_window::create_surface(&context.entry, &context.instance, &winit_window, None)?};

        let swapchain = Self::create_swapchain(&context, surface)?;

        let mut acquire_semaphores = Vec::with_capacity(context.max_frames_in_flight);
        for _ in 0..context.max_frames_in_flight {
            let sem = unsafe{context.device.create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?};
            acquire_semaphores.push(sem);
        }

        let mut render_semaphores = Vec::with_capacity(context.max_frames_in_flight);
        for _ in 0..context.max_frames_in_flight {
            let sem = unsafe{context.device.create_semaphore(&vk::SemaphoreCreateInfo::builder(), None)?};
            render_semaphores.push(sem);
        }

        Ok(Self {
            context,
            winit_window,
            surface,
            swapchain,

            acquire_semaphores,
            render_semaphores,
        })
    }

    pub(crate) fn recreate_swapchain(&mut self) -> GraphicsResult<()> {
        for view in &self.swapchain.views {
            unsafe{self.context.device.destroy_image_view(*view, None)};
        }
        unsafe{self.context.khr_swapchain.destroy_swapchain(self.swapchain.handle, None);}
        self.swapchain = Self::create_swapchain(&self.context, self.surface)?;

        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            for sem in &self.acquire_semaphores {
                self.context.device.destroy_semaphore(*sem, None);
            }
            for sem in &self.render_semaphores {
                self.context.device.destroy_semaphore(*sem, None);
            }

            for view in &self.swapchain.views {
                self.context.device.destroy_image_view(*view, None);
            }
            self.context.khr_swapchain.destroy_swapchain(self.swapchain.handle, None);
            self.context.khr_surface.destroy_surface(self.surface, None);
        }
    }
}

impl Window {
    fn create_swapchain(context: &Context, surface: vk::SurfaceKHR) -> GraphicsResult<Swapchain> {
        let caps = unsafe{context.khr_surface.get_physical_device_surface_capabilities(context.physical_device, surface)?};
        let formats = unsafe{context.khr_surface.get_physical_device_surface_formats(context.physical_device, surface)?};
        let present_modes = unsafe{context.khr_surface.get_physical_device_surface_present_modes(context.physical_device, surface)?};

        let format = formats.iter().find(|fmt| fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR && fmt.format == vk::Format::R8G8B8A8_SRGB)
            .or_else(|| formats.iter().find(|fmt| fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR && fmt.format == vk::Format::B8G8R8A8_SRGB))
            .ok_or(GraphicsError::WindowCreationFailed)?;

        let present_mode = if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        };

        let image_count = if caps.max_image_count != 0 {
            3.max(caps.min_image_count).min(caps.max_image_count)
        } else {
            3.max(caps.min_image_count)
        };

        let extent = caps.current_extent;
        if extent.width == 0 || extent.height == 0 {
            return Err(GraphicsError::WindowMinimized);
        }

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        
        let swapchain = unsafe{context.khr_swapchain.create_swapchain(&info, None)?};

        let images = unsafe{context.khr_swapchain.get_swapchain_images(swapchain)?};

        let mut views = Vec::with_capacity(images.len());
        for img in &images {
            let view_info = vk::ImageViewCreateInfo::builder()
                .image(*img)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            views.push(unsafe{context.device.create_image_view(&view_info, None)?});
        }

        Ok(Swapchain {
            handle: swapchain,
            images,
            views,
            format: format.format,
            size: extent,
        })
    }
}
