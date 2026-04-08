#![allow(unused)]
use crate::context::uniform::UniformBufferObject;
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

#[derive(Default)]
pub struct DescriptorManager {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
}

impl DescriptorManager {
    pub fn create(device: &Device, image_count: usize) -> Result<Self> {
        let layout = create_descriptor_set_layout(device)?;
        let pool = create_descriptor_pool(device, image_count)?;
        let sets = create_descriptor_sets(device, layout, &pool, image_count)?;

        Ok(Self {
            layout,
            pool,
            sets,
        })
    }

    pub fn update(
        &self,
        device: &Device,
        uniform_buffers: &[vk::Buffer],
        texture_view: vk::ImageView,
        texture_sampler: vk::Sampler,
    ) {
        for i in 0..self.sets.len() {
            let info = vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as u64);

            let buffer_info = &[info];
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(self.sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_view)
                .sampler(texture_sampler);

            let image_infos = &[image_info];
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(self.sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_infos);

            unsafe {
                device.update_descriptor_sets(&[ubo_write, sampler_write], &[] as &[vk::CopyDescriptorSet]);
            }
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_descriptor_pool(self.pool, None);
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

fn create_descriptor_set_layout(device: &Device) -> Result<vk::DescriptorSetLayout> {
    let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(1)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT);

    let bindings = &[ubo_binding, sampler_binding];
    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    let layout = unsafe { device.create_descriptor_set_layout(&info, None)? };

    Ok(layout)
}

fn create_descriptor_pool(device: &Device, image_count: usize) -> Result<vk::DescriptorPool> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(image_count as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(image_count as u32);

    let pool_sizes = &[ubo_size, sampler_size];

    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(image_count as u32);

    let pool = unsafe { device.create_descriptor_pool(&info, None)? };

    Ok(pool)
}

fn create_descriptor_sets(
    device: &Device,
    layout: vk::DescriptorSetLayout,
    pool: &vk::DescriptorPool,
    image_count: usize,
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![layout; image_count];
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(*pool)
        .set_layouts(&layouts);

    let sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

    Ok(sets)
}
