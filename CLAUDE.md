# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Sync:** 每次更新本文件时，请同步更新 `CLAUDE-cn.md`（中文翻译）。

## Project Overview

`light-wallpaper` (crate name `light-paper`) is a Wayland wallpaper engine written in Rust (edition 2024). It renders wallpapers using Vulkan via `vulkanalia`, attaches to outputs as a background layer via `wlr-layer-shell-unstable-v1`, plays background music via `rodio`, and supports timed wallpaper slideshows with configurable shader transitions.

## Build & Run

```bash
cargo build          # Build
cargo run --release  # Run (release recommended for Vulkan perf)
```

Build dependencies: Vulkan SDK (with `glslc` for shader compilation), Rust toolchain.

There are no automated tests.

## Shader Compilation

Fragment shaders live in `shader/`. Edit `.frag`/`.vert` files, then compile to SPIR-V:

```bash
glslc -O shader/shader.frag -o shader/frag.spv -fshader-stage=fragment
glslc -O shader/shader.vert -o shader/vert.spv -fshader-stage=vertex
```

**Important:** Shader files are loaded from disk at runtime — paths come from `WallpaperConfig` (`vert_shader` and `frag_shader`). They are NOT embedded at compile time. The old `include_bytes!` approach was replaced with `std::fs::read()`.

Two fragment shaders exist:
- `shader/shader.frag` — simple crossfade transition between two textures using `mix()` with a `progress` push constant
- `shader/shader_spark.frag` — noise-based dissolve transition with edge glow effect

## Runtime Configuration

Config is read from `$XDG_CONFIG_HOME/lightwallpaper/config.toml` with these defaults:

| Key | Default | Purpose |
|-----|---------|---------|
| `image_path` | `~/Pictures/assets/wallpapers/` | Directory scanned for wallpaper images |
| `audio_path` | `~/Music/assets/bgm` | Directory scanned for audio files |
| `vert_shader` | `shader/vert.spv` | Vertex shader SPIR-V path |
| `frag_shader` | `shader/frag.spv` | Fragment shader SPIR-V path |

All paths support `~` expansion via `shellexpand`. If the config file is missing, defaults are used.

## Architecture

### Thread Model

Three threads with the main thread blocking on `join()`:

```
main thread:  spawn → wallpaper thread (Wayland event loop + Vulkan rendering)
              spawn → audio thread (rodio playback loop)
              join both
```

The wallpaper thread runs `State::begin()` which owns the Wayland connection, event queue, Vulkan context, and wallpaper manager. The audio thread runs `MusicManager::begin()` with an `mpsc::Receiver<AudioCommand>` for play/pause/skip control. The main thread is idle — this is intentional, as the `tauri-ui` branch plans to occupy it with a Tauri WebView.

### Entry Point (`src/main.rs`)

1. Creates `State` (Wayland state, initially no Vulkan context)
2. Loads `WallpaperConfig` from disk
3. Creates `MusicManager` and spawns the audio thread via `channel<AudioCommand>()`
4. Spawns the wallpaper thread via `State::begin(state, image_path, config)`
5. Joins both threads

### Wayland Layer (`src/wayland/`)

- `wayland.rs`: `State` handles registry globals (`wl_compositor`, `wl_seat`, `zwlr_layer_shell_v1`, `wl_output`). Creates a `ZwlrLayerSurfaceV1` with `Layer::Background`, anchors to all edges, and uses `set_exclusive_zone(-1)` to cover panels. The Vulkan `Context` is lazily created on first `configure` event (which provides the swapchain extent). The render loop uses `blocking_dispatch()` and includes wallpaper auto-switching with a 1-second crossfade animation driven by the `switch()` method.
- `wlr_layer_shell.rs`: Generated Wayland protocol bindings from `protocols/wlr-layer-shell-unstable-v1.xml`.

### Vulkan Context (`src/context/`)

- `context.rs`: The main `Context` struct holding `Instance`, `Device`, and `ContextData`. Key methods:
  - `create_for_wayland()` — full initialization path for Wayland (creates surface from raw `*mut c_void` pointers)
  - `render_wayland()` — acquire swapchain image → update UBO → submit → present
  - `reload_texture()` — loads a new image to the inactive texture slot (dual-buffer scheme)
  - `switch(progress, first)` — updates descriptor sets to point at old+new textures, re-records command buffers with the `progress` push constant for crossfade. On `first` frame, binds both textures; on completion (`progress >= 1.0`), flips which texture is "active"
