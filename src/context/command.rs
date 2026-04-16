#![allow(unused)]
use crate::context::ContextData;
use crate::context::tool::QueueFamilyindices;
use crate::context::vertex;
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
    pub fn create(instance: &Instance, device: &Device, data: &mut ContextData) -> Result<Self> {
        let pool = create_command_pool(instance, device, data)?;

        Ok(Self {
            pool,
            buffers: Vec::new(),
        })
    }

    pub fn allocate_buffers(
        &mut self,
        device: &Device,
        count: u32,
    ) -> Result<&[vk::CommandBuffer]> {
        let info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        self.buffers = unsafe { device.allocate_command_buffers(&info)? };
        Ok(&self.buffers)
    }

    pub fn record_command_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
        for (i, command_buffer) in data.command_manager.buffers.iter().enumerate() {
            let info = vk::CommandBufferBeginInfo::builder();
            unsafe { device.begin_command_buffer(*command_buffer, &info)? };

            let render_area = vk::Rect2D::builder()
                .offset(vk::Offset2D::default())
                .extent(data.swapchain.extent);

            let color_clear_value = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            };

            let clear_values = &[color_clear_value];
            let info = vk::RenderPassBeginInfo::builder()
                .render_pass(data.pipeline.render_pass)
                .framebuffer(data.frame_buffers[i])
                .render_area(render_area)
                .clear_values(clear_values);

            unsafe {
                device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    data.pipeline.pipeline,
                );
                device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
                device.cmd_bind_index_buffer(
                    *command_buffer,
                    data.index_buffer,
                    0,
                    vk::IndexType::UINT16,
                );
                device.cmd_bind_descriptor_sets(
                    *command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    data.pipeline.layout,
                    0,
                    &[data.descriptor_manager.sets[i]],
                    &[],
                );
                device.cmd_draw_indexed(*command_buffer, vertex::INDICES.len() as u32, 1, 0, 0, 0);
                device.cmd_end_render_pass(*command_buffer);
                device.end_command_buffer(*command_buffer)?;
            }
        }
        Ok(())
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
    let indices =
        QueueFamilyindices::get(instance, data.device_manager.physical_device, data.surface)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty())
        .queue_family_index(indices.graphics);

    let pool = unsafe { device.create_command_pool(&info, None)? };
    Ok(pool)
}
