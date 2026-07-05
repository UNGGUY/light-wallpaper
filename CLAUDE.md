# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Sync:** 每次更新本文件时，请同步更新 `CLAUDE-cn.md`（中文翻译）。

## Project Overview

`light-wallpaper` is a Wayland wallpaper engine with a Tauri v2 settings GUI. It renders wallpapers using Vulkan via `vulkanalia`, attaches to outputs as a background layer via `wlr-layer-shell-unstable-v1`, plays background music via `rodio`, and supports timed wallpaper slideshows with configurable shader transitions.

- **Frontend:** Vue 3 + TypeScript + Vite (`src/`)
- **Backend:** Rust (edition 2021) via Tauri v2 (`src-tauri/src/`)
- **Package manager:** bun

## Build & Run

```bash
# Install frontend dependencies
bun install

# Development (Vite dev server + Tauri window)
bun run tauri dev

# Production build
bun run tauri build
```

Build dependencies: Vulkan SDK (with `glslc` for shader compilation), Rust toolchain, bun.

There are no automated tests.

## Project Structure

```
light-wallpaper/
├── src/                      # Vue 3 frontend (Vite)
│   ├── main.ts               # Vue app entry
│   ├── App.vue               # Root component
│   └── vite-env.d.ts
├── src-tauri/                # Rust backend (Tauri)
│   ├── Cargo.toml            # Rust dependencies
│   ├── tauri.conf.json       # Tauri v2 config
│   ├── build.rs              # Tauri build script
│   ├── shader/               # GLSL shaders + compiled SPIR-V
│   ├── protocols/            # Wayland XML protocol definitions
│   ├── icons/                # App + tray icons
│   └── src/
│       ├── main.rs           # Entry point: calls lib::run()
│       ├── lib.rs            # Tauri builder + commands (skeleton)
│       ├── config/config.rs  # WallpaperConfig (TOML, tilde expand)
│       ├── context/          # Vulkan rendering engine (17 modules)
│       ├── wallpaper/        # Wallpaper slideshow manager
│       ├── wayland/          # wlr-layer-shell integration
│       ├── music/            # rodio audio playback
│       └── gui/              # Design docs only (no active code)
├── vite.config.ts
├── tsconfig.json
├── package.json
└── bun.lock
```

## Architecture

### Thread Model

Three threads with the Tauri event loop on the main thread:

```
main thread:  Tauri event loop (WebView window + IPC)
    └── setup hook: spawn → wallpaper thread (Wayland event loop + Vulkan rendering)
                   spawn → audio thread (rodio playback loop)
```

The wallpaper thread runs `State::begin()` which owns the Wayland connection, event queue, Vulkan context, and wallpaper manager. The audio thread runs `MusicManager::begin()` with an `mpsc::Receiver<AudioCommand>` for play/pause/skip control.

> **Current state:** `lib.rs` is a skeleton — the wallpaper/audio threads are NOT yet wired into Tauri's setup hook. Only a `hello_world` command exists. The wallpaper engine code (wayland, context, music, wallpaper modules) is ready but hasn't been connected.

### Wayland Layer (`src-tauri/src/wayland/`)

- `wayland.rs`: `State` handles registry globals (`wl_compositor`, `wl_seat`, `zwlr_layer_shell_v1`, `wl_output`). Creates a `ZwlrLayerSurfaceV1` with `Layer::Background`, anchors to all edges, and uses `set_exclusive_zone(-1)` to cover panels. The Vulkan `Context` is lazily created on first `configure` event (which provides the swapchain extent). The render loop uses `blocking_dispatch()` and includes wallpaper auto-switching with a 1-second crossfade animation.
- `wlr_layer_shell.rs`: Generated Wayland protocol bindings from `src-tauri/protocols/wlr-layer-shell-unstable-v1.xml`.

### Vulkan Context (`src-tauri/src/context/`)

- `context.rs`: The main `Context` struct holding `Instance`, `Device`, and `ContextData`. Key methods:
  - `create_for_wayland()` — full Vulkan init for Wayland (surface from raw `*mut c_void` pointers)
  - `render_wayland()` — acquire swapchain image → update UBO → submit → present
  - `reload_texture()` — loads a new image to the inactive texture slot (dual-buffer scheme)
  - `switch(progress, first)` — updates descriptor sets to point at old+new textures, re-records command buffers with the `progress` push constant for crossfade. On completion (`progress >= 1.0`), flips which texture is "active"
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

