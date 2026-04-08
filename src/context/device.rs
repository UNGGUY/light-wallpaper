#![allow(unused)]
use crate::context::ContextData;
use crate::context::tool;
use crate::context::tool::QueueFamilyindices;
use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashSet;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::PhysicalDevice;
use vulkanalia::vk::SurfaceKHR;

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

#[derive(Default)]
pub struct DeviceManager {
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) graphics_family: u32,
    pub(crate) present_family: u32,
}
#[derive(Default)]
pub struct DeviceQueue {
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
}

impl DeviceManager {
    pub fn create(instance: &Instance, surface: vk::SurfaceKHR) -> Result<DeviceManager> {
        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        // 2. 遍历所有设备
        for physical_device in physical_devices {
            // 3. 尝试检查当前设备
            match Self::check_physical_device(instance, physical_device, surface) {
                Ok(indices) => {
                    // 找到了合适的设备，直接返回构造好的结构体
                    return Ok(Self {
                        physical_device,
                        graphics_family: indices.graphics,
                        present_family: indices.present,
                    });
                }
                Err(_) => {
                    // 当前设备不合适，不要返回错误，而是继续循环检查下一个设备
                    continue;
                }
            }
        }
        // 4. 如果循环结束还没返回，说明没有一个设备是合适的
        Err(anyhow::anyhow!("Failed to find a suitable GPU!"))
    }
    pub fn check_physical_device(
        instance: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
    ) -> Result<QueueFamilyindices> {
        let indices = tool::QueueFamilyindices::get(instance, physical_device, surface)?;
        Self::check_physical_device_extensions(instance, physical_device)?;

        let support = tool::SwapChainSupport::get(instance, physical_device, surface)?;

        if support.formats.is_empty() || support.present_modes.is_empty() {
            return Err(anyhow!("Do not support swapchain"));
        }

        let features = unsafe { instance.get_physical_device_features(physical_device) };

        if features.sampler_anisotropy != vk::TRUE {
            return Err(anyhow!("do not support anisotropy"));
        }

        Ok(indices)
    }

    pub fn check_physical_device_extensions(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> Result<()> {
        let extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device, None)? }
                .iter()
                .map(|e| e.extension_name)
                .collect::<HashSet<_>>();

        if DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e)) {
            return Ok(());
        }

        Err(anyhow!("extension error"))
    }

    //
    //logical device
    //
}

//
// Create LogicalDevice
//

pub fn create_logical_device(
    instance: &Instance,
    device_manager: &DeviceManager,
) -> Result<(Device, DeviceQueue)> {
    let mut unique_indices = HashSet::new();
    unique_indices.insert(device_manager.graphics_family);
    unique_indices.insert(device_manager.present_family);

    // QueueFamily
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_priorities(&[0.0])
                .queue_family_index(*i)
        })
        .collect::<Vec<_>>();

    //Layer
    let layer = vec![];

    // Feature
    let feature = vk::PhysicalDeviceFeatures::builder()
        .sampler_anisotropy(true)
        .sample_rate_shading(true);

    //Extension

    let extensions = DEVICE_EXTENSIONS
        .iter()
        .map(|f| f.as_ptr())
        .collect::<Vec<_>>();

    let device_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&extensions)
        .enabled_features(&feature)
        .enabled_layer_names(&layer);

    let device =
        unsafe { instance.create_device(device_manager.physical_device, &device_info, None)? };

    let graphics_queue = unsafe { device.get_device_queue(device_manager.graphics_family, 0) };
    let present_queue = unsafe { device.get_device_queue(device_manager.present_family, 0) };

    let device_queue = DeviceQueue {
        graphics_queue,
        present_queue,
    };

    Ok((device, device_queue))
}
