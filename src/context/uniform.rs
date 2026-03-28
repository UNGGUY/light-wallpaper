use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;

use crate::context::ContextData;
use crate::context::tool;

type Vec2 = cgmath::Vector2<f32>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformBufferObject {
    pub(crate) i_time: f32,
    pub(crate) _padding: f32,
    pub(crate) i_resolution: Vec2,
}

pub fn create_uniform_buffers(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    data.uniform_buffers.clear();
    data.uniform_buffers_memory.clear();

    for _ in 0..data.swapchain_images.len() {
        let (uniform_buffer, uniform_buffer_memory) = tool::create_buffer(
            instance,
            device,
            data,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        data.uniform_buffers.push(uniform_buffer);
        data.uniform_buffers_memory.push(uniform_buffer_memory);
    }

    Ok(())
}
