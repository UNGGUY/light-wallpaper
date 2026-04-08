# Context 模块重构计划

## 当前问题

`context.rs` 目前有 1500+ 行代码，存在以下问题：

1. **职责过于集中** - 一个文件包含了 Vulkan 初始化、设备管理、Swapchain、Pipeline、Buffer 等所有逻辑
2. **可读性差** - 大量自由函数混杂，难以快速定位特定功能
3. **维护困难** - 修改一个子系统（如 Swapchain）需要在大文件中跳转
4. **代码重复** - winit 和 wayland 两种后端的创建逻辑有大量重复代码

## 重构目标

1. 按 Vulkan 对象的生命周期和职责拆分模块
2. 每个模块只负责单一职责，对外暴露清晰的接口
3. 消除 winit/wayland 后端的重复代码
4. 保持现有功能不变，重构后渲染逻辑正常运作

## 新的模块结构

```
src/context/
├── mod.rs              # 模块聚合，暴露公开 API
├── context.rs          # Context 主结构体，只保留协调逻辑
├── instance.rs         # Vulkan Instance 创建
│   └── 支持 winit/wayland 两种后端
├── device.rs           # 物理设备选择 + 逻辑设备创建 + Queue 管理
├── swapchain.rs        # Surface、Swapchain、ImageView
├── pipeline.rs         # RenderPass、Pipeline、ShaderModule
├── buffer.rs           # Vertex Buffer、Index Buffer
├── command.rs          # CommandPool、CommandBuffer 分配
├── sync.rs             # Semaphore、Fence
├── descriptor.rs       # DescriptorSetLayout、DescriptorPool、DescriptorSet
├── render.rs           # 渲染循环逻辑 (render/render_wayland)
├── resource.rs         # 资源管理统一接口
│
└── 保留的辅助模块：
    ├── texture.rs      # 贴图加载和创建（已较独立）
    ├── uniform.rs      # Uniform Buffer 定义
    ├── vertex.rs       # 顶点定义
    ├── msaa.rs         # MSAA 颜色对象创建
    ├── mipmap.rs       # Mipmap 生成
    └── tool.rs         # 底层工具函数（buffer/image 创建等）
```

## 模块职责详解

### 1. `instance.rs`
**职责**：Vulkan 实例创建

**接口**：
```rust
pub fn create_for_winit(window: &Window, entry: &Entry) -> Result<Instance>;
pub fn create_for_wayland(entry: &Entry) -> Result<Instance>;
```

**包含当前代码**：
- `create_instance()`
- `create_instance_wayland()`
- `create_surface()` (wayland surface 创建)

---

### 2. `device.rs`
**职责**：物理设备选择 + 逻辑设备创建 + Queue 管理

**接口**：
```rust
pub struct DeviceManager {
    pub device: Device,                          // 逻辑设备
    pub physical_device: vk::PhysicalDevice,   // 物理设备
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub graphics_family: u32,
    pub present_family: u32,
}

impl DeviceManager {
    /// 选择物理设备并创建逻辑设备，一次性返回所有设备相关资源
    pub fn create(
        instance: &Instance, 
        surface: vk::SurfaceKHR
    ) -> Result<Self>;
    
    pub fn destroy(&self);
}
```

**设计说明**：
- `Device` 和 `Queue` 生命周期绑定，Queue 从 Device 创建，Device 销毁后 Queue 失效
- 几乎所有用 Queue 的地方（`queue_submit`、`queue_present`）都需要 Device，放在一起使用方便
- 创建时一次性完成：选择物理设备 → 创建逻辑设备 → 获取队列

**包含当前代码**：
- `pick_physical_device()`
- `check_physical_device()`
- `check_physical_device_extensions()`
- `QueueFamilyindices` 结构体和方法
- `create_logical_device()`

---

### 3. `swapchain.rs`
**职责**：Swapchain 和相关对象管理

**接口**：
```rust
pub struct Swapchain {
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

impl Swapchain {
    pub fn create_for_winit(window: &Window, instance: &Instance, device: &Device, surface: vk::SurfaceKHR) -> Result<Self>;
    pub fn create_for_wayland(width: u32, height: u32, instance: &Instance, device: &Device, surface: vk::SurfaceKHR) -> Result<Self>;
    pub fn destroy(&self, device: &Device);
}

// 辅助结构体
pub struct SwapChainSupport { ... }
```

**包含当前代码**：
- `create_swapchain()` / `create_swapchain_wayland()`
- `create_swapchain_image_view()`
- `get_swapchain_surface_format()`
- `get_swapchain_present_mode()`
- `get_swapchain_extent()` / `get_swapchain_extent_wayland()`
- `SwapChainSupport` 结构体

---

### 4. `pipeline.rs`
**职责**：渲染管线和 RenderPass

**接口**：
```rust
pub struct Pipeline {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
}

impl Pipeline {
    pub fn create(device: &Device, swapchain_format: vk::Format, swapchain_extent: vk::Extent2D, msaa_samples: vk::SampleCountFlags, descriptor_set_layout: vk::DescriptorSetLayout) -> Result<Self>;
    pub fn destroy(&self, device: &Device);
}
```

**包含当前代码**：
- `create_render_pass()`
- `create_pipeline()`
- `create_shader_module()`

---

### 5. `buffer.rs`
**职责**：顶点/索引 Buffer 创建