### Wallpaper Manager (`src-tauri/src/wallpaper/`)

- `manager.rs`: `Manager` scans a directory for supported images (png, jpg, jpeg, webp, bmp, gif), and supports `Sequential`, `Random`, and `Single` play modes. `update()` is called each frame and returns `Some(&Path)` when the interval has elapsed (default 15s).

### Music Player (`src-tauri/src/music/`)

- `music.rs`: `MusicManager` scans a directory for audio files (mp3, wav, flac, ogg, m4a, aac), uses `rodio::MixerDeviceSink` for playback. Runs in its own thread with a 100ms `recv_timeout` loop — checks for commands (`Stop`, `Resume`, `Next`, `Prev`) and auto-advances when a track finishes.

### Configuration (`src-tauri/src/config/`)

- `config.rs`: `WallpaperConfig` loaded via the `config` crate from `$XDG_CONFIG_HOME/lightwallpaper/config.toml`. Uses a `WallpaperConfigRaw` intermediate struct with `Option<String>` fields to handle missing config gracefully, then expands tildes via `shellexpand` and converts to `PathBuf`.

| Key | Default | Purpose |
|-----|---------|---------|
| `image_path` | `~/Pictures/assets/wallpapers/` | Directory scanned for wallpaper images |
| `audio_path` | `~/Music/assets/bgm` | Directory scanned for audio files |
| `vert_shader` | `shader/vert.spv` | Vertex shader SPIR-V path |
| `frag_shader` | `shader/frag.spv` | Fragment shader SPIR-V path |

If the config file is missing, defaults are used.

## Shader Compilation

Fragment shaders live in `src-tauri/shader/`. Edit `.frag`/`.vert` files, then compile to SPIR-V:

```bash
glslc -O src-tauri/shader/shader.frag -o src-tauri/shader/frag.spv -fshader-stage=fragment
glslc -O src-tauri/shader/shader.vert -o src-tauri/shader/vert.spv -fshader-stage=vertex
```

**Important:** Shader files are loaded from disk at runtime — paths come from `WallpaperConfig` (`vert_shader` and `frag_shader`). They are NOT embedded at compile time.

Two fragment shaders exist:
- `shader.frag` — simple crossfade transition between two textures using `mix()` with a `progress` push constant
- `shader_spark.frag` — noise-based dissolve transition with edge glow effect

## GUI Design (`src-tauri/src/gui/`)

Contains design documents only (no active Rust code):
- `a.md` — wry-based WebView GUI design (deprecated)
- `TAURI_DESIGN.md` — Tauri v2 frontend plan. The current implementation follows this design but `lib.rs` still needs to be wired up with wallpaper/audio thread spawning and Tauri commands.

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` v2 | Desktop app framework (Rust backend + WebView frontend) |
| `vulkanalia` | Vulkan bindings (Rust) |
| `wayland-client` | Wayland protocol client |
| `wayland-scanner` | Generate Rust bindings from Wayland XML protocols |
| `rodio` | Audio playback |
| `image` | Image decoding |
| `config` + `serde` | TOML config loading |
| `shellexpand` | Tilde expansion in paths |
| `cgmath` | Vector math for UBO |

| npm Package | Purpose |
|-----|---------|
| `vue` | Frontend UI framework |
| `@vitejs/plugin-vue` | Vite plugin for Vue SFC |
| `@tauri-apps/api` | Tauri IPC from JavaScript |
| `@tauri-apps/plugin-opener` | Tauri opener plugin |
| `@vue/language-server` | Vue LSP (for nvim) |
| `@vtsls/language-server` | TypeScript LSP (for nvim) |

## Important Behavior

- Swapchain size comes from the Wayland configure event — the layer surface requests size `(0, 0)` to receive the output's native resolution
- Output scale factor from `wl_output::Scale` is applied to the Vulkan surface dimensions
- ESC key exits the wallpaper thread
- The wallpaper slideshow interval defaults to 15 seconds
- Wallpaper transitions use a 1-second ease-in-out quadratic animation
- The descriptor set uses `binding = 1` as a 2-element `sampler2D` array — shaders sample both `texSamplers[0]` (old) and `texSamplers[1]` (new) simultaneously during transitions
- Vulkan MSAA is disabled (`SampleCountFlags::_1`) for 2D wallpaper rendering
- Tauri dev server runs on port 1420 (HMR on 1421)
- Frontend build output goes to `dist/` (configured as `frontendDist` in tauri.conf.json)
