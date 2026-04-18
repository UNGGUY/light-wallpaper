# 壁纸切换功能实现文档

## 功能概述

实现定时自动切换壁纸功能，支持多种图片格式（png, jpg, jpeg, webp, bmp, gif）。

## 核心问题与解决方案

### 问题：Intel 集成显卡驱动崩溃

**现象**：
- 首次加载壁纸正常
- 切换壁纸时 `create_image` 崩溃（段错误）
- 验证层报告设备丢失（Lost Device）

**根本原因**：
1. Intel 集显驱动在 `device_wait_idle` 后状态不一致
2. 运行时分配/销毁大纹理导致内存碎片
3. 临时创建命令缓冲与渲染命令缓冲冲突
4. `instance.get_physical_device_memory_properties` 在特定时机调用会崩溃

### 解决方案

#### 1. 双缓冲纹理（Double Buffering）

**位置**：`src/context/context.rs` - `ContextData`

```rust
// 主纹理
pub(crate) texture_image: vk::Image,
pub(crate) texture_image_view: vk::ImageView,

// 备用纹理（切换时使用）
pub(crate) texture_image_alt: vk::Image,
pub(crate) texture_image_alt_view: vk::ImageView,
pub(crate) use_alt_texture: bool,
```

**原理**：
- 初始化时同时创建两个纹理
- 切换时上传新数据到备用纹理
- 更新描述符集指向新纹理
- 永不销毁/重建纹理对象，避免内存碎片

#### 2. 专用上传命令缓冲

**位置**：`src/context/context.rs` - `ContextData`

```rust
pub(crate) upload_command_buffer: vk::CommandBuffer,
```

**原理**：
- 初始化时预分配专用命令缓冲
- 上传纹理时重置并复用该缓冲
- 避免运行时分配命令缓冲导致驱动状态混乱
- 与渲染命令缓冲完全分离

#### 3. Memory Type 缓存

**位置**：`src/context/context.rs` - `ContextData`

```rust
pub(crate) host_visible_memory_type: Option<u32>,
```

**原理**：
- 首次创建 staging buffer 时缓存 memory type index
- 后续上传直接使用缓存值
- 避免调用 `get_physical_device_memory_properties`（会触发 Intel 驱动崩溃）

#### 4. 严格的 GPU 同步

**流程**：
1. `device_wait_idle()` - 等待 GPU 空闲
2. 读取新图片
3. 上传数据到备用纹理
4. `device_wait_idle()` - 等待上传完成
5. 切换纹理标志
6. 更新描述符集

## 文件修改详情

### 新增文件

#### `src/wallpaper/mod.rs`
模块导出，暴露 `Manager` 和 `PlayMode`。

#### `src/wallpaper/manager.rs`
壁纸管理器实现：
- 目录扫描（支持多种图片格式）
- 定时切换逻辑
- 播放模式：顺序/随机/单张

### 修改的文件

#### `src/context/context.rs`

**新增字段**（`ContextData`）：
- `texture_image_alt`, `texture_image_alt_view`, `texture_image_alt_memory`
- `use_alt_texture: bool`
- `upload_command_buffer: vk::CommandBuffer`
- `host_visible_memory_type: Option<u32>`

**新增方法**（`Context`）：
- `reload_texture()` - 主切换逻辑

**修改方法**：
- `create_for_wayland()` - 初始化双纹理和专用命令缓冲
- `destroy()` - 清理双纹理资源

#### `src/context/texture.rs`

**新增函数**：
- `create_alt_texture_image()` - 创建备用纹理
- `create_alt_texture_image_view()` - 创建备用纹理视图
- `upload_to_texture()` - 上传数据到现有纹理（不复用旧函数）

**关键实现细节**：
- 使用预分配的 `upload_command_buffer`
- 直接使用 `device.reset_command_buffer()` 和 `device.begin_command_buffer()`
- 手动记录布局转换、buffer 到 image 拷贝
- 直接提交到队列并等待完成

#### `src/context/tool.rs`

**修改函数**：
- `create_buffer()` - 添加 memory type 缓存逻辑
- `get_memory_type_index()` - 优先使用缓存的 host-visible memory type

#### `src/main.rs`

**集成逻辑**：
- 创建 `Manager` 实例
- 主循环中每帧调用 `manager.update()`
- 检测到切换时调用 `context.reload_texture()`

## 使用说明

### 初始化

```rust
let directory = Path::new("assets/wallpapers/");
let mut manager = Manager::new(directory, 10).unwrap(); // 10秒切换间隔
```

### 主循环集成

```rust
if let Some(path) = manager.update() {
    context.reload_texture(path).unwrap();
}
```

### 播放模式

```rust
manager.set_mode(PlayMode::Sequential); // 顺序播放
manager.set_mode(PlayMode::Random);     // 随机播放
manager.set_mode(PlayMode::Single);     // 固定单张
```

## 注意事项

1. **Intel 集显兼容性**：
   - 纹理尺寸过大可能仍有问题（建议不超过 4096x4096）
   - mipmapping 可能触发驱动 bug（当前禁用）

2. **性能考虑**：
   - 切换时有短暂卡顿（上传数据到 GPU）
   - 双纹理占用双倍显存

3. **验证层警告**：
   - `UPDATE_AFTER_BIND` 警告可忽略
   - 不影响功能正确性

## 后续优化方向

1. **过渡动画**：使用 shader 实现淡入淡出
2. **后台加载**：在单独线程预加载下一张图片
3. **配置支持**：从配置文件读取目录和切换间隔
4. **动态壁纸**：支持视频格式
