use crate::context::ContextData;
use crate::context::tool;

use anyhow::Result;

use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::InstanceV1_0;

pub fn create_color_objects(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    let (color_image, color_image_memory) = tool::create_image(
        instance,
        device,
        data,
        data.swapchain_extent.width,
        data.swapchain_extent.height,
        1,
        data.swapchain_format,
        vk::ImageTiling::OPTIMAL,
        data.msaa_samples,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.color_image = color_image;
    data.color_image_memory = color_image_memory;

    data.color_image_view =
        tool::create_image_view(device, data.color_image, data.swapchain_format, 1)?;

    Ok(())
}

pub fn get_max_msaa_samples(instance: &Instance, data: &ContextData) -> vk::SampleCountFlags {
    let properties = unsafe { instance.get_physical_device_properties(data.physical_device) };
    let counts = properties.limits.framebuffer_color_sample_counts
        & properties.limits.framebuffer_depth_sample_counts;
    [
        vk::SampleCountFlags::_64,
        vk::SampleCountFlags::_32,
        vk::SampleCountFlags::_16,
        vk::SampleCountFlags::_8,
        vk::SampleCountFlags::_4,
        vk::SampleCountFlags::_2,
    ]
    .iter()
    .cloned()
    .find(|c| counts.contains(*c))
    .unwrap_or(vk::SampleCountFlags::_1)
}
