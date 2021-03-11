use ash::{version::DeviceV1_0, vk::{self, SubpassDependency}};

use crate::utils::color;

pub fn create_deferred_pass(
    color_format: vk::Format,
    depth_format: vk::Format,
    logical_device: &ash::Device,
) -> Result<vk::RenderPass, vk::Result> {
    let attachments = [
        // Resolve
        vk::AttachmentDescription::builder()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
        // Depth
        vk::AttachmentDescription::builder()
            .format(depth_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE) // TODO: needs to be STORE when used in PP effects
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
        // AlbedoRoughness
        vk::AttachmentDescription::builder()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
        // NormalMetallic
        vk::AttachmentDescription::builder()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),
    ];

    let color_attachment_references_0 = [
        vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }
    ];
    let depth_attachment_reference_0 = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let color_attachment_references_1 = [
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }
    ];
    let input_attachment_references_1 = [
        vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        },
        vk::AttachmentReference {
            attachment: 3,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        },
        vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
        },
    ];
    let depth_attachment_reference_1 = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
    };

    let subpasses = [
        vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_references_0)
            .depth_stencil_attachment(&depth_attachment_reference_0)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build(),
        vk::SubpassDescription::builder()
            .color_attachments(&color_attachment_references_1)
            .depth_stencil_attachment(&depth_attachment_reference_1)
            .input_attachments(&input_attachment_references_1)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build(),
    ];
    let subpass_dependencies = [
        // external to 0: wait for previous frame image blit reading g0 image
        vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::TRANSFER)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::TRANSFER_READ)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build(),
        // 0 to 1: wait for gpass writing of 0 before input attachment reading of 1
        vk::SubpassDependency::builder()
            .src_subpass(0)
            .dst_subpass(1)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ)
            .build(),
        // 0 to 1: wait for fragment tests of 0 before fragment tests and input attachment reading of 1
        vk::SubpassDependency::builder()
            .src_subpass(0)
            .dst_subpass(1)
            .src_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
            .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS | vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::INPUT_ATTACHMENT_READ)
            .build(),
        // 1 to pp: wait for color attachment writing of 1 before sampling of PP
        vk::SubpassDependency::builder()
            .src_subpass(1)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .build()
    ];
    let renderpass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);
    let renderpass = unsafe { logical_device.create_render_pass(&renderpass_info, None)? };
    Ok(renderpass)
}
