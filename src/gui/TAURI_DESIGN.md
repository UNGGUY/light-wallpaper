# light-wallpaper Tauri 前端设计方案

## 一、项目背景

当前 `light-wallpaper` 是一个纯 CLI 壁纸守护进程，包含两个线程：

- **Wallpaper 线程**：Wayland 事件循环 + Vulkan 渲染（wlr-layer-shell 协议）
- **Audio 线程**：rodio 音频播放

主线程空闲，仅 `join` 两个子线程。所有配置需要手动编辑 `config.toml` 并重启应用。用户希望通过 **Tauri v2** 构建一个前端设置界面，提供可视化的壁纸管理和音乐控制。

## 二、技术选型

| 层级 | 选型 | 说明 |
|------|------|------|
| 框架 | **Tauri v2** | Rust 后端 + WebView 前端，比直接用 wry 功能更完善 |
| 窗口 | Tauri 内置（基于 tao） | 自动创建 Wayland toplevel 窗口 |
| 前端 | HTML + CSS + vanilla JS | 无需 npm/webpack，静态文件直接由 Tauri 托管 |
| IPC | Tauri `#[tauri::command]` | 类型安全的 Rust ↔ JS 双向通信 |
| 系统托盘 | Tauri `tray-icon` feature | 关闭窗口后壁纸继续运行 |
| 原生对话框 | `tauri-plugin-dialog` | 浏览壁纸/音乐目录 |

## 三、架构总览

```
┌─ Tauri 主线程 (事件循环) ──────────────────────────────────────────┐
│                                                                      │
│  Tauri::Builder                                                      │
│  ├── setup 钩子: 启动 wallpaper + audio 线程，创建系统托盘           │
│  ├── 托管状态: AppManagedState (命令发送器 + 共享状态)               │
│  └── 18 个 #[tauri::command] 函数                                    │
│                                                                      │
│  ┌── WebView 窗口 ────────┐    ┌── 系统托盘 ──────┐                  │
│  │  frontend/              │    │  显示/隐藏窗口    │                  │
│  │  ├── index.html         │    │  下一张壁纸       │                  │
│  │  ├── style.css          │    │  播放/暂停        │                  │
│  │  └── script.js          │    │  退出             │                  │
│  └─────────────────────────┘    └──────────────────┘                  │
│                    │                                                  │
└────────────────────┼──────────────────────────────────────────────────┘
                     │ mpsc::Sender
          ┌──────────┴──────────┐
          ▼                     ▼
┌─ wallpaper 线程 ──────┐  ┌─ audio 线程 ────────────┐
│  Wayland 事件循环      │  │  rodio 播放循环          │
│  Vulkan 渲染           │  │  rx.recv_timeout(100ms)  │
│  Manager (壁纸管理)    │  │  MusicManager            │
│  非阻塞 dispatch       │  │                          │
│  + try_recv 接收命令   │  │  更新 AppStatus          │
│  更新 AppStatus        │  │                          │
└────────────────────────┘  └──────────────────────────┘
```

## 四、目录结构

```
light-wallpaper/
├── Cargo.toml                    # [修改] 添加 tauri 依赖
├── tauri.conf.json               # [新增] Tauri v2 配置
├── capabilities/
│   └── default.json              # [新增] Tauri 权限声明
├── build.rs                      # [新增] Tauri 构建脚本
├── icons/
│   └── icon.png                  # [新增] 应用图标（托盘用）
├── frontend/                     # [新增] 前端静态文件
│   ├── index.html                # 标签页设置界面
│   ├── style.css                 # 深色主题样式
│   └── script.js                 # IPC 调用 + 状态轮询
├── src/
│   ├── main.rs                   # [重写] 仅调用 lib::run()
│   ├── lib.rs                    # [新增] Tauri builder + 命令定义
│   ├── state.rs                  # [新增] 共享类型定义
│   ├── config/
│   │   └── config.rs             # [修改] 增加 save() 方法
│   ├── context/                  # [不变]
│   ├── wallpaper/                # [不变]
│   ├── wayland/
│   │   └── wayland.rs            # [修改] 增加 begin_with_commands()
│   └── music/
│       └── music.rs              # [修改] 扩展 AudioCommand，增加 begin_extended()
└── gui/                          # [删除] 被 Tauri 方案替代
```

