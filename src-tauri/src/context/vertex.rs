use crate::context::ContextData;
use crate::context::tool;

use anyhow::Result;

use cgmath::vec2;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;
use vulkanalia::vk::{self, HasBuilder};

use std::ptr::copy_nonoverlapping as memcpy;

type Vec2 = cgmath::Vector2<f32>;
pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
pub static VERTICES: [Vertex; 4] = [
    Vertex::new(vec2(1.0, -1.0), vec2(1.0, 0.0)),
    Vertex::new(vec2(-1.0, -1.0), vec2(0.0, 0.0)),
    Vertex::new(vec2(-1.0, 1.0), vec2(0.0, 1.0)),
    Vertex::new(vec2(1.0, 1.0), vec2(1.0, 1.0)),
];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pos: Vec2,
    tex_coord: Vec2,
}

impl Vertex {
    const fn new(pos: Vec2, tex_coord: Vec2) -> Self {
        Self { pos, tex_coord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(size_of::<Vertex>() as u32)
            .build()
    }

    pub fn attribute_description() -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(0)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(size_of::<Vec2>() as u32)
            .build();

        [pos, tex_coord]
    }
}

///
/// Create Vertex Buffer
///
pub fn create_vertex_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    let info = vk::BufferCreateInfo::builder()
        .size((size_of::<Vertex>() * VERTICES.len()) as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    data.vertex_buffer = unsafe { device.create_buffer(&info, None)? };

    let requirements = unsafe { device.get_buffer_memory_requirements(data.vertex_buffer) };

    let allocate_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(tool::get_memory_type_index(
            instance,
            data,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            requirements,
        )?);
    data.vertex_buffer_memory = unsafe { device.allocate_memory(&allocate_info, None)? };

    unsafe { device.bind_buffer_memory(data.vertex_buffer, data.vertex_buffer_memory, 0)? };

    let memory = unsafe {
        device.map_memory(
            data.vertex_buffer_memory,
            0,
            info.size,
            vk::MemoryMapFlags::empty(),
        )?
    };

    unsafe {
        memcpy(VERTICES.as_ptr(), memory.cast(), VERTICES.len());
        device.unmap_memory(data.vertex_buffer_memory);
    }

    Ok(())
}

///
/// Create Index Buffer
///
pub fn create_index_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    let size = (size_of::<u16>() * INDICES.len()) as u64;
    let (staging_buffer, staging_buffer_memory) = tool::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory =
        unsafe { device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())? };

    unsafe {
        memcpy(INDICES.as_ptr(), memory.cast(), INDICES.len());

        device.unmap_memory(staging_buffer_memory);
    }

    let (index_buffer, index_buffer_memory) = tool::create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.index_buffer = index_buffer;
    data.index_buffer_memory = index_buffer_memory;

    copy_buffer(device, data, staging_buffer, index_buffer, size)?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    Ok(())
}

fn copy_buffer(
    device: &Device,
    data: &ContextData,
    source: vk::Buffer,
    destination: vk::Buffer,
    size: vk::DeviceSize,
) -> Result<()> {
    // Allocate

    let info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_manager.pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&info)?[0] };

    // Commands

    let info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(command_buffer, &info)?;

        let regions = vk::BufferCopy::builder().size(size);
        device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

        device.end_command_buffer(command_buffer)?;
    };

    // Submit

    let command_buffers = &[command_buffer];
    let info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    unsafe {
        device.queue_submit(data.device_queue.graphics_queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(data.device_queue.graphics_queue)?;

        // Cleanup

        device.free_command_buffers(data.command_manager.pool, &[command_buffer]);
    }

    Ok(())
}
