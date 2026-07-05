# CLAUDE-cn.md

本文件为 Claude Code (claude.ai/code) 提供在此仓库中工作的指导。（CLAUDE.md 的中文翻译）

## 项目概述

`light-wallpaper` 是一个带有 Tauri v2 设置界面的 Wayland 壁纸引擎。通过 `vulkanalia` 使用 Vulkan 渲染壁纸，通过 `wlr-layer-shell-unstable-v1` 将壁纸作为背景层附加到显示器，通过 `rodio` 播放背景音乐，并支持带可配置着色器过渡效果的定时壁纸幻灯片。

- **前端:** Vue 3 + TypeScript + Vite (`src/`)
- **后端:** Rust (edition 2021) Tauri v2 (`src-tauri/src/`)
- **包管理器:** bun

## 构建与运行

```bash
# 安装前端依赖
bun install

# 开发模式（Vite 开发服务器 + Tauri 窗口）
bun run tauri dev

# 生产构建
bun run tauri build
```

构建依赖：Vulkan SDK（含 `glslc` 用于着色器编译）、Rust 工具链、bun。

项目没有自动化测试。

## 项目结构

```
light-wallpaper/
├── src/                      # Vue 3 前端（Vite）
│   ├── main.ts               # Vue 应用入口
│   ├── App.vue               # 根组件
│   └── vite-env.d.ts
├── src-tauri/                # Rust 后端（Tauri）
│   ├── Cargo.toml            # Rust 依赖
│   ├── tauri.conf.json       # Tauri v2 配置
│   ├── build.rs              # Tauri 构建脚本
│   ├── shader/               # GLSL 着色器 + 编译后的 SPIR-V
│   ├── protocols/            # Wayland XML 协议定义
│   ├── icons/                # 应用和托盘图标
│   └── src/
│       ├── main.rs           # 入口点：调用 lib::run()
│       ├── lib.rs            # Tauri builder + 命令（骨架）
│       ├── config/config.rs  # WallpaperConfig（TOML，波浪号展开）
│       ├── context/          # Vulkan 渲染引擎（17 个模块）
│       ├── wallpaper/        # 壁纸幻灯片管理器
│       ├── wayland/          # wlr-layer-shell 集成
│       ├── music/            # rodio 音频播放
│       └── gui/              # 仅设计文档（无活动代码）
├── vite.config.ts
├── tsconfig.json
├── package.json
└── bun.lock
```

## 架构

### 线程模型

三个线程，主线程运行 Tauri 事件循环：

```
主线程：  Tauri 事件循环（WebView 窗口 + IPC）
    └── setup 钩子: spawn → 壁纸线程（Wayland 事件循环 + Vulkan 渲染）
                   spawn → 音频线程（rodio 播放循环）
```

壁纸线程运行 `State::begin()`，拥有 Wayland 连接、事件队列、Vulkan 上下文和壁纸管理器。音频线程运行 `MusicManager::begin()`，使用 `mpsc::Receiver<AudioCommand>` 进行播放/暂停/跳过控制。

> **当前状态：** `lib.rs` 是骨架代码——壁纸/音频线程尚未接入 Tauri 的 setup 钩子。目前只有一个 `hello_world` 命令。壁纸引擎代码（wayland、context、music、wallpaper 模块）已就绪但尚未连接。

### Wayland 层 (`src-tauri/src/wayland/`)

- `wayland.rs`：`State` 处理注册表全局对象（`wl_compositor`、`wl_seat`、`zwlr_layer_shell_v1`、`wl_output`）。创建一个 `ZwlrLayerSurfaceV1` 并使用 `Layer::Background`，锚定到所有边缘，使用 `set_exclusive_zone(-1)` 覆盖面板。Vulkan `Context` 在第一个 `configure` 事件时延迟创建（该事件提供交换链范围）。渲染循环使用 `blocking_dispatch()`，并包含壁纸自动切换和 1 秒交叉淡入淡出动画。
- `wlr_layer_shell.rs`：从 `src-tauri/protocols/wlr-layer-shell-unstable-v1.xml` 生成的 Wayland 协议绑定。

### Vulkan 上下文 (`src-tauri/src/context/`)

- `context.rs`：主要的 `Context` 结构体，包含 `Instance`、`Device` 和 `ContextData`。关键方法：
  - `create_for_wayland()` — Wayland 的完整 Vulkan 初始化（从原始 `*mut c_void` 指针创建 surface）
  - `render_wayland()` — 获取交换链图像 → 更新 UBO → 提交 → 呈现
  - `reload_texture()` — 将新图像加载到非活动纹理槽（双缓冲方案）
  - `switch(progress, first)` — 更新描述符集以指向旧+新纹理，使用 `progress` push constant 重新录制命令缓冲区以实现交叉淡入淡出。完成后（`progress >= 1.0`），翻转哪个纹理是"活动的"
- `instance.rs`：使用 `KHR_WAYLAND_SURFACE` 扩展创建 Wayland 专用实例
- `device.rs`：GPU 选择，需要交换链 + 各向异性过滤支持
- `swapchain.rs`：优先使用 `R8G8B8A8_SRGB` 格式和 `MAILBOX` 呈现模式
- `pipeline.rs`：图形管线，使用 push constants 传递 `progress` 统一变量
- `descriptor.rs`：描述符集布局，UBO 在绑定 0，**2 元素**组合图像采样器数组在绑定 1（用于双纹理交叉淡入淡出）
- `texture.rs`：图像加载、GPU 暂存、mipmap 生成、采样器创建。支持双缓冲纹理（`texture_image` + `texture_image_alt`），用于壁纸切换时的 Intel GPU 变通方案
- `frame.rs`：为每个交换链图像创建帧缓冲（支持 MSAA 路径）
- `mipmap.rs`：通过 `cmd_blit_image` 进行 mipmap 生成，使用线性过滤
- `vertex.rs`：带位置 + UV 属性的全屏四边形
- `uniform.rs`：`UniformBufferObject { i_time, _padding, i_resolution }` — 每个交换链图像一个
- `buffer.rs`：使用暂存缓冲区创建顶点/索引缓冲区
- `tool.rs`：低级 Vulkan 辅助函数（缓冲区/图像创建、内存查询、一次性命令缓冲区）

