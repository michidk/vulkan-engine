use std::rc::Rc;

use ash::vk;

use crate::graphics::{context::Context, error::GraphicsResult};

use super::Renderer;

mod desc_layouts;
mod renderpasses;

pub(crate) struct DeferredRenderer {
    context: Rc<Context>,

    // DescriptorSetLayouts
    desc_layout_frame_data: vk::DescriptorSetLayout,

    // Renderpasses
    renderpass_deferred: vk::RenderPass,

    g0_alloc: gpu_allocator::vulkan::Allocation,
    g1_alloc: gpu_allocator::vulkan::Allocation,
    depth_alloc: gpu_allocator::vulkan::Allocation,
    output_alloc: gpu_allocator::vulkan::Allocation,

    g0_image: vk::Image,
    g1_image: vk::Image,
    depth_image: vk::Image,
    output_image: vk::Image,

    g0_view: vk::ImageView,
    g1_view: vk::ImageView,
    depth_view: vk::ImageView,
    output_view: vk::ImageView,

    framebuffer: vk::Framebuffer,

    width: u32,
    height: u32,
}

impl Renderer for DeferredRenderer {
    fn create(context: Rc<Context>) -> GraphicsResult<Self> where Self: Sized {
        let desc_layout_frame_data = desc_layouts::deferred_frame_data(&context.device)?;

        let renderpass_deferred = renderpasses::deferred(&context.device)?;

        let mut res = Self {
            context,
            desc_layout_frame_data,
            renderpass_deferred,
            g0_alloc: gpu_allocator::vulkan::Allocation::default(),
            g1_alloc: gpu_allocator::vulkan::Allocation::default(),
            depth_alloc: gpu_allocator::vulkan::Allocation::default(),
            output_alloc: gpu_allocator::vulkan::Allocation::default(),
            g0_image: vk::Image::null(),
            g1_image: vk::Image::null(),
            depth_image: vk::Image::null(),
            output_image: vk::Image::null(),
            g0_view: vk::ImageView::null(),
            g1_view: vk::ImageView::null(),
            depth_view: vk::ImageView::null(),
            output_view: vk::ImageView::null(),
            framebuffer: vk::Framebuffer::null(),

            width: 0,
            height: 0,
        };

        res.create_framebuffer(800, 600)?;

        Ok(res)
    }

