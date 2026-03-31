use crate::context::ContextData;
use crate::context::mipmap;
use crate::context::tool;

use anyhow::{Result, anyhow};
use image::DynamicImage;
use image::ImageReader;

use vulkanalia::Device;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::ImageLayout;

use std::ptr::copy_nonoverlapping as memcpy;

pub fn read_image(path: &str) -> Result<DynamicImage> {
    let image = ImageReader::open(path)?.decode()?;
    Ok(image)
}

///
/// Create Texture Image
///
pub fn create_texture_image(
    instance: &Instance,
    device: &Device,
    data: &mut ContextData,
    image: &DynamicImage,
) -> Result<()> {
    let image_rgba = image.to_rgba8();

    let (width, height) = image_rgba.dimensions();

    data.mip_levels = (width.max(height) as f32).log2().floor() as u32 + 1;

    let pixels = image_rgba.as_raw();

    let size = pixels.len() as u64;

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
        memcpy(pixels.as_ptr(), memory.cast(), pixels.len());
        device.unmap_memory(staging_buffer_memory);
    };

    let (texture_image, texture_image_momery) = tool::create_image(
        instance,
        device,
        data,
        width,
        height,
        data.mip_levels,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::SampleCountFlags::_1,
        vk::ImageUsageFlags::SAMPLED
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    data.texture_image = texture_image;
    data.texture_image_memory = texture_image_momery;

    transition_image_layout(
        device,
        data,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        data.texture_image,
        data.mip_levels,
    )?;

    copy_buffer_to_image(
        device,
        data,
        staging_buffer,
        data.texture_image,
        width,
        height,
    )?;

    unsafe {
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);
    }

    mipmap::generate_mipmaps(
        instance,
        device,
        data,
        data.texture_image,
        vk::Format::R8G8B8A8_SRGB,
        width,
        height,
        data.mip_levels,
    )?;

    Ok(())
}

fn transition_image_layout(
    device: &Device,
    data: &ContextData,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    image: vk::Image,
    mip_levels: u32,
) -> Result<()> {
    let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
        match (old_layout, new_layout) {
            (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vk::AccessFlags::empty(),
                vk::AccessFlags::TRANSFER_WRITE,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            ),
            (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            _ => return Err(anyhow!("Unsupported image layout transition!")),
        };

    let command_buffer = tool::begin_single_time_commands(device, data)?;

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[barrier],
        );
    }

    tool::end_single_time_commands(device, data, command_buffer)?;

    Ok(())
}

fn copy_buffer_to_image(
    device: &Device,
    data: &ContextData,
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
    width: u32,
    height: u32,
) -> Result<()> {
    let command_buffer = tool::begin_single_time_commands(device, data)?;

    let image_subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(image_subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
            width: width,
            height: height,
            depth: 1,
        });

    unsafe {
        device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    tool::end_single_time_commands(device, data, command_buffer)?;
    Ok(())
}

///
/// Create Texture ImageView
///
pub fn create_texture_image_view(device: &Device, data: &mut ContextData) -> Result<()> {
    data.texture_image_view = tool::create_image_view(
        device,
        data.texture_image,
        vk::Format::R8G8B8A8_SRGB,
        data.mip_levels,
    )?;
    Ok(())
}

///
/// Create Texture Sampler
///
pub fn create_texture_sampler(device: &Device, data: &mut ContextData) -> Result<()> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.0)
        .max_lod(data.mip_levels as f32)
        .mip_lod_bias(0.0);

    data.texture_image_sampler = unsafe { device.create_sampler(&info, None)? };
    Ok(())
}