- `instance.rs`: Wayland-specific instance creation with `KHR_WAYLAND_SURFACE` extension
- `device.rs`: GPU selection requiring swapchain + anisotropy support
- `swapchain.rs`: Prefers `R8G8B8A8_SRGB` format and `MAILBOX` present mode
- `pipeline.rs`: Graphics pipeline with push constants for the `progress` uniform
- `descriptor.rs`: Descriptor set layout with UBO at binding 0 and a **2-element** combined image sampler array at binding 1 (for dual-texture crossfade)
- `texture.rs`: Image loading, GPU staging, mipmap generation, sampler creation. Supports double-buffered textures (`texture_image` + `texture_image_alt`) for Intel GPU workaround during wallpaper switching
- `frame.rs`: Creates per-swapchain-image framebuffers (MSAA-aware path)
- `mipmap.rs`: Mipmap generation via `cmd_blit_image` with linear filtering
- `vertex.rs`: Full-screen quad with position + UV attributes
- `uniform.rs`: `UniformBufferObject { i_time, _padding, i_resolution }` — one per swapchain image
- `buffer.rs`: Vertex/index buffer creation with staging
- `tool.rs`: Low-level Vulkan helpers (buffer/image creation, memory queries, one-time command buffers)

**Intel GPU workaround:** Wallpaper switching uses a dual-texture approach — the new image is uploaded to the inactive texture slot while the old one is still displayed, then descriptor sets are atomically switched. This avoids pipeline stalls that caused rendering corruption on Intel iGPUs.

### Wallpaper Manager (`src/wallpaper/`)

- `manager.rs`: `Manager` scans a directory for supported images (png, jpg, jpeg, webp, bmp, gif), maintains a `VecDeque<DynamicImage>` queue, and supports `Sequential`, `Random`, and `Single` play modes. `update()` is called each frame and returns `Some(&Path)` when the interval has elapsed (default 15s).

### Music Player (`src/music/`)

- `music.rs`: `MusicManager` scans a directory for audio files (mp3, wav, flac, ogg, m4a, aac), uses `rodio::MixerDeviceSink` for playback. Runs in its own thread with a 100ms `recv_timeout` loop — checks for commands (`Stop`, `Resume`, `Next`, `Prev`) and auto-advances when a track finishes. Uses `rodio` only (no `symphonia` — `rodio` handles decoding internally via its system backends).

### Configuration (`src/config/`)

- `config.rs`: `WallpaperConfig` loaded via the `config` crate with TOML format. Uses a `WallpaperConfigRaw` intermediate struct with `Option<String>` fields to handle missing config gracefully, then expands tildes and converts to `PathBuf`.

### GUI Plans (`src/gui/`)

Contains design documents only (no active code):
- `a.md` — wry-based WebView GUI design
- `TAURI_DESIGN.md` — Tauri v2 frontend plan (current `tauri-ui` branch)

## Current Branch: `tauri-ui`

The `tauri-ui` branch is implementing a Tauri v2 settings GUI (per `src/gui/TAURI_DESIGN.md`). Key planned changes:
- Main thread will run Tauri event loop instead of idle `join()`
- Wallpaper/audio threads receive commands via `mpsc` channels
- New `src/state.rs` with `AppStatus`, `WallpaperCommand` (extended), `AudioCommand` (extended)
- Wayland event loop switches from `blocking_dispatch()` to non-blocking `dispatch()` + `try_recv()`
- 18 Tauri commands for wallpaper/music control + status polling
- `WallpaperConfig::save()` for config persistence

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `vulkanalia` | Vulkan bindings (Rust) |
| `wayland-client` | Wayland protocol client |
| `wayland-scanner` | Generate Rust bindings from Wayland XML protocols |
| `rodio` | Audio playback |
| `image` | Image decoding |
| `config` + `serde` | TOML config loading |
| `shellexpand` | Tilde expansion in paths |
| `cgmath` | Vector math for UBO |
| `winit` | Alternate window backend (not used in main wallpaper flow) |

## Important Behavior

- Swapchain size comes from the Wayland configure event — the layer surface requests size `(0, 0)` to receive the output's native resolution
- Output scale factor from `wl_output::Scale` is applied to the Vulkan surface dimensions
- ESC key exits the wallpaper thread
- The wallpaper slideshow interval defaults to 15 seconds
- Wallpaper transitions use a 1-second ease-in-out quadratic animation
- The descriptor set uses `binding = 1` as a 2-element `sampler2D` array — shaders sample both `texSamplers[0]` (old) and `texSamplers[1]` (new) simultaneously during transitions
- Vulkan MSAA is disabled (`SampleCountFlags::_1`) for 2D wallpaper rendering
