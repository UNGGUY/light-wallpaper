# 壁纸切换功能修改记录

## 新增文件

### src/wallpaper/README.md
- 设计文档，包含方案 A/B/C 的对比和选型

### src/wallpaper/manager.rs
- 新增 `Manager` 结构体：壁纸列表管理
- 新增 `PlayMode` 枚举：Sequential/Random/Single
- 实现目录扫描功能（支持 png/jpg/jpeg/webp/bmp/gif）
- 实现定时自动切换逻辑

### src/wallpaper/mod.rs
- 模块导出，暴露 `Manager` 和 `PlayMode`

---

## 修改文件

### src/context/context.rs

#### 新增方法
- `impl Context` 块中新增 `reload_texture()` 方法
- 位置：在 `destroy()` 方法之前
- 功能：运行时销毁旧纹理并加载新纹理

#### 方法内部调用链
1. `self.device.device_wait_idle()` - 等待 GPU 空闲
2. `self.device.destroy_image_view()` - 销毁旧 image view
3. `self.device.destroy_image()` - 销毁旧 image
4. `self.device.free_memory()` - 释放旧 image 内存
5. `self.device.destroy_sampler()` - 销毁旧 sampler
6. `texture::read_image()` - 读取新图片文件
7. `texture::create_texture_image()` - 创建新纹理
8. `texture::create_texture_image_view()` - 创建新 view
9. `texture::create_texture_sampler()` - 创建新 sampler
10. `self.data.descriptor_manager.update()` - 更新描述符集

---

## 待完成集成（未修改）

### src/main.rs 或 src/wayland/wayland.rs
- 需导入 `wallpaper::Manager`
- 需在初始化时创建 Manager 实例
- 需在主循环中每帧调用 `manager.update()`
- 需处理 `update()` 返回的 `Some(path)` 并调用 `context.reload_texture(path)`

### Cargo.toml
- 需确认已依赖 `tempfile` crate（用于 manager.rs 的测试）

---

## 后续扩展路径

1. **方案 B（过渡动画）**需修改：
   - `shader/shader.frag` - 添加第二张纹理和 blend uniform
   - `src/context/context.rs` - ContextData 添加第二组纹理资源
   - `src/context/descriptor.rs` - 描述符集布局添加新绑定

2. **方案 C（完整功能）**需修改：
   - 添加配置文件解析
   - 添加 IPC/命令行控制接口
