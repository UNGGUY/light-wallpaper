# CLAUDE.md

This file provides guidance to Claude Code (kimi.ai/code) when working with code in this repository.

## Project Overview

`light-paper` is a Wayland wallpaper engine written in Rust. It renders wallpapers using Vulkan (via the `vulkanalia` crate) and attaches to outputs as a background layer via `wlr-layer-shell-unstable-v1`. It currently supports static images with Bicubic filtering and aspect-ratio-preserving cover scaling.

## Development Environment

This project uses Nix for its development shell because Vulkan tools (`glslc`, validation layers, ICDs) are required.

- Enter the shell: `nix develop`
- Build: `cargo build`
- Run: `cargo run`

There are no automated tests in the project at this time.

## Shader Compilation

Fragment shaders are edited in `shader/shader.frag` and compiled manually:

```bash
nix develop --command sh -c 'glslc -O shader/shader.frag -o shader/frag.spv -fshader-stage=fragment'
```

The fragment shader implements Bicubic resampling and cover-style scaling using `textureLod(..., 0.0)` to avoid implicit LOD issues in loops. Modify `shader/shader.frag`, then recompile `shader/frag.spv`; the Rust code embeds `frag.spv` at compile time.

## Architecture

### Entry Point (`src/main.rs`)

The main loop is a Wayland event queue that owns `State`. `Context` (Vulkan) is lazily created once the layer surface receives its first `configure` event, providing the swapchain extent.

### Wayland Layer (`src/wayland/`)

- `wayland.rs`: `State` handles registry globals, layer surface configure events, and keyboard input. It creates a `ZwlrLayerSurfaceV1` with `Layer::Background`, anchors to all edges, and requests size `(0, 0)` so the compositor sends the output's native resolution.
- `wlr_layer_shell.rs`: Wayland protocol bindings for `wlr-layer-shell-unstable-v1`, generated at build time by `wayland_scanner` from `protocols/wlr-layer-shell-unstable-v1.xml`.

### Vulkan Context (`src/context/`)

- `mod.rs`: Module exports. Re-exports `Context`, `ContextData`, `DescriptorManager`, `DeviceManager`, `DeviceQueue`, `Pipeline`, `Swapchain`, `SyncObjects`, `UniformBufferObject`, and `Vertex`.
- `context.rs`: The main `Context` struct. Owns `Instance`, `Device`, `ContextData`, and the wallpaper image. Two constructors exist: `create_for_wayland` (used by main) and `create` (winit backend). `render_wayland` acquires an image, updates the uniform buffer, submits the command buffer, and presents.
- `instance.rs`: Instance creation. `create_instance` for winit (uses `vk_window::get_required_instance_extensions`) and `create_instance_wayland` for Wayland (explicitly enables `KHR_SURFACE` and `KHR_WAYLAND_SURFACE`).
- `device.rs`: Physical and logical device management. `DeviceManager` selects a suitable GPU (requires swapchain support and anisotropic filtering). `create_logical_device` creates the `Device` with graphics/present queues and enables `sampler_anisotropy` and `sample_rate_shading` features.
- `swapchain.rs`: `Swapchain` struct with `create_for_winit` and `create_for_wayland` variants. Handles surface format selection (prefers `R8G8B8A8_SRGB`), present mode selection (prefers `MAILBOX`), and image view creation.
- `pipeline.rs`: `Pipeline` struct containing the graphics pipeline, layout, and render pass. Supports both MSAA and non-MSAA render pass configurations based on `msaa_samples`.
- `descriptor.rs`: `DescriptorManager` handles descriptor set layout (uniform buffer at binding 0, combined image sampler at binding 1), descriptor pool allocation, and descriptor set updates.
- `command.rs`: `CommandManager` creates the command pool and allocates primary command buffers.
- `sync.rs`: `SyncObjects` creates per-frame semaphores (`image_available`, `render_finished`) and fences for GPU/CPU synchronization.
- `texture.rs`: Loads the hardcoded image `assets/wallhaven-3q3wj3.jpg`, stages it to GPU, generates mipmaps, and creates the image view + sampler (`LINEAR` mag/min, anisotropic, `LINEAR` mipmap mode).
- `mipmap.rs`: Generates mipmaps with `cmd_blit_image` and `Filter::LINEAR`.
- `msaa.rs`: `create_color_objects` creates MSAA color resolve targets; `get_max_msaa_samples` queries physical device limits. In `create_for_wayland`, MSAA is disabled (`msaa_samples = _1`) for 2D wallpapers.
- `vertex.rs`: Defines a full-screen quad with position and UV attributes. `VERTICES` and `INDICES` constants define the quad geometry.
- `uniform.rs`: Defines `UniformBufferObject { i_time, _padding, i_resolution }` and allocates one uniform buffer per swapchain image.
- `buffer.rs`: `Buffer` struct for creating vertex and index buffers. Uses staging buffers for device-local index buffers.
- `tool.rs`: Low-level helpers for buffer/image creation (`create_buffer`, `create_image`), memory type queries (`get_memory_type_index`), image view creation (`create_image_view`), one-time command buffers, and swapchain/queue family support queries.

### Alternate Window Backend (`src/app/`)

`app.rs` provides a winit-based `ApplicationHandler` and a `Context::create` path. This is an alternative desktop window backend and is **not** used by the main Wayland wallpaper flow.


### Wallpaper Manager (src/wallpaper/)


## Important Behavior Details

- Swapchain size is driven by the Wayland configure event, not hardcoded. The layer surface uses `set_size(0, 0)` to receive the output's native resolution.
- The wallpaper image path (`assets/wallhaven-3q3wj3.jpg`) is hardcoded in `Context::create_for_wayland`.
- The Vulkan pipeline enables MSAA and sample-rate shading (`min_sample_shading = 0.2`).
- The fragment shader preserves image aspect ratio and covers the screen (cropping if aspect ratios differ).
