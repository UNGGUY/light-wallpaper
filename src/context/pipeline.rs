#![allow(unused)]
use crate::context::ContextData;
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;

#[derive(Default)]
pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
}

impl Pipeline {
    pub fn create(
        device: &Device,
        swapchain_format: vk::Format,
        swapchain_extent: vk::Extent2D,
        msaa_samples: vk::SampleCountFlags,
        descriptor_set_layout: vk::DescriptorSetLayout,
        vert_shader_code: &[u8],
        frag_shader_code: &[u8],
    ) -> Result<Self> {
        let render_pass = create_render_pass(device, swapchain_format, msaa_samples)?;
        let (pipeline, layout) = create_graphics_pipeline(
            device,
            swapchain_extent,
            msaa_samples,
            render_pass,
            descriptor_set_layout,
            vert_shader_code,
            frag_shader_code,
        )?;

        Ok(Self {
            pipeline,
            layout,
            render_pass,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}

fn create_render_pass(
    device: &Device,
    swapchain_format: vk::Format,
    msaa_samples: vk::SampleCountFlags,
) -> Result<vk::RenderPass> {
    // 判断是否启用 MSAA
    let is_msaa = msaa_samples != vk::SampleCountFlags::_1;

    if is_msaa {
        // MSAA 模式：需要两个附件
        // 0: MSAA 颜色附件 (多重采样)
        // 1: Resolve 附件 (交换链图像，单采样)
        let msaa_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let resolve_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];
        let resolve_attachments = &[resolve_attachment_ref];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(color_attachments)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .resolve_attachments(resolve_attachments);

        let all_attachments = &[msaa_attachment, resolve_attachment];
        let subpasses = &[subpass];

        let render_info = vk::RenderPassCreateInfo::builder()
            .attachments(all_attachments)
            .subpasses(subpasses);

        let render_pass = unsafe { device.create_render_pass(&render_info, None)? };
        Ok(render_pass)
    } else {
        // 非 MSAA 模式：只需要一个附件
        // 0: 交换链图像 (单采样，直接呈现)
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = &[color_attachment_ref];

        let subpass = vk::SubpassDescription::builder()
            .color_attachments(color_attachments)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);
        // 注意：没有 resolve_attachments

        let all_attachments = &[color_attachment];
        let subpasses = &[subpass];

        let render_info = vk::RenderPassCreateInfo::builder()
            .attachments(all_attachments)
            .subpasses(subpasses);

        let render_pass = unsafe { device.create_render_pass(&render_info, None)? };
        Ok(render_pass)
    }
}

use crate::context::vertex::Vertex;

fn create_graphics_pipeline(
    device: &Device,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    vert_shader_code: &[u8],
    frag_shader_code: &[u8],
) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
    let vert_module = create_shader_module(device, vert_shader_code)?;
    let frag_module = create_shader_module(device, frag_shader_code)?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(b"main\0");

    // Vertex input
    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_description();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    // Input assembly
    let assembly_input_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // Viewport and scissor
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_extent.width as f32)
        .height(swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    // Rasterization
    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);

    // Multisampling
    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(msaa_samples)
        .sample_shading_enable(false);

    // Color blending
    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    // Pipeline layout
    let set_layouts = &[descriptor_set_layout];
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

    let stages = &[vert_stage, frag_stage];

    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&assembly_input_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .color_blend_state(&color_blend_state)
        .multisample_state(&multisample_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
            .0[0]
    };

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    Ok((pipeline, pipeline_layout))
}

fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code(bytecode.code())
        .code_size(bytecode.code_size());

    Ok(unsafe { device.create_shader_module(&info, None)? })
}
