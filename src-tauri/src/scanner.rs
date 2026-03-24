use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{fs, path::Path};

use chrono::{DateTime, Utc};
use walkdir::WalkDir;

use crate::{
    filename_parser::{parse_k6_filename_with_error, ParseFilenameError},
    models::{HealthStatus, ParseStatus, ScanDiagnostics, ScanResult, VideoAsset},
    pairing::{build_pairs, PairingConfig},
};

pub fn scan_folder(root_path: &str, recursive: bool, pairing_threshold_ms: i64) -> Result<ScanResult, String> {
    let root = Path::new(root_path);
    if !root.exists() {
        return Err(format!("Path does not exist: {root_path}"));
    }
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {root_path}"));
    }

    let mut assets: Vec<VideoAsset> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut total_files_discovered = 0usize;
    let mut parser_matched_files = 0usize;
    let mut parser_skipped_files = 0usize;
    let mut parser_failed_files = 0usize;

    let max_depth = if recursive { usize::MAX } else { 1 };
    let mut walker = WalkDir::new(root);
    walker = walker.max_depth(max_depth);

    for entry_result in walker {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(err) => {
                errors.push(format!("walk_error: {err}"));
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }
        total_files_discovered += 1;

        let file_name = entry.file_name().to_string_lossy().to_string();
        if !is_mp4_candidate(&file_name) {
            parser_skipped_files += 1;
            continue;
        }

        let parsed = match parse_k6_filename_with_error(&file_name) {
            Ok(parsed) => {
                parser_matched_files += 1;
                parsed
            }
            Err(ParseFilenameError::NoPatternMatch) => {
                parser_skipped_files += 1;
                continue;
            }
            Err(err) => {
                parser_failed_files += 1;
                errors.push(format!("parse_error [{}]: {:?}", entry.path().display(), err));
                continue;
            }
        };

        let metadata = match fs::metadata(entry.path()) {
            Ok(metadata) => metadata,
            Err(err) => {
                errors.push(format!("metadata_error [{}]: {err}", entry.path().display()));
                continue;
            }
        };

        let modified_iso = metadata
            .modified()
            .ok()
            .map(|time| DateTime::<Utc>::from(time).to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        assets.push(VideoAsset {
            id: deterministic_asset_id(entry.path().to_string_lossy().as_ref()),
            path: entry.path().to_string_lossy().to_string(),
            filename: file_name,
            side: parsed.side,
            parse_status: ParseStatus::Parsed,
            parsed_sequence: parsed.sequence,
            raw_timestamp_string: Some(parsed.raw_timestamp_string),
            parsed_timestamp: parsed
                .timestamp
                .map(|value| value.format("%Y-%m-%dT%H:%M:%S").to_string()),
            duration_sec: None,
            resolution: None,
            codec: None,
            size_bytes: metadata.len(),
            modified_at: modified_iso,
            health: HealthStatus::Ok,
            warnings: Vec::new(),
        });
    }

    let pairs = build_pairs(
        &assets,
        PairingConfig {
            threshold_ms: pairing_threshold_ms,
        },
        root_path,
    );

    let valid_pairs = pairs
        .iter()
        .filter(|pair| pair.front_asset_id.is_some() && pair.rear_asset_id.is_some())
        .count();
    let partial_pairs = pairs.len().saturating_sub(valid_pairs);

    println!("scan_folder selected={}", root_path);
    println!("scan_folder total_files_discovered={}", total_files_discovered);
    println!("scan_folder parser_matched_files={}", parser_matched_files);
    println!("scan_folder parser_skipped_files={}", parser_skipped_files);
    println!("scan_folder parser_failed_files={}", parser_failed_files);
    println!("scan_folder valid_pairs={}", valid_pairs);
    println!("scan_folder partial_pairs={}", partial_pairs);
    println!("scan_folder pairing_threshold_ms={}", pairing_threshold_ms);

    Ok(ScanResult {
        root_path: root_path.to_string(),
        scanned_at: Utc::now().to_rfc3339(),
        assets,
        pairs,
        diagnostics: ScanDiagnostics {
            total_files_discovered,
            parser_matched_files,
            parser_skipped_files,
            parser_failed_files,
            valid_pairs,
            partial_pairs,
            pairing_threshold_ms,
        },
        errors,
    })
}

fn is_mp4_candidate(file_name: &str) -> bool {
    file_name.to_ascii_lowercase().ends_with(".mp4")
}

fn deterministic_asset_id(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("asset_{:x}", hasher.finish())
}
