#![allow(unused)]
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::Instant;

use cgmath::vec2;

use crate::context::UniformBufferObject;
use crate::context::Vertex;
use crate::context::uniform::create_uniform_buffers;
use crate::context::vertex;
use image::DynamicImage;
use vertex::VERTICES;

use crate::context::tool;

use crate::context::msaa;
use crate::context::texture;

use anyhow::{Result, anyhow};
use vulkanalia::Device;
use vulkanalia::Entry;
use vulkanalia::Instance;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::loader::LIBRARY;
use vulkanalia::loader::LibloadingLoader;
use vulkanalia::vk;
use vulkanalia::vk::AttachmentLoadOp;
use vulkanalia::vk::AttachmentStoreOp;
use vulkanalia::vk::DescriptorSetLayoutCreateInfo;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::EntryV1_0;
use vulkanalia::vk::Framebuffer;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::ImageView;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::KhrSurfaceExtensionInstanceCommands;
use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;

use vulkanalia::vk::KhrWaylandSurfaceExtensionInstanceCommands;

use vulkanalia::vk::PhysicalDevice;
use vulkanalia::vk::SurfaceKHR;
use vulkanalia::window as vk_window;
use winit::window::Window;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

const DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];

pub struct Context {
    instance: Instance,
    data: ContextData,
    device: Device,
    frame: usize,

    start: Instant,
    image: DynamicImage,
}

#[derive(Default)]
pub struct ContextData {
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    swapchain_image_view: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    frame_buffers: Vec<vk::Framebuffer>,

    pub(crate) command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,

    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,

    pub(crate) uniform_buffers: Vec<vk::Buffer>,
    pub(crate) uniform_buffers_memory: Vec<vk::DeviceMemory>,

    pub(crate) texture_image: vk::Image,
    pub(crate) texture_image_memory: vk::DeviceMemory,
    pub(crate) texture_image_view: vk::ImageView,
    pub(crate) texture_image_sampler: vk::Sampler,

    // Msaa
    pub(crate) color_image: vk::Image,
    pub(crate) color_image_memory: vk::DeviceMemory,
    pub(crate) color_image_view: vk::ImageView,
    pub(crate) msaa_samples: vk::SampleCountFlags,

    image_available_semaphore: Vec<vk::Semaphore>,
    render_finished_semaphore: Vec<vk::Semaphore>,

    in_flight_fences: Vec<vk::Fence>,
    image_in_flight: Vec<vk::Fence>,
}

