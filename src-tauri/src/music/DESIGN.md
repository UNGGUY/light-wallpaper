# 背景音乐功能设计方案

## 目标

为 light-wallpaper 动态壁纸引擎添加背景音乐播放功能，与壁纸切换衔接。

## 技术栈约束

- **Rust edition 2024**
- **vulkanalia** + **wlr_layer_shell**：当前渲染管线，不能阻塞
- **单线程 Wayland 事件循环**：`event_queue.blocking_dispatch()` 驱动
- **无外部运行时**：目前没有 tokio / async-std

---

## 选型分析

### 方案 A：rodio + symphonia ⭐ 推荐

| 维度 | 说明 |
|------|------|
| **rodio** | 纯 Rust 音频播放库，基于 cpal（跨平台音频 I/O），支持混音、音量控制、Sink 队列 |
| **symphonia** | 纯 Rust 解码器，支持 MP3 / WAV / FLAC / OGG / AAC，零 C 依赖 |
| **组合方式** | symphonia 解码 → 转为 PCM → 喂给 rodio Sink |
| **线程模型** | rodio Sink 内部有独立播放线程，不会阻塞主循环 |
| **兼容性** | 纯 Rust，无 C 编译依赖，与 Vulkan 无冲突 |

**优点：**
- rodio 的 `Sink` 自动管理播放队列（append 后自动顺序播放）
- 内置音量控制、暂停/恢复
- 内部独立线程，主循环只需 `sink.empty()` 检查 + `sink.append()` 入队
- 纯 Rust 生态，无需系统编解码库

**缺点：**
- symphonia API 有一定学习曲线
- MP3 解码性能不如 FFmpeg（但壁纸场景完全够用）

### 方案 B：cpal + symphonia

直接使用 cpal 管理音频输出设备，symphonia 解码。

**优点：** 更底层，完全控制缓冲区和采样率转换

**缺点：** 需要手动处理环形缓冲区、格式转换、重采样，代码量大

### 方案 C：GStreamer Rust bindings

**优点：** 功能最强，支持所有格式

**缺点：** 引入 GStreamer 系统依赖，太重，与"轻量"理念冲突

### 推荐结论

**方案 A：rodio + symphonia**，原因：
1. 纯 Rust，零系统编解码依赖
2. `Sink` 的队列模型天然匹配壁纸的"顺序/随机/单曲"播放模式
3. 独立播放线程，不阻塞 Wayland 事件循环
4. 与项目现有的轻量理念一致

---

## 架构设计

### 新增依赖（Cargo.toml）

```toml
rodio = "0.20"
symphonia = { version = "0.6", default-features = false, features = ["mp3", "wav", "flac", "ogg", "aac", "isomp4"]  }
```

### 模块结构

```
src/music/
├── mod.rs          # pub mod music; pub use music::MusicManager;
├── music.rs        # MusicManager: 音频文件扫描、播放控制
└── decoder.rs      # symphonia 解码器封装：读取文件 → PCM → rodio Source
```

### MusicManager 设计

```rust
use rodio::{Sink, source::Source};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub enum MusicPlayMode {
    Sequential,  // 顺序播放
    Random,      // 随机
    Single,      // 单曲循环
    Off,         // 关闭
}

pub struct MusicManager {
    sink: Sink,                   // rodio 音频输出（内部独立线程）
    tracks: Vec<PathBuf>,         // 音乐文件列表
    current_index: usize,         // 当前曲目索引
    mode: MusicPlayMode,          // 播放模式
    volume: f32,                  // 音量 0.0 ~ 1.0
    enabled: bool,                // 是否启用
}
```

**关键方法：**

| 方法 | 职责 |
|------|------|
| `new(directory, volume)` | 创建 Sink + OutputStream，扫描音频文件 |
| `play_next()` | 解码下一首曲目，追加到 Sink 队列 |
| `update()` | 每帧调用：若 Sink 为空则播放下一首 |
| `set_mode(mode)` | 切换播放模式 |
| `set_volume(vol)` | 实时调音量 |
| `pause()` / `resume()` | 暂停 / 恢复 |
| `current_track()` | 当前播放的曲目路径 |

### 支持的音频格式

| 格式 | 扩展名 | symphonia 支持 |
|------|--------|----------------|
| MP3 | .mp3 | ✅ |
| WAV | .wav | ✅ |
| FLAC | .flac | ✅ |
| OGG Vorbis | .ogg | ✅ |
| AAC (M4A) | .m4a, .aac | ✅ |

