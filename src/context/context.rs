#![allow(unused)]
use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::path::Path;
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
use crate::context::command::{self, CommandManager};
use crate::context::frame;
use crate::context::instance;
use crate::context::msaa;
use crate::context::swapchain;
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

    // Double buffering for wallpaper switching (Intel GPU workaround)
    pub(crate) texture_image_alt: vk::Image,
    pub(crate) texture_image_alt_memory: vk::DeviceMemory,
    pub(crate) texture_image_alt_view: vk::ImageView,
    pub(crate) used_alt_texture: bool,
    pub(crate) texture_width: u32,
    pub(crate) texture_height: u32,

    // Msaa
    pub(crate) color_image: vk::Image,
    pub(crate) color_image_memory: vk::DeviceMemory,
    pub(crate) color_image_view: vk::ImageView,
    pub(crate) msaa_samples: vk::SampleCountFlags,

    // Cached memory type index for host-visible memory (used by reload_texture)
    pub(crate) host_visible_memory_type: Option<u32>,

    // Dedicated command buffer for texture uploads (Intel GPU workaround)
    pub(crate) upload_command_buffer: vk::CommandBuffer,
}

impl Context {
    pub fn create_for_wayland(
        surface: *mut c_void,
        display: *mut c_void,
        width: u32,
        height: u32,
        new_path: &std::path::Path,
    ) -> Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!(b))? };

        let instance = instance::create_instance_wayland(&entry)?;
        let image = texture::read_image(new_path.to_str().unwrap())?;

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
        let frag_shader = include_bytes!("../../shader/frag3.spv");
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
        //msaa::create_color_objects(&instance, &device, &mut data)?;
        //
        data.texture_width = width;
        data.texture_height = height;

        // Create frame buffers
        frame::create_frame_buffers(&device, &mut data)?;

        // Create texture (double buffering for Intel GPU workaround)
        texture::create_texture_image(&instance, &device, &mut data, &image)?;
        texture::create_texture_image_view(&device, &mut data)?;
        texture::create_texture_sampler(&device, &mut data)?;

        // Create alternate texture for wallpaper switching
        texture::create_alt_texture_image(&instance, &device, &mut data)?;
        texture::create_alt_texture_image_view(&device, &mut data)?;
        data.used_alt_texture = false;

        // Store texture dimensions for resizing during wallpaper switch

        // Allocate dedicated command buffer for texture uploads (Intel GPU workaround)
        let upload_cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(data.command_manager.pool)
            .command_buffer_count(1);
        data.upload_command_buffer =
            unsafe { device.allocate_command_buffers(&upload_cmd_alloc_info)?[0] };

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
        CommandManager::record_command_buffers(&device, &mut data, 0.0)?;

        Ok(Self {
            instance,
            data,
            device,
            frame: 0,
            start: Instant::now(),
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

    /// 运行时重新加载纹理（双缓冲方案 - Intel GPU workaround）
    pub fn reload_texture(&mut self, new_path: &std::path::Path) -> Result<()> {
        // 1. 等待 GPU 完全空闲
        unsafe { self.device.device_wait_idle()? };

        // 2. 读取新图片
        let new_image = texture::read_image(new_path.to_str().unwrap())?;

        // 3. 确定目标纹理
        let target_image = if self.data.used_alt_texture {
            self.data.texture_image
        } else {
            self.data.texture_image_alt
        };

        // 4. 上传数据
        texture::upload_to_texture(
            &self.instance,
            &self.device,
            &mut self.data,
            &new_image,
            target_image,
        )?;

        Ok(())
    }

    pub fn switch(&mut self, progress: f32, first: bool) -> Result<()> {
        // 5. 再次等待上传完成
        unsafe { self.device.device_wait_idle()? };

        // 7. 选择新的纹理视图
        let (image_view, old_image_view) = if self.data.used_alt_texture {
            (
                self.data.texture_image_view,
                self.data.texture_image_alt_view,
            )
        } else {
            (
                self.data.texture_image_alt_view,
                self.data.texture_image_view,
            )
        };

        if progress >= 1.0 {
            println!("finish");
            if self.data.used_alt_texture {
                println!("true");
            } else {
                println!("false");
            }
            // 8. 逐个更新描述符集（避免生命周期问题）
            for set in &self.data.descriptor_manager.sets {
                let image_info = vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(image_view)
                    .sampler(self.data.texture_image_sampler)
                    .build();
                let image_info1 = vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(image_view)
                    .sampler(self.data.texture_image_sampler)
                    .build();

                let image_infos = [image_info, image_info1];

                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(*set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_infos)
                    .build();

                unsafe {
                    self.device
                        .update_descriptor_sets(&[write], &[] as &[vk::CopyDescriptorSet]);
                }
            }
            self.data.used_alt_texture = !self.data.used_alt_texture;
        }
        if first {
            println!("first");
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(old_image_view)
                .sampler(self.data.texture_image_sampler)
                .build();

            let image_info1 = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(image_view)
                .sampler(self.data.texture_image_sampler)
                .build();

            let image_infos = [image_info, image_info1];
            // 8. 逐个更新描述符集（避免生命周期问题）
            for set in &self.data.descriptor_manager.sets {
                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(*set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_infos)
                    .build();

                unsafe {
                    self.device
                        .update_descriptor_sets(&[write], &[] as &[vk::CopyDescriptorSet]);
                }
            }
        }

        CommandManager::record_command_buffers(&self.device, &mut self.data, progress)?;
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

            // 清理主纹理
            if !self.data.texture_image_view.is_null() {
                self.device
                    .destroy_image_view(self.data.texture_image_view, None);
            }
            if !self.data.texture_image.is_null() {
                self.device.destroy_image(self.data.texture_image, None);
            }
            if !self.data.texture_image_memory.is_null() {
                self.device
                    .free_memory(self.data.texture_image_memory, None);
            }
            // 清理备用纹理
            if !self.data.texture_image_alt_view.is_null() {
                self.device
                    .destroy_image_view(self.data.texture_image_alt_view, None);
            }
            if !self.data.texture_image_alt.is_null() {
                self.device.destroy_image(self.data.texture_image_alt, None);
            }
            if !self.data.texture_image_alt_memory.is_null() {
                self.device
                    .free_memory(self.data.texture_image_alt_memory, None);
            }
            if !self.data.texture_image_sampler.is_null() {
                self.device
                    .destroy_sampler(self.data.texture_image_sampler, None);
            }

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
