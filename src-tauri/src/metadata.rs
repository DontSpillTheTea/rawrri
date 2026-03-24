use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetadataSnapshot {
    pub duration_sec: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub codec: Option<String>,
}

// Phase 2: implement ffprobe-backed extraction in background worker.
pub fn extract_metadata_placeholder() -> MetadataSnapshot {
    MetadataSnapshot::default()
}
