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

use scanner::scan_folder as scan_folder_impl;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![scan_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