impl Context {
    pub fn create_for_wayland(
        surface: *mut c_void,
        display: *mut c_void,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };

        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!(b))? };

        let instance = create_instance_wayland(&entry)?;

        let image = texture::read_image("assets/wallhaven-3q3wj3.jpg")?;

        let mut data = ContextData::default();

        create_surface(&instance, &mut data, surface, display)?;

        pick_physical_device(&instance, &mut data);

        let device = create_logical_device(&instance, &mut data)?;

        data.msaa_samples = msaa::get_max_msaa_samples(&instance, &data);

        //create swapchain
        create_swapchain_wayland(width, height, &instance, &device, &mut data)?;

        create_swapchain_image_view(&device, &mut data)?;

        // create render pass
        create_render_pass(&device, &mut data)?;

        //create descriptor set
        create_descriptor_set_layout(&device, &mut data)?;

        // create pipeline
        create_pipeline(&device, &mut data)?;

        //create command
        create_command_pool(&instance, &device, &mut data)?;

        msaa::create_color_objects(&instance, &device, &mut data)?;

        // create frame
        create_frame_buffers(&device, &mut data)?;

        //create image
        texture::create_texture_image(&instance, &device, &mut data, &image)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        // create vertex
        create_vertex_buffer(&instance, &device, &mut data)?;

        // create index
        create_index_buffer(&instance, &device, &mut data)?;

        create_uniform_buffers(&instance, &device, &mut data)?;

        // create descriptor
        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;

        create_command_buffers(&device, &mut data)?;

        // create sync
        create_sync_objects(&device, &mut data)?;
        Ok(Self {
            instance,
            data,
            device,
            frame: 0,
            start: Instant::now(),
            image,
        })
    }

    #[allow(unused)]
    pub fn create(window: &Window) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };

        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!(b))? };

        let instance = create_instance(window, &entry)?;

        let image = texture::read_image("assets/wallhaven-3q3wj3.jpg")?;

        let mut data = ContextData::default();

        data.surface = unsafe { vk_window::create_surface(&instance, window, window)? };

        pick_physical_device(&instance, &mut data);

        let device = create_logical_device(&instance, &mut data)?;

        //create swapchain
        create_swapchain(window, &instance, &device, &mut data)?;

        create_swapchain_image_view(&device, &mut data)?;

        // create render pass
        create_render_pass(&device, &mut data)?;

        //create descriptor set
        create_descriptor_set_layout(&device, &mut data)?;

        // create pipeline
        create_pipeline(&device, &mut data)?;

        // create frame
        create_frame_buffers(&device, &mut data)?;

        //create command
        create_command_pool(&instance, &device, &mut data)?;

        //create image
        texture::create_texture_image(&instance, &device, &mut data, &image)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        // create vertex
        create_vertex_buffer(&instance, &device, &mut data)?;

        // create index
        create_index_buffer(&instance, &device, &mut data)?;

        // create descriptor
        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;

        create_command_buffers(&device, &mut data)?;

        // create sync
        create_sync_objects(&device, &mut data)?;
        Ok(Self {
            instance,
            data,
            device,
            frame: 0,
            start: Instant::now(),
            image,
        })
    }

    pub fn render_wayland(&mut self) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];
        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        };

        let image_index = unsafe {
            self.device
                .acquire_next_image_khr(
                    self.data.swapchain,
                    u64::MAX,
                    self.data.image_available_semaphore[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize
        };

        let image_fence = self.data.image_in_flight[image_index];

        if !image_fence.is_null() {
            unsafe {
                self.device
                    .wait_for_fences(&[image_fence], true, u64::MAX)?;
            }
        };

        self.data.image_in_flight[image_index] = in_flight_fence;

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.data.image_available_semaphore[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphore[image_index as usize]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe { self.device.reset_fences(&[in_flight_fence])? };

        unsafe {
            self.device
                .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?
        };

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            self.device
                .queue_present_khr(self.data.present_queue, &present_info)?
        };

        self.frame = (self.frame + 1) % self.data.swapchain_images.len();

        Ok(())
    }

    #[allow(unused_variables, unused)]
    pub fn render(&mut self, window: &Window) -> Result<()> {
        let in_flight_fence = self.data.in_flight_fences[self.frame];
        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        };

        let image_index = unsafe {
            self.device
                .acquire_next_image_khr(
                    self.data.swapchain,
                    u64::MAX,
                    self.data.image_available_semaphore[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize
        };

        let image_fence = self.data.image_in_flight[image_index];

        self.update_uniform_buffer(image_index);

        if !image_fence.is_null() {
            unsafe {
                self.device
                    .wait_for_fences(&[image_fence], true, u64::MAX)?;
            }
        };

        self.data.image_in_flight[image_index] = in_flight_fence;

        let wait_semaphores = &[self.data.image_available_semaphore[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphore[image_index as usize]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe { self.device.reset_fences(&[in_flight_fence])? };

        unsafe {
            self.device
                .queue_submit(self.data.graphics_queue, &[submit_info], in_flight_fence)?
        };

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            self.device
                .queue_present_khr(self.data.present_queue, &present_info)?
        };

        self.frame = (self.frame + 1) % self.data.swapchain_images.len();

        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device
                .free_command_buffers(self.data.command_pool, &self.data.command_buffers);

            self.data
                .uniform_buffers
                .iter()
                .for_each(|b| self.device.destroy_buffer(*b, None));
            self.data
                .uniform_buffers_memory
                .iter()
                .for_each(|m| self.device.free_memory(*m, None));

            self.device
                .destroy_image_view(self.data.texture_image_view, None);

            self.device.destroy_image(self.data.texture_image, None);

            self.device
                .free_memory(self.data.texture_image_memory, None);

            self.device.destroy_buffer(self.data.vertex_buffer, None);
            self.device
                .free_memory(self.data.vertex_buffer_memory, None);
            self.device.destroy_buffer(self.data.index_buffer, None);
            self.device.free_memory(self.data.index_buffer_memory, None);

            self.device
                .destroy_sampler(self.data.texture_image_sampler, None);
            self.device
                .free_memory(self.data.texture_image_memory, None);
            self.device
                .destroy_command_pool(self.data.command_pool, None);
            self.data.swapchain_image_view.iter().for_each(|i| {
                self.device.destroy_image_view(*i, None);
            });
            self.data
                .frame_buffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));
            self.device
                .destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);
            self.device.destroy_pipeline(self.data.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.data.pipeline_layout, None);
            self.device.destroy_render_pass(self.data.render_pass, None);
            self.device.destroy_swapchain_khr(self.data.swapchain, None);
            self.data
                .image_available_semaphore
                .iter()
                .for_each(|f| self.device.destroy_semaphore(*f, None));

            self.data
                .render_finished_semaphore
                .iter()
                .for_each(|f| self.device.destroy_semaphore(*f, None));

            self.data
                .in_flight_fences
                .iter()
                .for_each(|f| self.device.destroy_fence(*f, None));

            self.device.destroy_buffer(self.data.vertex_buffer, None);
            self.device
                .free_memory(self.data.vertex_buffer_memory, None);

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }

    fn update_uniform_buffer(&mut self, image_index: usize) -> Result<()> {
        let i_time = self.start.elapsed().as_secs_f32();
        let i_resolution = vec2(
            self.data.swapchain_extent.width as f32,
            self.data.swapchain_extent.height as f32,
        );

        let ubo = UniformBufferObject {
            i_time,
            _padding: 0.0,
            i_resolution,
        };

        let memory = unsafe {
            self.device.map_memory(
                self.data.uniform_buffers_memory[image_index],
                0,
                size_of::<UniformBufferObject>() as u64,
                vk::MemoryMapFlags::empty(),
            )?
        };

        unsafe {
            memcpy(&ubo, memory.cast(), 1);

            self.device
                .unmap_memory(self.data.uniform_buffers_memory[image_index]);
        }

        Ok(())
    }
}