## 五、核心类型设计 (`src/state.rs`)

### 5.1 壁纸命令枚举

```rust
pub enum WallpaperCommand {
    Next,                              // 下一张
    Prev,                              // 上一张
    SwitchTo(usize),                   // 跳转到指定索引
    SetMode(PlayMode),                 // 设置播放模式
    SetInterval(u64),                  // 设置切换间隔（秒）
    ReloadImagePath(PathBuf),          // 更换壁纸目录
    ReloadShaders(PathBuf, PathBuf),   // 更换着色器 (vert, frag)
    Exit,                              // 退出
}
```

### 5.2 音频命令枚举（扩展现有）

```rust
pub enum AudioCommand {
    Stop,                              // 停止
    Resume,                            // 继续
    Next,                              // 下一首
    Prev,                              // 上一首
    SetMode(MusicPlayMode),            // [新增] 播放模式
    SetVolume(f32),                    // [新增] 音量 0.0-1.0
    SetAudioPath(PathBuf),             // [新增] 更换音频目录
}
```

### 5.3 共享状态（可序列化，供前端轮询）

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct AppStatus {
    // 壁纸状态
    pub wallpaper_index: usize,
    pub wallpaper_total: usize,
    pub wallpaper_mode: String,        // "sequential" | "random" | "single"
    pub wallpaper_interval: u64,
    pub wallpaper_current_path: String,

    // 音频状态
    pub audio_playing: bool,
    pub audio_track_index: usize,
    pub audio_track_total: usize,
    pub audio_mode: String,            // "sequential" | "random" | "single" | "off"
    pub audio_volume: f32,
    pub audio_current_track: String,

    // 配置路径（供界面显示）
    pub image_path: String,
    pub audio_path: String,
    pub vert_shader: String,
    pub frag_shader: String,
}
```

### 5.4 Tauri 托管状态

```rust
pub struct AppManagedState {
    pub wallpaper_tx: mpsc::Sender<WallpaperCommand>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub status: Arc<Mutex<AppStatus>>,
    pub wallpaper_paths: Arc<Mutex<Vec<String>>>,
    pub audio_tracks: Arc<Mutex<Vec<String>>>,
}
```

## 六、Tauri 命令列表

全部定义在 `src/lib.rs` 中，通过 `#[tauri::command]` 注解：

| 命令名 | 方向 | 功能 |
|--------|------|------|
| `next_wallpaper` | JS→Rust | 切换到下一张壁纸 |
| `prev_wallpaper` | JS→Rust | 切换到上一张壁纸 |
| `switch_wallpaper` | JS→Rust | 跳转到指定索引 |
| `set_wallpaper_mode` | JS→Rust | 设置模式 (sequential/random/single) |
| `set_wallpaper_interval` | JS→Rust | 设置切换间隔（秒） |
| `set_image_path` | JS→Rust | 更换壁纸目录 |
| `set_shader_path` | JS→Rust | 更换着色器路径 |
| `play_music` | JS→Rust | 继续播放 |
| `pause_music` | JS→Rust | 暂停播放 |
| `next_track` | JS→Rust | 下一首 |
| `prev_track` | JS→Rust | 上一首 |
| `set_volume` | JS→Rust | 设置音量 0.0–1.0 |
| `set_audio_mode` | JS→Rust | 设置音频模式 |
| `set_audio_path` | JS→Rust | 更换音频目录 |
| `get_status` | JS→Rust→JS | 获取当前状态 |
| `get_wallpapers` | JS→Rust→JS | 获取壁纸路径列表 |
| `get_tracks` | JS→Rust→JS | 获取音频轨道列表 |
| `save_config` | JS→Rust | 保存配置到 config.toml |

## 七、Wallpaper 线程改造 (`src/wayland/wayland.rs`)

### 当前问题

- 使用 `event_queue.blocking_dispatch()` 阻塞等待 Wayland 事件
- 无法接收外部命令
- 没有状态上报机制

### 改造方案

新增 `begin_with_commands()` 函数，采用**非阻塞事件循环**：

```rust
pub fn begin_with_commands(
    status: Arc<Mutex<AppStatus>>,
    wallpaper_rx: mpsc::Receiver<WallpaperCommand>,
    wallpaper_paths: Arc<Mutex<Vec<String>>>,
    config: WallpaperConfig,
)
```

