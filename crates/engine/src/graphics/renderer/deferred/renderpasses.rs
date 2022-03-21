use ash::vk;

use crate::graphics::error::GraphicsResult;


pub(crate) fn deferred(device: &ash::Device) -> GraphicsResult<vk::RenderPass> {
    let attachments = [
        // gbuffer0
        vk::AttachmentDescription2::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build(),
        // gbuffer1
        vk::AttachmentDescription2::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build(),
        // depth/stencil buffer
        vk::AttachmentDescription2::builder()
            .format(vk::Format::D24_UNORM_S8_UINT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .build(),
        // output image
        vk::AttachmentDescription2::builder()
            .format(vk::Format::R16G16B16A16_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .build(),
    ];

    let gpass_inputs = [];
    let gpass_colors = [
        vk::AttachmentReference2::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(),
        vk::AttachmentReference2::builder()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(),
    ];
    let gpass_depth = vk::AttachmentReference2::builder()
        .attachment(2)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        .aspect_mask(vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL);

    let rpass_inputs = [
        vk::AttachmentReference2::builder()
            .attachment(0)
            .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(),
        vk::AttachmentReference2::builder()
            .attachment(1)
            .layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(),
        vk::AttachmentReference2::builder()
            .attachment(2)
            .layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .build(),
    ];
    let rpass_colors = [
        vk::AttachmentReference2::builder()
            .attachment(3)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .build(),
    ];
    let rpass_depth = vk::AttachmentReference2::builder()
        .attachment(2)
        .layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
        .aspect_mask(vk::ImageAspectFlags::STENCIL);

    let subpasses = [
        vk::SubpassDescription2::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .input_attachments(&gpass_inputs)
            .color_attachments(&gpass_colors)
            .depth_stencil_attachment(&gpass_depth)
            .build(),
        vk::SubpassDescription2::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .input_attachments(&rpass_inputs)
            .color_attachments(&rpass_colors)
            .depth_stencil_attachment(&rpass_depth)
            .build(),
    ];

    // 0:
    //      Reads depth/stencil
    //      Writes g0, g1, depth/stencil
    // 1:
    //      Reads g0, g1, depth/stencil
    //      Writes output
    let dependencies = [
        // EXT to 0: wait for previous frame to read g0, g1 before writing them
        vk::SubpassDependency2::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ)
            .dst_subpass(0)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build(),
        // EXT to 0: wait for previous frame to read depth/stencil before fragment tests
        vk::SubpassDependency2::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ)
            .dst_subpass(0)
            .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .build(),
        // 0 to 1: wait for 0 to write g0, g1 before reading them
        vk::SubpassDependency2::builder()
            .src_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_subpass(1)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .dst_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ)
            .build(),
        // 0 to 1: wait for 0 to write d/s before reading it
        vk::SubpassDependency2::builder()
            .src_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ)
            .dst_subpass(1)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .dst_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ)
            .build(),
        // 1 to external: wait for 1 to write output image before transitioning to transfer
        vk::SubpassDependency2::builder()
            .src_subpass(1)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .dst_stage_mask(vk::PipelineStageFlags::TRANSFER)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
            .build(),
    ];

    let info = vk::RenderPassCreateInfo2::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);

    unsafe {
        Ok(device.create_render_pass2(&info, None)?)
    }
}
