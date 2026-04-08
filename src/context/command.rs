#![allow(unused)]
use crate::context::tool::QueueFamilyindices;
use crate::context::ContextData;
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

#[derive(Default)]
pub struct CommandManager {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl CommandManager {
    pub fn create(
        instance: &Instance,
        device: &Device,
        data: &mut ContextData,
    ) -> Result<Self> {
        let pool = create_command_pool(instance, device, data)?;

        Ok(Self {
            pool,
            buffers: Vec::new(),
        })
    }

    pub fn allocate_buffers(&mut self, device: &Device, count: u32) -> Result<&[vk::CommandBuffer]> {
        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        self.buffers = unsafe { device.allocate_command_buffers(&info)? };
        Ok(&self.buffers)
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_command_pool(self.pool, None);
        }
    }
}

fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &ContextData,
) -> Result<vk::CommandPool> {
    let indices = QueueFamilyindices::get(instance, data.device_manager.physical_device, data.surface)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty())
        .queue_family_index(indices.graphics);

    let pool = unsafe { device.create_command_pool(&info, None)? };
    Ok(pool)
}
