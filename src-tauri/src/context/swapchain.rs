#![allow(unused)]
use crate::context::ContextData;
use crate::context::DeviceManager;
use crate::context::swapchain;
use crate::context::tool::QueueFamilyindices;
use crate::context::tool::SwapChainSupport;
use anyhow::Result;
use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::ImageView;
use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;
use winit::window::Window;

#[derive(Default)]
pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn create_for_winit(
        window: &Window,
        instance: &Instance,
        device: &Device,
        device_manager: &DeviceManager,
        surface: vk::SurfaceKHR,
    ) -> Result<Self> {
        let support = SwapChainSupport::get(instance, device_manager.physical_device, surface)?;

        let surface_format = get_swapchain_surface_format(&support.formats);
        let present_mode = get_swapchain_present_mode(&support.present_modes);
        let extent = get_swapchain_extent(window, support.capabilities);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if device_manager.graphics_family != device_manager.present_family
        {
            queue_family_indices.push(device_manager.graphics_family);
            queue_family_indices.push(device_manager.present_family);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe { device.create_swapchain_khr(&swapchain_info, None)? };
        let images = unsafe { device.get_swapchain_images_khr(swapchain)? };

        let image_views = create_swapchain_image_views(device, &images, surface_format.format)?;

        Ok(Self {
            swapchain,
            images,
            format: surface_format.format,
            extent,
            image_views,
        })
    }

    pub fn create_for_wayland(
        width: u32,
        height: u32,
        instance: &Instance,
        device: &Device,
        device_manager: &DeviceManager,
        surface: vk::SurfaceKHR,
    ) -> Result<Self> {
        let indices = QueueFamilyindices::get(instance, device_manager.physical_device, surface)?;
        let support = SwapChainSupport::get(instance, device_manager.physical_device, surface)?;

        let surface_format = get_swapchain_surface_format(&support.formats);
        let present_mode = get_swapchain_present_mode(&support.present_modes);
        let extent = get_swapchain_extent_wayland(width, height, support.capabilities);

        let mut image_count = support.capabilities.min_image_count + 1;
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if indices.graphics != indices.present {
            queue_family_indices.push(indices.graphics);
            queue_family_indices.push(indices.present);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = unsafe { device.create_swapchain_khr(&swapchain_info, None)? };
        let images = unsafe { device.get_swapchain_images_khr(swapchain)? };

        let image_views = create_swapchain_image_views(device, &images, surface_format.format)?;

        Ok(Self {
            swapchain,
            images,
            format: surface_format.format,
            extent,
            image_views,
        })
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            for image_view in &self.image_views {
                device.destroy_image_view(*image_view, None);
            }
            device.destroy_swapchain_khr(self.swapchain, None);
        }
    }
}

fn create_swapchain_image_views(
    device: &Device,
    images: &Vec<vk::Image>,
    format: vk::Format,
) -> Result<Vec<vk::ImageView>> {
    let image_views = images
        .iter()
        .map(|i| {
            let components = vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY);

            let subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);

            let image_info = vk::ImageViewCreateInfo::builder()
                .image(*i)
                .view_type(vk::ImageViewType::_2D)
                .format(format)
                .components(components)
                .subresource_range(subresource_range);

            unsafe {
                device
                    .create_image_view(&image_info, None)
                    .unwrap_or(ImageView::null())
            }
        })
        .collect::<Vec<_>>();
    Ok(image_views)
}

fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    formats
        .iter()
        .cloned()
        .find(|f| {
            f.format == vk::Format::R8G8B8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| formats[0])
}

fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
    present_modes
        .iter()
        .cloned()
        .find(|f| *f == vk::PresentModeKHR::MAILBOX)
        .unwrap_or_else(|| present_modes[0])
}

fn get_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D::builder()
        .width(window.inner_size().width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ))
        .height(window.inner_size().height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ))
        .build()
}

fn get_swapchain_extent_wayland(
    width: u32,
    height: u32,
    capabilities: vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }
    vk::Extent2D::builder()
        .width(width.clamp(
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        ))
        .height(height.clamp(
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        ))
        .build()
}