主循环结构：

```
while state.running:
    1. event_queue.dispatch()        // 非阻塞处理 Wayland 事件
    2. while let Ok(cmd) = wallpaper_rx.try_recv():  // 处理 Tauri 命令
        匹配 cmd:
          Next/Prev        → 切换壁纸，触发动画
          SwitchTo(index)  → 跳转
          SetMode/SetInterval → 更新 Manager
          ReloadImagePath  → 重新扫描目录
          ReloadShaders    → 更新着色器路径
          Exit             → 设置 running = false
    3. 渲染逻辑:
       - 自动切换检测 (manager.update())
       - Vulkan 渲染 + surface.commit()
    4. sleep(16ms)                   // ~60fps
```

关键改动：将 `blocking_dispatch` 替换为非阻塞 `dispatch()`，在每次循环中用 `try_recv()` 拉取命令。

## 八、Audio 线程改造 (`src/music/music.rs`)

### 扩展 AudioCommand 枚举

新增 `SetMode`、`SetVolume`、`SetAudioPath` 三个变体。

### 新增 begin_extended()

```rust
pub fn begin_extended(
    rx: Receiver<AudioCommand>,
    music_manager: MusicManager,
    status: Arc<Mutex<AppStatus>>,
    tracks: Arc<Mutex<Vec<String>>>,
)
```

主循环处理新增命令：
- `SetMode(mode)` → 更新播放模式，写入 status
- `SetVolume(vol)` → 设置音量，写入 status
- `SetAudioPath(path)` → 更新路径，写入 status

已有 `recv_timeout(100ms)` 循环结构不变。

## 九、配置持久化 (`src/config/config.rs`)

新增 `WallpaperConfig::save()` 方法，将当前配置序列化写回 `$XDG_CONFIG_HOME/lightwallpaper/config.toml`。

## 十、Tauri 配置

### tauri.conf.json 关键配置

```json
{
  "productName": "Light Wallpaper",
  "identifier": "com.lightpaper.app",
  "build": { "frontendDist": "../frontend" },
  "app": {
    "windows": [{
      "label": "main",
      "title": "Light Wallpaper 设置",
      "width": 820,
      "height": 640,
      "center": true
    }],
    "trayIcon": { "iconPath": "icons/icon.png" }
  }
}
```

### capabilities/default.json

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default", "dialog:default", "dialog:allow-open"]
}
```

## 十一、前端界面设计

### 布局（820×640px，深色主题）

```
┌─────────────────────────────────────────────────────────────┐
│  Light Wallpaper 设置                                        │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  [壁纸]  [音乐]  [着色器]  [关于]                        ││
│  ├─────────────────────────────────────────────────────────┤│
│  │                                                         ││
│  │  === 壁纸标签页 ===                                      ││
│  │                                                         ││
│  │  图片目录: [~/Pictures/wallpapers/...        ] [浏览]   ││
│  │                                                         ││
│  │  切换间隔: [15 秒                          ▾]           ││
│  │                                                         ││
│  │  播放模式:  ( ) 顺序  (•) 随机  ( ) 单张                ││
│  │                                                         ││
│  │  ┌─ 壁纸列表 ───────────────────────────────────┐      ││
│  │  │  ○  01_sunset.jpg                             │      ││
│  │  │  ●  02_mountains.jpg   ← 当前                 │      ││
│  │  │  ○  03_forest.jpg                             │      ││
│  │  │  ○  04_ocean.jpg                             │      ││
│  │  │  ...                                          │      ││
│  │  └───────────────────────────────────────────────┘      ││
│  │                                                         ││
│  │  [◀ 上一张]  第 2 / 42 张  [下一张 ▶]                   ││
│  │                                                         ││
│  │  [保存配置]                                              ││
│  │                                                         ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  ● 壁纸运行中  ♪ 音乐: 播放中 - 02_song.mp3             ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 音乐标签页

```
  === 音乐标签页 ===

  音频目录: [~/Music/assets/bgm/              ] [浏览]

  播放模式:  (•) 顺序  ( ) 随机  ( ) 单曲  ( ) 关闭

  音量: [████████░░] 80%

  [⏮ 上一首]  [▶ 播放]  [⏭ 下一首]

  ┌─ 播放列表 ───────────────────────────────────┐
  │  ○  01_song.mp3                               │
  │  ●  02_song.mp3  ← 正在播放                   │
  │  ○  03_song.mp3                               │
  └───────────────────────────────────────────────┘
```

