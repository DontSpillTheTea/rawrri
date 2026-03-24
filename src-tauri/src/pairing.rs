use chrono::NaiveDateTime;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::models::{RecordingPair, VideoAsset, VideoSide};

#[derive(Debug, Clone)]
pub struct PairingConfig {
    pub threshold_ms: i64,
}

pub fn build_pairs(assets: &[VideoAsset], config: PairingConfig, source_folder: &str) -> Vec<RecordingPair> {
    let mut fronts = Vec::new();
    let mut rears = Vec::new();

    for asset in assets {
        match asset.side {
            VideoSide::Front => fronts.push(asset),
            VideoSide::Rear => rears.push(asset),
        }
    }

    fronts.sort_by_key(|asset| asset_sort_key(asset));
    rears.sort_by_key(|asset| asset_sort_key(asset));

    let mut used_rears = vec![false; rears.len()];
    let mut pairs = Vec::new();

    for front in fronts {
        let mut best_idx: Option<usize> = None;
        let mut best_delta_ms = i64::MAX;
        let mut candidates_within_threshold = 0;

        for (idx, rear) in rears.iter().enumerate() {
            if used_rears[idx] {
                continue;
            }
            let Some(delta_ms) = timestamp_delta_ms(front, rear) else {
                continue;
            };
            if delta_ms <= config.threshold_ms && delta_ms < best_delta_ms {
                best_delta_ms = delta_ms;
                best_idx = Some(idx);
                candidates_within_threshold = 1;
            } else if delta_ms <= config.threshold_ms && delta_ms == best_delta_ms {
                candidates_within_threshold += 1;
                if let Some(current_best_idx) = best_idx {
                    if rear.filename < rears[current_best_idx].filename {
                        best_idx = Some(idx);
                    }
                }
            }
        }

        match best_idx {
            Some(idx) => {
                used_rears[idx] = true;
                let rear = rears[idx];
                let pairing_reason = if best_delta_ms == 0 {
                    "exact_timestamp_match".to_string()
                } else if candidates_within_threshold > 1 {
                    format!(
                        "ambiguous_candidate_resolved_by_nearest_then_filename_within_{}ms",
                        config.threshold_ms
                    )
                } else {
                    format!("nearest_neighbor_within_{}ms", config.threshold_ms)
                };
                pairs.push(RecordingPair {
                    id: deterministic_pair_id(Some(front.id.as_str()), Some(rear.id.as_str())),
                    front_asset_id: Some(front.id.clone()),
                    rear_asset_id: Some(rear.id.clone()),
                    canonical_start_time: front
                        .parsed_timestamp
                        .clone()
                        .or_else(|| rear.parsed_timestamp.clone()),
                    estimated_duration_sec: front.duration_sec.or(rear.duration_sec),
                    pairing_confidence: confidence(best_delta_ms, config.threshold_ms),
                    pairing_reason,
                    source_folder: source_folder.to_string(),
                    warnings: Vec::new(),
                });
            }
            None => {
                pairs.push(RecordingPair {
                    id: deterministic_pair_id(Some(front.id.as_str()), None),
                    front_asset_id: Some(front.id.clone()),
                    rear_asset_id: None,
                    canonical_start_time: front.parsed_timestamp.clone(),
                    estimated_duration_sec: front.duration_sec,
                    pairing_confidence: 0.0,
                    pairing_reason: "no_rear_candidate_within_threshold".to_string(),
                    source_folder: source_folder.to_string(),
                    warnings: vec!["rear_missing".to_string()],
                });
            }
        }
    }

    for (idx, rear) in rears.iter().enumerate() {
        if used_rears[idx] {
            continue;
        }
        pairs.push(RecordingPair {
            id: deterministic_pair_id(None, Some(rear.id.as_str())),
            front_asset_id: None,
            rear_asset_id: Some(rear.id.clone()),
            canonical_start_time: rear.parsed_timestamp.clone(),
            estimated_duration_sec: rear.duration_sec,
            pairing_confidence: 0.0,
            pairing_reason: "unpaired_rear_leftover".to_string(),
            source_folder: source_folder.to_string(),
            warnings: vec!["front_missing".to_string()],
        });
    }

    pairs.sort_by_key(|pair| {
        (
            parse_dt(pair.canonical_start_time.as_deref()),
            pair.front_asset_id.clone().unwrap_or_default(),
            pair.rear_asset_id.clone().unwrap_or_default(),
        )
    });
    pairs
}

