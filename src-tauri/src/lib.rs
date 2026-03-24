mod cache;
mod export;
mod filename_parser;
mod logging;
mod metadata;
mod models;
mod pairing;
mod playback;
mod scanner;
mod settings;
mod state;
mod video_surface;

use playback::{PlaybackController, PlaybackSnapshot};
use scanner::scan_folder as scan_folder_impl;
use tauri::State;
use video_surface::{VideoRect, VideoSurfaceController, VideoSurfaceSnapshot};

#[tauri::command]
fn scan_folder(root_path: String, recursive: Option<bool>, pairing_threshold_ms: Option<i64>) -> Result<models::ScanResult, String> {
    let recursive = recursive.unwrap_or(false);
    let threshold = pairing_threshold_ms.unwrap_or(3000);

    if let Some(cached) = cache::load_cached_scan(&root_path, recursive, threshold) {
        println!("scan_folder cache_hit folder={} recursive={} threshold_ms={}", root_path, recursive, threshold);
        return Ok(cached);
    }

    println!("scan_folder cache_miss folder={} recursive={} threshold_ms={}", root_path, recursive, threshold);
    let result = scan_folder_impl(&root_path, recursive, threshold)?;
    let _ = cache::save_cached_scan(&root_path, recursive, threshold, &result);
    Ok(result)
}

#[tauri::command]
fn playback_load_pair(
    playback: State<'_, PlaybackController>,
    surfaces: State<'_, VideoSurfaceController>,
    window: tauri::Window,
    pair_id: String,
    front_path: Option<String>,
    rear_path: Option<String>,
) -> Result<PlaybackSnapshot, String> {
    let surface_snapshot = surfaces.ensure_for_window(&window)?;
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    manager.load_pair(
        pair_id,
        front_path,
        rear_path,
        surface_snapshot.front_wid,
        surface_snapshot.rear_wid,
    )
}

#[tauri::command]
fn playback_toggle_play_pause(playback: State<'_, PlaybackController>) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    manager.toggle_playing()
}

#[tauri::command]
fn playback_set_playing(
    playback: State<'_, PlaybackController>,
    is_playing: bool,
) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    manager.set_playing(is_playing)
}

#[tauri::command]
fn playback_seek(playback: State<'_, PlaybackController>, playhead_sec: f64) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    manager.seek_to(playhead_sec)
}

#[tauri::command]
fn playback_set_mute(
    playback: State<'_, PlaybackController>,
    side: String,
    muted: bool,
) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    manager.set_side_muted(&side, muted)
}

#[tauri::command]
fn playback_stop(playback: State<'_, PlaybackController>) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    Ok(manager.stop())
}

#[tauri::command]
fn playback_get_state(playback: State<'_, PlaybackController>) -> Result<PlaybackSnapshot, String> {
    let mut manager = playback
        .manager
        .lock()
        .map_err(|_| "Playback manager lock poisoned".to_string())?;
    Ok(manager.refresh_state())
}

#[tauri::command]
fn update_video_layout(
    surfaces: State<'_, VideoSurfaceController>,
    window: tauri::Window,
    front: VideoRect,
    rear: VideoRect,
) -> Result<VideoSurfaceSnapshot, String> {
    surfaces.update_layout(&window, front, rear)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init_logging();

    tauri::Builder::default()
        .manage(PlaybackController::default())
        .manage(VideoSurfaceController::default())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_folder,
            playback_load_pair,
            playback_toggle_play_pause,
            playback_set_playing,
            playback_seek,
            playback_set_mute,
            playback_stop,
            playback_get_state,
            update_video_layout
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