### 着色器标签页

```
  === 着色器标签页 ===

  顶点着色器: [shader/vert.spv                ] [浏览]
  片段着色器: [shader/frag.spv                ] [浏览]

  说明: 着色器决定了壁纸切换时的过渡动画效果。
        修改后需要重新加载才能生效。
```

### script.js 核心逻辑

```javascript
const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// ── 壁纸控制 ──
async function nextWallpaper() { await invoke('next_wallpaper'); refreshStatus(); }
async function setMode(mode) { await invoke('set_wallpaper_mode', { mode }); }
async function setInterval(secs) { await invoke('set_wallpaper_interval', { interval: secs }); }

// ── 音频控制 ──
async function playMusic() { await invoke('play_music'); }
async function pauseMusic() { await invoke('pause_music'); }
async function setVolume(vol) { await invoke('set_volume', { volume: vol }); }

// ── 状态轮询（每 2 秒）──
async function refreshStatus() {
    const status = await invoke('get_status');
    // 更新 UI 各元素...
}
setInterval(refreshStatus, 2000);

// ── 原生目录选择 ──
async function browseImageDir() {
    const result = await open({ directory: true, multiple: false, title: "选择壁纸目录" });
    if (result) {
        document.getElementById('image-path').value = result;
        await invoke('set_image_path', { path: result });
    }
}
```

## 十二、应用生命周期

```
启动 → 壁纸立即渲染 + 设置窗口打开 + 托盘图标出现
  │
  ├── 用户在设置窗口调整配置 → 实时生效
  │
  ├── 关闭设置窗口 → 窗口隐藏，壁纸继续运行，托盘图标保留
  │
  ├── 点击托盘图标 → 设置窗口重新显示
  │
  └── 托盘菜单 → 退出 → 发送 Exit 命令 → wallpaper 线程退出
                                      → audio 线程退出
                                      → 进程结束
```

## 十三、Cargo.toml 变更

```toml
[dependencies]
# 现有依赖保持不变
# ...

# 新增 Tauri 相关
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-dialog = "2"
serde_json = "1.0"

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

## 十四、实现步骤

| 步骤 | 文件 | 描述 |
|------|------|------|
| 1 | `Cargo.toml`、`build.rs` | 添加 tauri 依赖和构建脚本 |
| 2 | `tauri.conf.json` | Tauri v2 配置文件 |
| 3 | `capabilities/default.json` | 权限声明 |
| 4 | `icons/icon.png` | 应用/托盘图标 |
| 5 | `src/state.rs` | 定义 WallpaperCommand、扩展 AudioCommand、AppStatus、AppManagedState |
| 6 | `src/config/config.rs` | 增加 `WallpaperConfig::save()` |
| 7 | `src/wayland/wayland.rs` | 新增 `begin_with_commands()`，非阻塞事件循环 |
| 8 | `src/music/music.rs` | 扩展 AudioCommand，新增 `begin_extended()` |
| 9 | `src/lib.rs` | Tauri builder、setup 钩子、18 个 command 函数 |
| 10 | `src/main.rs` | 重写为 `fn main() { light_paper::run(); }` |
| 11 | `frontend/index.html` | 标签页布局 |
| 12 | `frontend/style.css` | 深色主题样式 |
| 13 | `frontend/script.js` | IPC 调用、状态轮询、对话框集成 |
| 14 | 删除 `src/gui/` | 清理旧的 wry 方案 |

## 十五、注意事项

1. **Linux 依赖**：Tauri 需要 `webkit2gtk-4.1`，需要更新 Nix flake 或通过系统包管理器安装。
2. **Wayland 非阻塞分发**：`dispatch()` 返回 `Ok(0)` 表示无事件，这是正常情况，继续循环即可。
3. **Vulkan 指针安全**：`surface_ptr` 和 `display_ptr` 在 wallpaper 线程内创建和使用，不跨线程传递。
4. **Edition 2024**：项目使用 Rust edition 2024，需验证与 Tauri v2 的兼容性。
5. **锁竞争**：AppStatus 每 2 秒读取一次，线程写入极短暂（仅赋值标量/字符串），不构成瓶颈。