fn timestamp_delta_ms(front: &VideoAsset, rear: &VideoAsset) -> Option<i64> {
    let front_ts = parse_dt(front.parsed_timestamp.as_deref())?;
    let rear_ts = parse_dt(rear.parsed_timestamp.as_deref())?;
    Some((front_ts - rear_ts).num_milliseconds().abs())
}

fn parse_dt(value: Option<&str>) -> Option<NaiveDateTime> {
    let value = value?;
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok()
}

fn asset_sort_key(asset: &VideoAsset) -> (Option<NaiveDateTime>, String) {
    (parse_dt(asset.parsed_timestamp.as_deref()), asset.filename.clone())
}

fn deterministic_pair_id(front_id: Option<&str>, rear_id: Option<&str>) -> String {
    let mut hasher = DefaultHasher::new();
    front_id.unwrap_or("none").hash(&mut hasher);
    "|".hash(&mut hasher);
    rear_id.unwrap_or("none").hash(&mut hasher);
    format!("pair_{:x}", hasher.finish())
}

fn confidence(delta_ms: i64, threshold_ms: i64) -> f64 {
    if threshold_ms <= 0 {
        return 1.0;
    }
    let ratio = (threshold_ms - delta_ms).max(0) as f64 / threshold_ms as f64;
    ratio.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HealthStatus, VideoAsset};

    fn mk_asset(id: &str, side: VideoSide, ts: &str) -> VideoAsset {
        VideoAsset {
            id: id.to_string(),
            path: format!("C:/tmp/{}.mp4", id),
            filename: format!("{}.mp4", id),
            side,
            parse_status: crate::models::ParseStatus::Parsed,
            parsed_sequence: None,
            raw_timestamp_string: None,
            parsed_timestamp: Some(ts.to_string()),
            duration_sec: None,
            resolution: None,
            codec: None,
            size_bytes: 100,
            modified_at: "2026-03-23T00:00:00Z".to_string(),
            health: HealthStatus::Ok,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn pairs_by_nearest_timestamp() {
        let assets = vec![
            mk_asset("f1", VideoSide::Front, "2026-03-23T11:43:24"),
            mk_asset("r1", VideoSide::Rear, "2026-03-23T11:43:25"),
            mk_asset("r2", VideoSide::Rear, "2026-03-23T11:50:00"),
        ];

        let pairs = build_pairs(
            &assets,
            PairingConfig { threshold_ms: 3000 },
            "C:/tmp/samples",
        );
        assert_eq!(pairs.len(), 2);
        assert!(pairs.iter().any(|p| p.front_asset_id.as_deref() == Some("f1") && p.rear_asset_id.as_deref() == Some("r1")));
        assert!(pairs.iter().any(|p| p.front_asset_id.is_none() && p.rear_asset_id.as_deref() == Some("r2")));
    }

    #[test]
    fn real_fixture_like_samples_pair_as_expected() {
        let assets = vec![
            mk_asset("f23", VideoSide::Front, "2026-03-23T11:43:24"),
            mk_asset("r24", VideoSide::Rear, "2026-03-23T11:43:25"),
            mk_asset("f5", VideoSide::Front, "2026-03-23T11:25:20"),
            mk_asset("r6", VideoSide::Rear, "2026-03-23T11:25:21"),
        ];

        let pairs = build_pairs(
            &assets,
            PairingConfig { threshold_ms: 3000 },
            "C:/tmp/.test_examples",
        );
        assert_eq!(pairs.len(), 2);
        assert!(pairs
            .iter()
            .any(|p| p.front_asset_id.as_deref() == Some("f5") && p.rear_asset_id.as_deref() == Some("r6")));
        assert!(pairs
            .iter()
            .any(|p| p.front_asset_id.as_deref() == Some("f23") && p.rear_asset_id.as_deref() == Some("r24")));
    }

    #[test]
    fn same_input_produces_stable_pair_id() {
        let assets = vec![
            mk_asset("f1", VideoSide::Front, "2026-03-23T11:43:24"),
            mk_asset("r1", VideoSide::Rear, "2026-03-23T11:43:25"),
        ];
        let first = build_pairs(&assets, PairingConfig { threshold_ms: 3000 }, "C:/tmp/samples");
        let second = build_pairs(&assets, PairingConfig { threshold_ms: 3000 }, "C:/tmp/samples");
        assert_eq!(first[0].id, second[0].id);
    }
}
