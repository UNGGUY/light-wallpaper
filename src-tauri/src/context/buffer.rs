#![allow(unused)]
use crate::context::ContextData;
use crate::context::tool;
use anyhow::Result;
use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;

pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl Buffer {
    pub fn create_vertex_buffer(
        instance: &Instance,
        device: &Device,
        data: &mut ContextData,
        vertices: &[crate::context::vertex::Vertex],
    ) -> Result<Self> {
        let size = (std::mem::size_of::<crate::context::vertex::Vertex>() * vertices.len()) as u64;

        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(tool::get_memory_type_index(
                instance,
                data,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                requirements,
            )?);

        let memory = unsafe { device.allocate_memory(&allocate_info, None)? };

        unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

        let mapped = unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty())? };

        unsafe {
            memcpy(vertices.as_ptr(), mapped.cast(), vertices.len());
            device.unmap_memory(memory);
        }

        Ok(Self { buffer, memory })
    }

    pub fn create_index_buffer(
        instance: &Instance,
        device: &Device,
        data: &mut ContextData,
        indices: &[u16],
    ) -> Result<Self> {
        let size = (std::mem::size_of::<u16>() * indices.len()) as u64;

        // Staging buffer
        let (staging_buffer, staging_memory) = tool::create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let mapped =
            unsafe { device.map_memory(staging_memory, 0, size, vk::MemoryMapFlags::empty())? };

        unsafe {
            memcpy(indices.as_ptr(), mapped.cast(), indices.len());
            device.unmap_memory(staging_memory);
        }

        // Device local buffer
        let (buffer, memory) = tool::create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        copy_buffer(device, data, staging_buffer, buffer, size)?;

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_memory, None);
        }

        Ok(Self { buffer, memory })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}

fn copy_buffer(
    device: &Device,
    data: &ContextData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_manager.pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info)?[0] };

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(command_buffer, &info)?;

        let regions = vk::BufferCopy::builder().size(size);
        device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

        device.end_command_buffer(command_buffer)?;
    };

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    unsafe {
        device.queue_submit(data.device_queue.graphics_queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(data.device_queue.graphics_queue)?;
        device.free_command_buffers(data.command_manager.pool, &[command_buffer]);
    }

    Ok(())
}
