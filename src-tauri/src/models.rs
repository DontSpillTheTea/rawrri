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
    pub metadata: Option<MediaMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    pub duration_sec: f64,
    pub creation_time: Option<String>,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub has_audio: bool,
    pub is_corrupt: bool,
    pub stream_count: usize,
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
    pub observations: Vec<ObservationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationEvent {
    pub id: String,
    pub asset_id: String,
    pub pair_id: String,
    pub start_time_sec: f64,
    pub end_time_sec: f64,
    pub pair_canonical_time_sec: f64,
    pub observation_type: ObservationType,
    pub confidence: f64,
    pub bounding_box: Option<Rect>,
    pub is_user_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObservationType {
    Vehicle {
        color: Option<String>,
        vehicle_type: String,
    },
    LicensePlate {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
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
