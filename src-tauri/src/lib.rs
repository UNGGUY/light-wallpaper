// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::path::Path;

use std::sync::mpsc::channel;

use config::WallpaperConfig;
use wayland::State;

use crate::music::music::{AudioCommand, MusicManager};

mod config;
mod context;
mod music;
mod wallpaper;
mod wayland;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tauri::command]
fn begin() {
    let state = State {
        running: true,
        base_surface: None,
        configured: false,
        render: false,
        context: None,
        layer_shell: None,
        output: None,
        layer_surface: None,
        width: 0,
        height: 0,
        output_scale: 1,
    };

    let config = WallpaperConfig::load().unwrap();

    let image_path = Path::new(&config.image_path);
    let audio_path = Path::new(&config.audio_path);

    let music_manager = MusicManager::new(audio_path, 0.3).unwrap();

    let (_tx, rx) = channel::<AudioCommand>();

    let audio_handle = MusicManager::begin(rx, music_manager);

    let wayland_handle = State::begin(state, image_path.to_path_buf(), config);

    audio_handle.join().unwrap();
    wayland_handle.join().unwrap();

    println!("init finish");
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![begin])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
