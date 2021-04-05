use ash::{version::DeviceV1_0, vk};

use crate::vulkan::descriptor_manager::DescriptorData;

pub fn compile_descriptor_set_layout(
    device: &ash::Device,
    resources: &[DescriptorData],
) -> Result<vk::DescriptorSetLayout, vk::Result> {
    let mut bindings = Vec::with_capacity(resources.len());
    for (i, r) in resources.iter().enumerate() {
        let vk_type = match r {
            DescriptorData::None => continue,
            DescriptorData::UniformBuffer { .. } => vk::DescriptorType::UNIFORM_BUFFER,
            DescriptorData::DynamicUniformBuffer { .. } => {
                vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
            }
            DescriptorData::ImageSampler { .. } => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorData::StorageBuffer { .. } => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorData::InputAttachment { .. } => vk::DescriptorType::INPUT_ATTACHMENT,
        };

        bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(i as u32)
                .descriptor_count(1)
                .descriptor_type(vk_type)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build(),
        );
    }

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings)
        .build();

    let set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };
    Ok(set_layout)
}

pub fn compile_pipeline_layout(
    device: &ash::Device,
    layouts: &[vk::DescriptorSetLayout],
) -> Result<vk::PipelineLayout, vk::Result> {
    // all devices must at least support 128 bytes of push constants, so this is safe
    let push_constants = [vk::PushConstantRange::builder()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .offset(0)
        .size(128)
        .build()];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(layouts)
        .push_constant_ranges(&push_constants)
        .build();
    unsafe { device.create_pipeline_layout(&layout_info, None) }
}

pub fn compile_resources(
    data: &[DescriptorData],
    allocator: &vk_mem::Allocator,
) -> Result<(Vec<DescriptorData>, Vec<vk_mem::Allocation>), vk_mem::Error> {
    let mut resources = Vec::with_capacity(data.len());
    let mut allocations = Vec::with_capacity(data.len());

    for res in data {
        match res {
            DescriptorData::UniformBuffer {
                buffer: _,
                offset: _,
                size,
            } => {
                let buffer_info = vk::BufferCreateInfo::builder()
                    .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                    .size(*size)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build();
                let alloc_info = vk_mem::AllocationCreateInfo {
                    usage: vk_mem::MemoryUsage::CpuToGpu,
                    ..Default::default()
                };
                let (buffer, alloc, _) = allocator.create_buffer(&buffer_info, &alloc_info)?;

                allocations.push(alloc);
                resources.push(DescriptorData::UniformBuffer {
                    buffer,
                    offset: 0,
                    size: *size,
                });
            }
            DescriptorData::ImageSampler { .. } => {
                resources.push(DescriptorData::ImageSampler {
                    image: vk::ImageView::null(),
                    layout: vk::ImageLayout::UNDEFINED,
                    sampler: vk::Sampler::null(),
                });
            }
            _ => continue,
        }
    }

    Ok((resources, allocations))
}
