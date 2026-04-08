use crate::context::ContextData;
use anyhow::{Result, anyhow};
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::InstanceV1_0;

use vulkanalia::vk::CommandBufferBeginInfo;
use vulkanalia::vk::Handle;

use vulkanalia::vk::KhrSurfaceExtensionInstanceCommands;
///
/// Get Memory type index
///
pub fn get_memory_type_index(
    instance: &Instance,
    data: &ContextData,
    properties: vk::MemoryPropertyFlags,
    requirements: vk::MemoryRequirements,
) -> Result<u32> {
    let memroy = unsafe {
        instance.get_physical_device_memory_properties(data.device_manager.physical_device)
    };
    (0..memroy.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memroy.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("get memory type error"))
}

///
/// Create Buffer
///
pub fn create_buffer(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            data,
            properties,
            requirements,
        )?);

    let buffer_memory = unsafe { device.allocate_memory(&memory_info, None)? };

    unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0)? }

    Ok((buffer, buffer_memory))
}

pub fn create_image(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
    width: u32,
    height: u32,
    mip_levels: u32,
    format: vk::Format,
    tiling: vk::ImageTiling,
    samples: vk::SampleCountFlags,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory)> {
    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_levels)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .samples(samples)
        .flags(vk::ImageCreateFlags::empty());
    let texture_image = unsafe { device.create_image(&image_info, None)? };

    let requirements = unsafe { device.get_image_memory_requirements(texture_image) };

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            instance,
            data,
            properties,
            requirements,
        )?);

    let texture_image_memory = unsafe { device.allocate_memory(&memory_info, None)? };

    unsafe { device.bind_image_memory(texture_image, texture_image_memory, 0)? };

    Ok((texture_image, texture_image_memory))
}

///
/// Create Image View
///
pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    mip_levels: u32,
) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(unsafe { device.create_image_view(&info, None)? })
}

pub fn begin_single_time_commands(
    device: &Device,
    data: &ContextData,
) -> Result<vk::CommandBuffer> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(data.command_manager.pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info)?[0] };

    let begin_info =
        CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe { device.begin_command_buffer(command_buffer, &begin_info)? }

    Ok(command_buffer)
}

pub fn end_single_time_commands(
    device: &Device,
    data: &ContextData,
    command_buffer: vk::CommandBuffer,
) -> Result<()> {
    unsafe { device.end_command_buffer(command_buffer)? };

    let command_buffers = &[command_buffer];
    let submit_info = vk::SubmitInfo::builder().command_buffers(command_buffers);

    unsafe {
        device.queue_submit(
            data.device_queue.graphics_queue,
            &[submit_info],
            vk::Fence::null(),
        )?;
        device.queue_wait_idle(data.device_queue.graphics_queue)?;

        device.free_command_buffers(data.command_manager.pool, command_buffers);
    }

    Ok(())
}

///
/// QueueFamilyindices
///
pub struct QueueFamilyindices {
    pub(crate) graphics: u32,
    pub(crate) present: u32,
}

impl QueueFamilyindices {
    pub fn get(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<Self> {
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;

        for (index, _) in properties.iter().enumerate() {
            if unsafe {
                instance.get_physical_device_surface_support_khr(
                    physical_device,
                    index as u32,
                    surface,
                )?
            } {
                present = Some(index as u32);
                break;
            }
        }

        if let (Some(graphics), Some(present)) = (graphics, present) {
            Ok(Self { graphics, present })
        } else {
            Err(anyhow!("error"))
        }
    }
}

// SwapChain
pub struct SwapChainSupport {
    pub(crate) capabilities: vk::SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<vk::SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupport {
    pub fn get(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
    ) -> Result<Self> {
        Ok(Self {
            capabilities: unsafe {
                instance.get_physical_device_surface_capabilities_khr(physical_device, surface)?
            },
            formats: unsafe {
                instance.get_physical_device_surface_formats_khr(physical_device, surface)?
            },
            present_modes: unsafe {
                instance.get_physical_device_surface_present_modes_khr(physical_device, surface)?
            },
        })
    }
}