///
/// Create Surface
///
fn create_surface(
    instance: &Instance,
    data: &mut ContextData,
    surface: *mut c_void,
    display: *mut c_void,
) -> Result<()> {
    let info = vk::WaylandSurfaceCreateInfoKHR::builder()
        .surface(surface)
        .display(display);

    data.surface = unsafe { instance.create_wayland_surface_khr(&info, None)? };

    Ok(())
}

//
//Create Instance
//
fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(b"light paper")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers = unsafe { entry.enumerate_instance_layer_properties()? }
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let extension = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|p| p.as_ptr())
        .collect::<Vec<_>>();

    let instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extension);

    Ok(unsafe { entry.create_instance(&instance_info, None)? })
}
fn create_instance_wayland(entry: &Entry) -> Result<Instance> {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(b"light paper")
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(b"No Engine")
        .engine_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let available_layers = unsafe { entry.enumerate_instance_layer_properties()? }
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let extension = &[
        &vk::KHR_SURFACE_EXTENSION.name,
        &vk::KHR_WAYLAND_SURFACE_EXTENSION.name,
    ]
    .iter()
    .map(|p| p.as_ptr())
    .collect::<Vec<_>>();

    let instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extension);

    Ok(unsafe { entry.create_instance(&instance_info, None)? })
}

//
// Create PhysicalDevice
//
fn pick_physical_device(instance: &Instance, data: &mut ContextData) {
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

    data.physical_device = physical_devices[0];

    //for physical_device in physical_devices {
    //    let propertics = unsafe { instance.get_physical_device_properties(physical_device) };

    //    println!("{}", propertics.device_name);

    //    if let Err(error) = check_physical_device(instance, physical_device) {
    //        continue;
    //    } else {
    //        data.physical_device = physical_device;
    //    }
    //}
}

