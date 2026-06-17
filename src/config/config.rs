use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct WallpaperConfigRaw {
    // 用 String 接收，避免 config crate 对 PathBuf 默认值的类型兼容问题
    pub image_path: Option<String>,
    pub audio_path: Option<String>,
    pub vert_shader: Option<String>,
    pub frag_shader: Option<String>,
}

#[derive(Debug)]
pub struct WallpaperConfig {
    pub image_path: PathBuf,
    pub audio_path: PathBuf,
    pub vert_shader: PathBuf,
    pub frag_shader: PathBuf,
}

impl WallpaperConfig {
    pub fn load() -> Result<Self> {
        let base_path = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("lightwallpaper");

        let full_path = base_path.join("config.toml");
        println!("Config path: {:?}", full_path);

        let config = Config::builder()
            // ✅ 修复拼写错误
            .set_default("image_path", "~/Pictures/assets/wallpapers/")?
            .set_default("audio_path", "~/Music/assets/bgm")?
            .set_default("vert_shader", "shader/vert.spv")?
            .set_default("frag_shader", "shader/frag.spv")?
            // ✅ 配置文件不存在时不报错，回退到默认值
            .add_source(File::from(full_path).required(false))
            .build()
            .context("Failed to build config")?;

        let raw: WallpaperConfigRaw = config
            .try_deserialize()
            .context("Failed to deserialize config")?;

        // ✅ 统一的 tilde 展开 + PathBuf 转换辅助函数
        let expand = |s: Option<String>, fallback: &str| -> PathBuf {
            let path_str = s.as_deref().unwrap_or(fallback);
            PathBuf::from(shellexpand::tilde(path_str).into_owned())
        };

        Ok(Self {
            image_path: expand(raw.image_path, "~/Pictures/assets/wallpapers/"),
            audio_path: expand(raw.audio_path, "~/Music/assets/bgm"),
            vert_shader: expand(raw.vert_shader, "shader/vert.spv"),
            frag_shader: expand(raw.frag_shader, "shader/frag.spv"),
        })
    }
}
