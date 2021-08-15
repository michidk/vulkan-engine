use ash::vk;
use gpu_allocator::SubAllocation;

use super::{GraphicsResult, allocator::Allocator, queue, surface};

const PREFERRED_IMAGE_COUNT: u32 = 3;

#[allow(dead_code)]
pub struct SwapchainWrapper {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub imageviews: Vec<vk::ImageView>,
    pub depth_image: vk::Image, // used in gpass and resolve pass
    pub depth_image_alloc: SubAllocation,
    pub depth_imageview: vk::ImageView,
    pub depth_imageview_depth_only: vk::ImageView,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub amount_of_images: u32,
    pub resolve_image: vk::Image, // will contain the finished deferred scene rendering
    pub resolve_imageview: vk::ImageView,
    pub resolve_image_alloc: SubAllocation,
    pub g0_image: vk::Image,
    pub g0_imageview: vk::ImageView,
    pub g0_image_alloc: SubAllocation,
    pub g1_image: vk::Image,
    pub g1_imageview: vk::ImageView,
    pub g1_image_alloc: SubAllocation,
    pub framebuffer_deferred: vk::Framebuffer, // used for gpass and resolve pass, renders to resolve_image
    pub framebuffer_pp_a: vk::Framebuffer,     // used for pp, renders to g0_image
    pub framebuffer_pp_b: vk::Framebuffer,     // used for pp, renders to resolve_image
}

impl SwapchainWrapper {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &surface::SurfaceWrapper,
        #[allow(unused_variables)] queue_families: &queue::QueueFamilies,
        allocator: &Allocator,
    ) -> GraphicsResult<SwapchainWrapper> {
        let surface_capabilities = surface.get_capabilities(physical_device)?;
        let extent = surface_capabilities.current_extent; // TODO: handle 0xFFFF x 0xFFFF extent
        let surface_format = surface.choose_format(physical_device)?;
        let present_mode = surface.choose_present_mode(physical_device)?;

        let image_count = if surface_capabilities.max_image_count > 0 {
            PREFERRED_IMAGE_COUNT
                .max(surface_capabilities.min_image_count)
                .min(surface_capabilities.max_image_count)
        } else {
            PREFERRED_IMAGE_COUNT.max(surface_capabilities.min_image_count)
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode);

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let amount_of_images = swapchain_images.len() as u32;
        let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
        for image in &swapchain_images {
            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let imageview_create_info = vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .subresource_range(*subresource_range);
            let imageview =
                unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;
            swapchain_imageviews.push(imageview);
        }
        let extend_3d = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };

        let (depth_image, depth_image_alloc) =
            allocator.create_image(extent.width, extent.height, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT, vk::Format::D24_UNORM_S8_UINT);
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::D24_UNORM_S8_UINT)
            .subresource_range(*subresource_range);
        let depth_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::D24_UNORM_S8_UINT)
            .subresource_range(*subresource_range);
        let depth_imageview_depth_only =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        let (resolve_image, resolve_image_alloc) =
            allocator.create_image(extent.width, extent.height, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED, vk::Format::R16G16B16A16_SFLOAT);
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(resolve_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .subresource_range(*subresource_range);
        let resolve_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        let (g0_image, g0_image_alloc) =
            allocator.create_image(extent.width, extent.height, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::SAMPLED, vk::Format::R16G16B16A16_SFLOAT);
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(g0_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .subresource_range(*subresource_range);
        let g0_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        let (g1_image, g1_image_alloc) =
            allocator.create_image(extent.width, extent.height, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT, vk::Format::R16G16B16A16_SFLOAT);
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(g1_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .subresource_range(*subresource_range);
        let g1_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        Ok(SwapchainWrapper {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            depth_image,
            depth_image_alloc,
            depth_imageview,
            depth_imageview_depth_only,
            surface_format,
            extent,
            amount_of_images,
            g0_image,
            g0_image_alloc,
            g0_imageview,
            g1_image,
            g1_image_alloc,
            g1_imageview,
            resolve_image,
            resolve_imageview,
            resolve_image_alloc,
            framebuffer_deferred: vk::Framebuffer::null(),
            framebuffer_pp_a: vk::Framebuffer::null(),
            framebuffer_pp_b: vk::Framebuffer::null(),
        })
    }

    // TODO: handle error
    pub fn aquire_next_image(&self, signal_semaphore: vk::Semaphore) -> u32 {
        let (image_index, _) = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    signal_semaphore,
                    vk::Fence::null(),
                )
                .expect("image acquisition trouble")
        };
        image_index
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
        pp_renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        // deferred framebuffer
        let views = [
            self.resolve_imageview,
            self.depth_imageview,
            self.g0_imageview,
            self.g1_imageview,
        ];
        let fb_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass)
            .attachments(&views)
            .width(self.extent.width)
            .height(self.extent.height)
            .layers(1)
            .build();
        self.framebuffer_deferred = unsafe { logical_device.create_framebuffer(&fb_info, None)? };

        // PP a framebuffer
        let views = [self.g0_imageview];
        let fb_info = vk::FramebufferCreateInfo::builder()
            .render_pass(pp_renderpass)
            .attachments(&views)
            .width(self.extent.width)
            .height(self.extent.height)
            .layers(1)
            .build();
        self.framebuffer_pp_a = unsafe { logical_device.create_framebuffer(&fb_info, None)? };

        // PP b framebuffer
        let views = [self.resolve_imageview];
        let fb_info = vk::FramebufferCreateInfo::builder()
            .render_pass(pp_renderpass)
            .attachments(&views)
            .width(self.extent.width)
            .height(self.extent.height)
            .layers(1)
            .build();
        self.framebuffer_pp_b = unsafe { logical_device.create_framebuffer(&fb_info, None)? };

        Ok(())
    }

    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &Allocator) {
        logical_device.destroy_framebuffer(self.framebuffer_deferred, None);
        logical_device.destroy_framebuffer(self.framebuffer_pp_a, None);
        logical_device.destroy_framebuffer(self.framebuffer_pp_b, None);

        logical_device.destroy_image_view(self.depth_imageview, None);
        logical_device.destroy_image_view(self.depth_imageview_depth_only, None);
        allocator.destroy_image(self.depth_image, &self.depth_image_alloc);

        logical_device.destroy_image_view(self.g0_imageview, None);
        allocator.destroy_image(self.g0_image, &self.g0_image_alloc);

        logical_device.destroy_image_view(self.g1_imageview, None);
        allocator.destroy_image(self.g1_image, &self.g1_image_alloc);

        logical_device.destroy_image_view(self.resolve_imageview, None);
        allocator.destroy_image(self.resolve_image, &self.resolve_image_alloc);

        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}