**Intel GPU 变通方案：** 壁纸切换使用双纹理方法——新图像上传到非活动纹理槽，而旧图像仍在显示，然后原子地切换描述符集。这避免了在 Intel iGPU 上导致渲染损坏的管线停顿。

### 壁纸管理器 (`src-tauri/src/wallpaper/`)

- `manager.rs`：`Manager` 扫描目录中支持的图片（png、jpg、jpeg、webp、bmp、gif），支持 `Sequential`、`Random` 和 `Single` 播放模式。`update()` 每帧调用，当间隔时间到（默认 15 秒）时返回 `Some(&Path)`。

### 音乐播放器 (`src-tauri/src/music/`)

- `music.rs`：`MusicManager` 扫描目录中的音频文件（mp3、wav、flac、ogg、m4a、aac），使用 `rodio::MixerDeviceSink` 播放。在独立线程中运行，使用 100ms 的 `recv_timeout` 循环——检查命令（`Stop`、`Resume`、`Next`、`Prev`），并在曲目结束时自动推进。

### 配置 (`src-tauri/src/config/`)

- `config.rs`：通过 `config` 包从 `$XDG_CONFIG_HOME/lightwallpaper/config.toml` 加载 `WallpaperConfig`。使用 `WallpaperConfigRaw` 中间结构体，其中包含 `Option<String>` 字段，以优雅地处理缺失的配置，然后通过 `shellexpand` 展开波浪号并转换为 `PathBuf`。

| 键 | 默认值 | 用途 |
|-----|---------|---------|
| `image_path` | `~/Pictures/assets/wallpapers/` | 扫描壁纸图片的目录 |
| `audio_path` | `~/Music/assets/bgm` | 扫描音频文件的目录 |
| `vert_shader` | `shader/vert.spv` | 顶点着色器 SPIR-V 路径 |
| `frag_shader` | `shader/frag.spv` | 片段着色器 SPIR-V 路径 |

如果配置文件缺失，则使用默认值。

## 着色器编译

片段着色器位于 `src-tauri/shader/`。编辑 `.frag`/`.vert` 文件，然后编译为 SPIR-V：

```bash
glslc -O src-tauri/shader/shader.frag -o src-tauri/shader/frag.spv -fshader-stage=fragment
glslc -O src-tauri/shader/shader.vert -o src-tauri/shader/vert.spv -fshader-stage=vertex
```

**重要：** 着色器文件在运行时从磁盘加载——路径来自 `WallpaperConfig`（`vert_shader` 和 `frag_shader`）。它们不会在编译时嵌入。

存在两个片段着色器：
- `shader.frag` — 使用 `mix()` 和 `progress` push constant 在两个纹理之间进行简单的交叉淡入淡出过渡
- `shader_spark.frag` — 基于噪声的溶解过渡，带有边缘发光效果

## GUI 设计 (`src-tauri/src/gui/`)

仅包含设计文档（无活动 Rust 代码）：
- `a.md` — 基于 wry 的 WebView GUI 设计（已废弃）
- `TAURI_DESIGN.md` — Tauri v2 前端方案。当前实现遵循此设计，但 `lib.rs` 仍需接入壁纸/音频线程启动和 Tauri 命令。

## 关键依赖

| Crate | 用途 |
|-------|---------|
| `tauri` v2 | 桌面应用框架（Rust 后端 + WebView 前端） |
| `vulkanalia` | Vulkan 绑定（Rust） |
| `wayland-client` | Wayland 协议客户端 |
| `wayland-scanner` | 从 Wayland XML 协议生成 Rust 绑定 |
| `rodio` | 音频播放 |
| `image` | 图像解码 |
| `config` + `serde` | TOML 配置加载 |
| `shellexpand` | 路径中的波浪号展开 |
| `cgmath` | UBO 的向量数学 |

| npm 包 | 用途 |
|-----|---------|
| `vue` | 前端 UI 框架 |
| `@vitejs/plugin-vue` | Vue SFC 的 Vite 插件 |
| `@tauri-apps/api` | JavaScript 端 Tauri IPC |
| `@tauri-apps/plugin-opener` | Tauri opener 插件 |
| `@vue/language-server` | Vue LSP（用于 nvim） |
| `@vtsls/language-server` | TypeScript LSP（用于 nvim） |

## 重要行为

- 交换链大小来自 Wayland configure 事件——layer surface 请求大小 `(0, 0)` 以接收显示器的原生分辨率
- `wl_output::Scale` 的输出缩放因子应用于 Vulkan surface 尺寸
- ESC 键退出壁纸线程
- 壁纸幻灯片切换间隔默认为 15 秒
- 壁纸过渡使用 1 秒的 ease-in-out 二次动画
- 描述符集使用 `binding = 1` 作为 2 元素 `sampler2D` 数组——着色器在过渡期间同时采样 `texSamplers[0]`（旧纹理）和 `texSamplers[1]`（新纹理）
- Vulkan MSAA 已禁用（`SampleCountFlags::_1`），适用于 2D 壁纸渲染
- Tauri 开发服务器运行在端口 1420（HMR 在 1421）
- 前端构建输出到 `dist/`（在 tauri.conf.json 中配置为 `frontendDist`）
