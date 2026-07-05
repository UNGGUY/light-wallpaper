use crate::context::ContextData;
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

pub fn create_frame_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
    data.frame_buffers = data
        .swapchain
        .image_views
        .iter()
        .map(|image_view| {
            let attachments = if data.msaa_samples != vk::SampleCountFlags::_1 {
                vec![data.color_image_view, *image_view]
            } else {
                vec![*image_view]
            };

            let info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.pipeline.render_pass)
                .attachments(&attachments)
                .width(data.swapchain.extent.width)
                .height(data.swapchain.extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&info, None).unwrap() }
        })
        .collect::<Vec<_>>();
    Ok(())
}
