use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub pairing_threshold_ms: i64,
    pub seek_small_sec: f64,
    pub seek_large_sec: f64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pairing_threshold_ms: 3000,
            seek_small_sec: 1.0,
            seek_large_sec: 5.0,
        }
    }
}
