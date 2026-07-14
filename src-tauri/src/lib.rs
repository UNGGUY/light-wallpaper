use std::path::Path;

use std::sync::mpsc::{self, channel};

use config::WallpaperConfig;
use tauri::Manager;
use wayland::State;

use music::music::AudioCommand;
use music::music::MusicManager;

mod config;
mod context;
mod music;
mod wallpaper;
mod wayland;

/// Tauri 托管状态：音频命令发送器
struct MusicState {
    tx: mpsc::Sender<AudioCommand>,
}

/// 启动壁纸引擎（由 setup 钩子在应用启动时自动调用）
fn start_engine(app: &tauri::AppHandle) {
    let Ok(config) = WallpaperConfig::load() else {
        eprintln!("[light-wallpaper] Failed to load config");
        return;
    };

    let image_path = Path::new(&config.image_path);
    let audio_path = Path::new(&config.audio_path);

    let Ok(music_manager) = MusicManager::new(audio_path, 0.3) else {
        eprintln!("[light-wallpaper] Failed to init music manager");
        return;
    };
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

    let (tx, rx) = channel::<AudioCommand>();

    // 启动音频线程
    let _audio_handle = MusicManager::begin(rx, music_manager);

    // 启动壁纸线程
    let _wayland_handle = State::begin(state, image_path.to_path_buf(), config);

    // 保存 sender 到 Tauri 状态，供后续命令使用
    app.manage(MusicState { tx });
}

// ── 音乐控制命令 ──

#[tauri::command]
fn resume_music(music_state: tauri::State<'_, MusicState>) {
    let _ = music_state.tx.send(AudioCommand::Resume);
}

#[tauri::command]
fn pause_music(music_state: tauri::State<'_, MusicState>) {
    let _ = music_state.tx.send(AudioCommand::Stop);
}

#[tauri::command]
fn next_track(music_state: tauri::State<'_, MusicState>) {
    let _ = music_state.tx.send(AudioCommand::Next);
}

#[tauri::command]
fn prev_track(music_state: tauri::State<'_, MusicState>) {
    let _ = music_state.tx.send(AudioCommand::Prev);
}

// ── 应用入口 ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            start_engine(&app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            resume_music,
            pause_music,
            next_track,
            prev_track
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
