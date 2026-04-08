#![allow(unused)]
use std::collections::HashSet;
use std::ffi::c_void;
use std::ptr::copy_nonoverlapping as memcpy;
use std::time::Instant;

use cgmath::vec2;
use vulkanalia::vk::SampleCountFlags;

use crate::context::DescriptorManager;
use crate::context::DeviceManager;
use crate::context::DeviceQueue;
use crate::context::Pipeline;
use crate::context::Swapchain;
use crate::context::SyncObjects;
use crate::context::UniformBufferObject;
use crate::context::Vertex;
use crate::context::command::CommandManager;
use crate::context::instance;
use crate::context::msaa;
use crate::context::texture;
use crate::context::tool;
use crate::context::uniform;
use crate::context::vertex;

use image::DynamicImage;
use vertex::VERTICES;

use anyhow::{Result, anyhow};
use vulkanalia::Device;
use vulkanalia::Entry;
use vulkanalia::Instance;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::loader::LIBRARY;
use vulkanalia::loader::LibloadingLoader;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::EntryV1_0;
use vulkanalia::vk::Framebuffer;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::ImageView;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::KhrSwapchainExtensionDeviceCommands;

use vulkanalia::vk::KhrWaylandSurfaceExtensionInstanceCommands;

use vulkanalia::vk::PhysicalDevice;
use vulkanalia::vk::SurfaceKHR;
use vulkanalia::window as vk_window;
use winit::window::Window;

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
    pub(crate) device_manager: DeviceManager,
    pub(crate) device_queue: DeviceQueue,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) swapchain: Swapchain,
    pub(crate) pipeline: Pipeline,
    pub(crate) descriptor_manager: DescriptorManager,
    pub(crate) sync_objects: SyncObjects,
    pub(crate) command_manager: CommandManager,

    pub(crate) frame_buffers: Vec<vk::Framebuffer>,

    pub(crate) vertex_buffer: vk::Buffer,
    pub(crate) vertex_buffer_memory: vk::DeviceMemory,

    pub(crate) index_buffer: vk::Buffer,
    pub(crate) index_buffer_memory: vk::DeviceMemory,

    pub(crate) uniform_buffers: Vec<vk::Buffer>,
    pub(crate) uniform_buffers_memory: Vec<vk::DeviceMemory>,

    pub(crate) texture_image: vk::Image,
    pub(crate) texture_image_memory: vk::DeviceMemory,
    pub(crate) texture_image_view: vk::ImageView,
    pub(crate) texture_image_sampler: vk::Sampler,
    pub(crate) mip_levels: u32,

    // Msaa
    pub(crate) color_image: vk::Image,
    pub(crate) color_image_memory: vk::DeviceMemory,
    pub(crate) color_image_view: vk::ImageView,
    pub(crate) msaa_samples: vk::SampleCountFlags,
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

        let instance = instance::create_instance_wayland(&entry)?;
        let image = texture::read_image("assets/wallhaven-3q3wj3.jpg")?;

        let mut data = ContextData::default();

        create_surface(&instance, &mut data, surface, display)?;

        data.device_manager = DeviceManager::create(&instance, data.surface)?;
        let (device, device_queue) =
            crate::context::device::create_logical_device(&instance, &data.device_manager)?;
        data.device_queue = device_queue;

        // Wallpaper is 2D full-screen image; MSAA only blurs during resolve.
        data.msaa_samples = vk::SampleCountFlags::_1;

        // Create swapchain
        data.swapchain = Swapchain::create_for_wayland(
            width,
            height,
            &instance,
            &device,
            &data.device_manager,
            data.surface,
        )?;

        // Create descriptor manager
        data.descriptor_manager = DescriptorManager::create(&device, data.swapchain.images.len())?;

        // Create pipeline
        let vert_shader = include_bytes!("../../shader/vert.spv");
        let frag_shader = include_bytes!("../../shader/frag1.spv");
        data.pipeline = Pipeline::create(
            &device,
            data.swapchain.format,
            data.swapchain.extent,
            data.msaa_samples,
            data.descriptor_manager.layout,
            vert_shader,
            frag_shader,
        )?;

        // Create command manager
        data.command_manager = CommandManager::create(&instance, &device, &mut data)?;
        data.command_manager
            .allocate_buffers(&device, data.swapchain.images.len() as u32)?;

        // Create MSAA color objects (if needed)
        msaa::create_color_objects(&instance, &device, &mut data)?;

        // Create frame buffers
        create_frame_buffers(&device, &mut data)?;

        // Create texture
        texture::create_texture_image(&instance, &device, &mut data, &image)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        // Create vertex and index buffers
        vertex::create_vertex_buffer(&instance, &device, &mut data)?;
        vertex::create_index_buffer(&instance, &device, &mut data)?;

        // Create uniform buffers
        uniform::create_uniform_buffers(&instance, &device, &mut data)?;

        // Update descriptor sets
        data.descriptor_manager.update(
            &device,
            &data.uniform_buffers,
            data.texture_image_view,
            data.texture_image_sampler,
        );

        // Create sync objects
        data.sync_objects = SyncObjects::create(&device, data.swapchain.images.len())?;

        // Record command buffers
        record_command_buffers(&device, &mut data)?;

        Ok(Self {
            instance,
            data,
            device,
            frame: 0,
            start: Instant::now(),
            image,
        })
    }

    pub fn create(window: &Window) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!(b))? };

        let instance = instance::create_instance(window, &entry)?;
        let image = texture::read_image("assets/wallhaven-3q3wj3.jpg")?;

        let mut data = ContextData::default();

        data.surface = unsafe { vk_window::create_surface(&instance, window, window)? };

        data.device_manager = DeviceManager::create(&instance, data.surface)?;
        let (device, device_queue) =
            crate::context::device::create_logical_device(&instance, &data.device_manager)?;
        data.device_queue = device_queue;

        // Create swapchain
        data.swapchain = Swapchain::create_for_winit(
            window,
            &instance,
            &device,
            &data.device_manager,
            data.surface,
        )?;

        // Create descriptor manager
        data.descriptor_manager = DescriptorManager::create(&device, data.swapchain.images.len())?;

        // Create pipeline
        let vert_shader = include_bytes!("../../shader/vert.spv");
        let frag_shader = include_bytes!("../../shader/frag1.spv");
        data.pipeline = Pipeline::create(
            &device,
            data.swapchain.format,
            data.swapchain.extent,
            data.msaa_samples,
            data.descriptor_manager.layout,
            vert_shader,
            frag_shader,
        )?;

        // Create command manager
        data.command_manager = CommandManager::create(&instance, &device, &mut data)?;
        data.command_manager
            .allocate_buffers(&device, data.swapchain.images.len() as u32)?;

        // Create frame buffers
        create_frame_buffers(&device, &mut data)?;

        // Create texture
        texture::create_texture_image(&instance, &device, &mut data, &image)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        // Create vertex and index buffers
        vertex::create_vertex_buffer(&instance, &device, &mut data)?;
        vertex::create_index_buffer(&instance, &device, &mut data)?;

        // Create uniform buffers
        uniform::create_uniform_buffers(&instance, &device, &mut data)?;

        // Update descriptor sets
        data.descriptor_manager.update(
            &device,
            &data.uniform_buffers,
            data.texture_image_view,
            data.texture_image_sampler,
        );

        // Create sync objects
        data.sync_objects = SyncObjects::create(&device, data.swapchain.images.len())?;

        // Record command buffers
        record_command_buffers(&device, &mut data)?;

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
        let in_flight_fence = self.data.sync_objects.in_flight_fences[self.frame];
        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        };

        let image_index = unsafe {
            self.device
                .acquire_next_image_khr(
                    self.data.swapchain.swapchain,
                    u64::MAX,
                    self.data.sync_objects.image_available[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize
        };

        let image_fence = self.data.sync_objects.images_in_flight[image_index];

        if !image_fence.is_null() {
            unsafe {
                self.device
                    .wait_for_fences(&[image_fence], true, u64::MAX)?;
            }
        };

        self.data.sync_objects.images_in_flight[image_index] = in_flight_fence;

        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.data.sync_objects.image_available[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_manager.buffers[image_index as usize]];
        let signal_semaphores = &[self.data.sync_objects.render_finished[image_index as usize]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe { self.device.reset_fences(&[in_flight_fence])? };

        unsafe {
            self.device.queue_submit(
                self.data.device_queue.graphics_queue,
                &[submit_info],
                in_flight_fence,
            )?
        };

        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            self.device
                .queue_present_khr(self.data.device_queue.present_queue, &present_info)?
        };

        self.frame = (self.frame + 1) % self.data.swapchain.images.len();

        Ok(())
    }

    pub fn render(&mut self, window: &Window) -> Result<()> {
        let in_flight_fence = self.data.sync_objects.in_flight_fences[self.frame];
        unsafe {
            self.device
                .wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
        };

        let image_index = unsafe {
            self.device
                .acquire_next_image_khr(
                    self.data.swapchain.swapchain,
                    u64::MAX,
                    self.data.sync_objects.image_available[self.frame],
                    vk::Fence::null(),
                )?
                .0 as usize
        };

        let image_fence = self.data.sync_objects.images_in_flight[image_index];

        self.update_uniform_buffer(image_index);

        if !image_fence.is_null() {
            unsafe {
                self.device
                    .wait_for_fences(&[image_fence], true, u64::MAX)?;
            }
        };

        self.data.sync_objects.images_in_flight[image_index] = in_flight_fence;

        let wait_semaphores = &[self.data.sync_objects.image_available[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_manager.buffers[image_index as usize]];
        let signal_semaphores = &[self.data.sync_objects.render_finished[image_index as usize]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);

        unsafe { self.device.reset_fences(&[in_flight_fence])? };

        unsafe {
            self.device.queue_submit(
                self.data.device_queue.graphics_queue,
                &[submit_info],
                in_flight_fence,
            )?
        };

        let swapchains = &[self.data.swapchain.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        unsafe {
            self.device
                .queue_present_khr(self.data.device_queue.present_queue, &present_info)?
        };

        self.frame = (self.frame + 1) % self.data.swapchain.images.len();

        Ok(())
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.data.sync_objects.destroy(&self.device);
            self.data.descriptor_manager.destroy(&self.device);
            self.data.pipeline.destroy(&self.device);
            self.data.command_manager.destroy(&self.device);

            for framebuffer in &self.data.frame_buffers {
                self.device.destroy_framebuffer(*framebuffer, None);
            }

            self.data.swapchain.destroy(&self.device);

            self.device.destroy_buffer(self.data.vertex_buffer, None);
            self.device
                .free_memory(self.data.vertex_buffer_memory, None);
            self.device.destroy_buffer(self.data.index_buffer, None);
            self.device.free_memory(self.data.index_buffer_memory, None);

            for (buffer, memory) in self
                .data
                .uniform_buffers
                .iter()
                .zip(&self.data.uniform_buffers_memory)
            {
                self.device.destroy_buffer(*buffer, None);
                self.device.free_memory(*memory, None);
            }

            self.device
                .destroy_image_view(self.data.texture_image_view, None);
            self.device.destroy_image(self.data.texture_image, None);
            self.device
                .free_memory(self.data.texture_image_memory, None);
            self.device
                .destroy_sampler(self.data.texture_image_sampler, None);

            if self.data.msaa_samples != vk::SampleCountFlags::_1 {
                self.device
                    .destroy_image_view(self.data.color_image_view, None);
                self.device.destroy_image(self.data.color_image, None);
                self.device.free_memory(self.data.color_image_memory, None);
            }

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }

    fn update_uniform_buffer(&mut self, image_index: usize) -> Result<()> {
        let i_time = self.start.elapsed().as_secs_f32();
        let i_resolution = vec2(
            self.data.swapchain.extent.width as f32,
            self.data.swapchain.extent.height as f32,
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
                std::mem::size_of::<UniformBufferObject>() as u64,
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

fn create_frame_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
    data.frame_buffers = data
        .swapchain
        .image_views
        .iter()
        .map(|image_view| {
            let attachments = if data.msaa_samples != vk::SampleCountFlags::_1 {
                vec![data.color_image_view, *image_view]
            } else {
                vec![*image_view]
            };

            let info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.pipeline.render_pass)
                .attachments(&attachments)
                .width(data.swapchain.extent.width)
                .height(data.swapchain.extent.height)
                .layers(1);

            unsafe { device.create_framebuffer(&info, None).unwrap() }
        })
        .collect::<Vec<_>>();
    Ok(())
}

fn record_command_buffers(device: &Device, data: &mut ContextData) -> Result<()> {
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
