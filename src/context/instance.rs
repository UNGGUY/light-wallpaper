use anyhow::Result;
use anyhow::anyhow;
use std::collections::HashSet;
use vulkanalia::Entry;
use vulkanalia::Instance;
use vulkanalia::vk;
use vulkanalia::vk::EntryV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::window as vk_window;
use winit::window::Window;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

//
//Create Instance
//
pub fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
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
pub fn create_instance_wayland(entry: &Entry) -> Result<Instance> {
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