### 解码器封装 (decoder.rs)

symphonia 解码流程：

```
File::open(path)
  → symphonia MediaSourceStream
  → symphonia ProbeFormat (自动检测格式)
  → symphonia Track 选择（取第一个音轨）
  → Decoder 解码 → AudioBufferRef → f32/f64 PCM samples
  → 封装为 rodio::Source trait (实现 Iterator<Item = f32> + CurrentFrameLen)
```

### 配置扩展 (config.toml)

```toml
[music]
# 音乐目录
dir = "~/Music/wallpaper/"
# 是否启用
enabled = true
# 音量 0.0 ~ 1.0
volume = 0.5
# 播放模式: sequential | random | single
mode = "sequential"
```

对应的 `WallpaperConfigRaw` 扩展：

```rust
#[derive(Debug, Deserialize)]
pub struct MusicConfig {
    pub dir: Option<PathBuf>,
    pub enabled: Option<bool>,
    pub volume: Option<f32>,
    pub mode: Option<String>,
}
```

---

## 主循环集成

修改 `main.rs`，在以下几个点插入音频逻辑：

```rust
// 1. 初始化（在创建 Manager 之后）
let music_manager = if music_config.enabled {
    Some(MusicManager::new(&music_config.dir, music_config.volume)?)
} else {
    None
};

// 2. 主循环内（与 wallpaper switching 并列）
while state.running {
    event_queue.blocking_dispatch(&mut state)?;

    if state.configured && state.render {
        if let Some(context) = state.context.as_mut() {
            // --- 壁纸切换 ---
            if !switch {
                if let Some(path) = manager.update() {
                    switch = true;
                    first = true;
                    context.reload_texture(path)?;
                    // 壁纸切换时也切歌（如果模式不是 Off）
                    if let Some(ref mut music) = music_manager {
                        music.play_next();
                    }
                }
            }

            // --- 音频自动切换（独立于壁纸，Sink 播完自动下一首）---
            if let Some(ref mut music) = music_manager {
                music.update();  // 内部检查 sink.empty()
            }

            // ... 切换动画 ...
            context.render_wayland()?;
        }
        surface.commit();
    }
}

// 3. 退出前清理
// rodio Sink + OutputStream 在 Drop 时自动停止
```

### 音频与壁纸的同步策略

有两种耦合级别：

**A. 独立运行（推荐初版）**
- 音乐独立播放，有自己的间隔/循环
- 与壁纸切换不绑定
- 实现最简单

**B. 与壁纸同步**
- 壁纸切换时同时切歌
- 共享同一个 `PlayMode` 或各自独立配置
- 适合"幻灯片+背景音乐"的体验

建议先实现 A，后续可扩展为 B。

---

## 实现步骤

| 步骤 | 内容 | 文件 |
|------|------|------|
| 1 | 添加 rodio + symphonia 依赖 | `Cargo.toml` |
| 2 | 实现 symphonia 解码器封装 | `src/music/decoder.rs` |
| 3 | 实现 MusicManager | `src/music/music.rs` |
| 4 | 扩展配置支持 `[music]` 段 | `src/config/config.rs` |
| 5 | 集成到主循环 | `src/main.rs` |
| 6 | 测试多媒体键或键盘快捷键控制 | `src/wayland/wayland.rs` |

---

## 潜在问题和解决方案

| 问题 | 解决方案 |
|------|----------|
| rodio 的 cpal 后端在 Wayland 下可能检测不到音频设备 | cpal 通过 ALSA/PulseAudio/PipeWire 访问音频，与 Wayland 无关。Linux 上 rodio 可以使用 ALSA backend |
| symphonia 解码耗时可能造成帧卡顿 | 解码放在后台线程，通过 channel 发回 PCM 数据。或者利用 rodio Sink 的内部缓冲提前解码 |
| 内存占用（PCM 缓冲） | 每首解码后立即释放，不缓存解码后的音频 |
| Intel GPU 音频共享内存冲突 | 不会，音频通过 ALSA/Pipewire，与 Vulkan GPU 操作无关 |

---

## 参考

- [rodio docs](https://docs.rs/rodio/latest/rodio/)
- [symphonia docs](https://docs.rs/symphonia/latest/symphonia/)
- [rodio + symphonia 示例](https://github.com/RustAudio/rodio/blob/master/examples/symphonia.rs)
