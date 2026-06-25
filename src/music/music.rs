use anyhow::{Context as _, Result};
use rodio::{Decoder, MixerDeviceSink, Player};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub enum MusicPlayMode {
    Sequential, // 顺序播放
    Random,     // 随机
    Single,     // 单曲循环
    Off,        // 关闭
}

pub enum AudioCommand {
    Stop,
    Resume,
    Next,
    Prev,
}

pub struct MusicManager {
    handle: MixerDeviceSink, // 必须保持存活，音频输出设备句柄
    tracks: Vec<PathBuf>,    // 音乐文件列表
    current_index: usize,    // 当前曲目索引
    mode: MusicPlayMode,     // 播放模式
    volume: f32,             // 音量 0.0 ~ 1.0
}

impl MusicManager {
    /// 支持的音频格式扩展名
    const SUPPORTED_EXTENSIONS: &'static [&'static str] =
        &["mp3", "wav", "flac", "ogg", "m4a", "aac"];

    /// 启动音频播放线程，消费 MusicManager 的所有权。
    /// 返回 JoinHandle，主线程可借此等待音频线程结束。
    pub fn begin(rx: Receiver<AudioCommand>, mut music_manager: MusicManager) -> JoinHandle<()> {
        thread::spawn(move || {
            // 从 MusicManager 已有的 handle 创建 Player
            let player = Player::connect_new(music_manager.handle.mixer());
            player.set_volume(music_manager.volume);

            // 加载并播放第一首曲目
            if !music_manager.tracks.is_empty() {
                let path = &music_manager.tracks[music_manager.current_index];
                if let Ok(file) = File::open(path) {
                    if let Ok(source) = Decoder::try_from(file) {
                        player.append(source);
                    } else {
                        eprintln!("[Music] Failed to decode: {:?}", path);
                    }
                } else {
                    eprintln!("[Music] Failed to open: {:?}", path);
                }
            }

            loop {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(command) => match command {
                        AudioCommand::Resume => {
                            player.play();
                        }
                        AudioCommand::Stop => {
                            player.stop();
                        }
                        AudioCommand::Next => {
                            music_manager.play_next_track(&player);
                        }
                        AudioCommand::Prev => {
                            music_manager.play_prev_track(&player);
                        }
                    },
                    Err(RecvTimeoutError::Timeout) => {
                        // 超时：检查当前曲目是否播完，自动切下一首
                        if player.empty()
                            && !music_manager.tracks.is_empty()
                            && !matches!(music_manager.mode, MusicPlayMode::Off)
                        {
                            music_manager.advance_and_play(&player);
                        }
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        // 发送端已断开，退出播放线程
                        player.stop();
                        break;
                    }
                }
            }
        })
    }

    /// 播放下一首曲目
    fn play_next_track(&mut self, player: &Player) {
        if self.tracks.is_empty() {
            return;
        }
        player.stop();
        self.advance_index();
        self.load_and_play(player);
    }

    /// 播放上一首曲目
    fn play_prev_track(&mut self, player: &Player) {
        if self.tracks.is_empty() {
            return;
        }
        player.stop();
        // 如果当前播放超过 2 秒则重放当前曲目，否则切到上一首
        if player.get_pos() > Duration::from_secs(2) {
            // 重放当前曲目
        } else if self.current_index == 0 {
            self.current_index = self.tracks.len() - 1;
        } else {
            self.current_index -= 1;
        }
        self.load_and_play(player);
    }

    /// 自动前进索引并播放（用于曲目自然结束后的自动切换）
    fn advance_and_play(&mut self, player: &Player) {
        self.advance_index();
        self.load_and_play(player);
    }

    /// 根据播放模式计算下一个索引
    fn advance_index(&mut self) {
        self.current_index = match self.mode {
            MusicPlayMode::Sequential => (self.current_index + 1) % self.tracks.len(),
            MusicPlayMode::Random => {
                // 基于当前时间的简单随机
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos();
                (seed as usize) % self.tracks.len()
            }
            MusicPlayMode::Single => {
                // 单曲循环：索引不变
                self.current_index
            }
            MusicPlayMode::Off => self.current_index,
        };
    }

    /// 加载当前索引的音频文件并追加到播放队列
    fn load_and_play(&self, player: &Player) {
        let path = &self.tracks[self.current_index];
        match File::open(path) {
            Ok(file) => match Decoder::try_from(file) {
                Ok(source) => {
                    player.append(source);
                }
                Err(e) => {
                    eprintln!("[Music] Failed to decode {:?}: {e}", path);
                }
            },
            Err(e) => {
                eprintln!("[Music] Failed to open {:?}: {e}", path);
            }
        }
    }

    /// 创建一个新的 MusicManager，扫描目录并打开默认音频设备。
    /// 注意：此时仅完成初始化，播放需调用 `begin()` 启动独立线程。
    pub fn new(directory: &Path, volume: f32) -> Result<Self> {
        let handle =
            rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| anyhow::anyhow!("{e}"))?;

        let tracks = Self::scan_directory(&directory)?;
        Ok(Self {
            handle,
            tracks,
            current_index: 0,
            mode: MusicPlayMode::Sequential,
            volume,
        })
    }

    /// 获取当前播放模式
    pub fn mode(&self) -> &MusicPlayMode {
        &self.mode
    }

    /// 设置播放模式
    pub fn set_mode(&mut self, mode: MusicPlayMode) {
        self.mode = mode;
    }

    /// 获取音量
    pub fn volume(&self) -> f32 {
        self.volume
    }

    /// 获取曲目总数
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// 扫描目录收集支持的音频文件
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

            if path.is_file() && Self::is_supported_audio(&path) {
                tracks.push(path);
            }
        }

        // 按文件名排序，确保顺序一致
        tracks.sort();

        Ok(tracks)
    }

    /// 检查文件是否是支持的音频格式
    fn is_supported_audio(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return Self::SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str());
            }
        }
        false
    }
}
