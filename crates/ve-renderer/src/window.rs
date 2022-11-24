//! This module contains the [`Window`] struct.
//!
//! See the [`Window`] documentation.

use std::rc::Rc;

use ash::vk;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{error::WindowCreationError, renderer::Renderer};

/// Contains a [`winit::window::Window`] and all Vulkan objects required for rendering to it.
#[derive(Debug)]
pub struct Window {
    renderer: Rc<Renderer>,
    /// The [`winit Window`](winit::window::Window) passed into [`Window::new()`]
    pub window: winit::window::Window,

    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
}

impl Window {
    /// Creates a new [`Window`] object.
    ///
    /// # Errors
    /// - [`WindowCreationError::Vk`] if a Vulkan API call returns an unexpected error
    pub fn new(
        renderer: Rc<Renderer>,
        window: winit::window::Window,
    ) -> Result<Self, WindowCreationError> {
        let surface = Self::create_surface(&renderer, &window)?;
        let swapchain = Self::create_swapchain(&renderer, surface, &window)?;

        Ok(Self {
            renderer,
            window,
            surface,
            swapchain,
        })
    }

    fn create_surface(
        renderer: &Renderer,
        window: &winit::window::Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        log::debug!("Creating vk::SurfaceKHR with ash_window");
        let surface = unsafe {
            ash_window::create_surface(
                &renderer.entry,
                &renderer.instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        Ok(surface)
    }

    fn create_swapchain(
        renderer: &Renderer,
        surface: vk::SurfaceKHR,
        window: &winit::window::Window,
    ) -> Result<vk::SwapchainKHR, vk::Result> {
        log::debug!("Creating vk::SwapchainKHR");

        let caps;
        let formats;
        let present_modes;
        unsafe {
            caps = renderer
                .khr_surface
                .get_physical_device_surface_capabilities(
                    renderer.device_info.physical_device,
                    surface,
                )?;
            formats = renderer.khr_surface.get_physical_device_surface_formats(
                renderer.device_info.physical_device,
                surface,
            )?;
            present_modes = renderer
                .khr_surface
                .get_physical_device_surface_present_modes(
                    renderer.device_info.physical_device,
                    surface,
                )?;
        };

        let mut image_count = u32::max(caps.min_image_count, 3);
        if caps.max_image_count > 0 {
            image_count = u32::min(caps.max_image_count, image_count);
        }
        log::debug!("Using {} images", image_count);

        let format = formats
            .iter()
            .find(|fmt| {
                fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    && (fmt.format == vk::Format::R8G8B8A8_SRGB
                        || fmt.format == vk::Format::B8G8R8A8_SRGB)
            })
            .unwrap_or_else(|| {
                formats
                    .first()
                    .expect("vkGetPhysicalDeviceSurfaceFormatsKHR() returned 0 formats")
            });
        log::debug!("Using format {:?}", format);

        let extent = if caps.current_extent.width == u32::MAX {
            let window_size = window.inner_size();
            vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            }
        } else {
            caps.current_extent
        };
        log::debug!("Using size {:?}", extent);

        let present_mode = if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        };
        log::debug!("Using present mode {:?}", present_mode);

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            renderer
                .khr_swapchain
                .create_swapchain(&create_info, None)?
        };
        Ok(swapchain)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            log::debug!("Destroying swapchain");
            self.renderer
                .khr_swapchain
                .destroy_swapchain(self.swapchain, None);
            log::debug!("Destroying surface");
            self.renderer
                .khr_surface
                .destroy_surface(self.surface, None);
        }
    }
}
