use ash::{version::DeviceV1_0, vk};

use super::buffer::BufferWrapper;

#[allow(dead_code)]
pub struct Texture {
    pub image: image::RgbaImage,
    pub vk_image: vk::Image,
    pub allocation: vk_mem::Allocation,
    pub allocation_info: vk_mem::AllocationInfo,
    pub imageview: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    #[allow(dead_code)]
    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        device: &ash::Device,
        allocator: &vk_mem::Allocator,
        commandpool_graphics: &vk::CommandPool,
        graphics_queue: &vk::Queue,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)
            .map(|img| img.to_rgba8())
            .expect("unable to open image");
        let (width, height) = image.dimensions();
        let img_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_SRGB)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);

        let alloc_create_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (vk_image, allocation, allocation_info) = allocator
            .create_image(&img_create_info, &alloc_create_info)
            .expect("creating vkImage for texture");

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });
        let imageview = unsafe { device.create_image_view(&view_create_info, None) }
            .expect("image view creation");

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR);
        let sampler =
            unsafe { device.create_sampler(&sampler_info, None) }.expect("sampler creation");

        let data = image.clone().into_raw();
        let mut buffer = BufferWrapper::new(
            allocator,
            data.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu, // TODO: explain to Rob why CpuOnly is wrong // maybe reconsider and use CpuOnly
        )?;
        buffer.fill(allocator, &data)?;

        let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(*commandpool_graphics)
            .command_buffer_count(1);
        let copycmdbuffer =
            unsafe { device.allocate_command_buffers(&commandbuf_allocate_info) }.unwrap()[0];

        let cmdbegininfo = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(copycmdbuffer, &cmdbegininfo) }?;

        // is this the correct location?
        let barrier = vk::ImageMemoryBarrier::builder()
            .image(vk_image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };

        //Insert commands here.
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        };
        let (width, height) = image.dimensions();
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            image_subresource,
        };
        unsafe {
            device.cmd_copy_buffer_to_image(
                copycmdbuffer,
                buffer.buffer,
                vk_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        let barrier = vk::ImageMemoryBarrier::builder()
            .image(vk_image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };

        unsafe { device.end_command_buffer(copycmdbuffer) }?;
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&[copycmdbuffer])
            .build()];
        let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }?;
        unsafe { device.queue_submit(*graphics_queue, &submit_infos, fence) }?;
        unsafe { device.wait_for_fences(&[fence], true, std::u64::MAX) }?;
        unsafe { device.destroy_fence(fence, None) };
        allocator.destroy_buffer(buffer.buffer, &allocation);
        unsafe { device.free_command_buffers(*commandpool_graphics, &[copycmdbuffer]) };

        Ok(Texture {
            image,
            vk_image,
            imageview,
            allocation,
            allocation_info,
            sampler,
        })
    }
}

pub struct TextureStorage {
    textures: Vec<Texture>,
}

#[allow(clippy::new_without_default)]
impl TextureStorage {
    pub fn new() -> Self {
        TextureStorage { textures: vec![] }
    }
    pub fn cleanup(&mut self, device: &ash::Device, allocator: &vk_mem::Allocator) {
        for texture in &self.textures {
            unsafe {
                device.destroy_sampler(texture.sampler, None);
                device.destroy_image_view(texture.imageview, None);
            }
            allocator.destroy_image(texture.vk_image, &texture.allocation);
        }
    }
    pub fn new_texture_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
        device: &ash::Device,
        allocator: &vk_mem::Allocator,
        commandpool_graphics: &vk::CommandPool,
        graphics_queue: &vk::Queue,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let new_texture = Texture::from_file(
            path,
            device,
            allocator,
            commandpool_graphics,
            graphics_queue,
        )?;
        let new_id = self.textures.len();
        self.textures.push(new_texture);
        Ok(new_id)
    }
    pub fn get(&self, index: usize) -> Option<&Texture> {
        self.textures.get(index)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Texture> {
        self.textures.get_mut(index)
    }
}