    fn render_frame(&mut self, command_buffer: vk::CommandBuffer) -> GraphicsResult<vk::Image> {
        unsafe {
            let clear_values = [
                vk::ClearValue::default(),
                vk::ClearValue::default(),
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 1.0,
                        stencil: 0,
                    }
                },
                vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [1.0, 0.0, 1.0, 1.0]
                    }
                }
            ];

            let attachments = [
                self.g0_view,
                self.g1_view,
                self.depth_view,
                self.output_view,
            ];

            let mut attachment_info = vk::RenderPassAttachmentBeginInfo::builder()
                .attachments(&attachments);

            let begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.renderpass_deferred)
                .framebuffer(self.framebuffer)
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D {
                        x: 0,
                        y: 0,
                    },
                    extent: vk::Extent2D {
                        width: self.width,
                        height: self.height,
                    },
                })
                .clear_values(&clear_values)
                .push_next(&mut attachment_info);
            self.context.device.cmd_begin_render_pass(command_buffer, &begin_info, vk::SubpassContents::INLINE);

            // gpass

            self.context.device.cmd_next_subpass(command_buffer, vk::SubpassContents::INLINE);

            // resolve pass

            self.context.device.cmd_end_render_pass(command_buffer);
        }

        Ok(self.output_image)
    }

    fn set_size(&mut self, size: (u32, u32)) -> GraphicsResult<()> {
        self.destroy_framebuffer();
        self.create_framebuffer(size.0, size.1)?;

        Ok(())
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl DeferredRenderer {
    fn create_framebuffer(&mut self, width: u32, height: u32) -> GraphicsResult<()> {
        let (g0_alloc, g0_image) = self.context.create_image(width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT, "deferred g0 image")?;
        let (g1_alloc, g1_image) = self.context.create_image(width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT, "deferred g1 image")?;
        let (depth_alloc, depth_image) = self.context.create_image(width, height, vk::Format::D24_UNORM_S8_UINT, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT, "deferred depth/stencil image")?;
        let (output_alloc, output_image) = self.context.create_image(width, height, vk::Format::R16G16B16A16_SFLOAT, vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC, "deferred output image")?;

        let g0_view = unsafe {
            let info = vk::ImageViewCreateInfo::builder()
                .image(g0_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R16G16B16A16_SFLOAT)
                .components(vk::ComponentMapping::default())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            self.context.device.create_image_view(&info, None)?
        };
        let g1_view = unsafe {
            let info = vk::ImageViewCreateInfo::builder()
                .image(g1_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R16G16B16A16_SFLOAT)
                .components(vk::ComponentMapping::default())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            self.context.device.create_image_view(&info, None)?
        };
        let depth_view = unsafe {
            let info = vk::ImageViewCreateInfo::builder()
                .image(depth_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::D24_UNORM_S8_UINT)
                .components(vk::ComponentMapping::default())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            self.context.device.create_image_view(&info, None)?
        };
        let output_view = unsafe {
            let info = vk::ImageViewCreateInfo::builder()
                .image(output_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(vk::Format::R16G16B16A16_SFLOAT)
                .components(vk::ComponentMapping::default())
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            self.context.device.create_image_view(&info, None)?
        };

        let framebuffer = unsafe {
            let mut info = vk::FramebufferCreateInfo::builder()
                .flags(vk::FramebufferCreateFlags::IMAGELESS)
                .render_pass(self.renderpass_deferred)
                .width(width)
                .height(height)
                .layers(1);
            info.attachment_count = 4;

            let color_formats = [vk::Format::R16G16B16A16_SFLOAT];
            let depth_formats = [vk::Format::D24_UNORM_S8_UINT];

            let attachment_infos = [
                vk::FramebufferAttachmentImageInfo::builder()
                    .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
                    .width(width)
                    .height(height)
                    .layer_count(1)
                    .view_formats(&color_formats)
                    .build(),
                vk::FramebufferAttachmentImageInfo::builder()
                    .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
                    .width(width)
                    .height(height)
                    .layer_count(1)
                    .view_formats(&color_formats)
                    .build(),
                vk::FramebufferAttachmentImageInfo::builder()
                    .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT)
                    .width(width)
                    .height(height)
                    .layer_count(1)
                    .view_formats(&depth_formats)
                    .build(),
                vk::FramebufferAttachmentImageInfo::builder()
                    .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC)
                    .width(width)
                    .height(height)
                    .layer_count(1)
                    .view_formats(&color_formats)
                    .build(),
            ];

            let mut attachments_info = vk::FramebufferAttachmentsCreateInfo::builder()
                .attachment_image_infos(&attachment_infos);
            info = info.push_next(&mut attachments_info);

            self.context.device.create_framebuffer(&info, None)?
        };

        self.g0_alloc = g0_alloc;
        self.g1_alloc = g1_alloc;
        self.depth_alloc = depth_alloc;
        self.output_alloc = output_alloc;

        self.g0_image = g0_image;
        self.g1_image = g1_image;
        self.depth_image = depth_image;
        self.output_image = output_image;

        self.g0_view = g0_view;
        self.g1_view = g1_view;
        self.depth_view = depth_view;
        self.output_view = output_view;

        self.framebuffer = framebuffer;

        self.width = width;
        self.height = height;

        Ok(())
    }

    fn destroy_framebuffer(&mut self) {
        unsafe {
            self.context.device.destroy_framebuffer(self.framebuffer, None);

            self.context.device.destroy_image_view(self.g0_view, None);
            self.context.device.destroy_image_view(self.g1_view, None);
            self.context.device.destroy_image_view(self.depth_view, None);
            self.context.device.destroy_image_view(self.output_view, None);
        }

        self.context.destroy_image(self.g0_alloc.clone(), self.g0_image);
        self.context.destroy_image(self.g1_alloc.clone(), self.g1_image);
        self.context.destroy_image(self.depth_alloc.clone(), self.depth_image);
        self.context.destroy_image(self.output_alloc.clone(), self.output_image);
    }
}

impl Drop for DeferredRenderer {
    fn drop(&mut self) {
        self.destroy_framebuffer();

        unsafe {
            self.context.device.destroy_render_pass(self.renderpass_deferred, None);
            self.context.device.destroy_descriptor_set_layout(self.desc_layout_frame_data, None);
        }
    }
}
