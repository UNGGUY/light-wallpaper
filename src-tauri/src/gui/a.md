# light-wallpaper WebView GUI 方案

## 需求分析

- **低频操作**：仅用于设置、管理资源，不需要每帧交互
- **功能范围**：配置壁纸目录、切换间隔、播放模式、音乐管理、着色器选择
- **用户选择**：WebView（HTML/CSS/JS 写 UI）

## 技术栈

| 层 | 选型 | 说明 |
|---|------|------|
| WebView | **wry** | Tauri 底层库，纯 Rust，Linux 用 WebKitGTK |
| 窗口 | wry 内置（基于 tao） | 自动创建 Wayland toplevel 窗口 |
| 前端 | HTML + CSS + vanilla JS | 无需 npm/webpack，单文件内联 |
| IPC | wry `ipc_handler` | Rust ↔ JS 双向通信 |

## 项目结构方案

### 三种选择

| 方案 | 结构 | 启动方式 | 优缺点 |
|------|------|---------|--------|
| **A：单项目单二进制** ⭐ | 一个 Cargo project，一个 binary | `light-wallpaper` 启动壁纸 + GUI | 最简单，一个进程，mpsc 通信 |
| B：单项目多二进制 | 一个 Cargo project，两个 `[[bin]]` | `light-wallpaper-daemon` + `light-wallpaper-gui` | daemon 可独立后台运行 |
| C：两个独立项目 | 两个 Cargo project | 同上，但分开编译 | 依赖完全隔离，但共享代码需抽取 |

### 推荐：方案 A（单项目单二进制）

**理由：**

```
当前 main 线程状态： 空闲（只 join 等待）
wry 事件循环：       跑在主线程
wallpaper + Vulkan：  已在独立线程 (State::begin)
音频：               已在独立线程 (MusicManager::begin)
```

主线程刚好被 wry 占用，wallpaper 和 audio 已经在子线程，无需任何进程拆分。

**默认行为：** 启动后壁纸渲染 + WebView 设置窗口同时出现。关闭窗口 → 程序退出。

**可选 daemon 模式：** 加一个 CLI 参数实现：

```bash
# 完整模式（默认）：壁纸 + GUI
light-wallpaper

# daemon 模式：只有壁纸，无 GUI
light-wallpaper --daemon

# GUI 模式：只打开设置窗口（连接到已有 daemon）
light-wallpaper --gui
```

这样只需一个二进制，三种用法，无需拆项目。

### 为什么不需要拆项目

1. **通信简单** — 同进程内 `mpsc::channel` 零开销，不需要 Unix socket / D-Bus
2. **编译一次** — 一个 `cargo build --release` 产出所有功能
3. **分发简单** — 只维护一个二进制文件
4. **wry/tao 不冲突** — 各自创建独立的 Wayland 连接，不影响 wallpaper 线程的 layer shell
5. **WebKitGTK 按需加载** — 只有在创建 WebView 时才初始化，daemon 模式不加载

### 目录结构

```
light-wallpaper/
├── Cargo.toml
├── src/
│   ├── main.rs          # 入口：解析 CLI，启动对应模式
│   ├── config/          # 配置加载/持久化
│   ├── context/         # Vulkan 渲染引擎（不变）
│   ├── wallpaper/       # 壁纸管理器（不变）
│   ├── wayland/         # wlr_layer_shell + 事件循环（需加命令接收）
│   ├── music/           # 音频播放（不变）
│   └── gui/             # WebView GUI [新增]
│       ├── mod.rs
│       ├── gui.rs       # wry 窗口 + HTML 内联
│       ├── commands.rs  # UiCommand 定义
│       └── DESIGN.md    # 本文档
```

---

## 架构

```
┌─ main 线程 (wry WebView) ────────────────────────────────┐
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │  WebView 窗口                                     │   │
│  │  ┌────────────────────────────────────────────┐  │   │
│  │  │  ⚙ Settings                    light-wallpaper │   │
│  │  │────────────────────────────────────────────│  │   │
│  │  │  [壁纸] [音乐] [着色器] [关于]              │  │   │
│  │  │                                            │  │   │
│  │  │  壁纸目录  [~/Pictures/wallpapers/  📂]    │  │   │
│  │  │  切换间隔  [15s               ▾]           │  │   │
│  │  │  播放模式  ○ 顺序  ○ 随机  ○ 单张          │  │   │
│  │  │                                            │  │   │
│  │  │  当前壁纸  3 / 42                          │  │   │
│  │  │  [⏮ 上一张]  [下一张 ⏭]                    │  │   │
│  │  │                                            │  │   │
│  │  │  [应用]  [取消]                             │  │   │
│  │  └────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  JS ──ipc──→ Rust handler ──mpsc──→ wallpaper thread     │
│                                  ──→ audio thread         │
└──────────────────────────────────────────────────────────┘
```

