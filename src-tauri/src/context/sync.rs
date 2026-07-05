#![allow(unused)]
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;

#[derive(Default)]
pub struct SyncObjects {
    pub image_available: Vec<vk::Semaphore>,
    pub render_finished: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
}

impl SyncObjects {
    pub fn create(device: &Device, image_count: usize) -> Result<Self> {
        let mut image_available = Vec::new();
        let mut render_finished = Vec::new();
        let mut in_flight_fences = Vec::new();

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..image_count {
            image_available.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            render_finished.push(unsafe { device.create_semaphore(&semaphore_info, None)? });
            in_flight_fences.push(unsafe { device.create_fence(&fence_info, None)? });
        }

        let images_in_flight = vec![vk::Fence::null(); image_count];

        Ok(Self {
            image_available,
            render_finished,
            in_flight_fences,
            images_in_flight,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            for semaphore in &self.image_available {
                device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.render_finished {
                device.destroy_semaphore(*semaphore, None);
            }
            for fence in &self.in_flight_fences {
                device.destroy_fence(*fence, None);
            }
        }
    }
}

