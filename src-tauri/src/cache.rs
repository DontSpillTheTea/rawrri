use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::models::ScanResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFolderScan {
    pub folder: String,
    pub recursive: bool,
    pub pairing_threshold_ms: i64,
    pub result: ScanResult,
}

fn cache_file_path() -> Option<PathBuf> {
    let base = dirs::data_local_dir()?;
    Some(base.join("rawrii").join("cache.json"))
}

pub fn load_cached_scan(folder: &str, recursive: bool, pairing_threshold_ms: i64) -> Option<ScanResult> {
    let cache_file = cache_file_path()?;
    let raw = fs::read_to_string(cache_file).ok()?;
    let entries = serde_json::from_str::<Vec<CachedFolderScan>>(&raw).ok()?;
    entries
        .into_iter()
        .find(|entry| {
            entry.folder.eq_ignore_ascii_case(folder)
                && entry.recursive == recursive
                && entry.pairing_threshold_ms == pairing_threshold_ms
        })
        .map(|entry| entry.result)
}

pub fn save_cached_scan(folder: &str, recursive: bool, pairing_threshold_ms: i64, result: &ScanResult) -> Result<(), String> {
    let cache_file = cache_file_path().ok_or_else(|| "Cannot resolve local app data path".to_string())?;
    let parent = Path::new(&cache_file)
        .parent()
        .ok_or_else(|| "Cannot resolve cache parent path".to_string())?;
    fs::create_dir_all(parent).map_err(|err| format!("cache_create_dir_error: {err}"))?;

    let mut entries = if cache_file.exists() {
        let raw = fs::read_to_string(&cache_file).unwrap_or_default();
        serde_json::from_str::<Vec<CachedFolderScan>>(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };

    entries.retain(|entry| {
        !(entry.folder.eq_ignore_ascii_case(folder)
            && entry.recursive == recursive
            && entry.pairing_threshold_ms == pairing_threshold_ms)
    });
    entries.push(CachedFolderScan {
        folder: folder.to_string(),
        recursive,
        pairing_threshold_ms,
        result: result.clone(),
    });

    let raw = serde_json::to_string_pretty(&entries).map_err(|err| format!("cache_serialize_error: {err}"))?;
    fs::write(cache_file, raw).map_err(|err| format!("cache_write_error: {err}"))?;
    Ok(())
}