**接口**：
```rust
pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

impl VertexBuffer {
    pub fn create(instance: &Instance, device: &Device, data: &ContextData) -> Result<Self>;
    pub fn destroy(&self, device: &Device);
}

impl IndexBuffer {
    pub fn create(instance: &Instance, device: &Device, data: &ContextData) -> Result<Self>;
    pub fn destroy(&self, device: &Device);
}
```

**包含当前代码**：
- `create_vertex_buffer()`
- `create_index_buffer()`
- `copy_buffer()` (可能需要移到 tool.rs)

---

### 6. `command.rs`
**职责**：CommandPool 和 CommandBuffer

**接口**：
```rust
pub struct CommandManager {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}

impl CommandManager {
    pub fn create(instance: &Instance, device: &Device, graphics_family: u32, frame_count: usize) -> Result<Self>;
    pub fn record(&self, device: &Device, framebuffers: &[vk::Framebuffer], render_pass: vk::Pipeline, ...) -> Result<()>;
    pub fn destroy(&self, device: &Device);
}
```

**包含当前代码**：
- `create_command_pool()`
- `create_command_buffers()`

---

### 7. `sync.rs`
**职责**：同步对象（Semaphore、Fence）

**接口**：
```rust
pub struct SyncObjects {
    pub image_available: Vec<vk::Semaphore>,
    pub render_finished: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
}

impl SyncObjects {
    pub fn create(device: &Device, image_count: usize) -> Result<Self>;
    pub fn destroy(&self, device: &Device);
}
```

**包含当前代码**：
- `create_sync_objects()`

---

### 8. `descriptor.rs`
**职责**：Descriptor 相关

**接口**：
```rust
pub struct DescriptorManager {
    pub layout: vk::DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
}

impl DescriptorManager {
    pub fn create(device: &Device, image_count: usize) -> Result<Self>;
    pub fn update(&self, device: &Device, uniform_buffers: &[vk::Buffer], texture_view: vk::ImageView, texture_sampler: vk::Sampler);
    pub fn destroy(&self, device: &Device);
}
```

**包含当前代码**：
- `create_descriptor_set_layout()`
- `create_descriptor_pool()`
- `create_descriptor_sets()`

---

### 9. `render.rs`
**职责**：渲染循环逻辑

**接口**：
```rust
pub struct Renderer {
    // 引用其他模块创建的对象
}

impl Renderer {
    pub fn render_wayland(&mut self) -> Result<()>;
    pub fn render(&mut self, window: &Window) -> Result<()>;
    pub fn update_uniform_buffer(&mut self, image_index: usize, start: Instant, extent: vk::Extent2D) -> Result<()>;
}
```

**包含当前代码**：
- `render_wayland()`
- `render()`
- `update_uniform_buffer()`

---

### 10. `context.rs`（简化后）
**职责**：协调各个模块，作为主入口

**结构**：
```rust
pub struct Context {
    instance: Instance,
    device: Device,
    
    // 子模块
    swapchain: Swapchain,
    pipeline: Pipeline,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    command_manager: CommandManager,
    descriptor_manager: DescriptorManager,
    sync_objects: SyncObjects,
    renderer: Renderer,
    
    // 其他资源
    frame: usize,
    start: Instant,
    image: DynamicImage,
}
```

**方法**：
- `create_for_wayland()` - 协调各模块创建
- `create()` - winit 版本
- `render_wayland()` - 委托给 Renderer
- `render()` - 委托给 Renderer
- `destroy()` - 按顺序销毁各模块

---

### 11. `resource.rs`（可选）
**职责**：统一资源管理，避免重复代码

可以考虑将 texture、uniform buffer、framebuffer 的创建抽象为资源管理器。

## 重构步骤

### Phase 1: 创建基础模块（不改动现有代码）
1. 创建 `instance.rs`，复制 `create_instance` 相关代码
2. 创建 `device.rs`，复制设备相关代码
3. 创建 `swapchain.rs`，复制 swapchain 相关代码

### Phase 2: 创建渲染相关模块
4. 创建 `pipeline.rs`
5. 创建 `descriptor.rs`
6. 创建 `buffer.rs`
7. 创建 `command.rs`
8. 创建 `sync.rs`

### Phase 3: 重构主 Context
9. 创建新的 `context.rs`，使用新模块
10. 创建 `render.rs`，提取渲染逻辑
11. 更新 `mod.rs`

### Phase 4: 清理和测试
12. 删除旧 `context.rs`（备份）
13. 验证 winit 和 wayland 两种模式都能正常运行
14. 清理未使用的代码

## 注意事项

1. **依赖管理** - 各模块之间要明确定义依赖关系，避免循环依赖
2. **错误处理** - 保持现有的 `anyhow::Result` 错误处理方式
3. **资源销毁** - 确保 `destroy()` 顺序正确，先销毁依赖对象
4. **测试** - 每完成一个模块都要测试编译通过
5. **性能** - 重构不应影响运行时性能，只是代码组织方式改变

## 预期结果

重构后代码行数分布（估算）：

| 文件 | 行数（估算） | 职责 |
|------|-------------|------|
| context.rs | ~150 | 协调各模块 |
| instance.rs | ~100 | 实例创建 |
| device.rs | ~150 | 设备管理 |
| swapchain.rs | ~200 | Swapchain |
| pipeline.rs | ~150 | 管线 |
| buffer.rs | ~100 | Buffer |
| command.rs | ~100 | Command |
| sync.rs | ~50 | 同步对象 |
| descriptor.rs | ~100 | Descriptor |
| render.rs | ~150 | 渲染循环 |
| **总计** | **~1250** | 分散到 10 个文件 |

（当前 context.rs 单独就有 1500+ 行）
