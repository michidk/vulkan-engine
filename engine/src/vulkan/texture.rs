use std::rc::Rc;

use ash::vk;

use super::uploader::Uploader;

/// The filtering mode with which a [`Texture2D`] should be sampled.
pub enum TextureFilterMode {
    /// Take the average of the surrounding texels
    Linear,
    // Take the texel nearest to the UV coords
    Nearest,
}

/// Manages a 2D Texture.
///
/// A Texture2D will always be in R8G8B8A8 format.
pub struct Texture2D {
    allocator: Rc<vk_mem::Allocator>,
    device: Rc<ash::Device>,
    image: vk::Image,
    alloc: vk_mem::Allocation,
    /// The [`vk::ImageView`] that can be used to refer to this [`Texture2D`].
    pub view: vk::ImageView,
    pub width: u32,
    pub height: u32,
    /// The [`vk::Sampler`] that can be used to sample from this [`Texture2D`].
    pub sampler: vk::Sampler,
}

impl Texture2D {
    /// Creates a new [`Texture2D`].
    ///
    /// # Parameters
    /// - `pixels`: A `width` * `height` * 4 slice of u8. A group of 4 bytes is a single pixel.
    ///   The slice must be in row major memory order and tightly packed.
    ///   The row with UV (xx, 0.0) is the first one in the buffer.
    pub fn new(
        width: u32,
        height: u32,
        pixels: &[u8],
        filter: TextureFilterMode,
        allocator: Rc<vk_mem::Allocator>,
        uploader: &mut Uploader,
        device: Rc<ash::Device>,
    ) -> Result<Rc<Texture2D>, vk_mem::Error> {
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .build();
        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };

        let (image, alloc, _) = allocator.create_image(&image_info, &alloc_info)?;

        uploader.enqueue_image_upload(
            image,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            width,
            height,
            pixels,
        );

        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
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
            })
            .build();
        let view = unsafe { device.create_image_view(&view_info, None) }.unwrap();

        let vk_filter = match filter {
            TextureFilterMode::Linear => vk::Filter::LINEAR,
            TextureFilterMode::Nearest => vk::Filter::NEAREST,
        };

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk_filter)
            .min_filter(vk_filter)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .mip_lod_bias(0.0)
            .anisotropy_enable(false)
            .compare_enable(false)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE)
            .unnormalized_coordinates(false)
            .build();
        let sampler = unsafe { device.create_sampler(&sampler_info, None) }.unwrap();

        Ok(Rc::new(Texture2D {
            allocator,
            device,
            image,
            alloc,
            view,
            width,
            height,
            sampler,
        }))
    }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.view, None);
        }
        self.allocator.destroy_image(self.image, &self.alloc);
    }
}