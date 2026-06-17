use anyhow::{Context as _, Result};
use rodio::{Decoder, MixerDeviceSink};
use std::fs::File;
use std::path::{Path, PathBuf};

pub enum MusicPlayMode {
    Sequential, // 顺序播放
    Random,     // 随机
    Single,     // 单曲循环
    Off,        // 关闭
}

pub struct MusicManager {
    handle: MixerDeviceSink, // rodio 音频输出（内部独立线程）
    tracks: Vec<PathBuf>,    // 音乐文件列表
    current_index: usize,    // 当前曲目索引
    mode: MusicPlayMode,     // 播放模式
    volume: f32,             // 音量 0.0 ~ 1.0
    enabled: bool,           // 是否启用
}

impl MusicManager {
    /// 支持的图片格式扩展名
    const SUPPORTED_EXTENSIONS: &'static [&'static str] =
        &["mp3", "wav", "flac", "ogg", "m4a", "acc"];

    pub fn new(directory: &Path, volume: f32) -> Result<Self> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().expect("open sink error");

        let tracks = Self::scan_directory(&directory)?;
        Ok(Self {
            handle: handle,
            tracks: tracks,
            current_index: 0,
            mode: MusicPlayMode::Sequential,
            volume: volume,
            enabled: false,
        })
    }

    pub fn play(&self) -> Result<()> {
        let player = rodio::Player::connect_new(&self.handle.mixer());

        let file = File::open(&self.tracks[self.current_index])?;

        let source = Decoder::try_from(file)?;

        self.handle.mixer().add(source);

        println!("play music");

        Ok(())
    }

    /// 扫描目录收集支持的图片文件
    fn scan_directory(directory: &Path) -> Result<Vec<PathBuf>> {
        let mut tracks = Vec::new();

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
                tracks.push(path);
            }
        }

        // 按文件名排序，确保顺序一致
        tracks.sort();

        Ok(tracks)
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
}
