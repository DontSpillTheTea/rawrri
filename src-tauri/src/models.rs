use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VideoSide {
    Front,
    Rear,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Parsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoAsset {
    pub id: String,
    pub path: String,
    pub filename: String,
    pub side: VideoSide,
    pub parse_status: ParseStatus,
    pub parsed_sequence: Option<u32>,
    pub raw_timestamp_string: Option<String>,
    pub parsed_timestamp: Option<String>,
    pub duration_sec: Option<f64>,
    pub resolution: Option<Resolution>,
    pub codec: Option<String>,
    pub size_bytes: u64,
    pub modified_at: String,
    pub health: HealthStatus,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingPair {
    pub id: String,
    pub front_asset_id: Option<String>,
    pub rear_asset_id: Option<String>,
    pub canonical_start_time: Option<String>,
    pub estimated_duration_sec: Option<f64>,
    pub pairing_confidence: f64,
    pub pairing_reason: String,
    pub source_folder: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanDiagnostics {
    pub total_files_discovered: usize,
    pub parser_matched_files: usize,
    pub parser_skipped_files: usize,
    pub parser_failed_files: usize,
    pub valid_pairs: usize,
    pub partial_pairs: usize,
    pub pairing_threshold_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub root_path: String,
    pub scanned_at: String,
    pub assets: Vec<VideoAsset>,
    pub pairs: Vec<RecordingPair>,
    pub diagnostics: ScanDiagnostics,
    pub errors: Vec<String>,
}