fn check_physical_device(
    instance: &Instance,
    physical_device: PhysicalDevice,
    data: &ContextData,
) -> Result<()> {
    QueueFamilyindices::get(instance, physical_device, data.surface)?;
    check_physical_device_extensions(instance, physical_device)?;
    let support = SwapChainSupport::get(instance, physical_device, data.surface)?;

    if support.formats.is_empty() || support.present_modes.is_empty() {
        return Err(anyhow!("Do not support swapchain"));
    }

    let features = unsafe { instance.get_physical_device_features(physical_device) };

    if features.sampler_anisotropy != vk::TRUE {
        return Err(anyhow!("do not support anisotropy"));
    }

    Ok(())
}

fn check_physical_device_extensions(
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
fn create_logical_device(instance: &Instance, data: &mut ContextData) -> Result<Device> {
    let indices = QueueFamilyindices::get(instance, data.physical_device, data.surface)?;
    let mut unique_indices = HashSet::new();
    unique_indices.insert(indices.graphics);
    unique_indices.insert(indices.present);

    // QueueFamily
    let queue_infos = unique_indices
        .iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::builder()
                .queue_priorities(&[1.0])
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

    let device = unsafe { instance.create_device(data.physical_device, &device_info, None)? };

    data.graphics_queue = unsafe { device.get_device_queue(indices.graphics, 0) };
    data.present_queue = unsafe { device.get_device_queue(indices.present, 0) };

    Ok(device)
}

//
// Create SwapChain
//
fn create_swapchain(
    window: &Window,
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    let indices = QueueFamilyindices::get(instance, data.physical_device, data.surface)?;
    let support = SwapChainSupport::get(instance, data.physical_device, data.surface)?;

    // surface format
    let surface_format = get_swapchain_surface_format(&support.formats);
    // present mode
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    // extent 1920*1080
    let extent2d = get_swapchain_extent(window, support.capabilities);

    // image count
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
        // set surface
        .surface(data.surface)
        // image setting
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent2d)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        // image sharing settings
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        // transforms setting
        .pre_transform(support.capabilities.current_transform)
        // composite_alpha
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        // present_mode setting
        .present_mode(present_mode)
        .clipped(true)
        //old swapchain
        .old_swapchain(vk::SwapchainKHR::null());

    data.swapchain = unsafe { device.create_swapchain_khr(&swapchain_info, None)? };
    data.swapchain_images = unsafe { device.get_swapchain_images_khr(data.swapchain)? };
    data.swapchain_format = surface_format.format;
    data.swapchain_extent = extent2d;

    Ok(())
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

/// Create SwapChain
fn create_swapchain_wayland(
    width: u32,
    height: u32,
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
) -> Result<()> {
    let indices = QueueFamilyindices::get(instance, data.physical_device, data.surface)?;
    let support = SwapChainSupport::get(instance, data.physical_device, data.surface)?;

    // surface format
    let surface_format = get_swapchain_surface_format(&support.formats);
    // present mode
    let present_mode = get_swapchain_present_mode(&support.present_modes);
    // extent 1920*1080
    let extent2d = get_swapchain_extent_wayland(width, height, support.capabilities);

    // image count
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
        // set surface
        .surface(data.surface)
        // image setting
        .min_image_count(image_count)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .image_extent(extent2d)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        // image sharing settings
        .image_sharing_mode(image_sharing_mode)
        .queue_family_indices(&queue_family_indices)
        // transforms setting
        .pre_transform(support.capabilities.current_transform)
        // composite_alpha
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        // present_mode setting
        .present_mode(present_mode)
        .clipped(true)
        //old swapchain
        .old_swapchain(vk::SwapchainKHR::null());

    data.swapchain = unsafe { device.create_swapchain_khr(&swapchain_info, None)? };
    data.swapchain_images = unsafe { device.get_swapchain_images_khr(data.swapchain)? };
    data.swapchain_format = surface_format.format;
    data.swapchain_extent = extent2d;

    Ok(())
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

///
/// Create Image View
///
fn create_swapchain_image_view(device: &Device, data: &mut ContextData) -> Result<()> {
    data.swapchain_image_view = data
        .swapchain_images
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
                .format(data.swapchain_format)
                .components(components)
                .subresource_range(subresource_range);

            match unsafe { device.create_image_view(&image_info, None) } {
                Ok(t) => t,
                #[allow(unused_variables)]
                Err(e) => ImageView::null(),
            }
        })
        .collect::<Vec<_>>();
    Ok(())
}

