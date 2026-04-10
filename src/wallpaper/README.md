# Wallpaper 模块设计文档

## 方案 A：基础切换（重新加载）

### 核心思路
最简单的壁纸切换方案：销毁旧纹理 → 加载新纹理 → 更新描述符。

### 优点
- 实现简单，代码量小
- 内存占用少（只保留当前壁纸）
- 无需修改 shader

### 缺点
- 切换时有短暂卡顿（需要重新生成 mipmap）
- 无过渡动画，切换突兀

---

## 文件结构

```
src/wallpaper/
├── mod.rs          # 模块导出
├── manager.rs      # WallpaperManager: 管理壁纸列表和切换逻辑
└── README.md       # 本文档
```

---

## WallpaperManager 设计

### 职责
1. 扫描指定目录，收集所有支持的图片格式
2. 维护当前壁纸索引
3. 提供定时切换控制
4. 通知外部何时需要加载新壁纸

### 数据结构

```rust
pub struct WallpaperManager {
    /// 壁纸文件路径列表
    wallpapers: Vec<PathBuf>,
    /// 当前显示壁纸的索引
    current_index: usize,
    /// 上次切换时间
    last_switch: Instant,
    /// 自动切换间隔
    interval: Duration,
    /// 播放模式
    mode: PlayMode,
}

pub enum PlayMode {
    /// 顺序播放
    Sequential,
    /// 随机播放
    Random,
    /// 固定单张
    Single,
}
```

### 主要方法

```rust
impl WallpaperManager {
    /// 创建管理器，扫描指定目录
    pub fn new(directory: &Path, interval_secs: u64) -> Result<Self>;
    
    /// 每帧调用，检查是否需要切换
    /// 返回：Some(index) 表示需要切换到指定壁纸，None 表示保持当前
    pub fn update(&mut self) -> Option<&Path>;
    
    /// 手动切换到下一个
    pub fn next(&mut self) -> &Path;
    
    /// 手动切换到上一个
    pub fn prev(&mut self) -> &Path;
    
    /// 获取当前壁纸路径
    pub fn current(&self) -> &Path;
    
    /// 设置播放模式
    pub fn set_mode(&mut self, mode: PlayMode);
    
    /// 设置切换间隔
    pub fn set_interval(&mut self, secs: u64);
}
```

---

## Context 需要添加的方法

```rust
impl Context {
    /// 运行时重新加载纹理
    /// 步骤：
    /// 1. 等待设备空闲
    /// 2. 销毁旧纹理资源（image, view, sampler）
    /// 3. 读取新图片
    /// 4. 创建新纹理资源
    /// 5. 更新描述符集
    /// 6. 重新记录命令缓冲区
    pub fn reload_texture(&mut self, new_path: &Path) -> Result<()>;
}
```

### reload_texture 实现细节

```rust
pub fn reload_texture(&mut self, new_path: &Path) -> Result<()> {
    // 1. 等待 GPU 完成当前工作
    unsafe { self.device.device_wait_idle()? };
    
    // 2. 销毁旧纹理资源
    unsafe {
        self.device.destroy_image_view(self.data.texture_image_view, None);
        self.device.destroy_image(self.data.texture_image, None);
        self.device.free_memory(self.data.texture_image_memory, None);
        self.device.destroy_sampler(self.data.texture_image_sampler, None);
    }
    
    // 3. 读取新图片
    let new_image = texture::read_image(new_path.to_str().unwrap())?;
    
    // 4. 创建新纹理资源
    texture::create_texture_image(&self.instance, &self.device, &mut self.data, &new_image)?;
    texture::create_texture_image_view(&self.device, &mut self.data)?;
    texture::create_texture_sampler(&self.device, &mut self.data)?;
    
    // 5. 更新描述符集
    self.data.descriptor_manager.update(
        &self.device,
        &self.data.uniform_buffers,
        self.data.texture_image_view,
        self.data.texture_image_sampler,
    );
    
    // 6. 重新记录命令缓冲区（因为描述符集已更新）
    // 实际上描述符集指针没变，只是内容变了，不需要重新记录
    
    self.image = new_image;
    Ok(())
}
```

---

## 主循环集成

```rust
// src/main.rs 或 wayland.rs

fn main() {
    // ... 初始化 ...
    
    let mut wallpaper_manager = WallpaperManager::new(
        Path::new("~/Pictures/Wallpapers"),
        300, // 5分钟切换一次
    ).unwrap();
    
    // ... 创建 Context ...
    
    loop {
        // 处理 Wayland 事件
        event_queue.blocking_dispatch(&mut state).unwrap();
        
        // 检查是否需要切换壁纸
        if let Some(new_path) = wallpaper_manager.update() {
            context.reload_texture(new_path).unwrap();
        }
        
        // 渲染
        context.render_wayland().unwrap();
    }
}
```

---

## 支持的图片格式

使用 `image` crate 自动识别，支持：
- PNG
- JPEG/JPG
- WebP
- BMP
- GIF（静态）
- TIFF

通过文件扩展名过滤：
```rust
const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "gif"];
```

---

## 后续扩展路径

### 到方案 B（过渡动画）
1. 修改 shader 支持双纹理混合
2. Context 改为同时维护两张纹理
3. 切换时异步加载新纹理，同时渐变 blend 因子

### 到方案 C（完整功能）
1. 添加配置文件支持
2. 添加 IPC 控制（如通过命令行切换）
3. 添加缩略图缓存
4. 支持动态壁纸（视频）

---

## 实现顺序

1. **manager.rs** - 实现 WallpaperManager 基础功能
2. **mod.rs** - 模块导出
3. **context.rs 修改** - 添加 reload_texture 方法
4. **main.rs/wayland.rs 修改** - 集成到主循环
5. **测试** -