## 线程模型

```
main 线程:  wry event_loop.run()  ← 阻塞主线程
   │
   ├── spawn → wallpaper 线程 (Vulkan + wlr_layer_shell)
   │              mpsc::Receiver<UiCommand> 接收命令
   │
   ├── spawn → audio 线程 (rodio)
   │              mpsc::Receiver<AudioCommand> 接收命令
   │
   └── wry ipc_handler
           JS → Rust 转换，发送 UiCommand / AudioCommand
           Rust → JS 状态查询
```

### 退出流程

```
用户关闭 WebView 窗口
  → wry Event::WindowEvent::CloseRequested
  → 发送 UiCommand::Exit 到 wallpaper 线程
  → 发送 AudioCommand::Stop 到 audio 线程
  → audio_handle.join()
  → wallpaper_handle.join()
  → 进程退出
```

## 模块结构

```
src/gui/
├── mod.rs        # pub mod gui;
├── gui.rs        # WebView 窗口创建、ipc_handler、事件循环
├── commands.rs   # UiCommand 枚举定义
└── DESIGN.md     # 本文档
```

## IPC 协议

### JS → Rust（通过 `window.ipc.postMessage()`）

```js
// 壁纸控制
window.ipc.postMessage('{"action":"next_wallpaper"}')
window.ipc.postMessage('{"action":"prev_wallpaper"}')
window.ipc.postMessage('{"action":"set_interval","value":30}')
window.ipc.postMessage('{"action":"set_mode","value":"random"}')
window.ipc.postMessage('{"action":"set_image_path","value":"~/Wallpapers/"}')
window.ipc.postMessage('{"action":"set_shader_path","value":"shader/fade.spv"}')

// 音乐控制
window.ipc.postMessage('{"action":"next_track"}')
window.ipc.postMessage('{"action":"prev_track"}')
window.ipc.postMessage('{"action":"pause_music"}')
window.ipc.postMessage('{"action":"resume_music"}')
window.ipc.postMessage('{"action":"set_volume","value":0.7}')
window.ipc.postMessage('{"action":"set_audio_path","value":"~/Music/bgm/"}')

// 系统
window.ipc.postMessage('{"action":"exit"}')
window.ipc.postMessage('{"action":"get_status"}')
```

### Rust → JS（通过 `webview.evaluate_script()`）

```rust
// 状态回传
webview.evaluate_script(&format!(
    "window.__onStatus({})",
    serde_json::to_string(&status).unwrap()
))?;
```

### UiCommand（Rust 侧）

```rust
pub enum UiCommand {
    // 壁纸
    NextWallpaper,
    PrevWallpaper,
    SetInterval(u64),
    SetMode(PlayMode),
    SetImagePath(PathBuf),
    SetShaderPath(PathBuf),

    // 音乐
    NextTrack,
    PrevTrack,
    PauseMusic,
    ResumeMusic,
    SetVolume(f32),
    SetAudioPath(PathBuf),

    // 系统
    GetStatus,
    Exit,
}
```

## 实现步骤

| 步骤 | 内容 | 涉及文件 |
|------|------|---------|
| 1 | 添加 wry + serde_json 依赖 | `Cargo.toml` |
| 2 | 定义 `UiCommand` 枚举 + JSON 解析 | `src/gui/commands.rs` |
| 3 | 实现 WebView 窗口 + ipc_handler + 事件循环 | `src/gui/gui.rs` |
| 4 | 重构 `main.rs`：主线程跑 GUI，子线程跑 wallpaper+audio | `src/main.rs` |
| 5 | wallpaper 线程接收 UiCommand 并执行 | `src/wayland/wayland.rs` |
| 6 | 编写内联 HTML/CSS UI | `src/gui/gui.rs` (内嵌字符串) |
| 7 | 实现配置持久化（读/写 config.toml） | `src/config/config.rs` |

## 新增依赖

```toml
wry = "0.50"
tao = "0.32"       # wry 的窗口后端（可能自动引入）
serde_json = "1.0"  # IPC JSON 解析
```

> 注：wry 0.50 对应 tao 0.32（tao 是 winit 的 fork，wry 使用它作为窗口后端）。
> 项目已有 winit 0.30，但 wry 的 tao 是独立依赖，不会冲突（各自创建 Wayland 连接）。

## 验证方式

```bash
cargo run
# 预期：壁纸正常渲染 + 弹出 WebView 设置窗口
# 关闭设置窗口 → 壁纸进程退出
```