///
/// Create Pipeline
///

fn create_pipeline(device: &Device, data: &mut ContextData) -> Result<()> {
    let vert = include_bytes!("../../shader/vert.spv");
    let frag = include_bytes!("../../shader/frag.spv");

    let vert_mode = create_shader_module(device, &vert[..])?;
    let frag_mode = create_shader_module(device, &frag[..])?;

    let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_mode)
        .name(b"main\0");

    let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_mode)
        .name(b"main\0");

    // vertex input
    let binding_descriptions = &[Vertex::binding_description()];
    let attribute_descriptions = Vertex::attribute_description();
    let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
        .vertex_binding_descriptions(binding_descriptions)
        .vertex_attribute_descriptions(&attribute_descriptions);

    // assembly input
    let assembly_input_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);

    // view port state 1. viewport 2. scissor
    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(data.swapchain_extent.width as f32)
        .height(data.swapchain_extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(data.swapchain_extent);

    let viewports = &[viewport];
    let scissors = &[scissor];

    let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(viewports)
        .scissors(scissors);

    let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .line_width(1.0);

    let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(data.msaa_samples)
        .sample_shading_enable(true)
        .min_sample_shading(0.2);

    let attachment = vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(vk::ColorComponentFlags::all())
        .blend_enable(false);

    let attachments = &[attachment];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0]);

    let set_layouts = &[data.descriptor_set_layout];
    let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(set_layouts);

    data.pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

    let stages = &[vert_stage, frag_stage];

    let info = vk::GraphicsPipelineCreateInfo::builder()
        .stages(stages)
        .vertex_input_state(&vertex_input_state)
        .input_assembly_state(&assembly_input_state)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterization_state)
        .color_blend_state(&color_blend_state)
        .multisample_state(&multisample_state)
        .layout(data.pipeline_layout)
        .render_pass(data.render_pass)
        .subpass(0);

    data.pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?
            .0[0]
    };

    unsafe {
        device.destroy_shader_module(vert_mode, None);
        device.destroy_shader_module(frag_mode, None);
    }

    Ok(())
}

fn create_frame_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
    data.frame_buffers = data
        .swapchain_image_view
        .iter()
        .map(|i| {
            let attachments = &[data.color_image_view, *i];
            let info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.render_pass)
                .attachments(attachments)
                .width(data.swapchain_extent.width)
                .height(data.swapchain_extent.height)
                .layers(1);

            //unsafe { device.create_framebuffer(&info, None).unwrap() }
            match unsafe { device.create_framebuffer(&info, None) } {
                Ok(f) => f,
                Err(e) => Framebuffer::null(),
            }
        })
        .collect::<Vec<_>>();
    Ok(())
}

fn create_shader_module(device: &Device, bytecode: &[u8]) -> Result<vk::ShaderModule> {
    let bytecode = Bytecode::new(bytecode).unwrap();

    let info = vk::ShaderModuleCreateInfo::builder()
        .code(bytecode.code())
        .code_size(bytecode.code_size());

    Ok(unsafe { device.create_shader_module(&info, None)? })
}

///
/// Create Render Pass
///
fn create_render_pass(device: &Device, data: &mut ContextData) -> Result<()> {
    let attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(data.msaa_samples)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment = vk::AttachmentDescription::builder()
        .format(data.swapchain_format)
        .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::DONT_CARE)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let color_resolve_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let attachmnets = &[color_attachment_ref];
    let resolve_attachments = &[color_resolve_attachment_ref];

    let subpass = vk::SubpassDescription::builder()
        .color_attachments(attachmnets)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .resolve_attachments(resolve_attachments);

    let attachments = &[attachment, color_resolve_attachment];

    let subpasses = &[subpass];

    let render_info = vk::RenderPassCreateInfo::builder()
        .attachments(attachments)
        .subpasses(subpasses);
    data.render_pass = unsafe { device.create_render_pass(&render_info, None)? };

    Ok(())
}

