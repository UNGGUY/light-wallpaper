use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use shellexpand;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct WallpaperConfigRaw {
    pub path: PathBuf,
    pub vert_shader: PathBuf,
    pub frag_shader: PathBuf,
}

#[derive(Debug)]
pub struct WallpaperConfig {
    pub path: PathBuf,
    pub vert_shader: PathBuf,
    pub frag_shader: PathBuf,
}

impl WallpaperConfig {
    pub fn load() -> Result<Self> {
        let base_path = dirs::config_dir()
            .context("dirs err")?
            .join("lightwallpaper");

        let full_path = base_path.join("config.toml");

        println!("{:?}", full_path);

        let config = Config::builder()
            .set_default("path", "~/Pictures/assets/wallpapers/")?
            .set_default("vert_shader", "shader/vert.spv")?
            .set_default("frag_shader", "shader/frag.spv")?
            .add_source(File::from(full_path.clone()))
            .build()?;

        let settings: WallpaperConfigRaw = config.try_deserialize()?;

        Ok(Self {
            path: PathBuf::from(shellexpand::tilde(&settings.path.to_string_lossy()).into_owned()),
            vert_shader: PathBuf::from(
                shellexpand::tilde(&settings.vert_shader.to_string_lossy()).into_owned(),
            ),
            frag_shader: PathBuf::from(
                shellexpand::tilde(&settings.frag_shader.to_string_lossy()).into_owned(),
            ),
        })
    }
}
