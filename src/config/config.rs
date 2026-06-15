use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use shellexpand;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct WallpaperConfig {
    pub path: PathBuf,
    pub shader: PathBuf,
}

impl WallpaperConfig {
    pub fn load() -> Result<Self> {
        let base_path = dirs::config_dir()
            .context("dirs err")?
            .join("lightwallpaper");

        let full_path = base_path.join("config.toml");

        println!("{:?}", full_path);

        let config = Config::builder()
            .add_source(File::from(full_path.clone()))
            .build()?;

        let mut settings: Self = config.try_deserialize()?;

        settings.path =
            PathBuf::from(shellexpand::tilde(&settings.path.to_string_lossy()).into_owned());
        settings.shader =
            PathBuf::from(shellexpand::tilde(&settings.shader.to_string_lossy()).into_owned());

        Ok(settings)
    }
}