///
/// Create Command pool
///
fn create_command_pool(instance: &Instance, device: &Device, data: &mut ContextData) -> Result<()> {
    let indices = QueueFamilyindices::get(instance, data.physical_device, data.surface)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty())
        .queue_family_index(indices.graphics);

    data.command_pool = unsafe { device.create_command_pool(&info, None)? };
    Ok(())
}

///
/// Create command buffer
///
fn create_command_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
    let info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.frame_buffers.len() as u32);
    data.command_buffers = unsafe { device.allocate_command_buffers(&info)? };

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let info = vk::CommandBufferBeginInfo::builder();

        unsafe { device.begin_command_buffer(*command_buffer, &info)? };

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.frame_buffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        unsafe {
            device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                data.pipeline,
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
                data.pipeline_layout,
                0,
                &[data.descriptor_sets[i]],
                &[],
            );

            device.cmd_draw_indexed(*command_buffer, vertex::INDICES.len() as u32, 1, 0, 0, 0);
            device.cmd_end_render_pass(*command_buffer);

            device.end_command_buffer(*command_buffer)?;
        }
    }

    Ok(())
}

///
/// Create Vertex Buffer
///
fn create_vertex_buffer(
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
fn create_index_buffer(instance: &Instance, device: &Device, data: &mut ContextData) -> Result<()> {
    let size = (size_of::<u16>() * vertex::INDICES.len()) as u64;
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
        memcpy(
            vertex::INDICES.as_ptr(),
            memory.cast(),
            vertex::INDICES.len(),
        );

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
        .command_pool(data.command_pool)
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
        device.queue_submit(data.graphics_queue, &[info], vk::Fence::null())?;
        device.queue_wait_idle(data.graphics_queue)?;

        // Cleanup

        device.free_command_buffers(data.command_pool, &[command_buffer]);
    }

    Ok(())
}

///
/// Create descriptor
///
fn create_descriptor_set_layout(device: &Device, data: &mut ContextData) -> Result<()> {
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
    let info = DescriptorSetLayoutCreateInfo::builder().bindings(bindings);

    data.descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&info, None)? };

    Ok(())
}

fn create_descriptor_pool(device: &Device, data: &mut ContextData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let sampler_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size, sampler_size];

    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32);

    data.descriptor_pool = unsafe { device.create_descriptor_pool(&info, None)? };

    Ok(())
}

fn create_descriptor_sets(device: &Device, data: &mut ContextData) -> Result<()> {
    let layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];
    let allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);
    data.descriptor_sets = unsafe { device.allocate_descriptor_sets(&allocate_info)? };

    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(data.texture_image_view)
            .sampler(data.texture_image_sampler);

        let image_infos = &[image_info];

        let sampler_set = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(1)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);
        unsafe {
            device
                .update_descriptor_sets(&[ubo_write, sampler_set], &[] as &[vk::CopyDescriptorSet]);
        }
    }

    Ok(())
}

fn create_sync_objects(device: &Device, data: &mut ContextData) -> Result<()> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

    for _ in 0..data.swapchain_images.len() {
        data.image_available_semaphore
            .push(unsafe { device.create_semaphore(&semaphore_info, None)? });

        data.render_finished_semaphore
            .push(unsafe { device.create_semaphore(&semaphore_info, None)? });

        data.in_flight_fences
            .push(unsafe { device.create_fence(&fence_info, None)? });
    }

    data.image_in_flight = data
        .swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok(())
}

///
/// QueueFamilyindices
///
struct QueueFamilyindices {
    graphics: u32,
    present: u32,
}

impl QueueFamilyindices {
    fn get(
        instance: &Instance,
        physical_device: PhysicalDevice,
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
struct SwapChainSupport {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupport {
    fn get(
        instance: &Instance,
        physical_device: PhysicalDevice,
        surface: SurfaceKHR,
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
