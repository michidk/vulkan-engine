use ash::{version::DeviceV1_0, vk};

use super::{MaterialData, MaterialDataLayout};
use crate::{vulkan::descriptor_manager::DescriptorData};

pub fn compile_descriptor_set_layout(
    device: &ash::Device,
    layout: &MaterialDataLayout,
) -> Result<vk::DescriptorSetLayout, vk::Result> {
    let mut bindings = Vec::with_capacity(layout.bindings.len());
    for (i, b) in layout.bindings.iter().enumerate() {
        let stage;
        match &b.binding_stage {
            super::MaterialDataBindingStage::Vertex => {
                stage = vk::ShaderStageFlags::VERTEX;
            }
            super::MaterialDataBindingStage::Fragment => {
                stage = vk::ShaderStageFlags::FRAGMENT;
            }
        }

        let vk_type;
        match &b.binding_type {
            super::MaterialDataBindingType::Uniform => {
                vk_type = vk::DescriptorType::UNIFORM_BUFFER;
            }
        }

        bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(i as u32)
                .descriptor_count(1)
                .descriptor_type(vk_type)
                .stage_flags(stage)
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
    let push_constants = [
        vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(128)
            .build()
    ];
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        .set_layouts(layouts)
        .push_constant_ranges(&push_constants)
        .build();
    unsafe { device.create_pipeline_layout(&layout_info, None) }
}

pub fn compile_resources<T: MaterialData>(
    resources: &T,
    allocator: &vk_mem::Allocator,
) -> Result<(Vec<DescriptorData>, Vec<vk_mem::Allocation>), vk_mem::Error> {
    let helpers = resources.get_material_resource_helpers();

    let mut resources = Vec::with_capacity(helpers.len());
    let mut allocations = Vec::with_capacity(helpers.len());

    for res in helpers {
        match res {
            super::MaterialResourceHelper::UniformBuffer(data) => {
                let buffer_info = vk::BufferCreateInfo::builder()
                    .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
                    .size(data.len() as u64)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .build();
                let alloc_info = vk_mem::AllocationCreateInfo {
                    usage: vk_mem::MemoryUsage::CpuToGpu,
                    ..Default::default()
                };
                let (buffer, alloc, _) = allocator.create_buffer(&buffer_info, &alloc_info)?;

                let map = allocator.map_memory(&alloc)?;
                unsafe {
                    map.copy_from_nonoverlapping(data.as_ptr(), data.len());
                }
                allocator.unmap_memory(&alloc);

                allocations.push(alloc);
                resources.push(DescriptorData::UniformBuffer {
                    buffer,
                    offset: 0,
                    size: data.len() as u64,
                });
            }
        }
    }

    Ok((resources, allocations))
}
