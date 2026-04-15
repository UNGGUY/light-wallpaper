use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// 播放模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Sequential,
    Random,
    Single,
}

/// 壁纸管理器
pub struct Manager {
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

impl Manager {
    /// 支持的图片格式扩展名
    const SUPPORTED_EXTENSIONS: &'static [&'static str] =
        &["png", "jpg", "jpeg", "webp", "bmp", "gif"];

    /// 创建管理器，扫描指定目录
    pub fn new(directory: &Path, interval_secs: u64) -> Result<Self> {
        let wallpapers = Self::scan_directory(directory)?;

        if wallpapers.is_empty() {
            anyhow::bail!("No supported wallpaper images found in: {:?}", directory);
        }

        Ok(Self {
            wallpapers,
            current_index: 0,
            last_switch: Instant::now(),
            interval: Duration::from_secs(interval_secs),
            mode: PlayMode::Sequential,
        })
    }

    /// 扫描目录收集支持的图片文件
    fn scan_directory(directory: &Path) -> Result<Vec<PathBuf>> {
        let mut wallpapers = Vec::new();

        if !directory.exists() {
            anyhow::bail!("Directory does not exist: {:?}", directory);
        }

        if !directory.is_dir() {
            anyhow::bail!("Path is not a directory: {:?}", directory);
        }

        for entry in std::fs::read_dir(directory)
            .with_context(|| format!("Failed to read directory: {:?}", directory))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && Self::is_supported_image(&path) {
                println!("{0}", &path.to_str().unwrap());
                wallpapers.push(path);
            }
        }

        // 按文件名排序，确保顺序一致
        wallpapers.sort();

        Ok(wallpapers)
    }

    /// 检查文件是否是支持的图片格式
    fn is_supported_image(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return Self::SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str());
            }
        }
        false
    }

    /// 每帧调用，检查是否需要切换
    /// 返回：Some(path) 表示需要切换到指定壁纸，None 表示保持当前
    pub fn update(&mut self) -> Option<&Path> {
        // 单张模式不自动切换
        if self.mode == PlayMode::Single {
            return None;
        }

        let now = Instant::now();
        if now.duration_since(self.last_switch) >= self.interval {
            self.last_switch = now;
            let next_path = self.move_to_next();

            return Some(next_path);
        }

        None
    }

    /// 获取下一个索引（根据播放模式）
    fn next_index(&self) -> usize {
        match self.mode {
            PlayMode::Sequential => {
                // 顺序模式：循环到下一个
                (self.current_index + 1) % self.wallpapers.len()
            }
            PlayMode::Random => {
                // 随机模式：随机选择一个（可能和当前相同）
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let now = Instant::now();
                let mut hasher = DefaultHasher::new();
                now.hash(&mut hasher);
                let hash = hasher.finish();
                (hash as usize) % self.wallpapers.len()
            }
            PlayMode::Single => self.current_index,
        }
    }

    /// 移动到下一个壁纸，返回新壁纸路径
    fn move_to_next(&mut self) -> &Path {
        self.current_index = self.next_index();
        &self.wallpapers[self.current_index]
    }

    /// 手动切换到下一个
    pub fn next(&mut self) -> &Path {
        self.current_index = self.next_index();
        self.last_switch = Instant::now();
        &self.wallpapers[self.current_index]
    }

    /// 手动切换到上一个
    pub fn prev(&mut self) -> &Path {
        if self.wallpapers.len() <= 1 {
            return &self.wallpapers[self.current_index];
        }

        self.current_index = if self.current_index == 0 {
            self.wallpapers.len() - 1
        } else {
            self.current_index - 1
        };
        self.last_switch = Instant::now();
        &self.wallpapers[self.current_index]
    }

    /// 获取当前壁纸路径
    pub fn current(&self) -> &Path {
        &self.wallpapers[self.current_index]
    }

    /// 设置播放模式
    pub fn set_mode(&mut self, mode: PlayMode) {
        self.mode = mode;
    }

    /// 获取播放模式
    pub fn mode(&self) -> PlayMode {
        self.mode
    }

    /// 设置切换间隔
    pub fn set_interval(&mut self, secs: u64) {
        self.interval = Duration::from_secs(secs);
    }

    /// 获取切换间隔（秒）
    pub fn interval_secs(&self) -> u64 {
        self.interval.as_secs()
    }

    /// 获取壁纸总数
    pub fn len(&self) -> usize {
        self.wallpapers.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.wallpapers.is_empty()
    }

    /// 获取所有壁纸路径（只读）
    pub fn wallpapers(&self) -> &[PathBuf] {
        &self.wallpapers
    }

    /// 获取当前索引
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// 手动切换到指定索引
    pub fn switch_to(&mut self, index: usize) -> Result<&Path> {
        if index >= self.wallpapers.len() {
            anyhow::bail!(
                "Index out of bounds: {} (total: {})",
                index,
                self.wallpapers.len()
            );
        }
        self.current_index = index;
        self.last_switch = Instant::now();
        Ok(&self.wallpapers[self.current_index])
    }
}
