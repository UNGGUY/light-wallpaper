# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

- `context.rs`: The main `Context` struct. Holds `ContextData` (physical device, swapchain, pipeline, framebuffers, command buffers, sync objects, etc.) and the `Device`/`Instance`. Two constructors exist: `create_for_wayland` (used by main) and `create` (winit backend). `render_wayland` acquires an image, updates the uniform buffer, submits the command buffer, and presents.
- `texture.rs`: Loads the hardcoded image `assets/wallhaven-3q3wj3.jpg`, stages it to GPU, generates mipmaps, and creates the image view + sampler (`LINEAR` mag/min, anisotropic, `LINEAR` mipmap mode).
- `mipmap.rs`: Generates mipmaps with `cmd_blit_image` and `Filter::LINEAR`.
- `msaa.rs`: Creates MSAA color resolve targets; `msaa_samples` is queried from the physical device limits.
- `vertex.rs`: Defines a full-screen quad with position and UV attributes.
- `uniform.rs`: Defines `UniformBufferObject { i_time, _padding, i_resolution }` and allocates one uniform buffer per swapchain image.
- `tool.rs`: Helpers for buffer/image creation, memory type queries, and one-time command buffers.

### Alternate Window Backend (`src/app/`)

`app.rs` provides a winit-based `ApplicationHandler` and a `Context::create` path. This is an alternative desktop window backend and is **not** used by the main Wayland wallpaper flow.

## Important Behavior Details

- Swapchain size is driven by the Wayland configure event, not hardcoded. The layer surface uses `set_size(0, 0)` to receive the output's native resolution.
- The wallpaper image path (`assets/wallhaven-3q3wj3.jpg`) is hardcoded in `Context::create_for_wayland`.
- The Vulkan pipeline enables MSAA and sample-rate shading (`min_sample_shading = 0.2`).
- The fragment shader preserves image aspect ratio and covers the screen (cropping if aspect ratios differ).
