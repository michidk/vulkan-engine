use ash::{version::DeviceV1_0, vk};

use super::{error::VulkanError, queue, surface};

const PREFERRED_IMAGE_COUNT: u32 = 3;

#[allow(dead_code)]
pub struct SwapchainWrapper {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub imageviews: Vec<vk::ImageView>,
    pub depth_image: vk::Image,
    pub depth_image_allocation: vk_mem::Allocation,
    pub depth_image_allocation_info: vk_mem::AllocationInfo,
    pub depth_imageview: vk::ImageView,
    pub depth_imageview_depth_only: vk::ImageView,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,
    pub amount_of_images: u32,
    pub g0_image: vk::Image,
    pub g0_imageview: vk::ImageView,
    pub g0_image_alloc: vk_mem::Allocation,
    pub g1_image: vk::Image,
    pub g1_imageview: vk::ImageView,
    pub g1_image_alloc: vk_mem::Allocation,
}

impl SwapchainWrapper {
    pub fn init(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &surface::SurfaceWrapper,
        #[allow(unused_variables)] queue_families: &queue::QueueFamilies,
        allocator: &vk_mem::Allocator,
    ) -> Result<SwapchainWrapper, VulkanError> {
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

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode);

        swapchain_create_info =
            swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);

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

        let depth_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            // TODO: maybe optimize wit D24 bit instead
            .format(vk::Format::D24_UNORM_S8_UINT)
            .extent(extend_3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (depth_image, depth_image_allocation, depth_image_allocation_info) =
            allocator.create_image(&depth_image_info, &allocation_info)?;
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

        let g0_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .extent(extend_3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (g0_image, g0_image_alloc, _) =
            allocator.create_image(&g0_image_info, &allocation_info)?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(g0_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .subresource_range(*subresource_range);
        let g0_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        let g1_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .extent(extend_3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let allocation_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (g1_image, g1_image_alloc, _) =
            allocator.create_image(&g1_image_info, &allocation_info)?;
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(g1_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .subresource_range(*subresource_range);
        let g1_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }?;

        Ok(SwapchainWrapper {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            imageviews: swapchain_imageviews,
            depth_image,
            depth_image_allocation,
            depth_image_allocation_info,
            depth_imageview,
            depth_imageview_depth_only,
            framebuffers: vec![],
            surface_format,
            extent,
            amount_of_images,
            g0_image,
            g0_image_alloc,
            g0_imageview,
            g1_image,
            g1_image_alloc,
            g1_imageview
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
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv, self.depth_imageview, self.g0_imageview, self.g1_imageview];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
    }

    pub unsafe fn cleanup(&mut self, logical_device: &ash::Device, allocator: &vk_mem::Allocator) {
        logical_device.destroy_image_view(self.depth_imageview, None);
        logical_device.destroy_image_view(self.depth_imageview_depth_only, None);
        allocator.destroy_image(self.depth_image, &self.depth_image_allocation);

        logical_device.destroy_image_view(self.g0_imageview, None);
        allocator.destroy_image(self.g0_image, &self.g0_image_alloc);

        logical_device.destroy_image_view(self.g1_imageview, None);
        allocator.destroy_image(self.g1_image, &self.g1_image_alloc);

        for fb in &self.framebuffers {
            logical_device.destroy_framebuffer(*fb, None);
        }
        for iv in &self.imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        self.swapchain_loader
            .destroy_swapchain(self.swapchain, None)
    }
}
