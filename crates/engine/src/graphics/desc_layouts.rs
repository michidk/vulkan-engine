use ash::vk;

use super::error::GraphicsResult;

pub(crate) fn deferred_frame_data(device: &ash::Device) -> GraphicsResult<vk::DescriptorSetLayout> {
    let bindings = [
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build(),
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::INPUT_ATTACHMENT)
            .descriptor_count(3)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build(),
    ];
    let info = vk::DescriptorSetLayoutCreateInfo::builder()
        .bindings(&bindings);
    
    unsafe {
        Ok(device.create_descriptor_set_layout(&info, None)?)
    }
}
