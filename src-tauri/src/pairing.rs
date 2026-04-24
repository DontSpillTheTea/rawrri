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
                let mut pairing_reason = if best_delta_ms == 0 {
                    "exact_timestamp_match".to_string()
                } else if candidates_within_threshold > 1 {
                    format!(
                        "ambiguous_candidate_resolved_by_nearest_then_filename_within_{}ms",
                        config.threshold_ms
                    )
                } else {
                    format!("nearest_neighbor_within_{}ms", config.threshold_ms)
                };

                if front.metadata.is_some() && rear.metadata.is_some() {
                    pairing_reason.push_str(" (validated_by_metadata)");
                }

                let mut warnings = Vec::new();
                if let (Some(f_meta), Some(r_meta)) = (&front.metadata, &rear.metadata) {
                    let duration_delta = (f_meta.duration_sec - r_meta.duration_sec).abs();
                    if duration_delta > 5.0 {
                        warnings.push(format!("duration_mismatch_{:.1}s", duration_delta));
                    }
                }

                pairs.push(RecordingPair {
                    id: deterministic_pair_id(Some(front.id.as_str()), Some(rear.id.as_str())),
                    front_asset_id: Some(front.id.clone()),
                    rear_asset_id: Some(rear.id.clone()),
                    canonical_start_time: get_best_timestamp(front)
                        .or_else(|| get_best_timestamp(rear))
                        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
                    estimated_duration_sec: front
                        .metadata
                        .as_ref()
                        .map(|m| m.duration_sec)
                        .or(rear.metadata.as_ref().map(|m| m.duration_sec))
                        .or(front.duration_sec)
                        .or(rear.duration_sec),
                    pairing_confidence: calculate_confidence(front, rear, best_delta_ms, config.threshold_ms),
                    pairing_reason,
                    source_folder: source_folder.to_string(),
                    warnings,
                    observations: Vec::new(),
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
                    observations: Vec::new(),
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
            observations: Vec::new(),
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
    let front_ts = get_best_timestamp(front)?;
    let rear_ts = get_best_timestamp(rear)?;
    Some((front_ts - rear_ts).num_milliseconds().abs())
}

fn get_best_timestamp(asset: &VideoAsset) -> Option<NaiveDateTime> {
    // metadata creation_time is authoritative
    if let Some(metadata) = &asset.metadata {
        if let Some(ct) = &metadata.creation_time {
            if let Some(dt) = parse_dt(Some(ct)) {
                return Some(dt);
            }
        }
    }
    // fallback to filename parsed timestamp
    parse_dt(asset.parsed_timestamp.as_deref())
}

fn parse_dt(value: Option<&str>) -> Option<NaiveDateTime> {
    let value = value?;
    // try ISO 8601 first (metadata)
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%SZ") {
        return Some(dt);
    }
    // then fallback to simple format (filename parser)
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S").ok()
}

fn asset_sort_key(asset: &VideoAsset) -> (Option<NaiveDateTime>, String) {
    (get_best_timestamp(asset), asset.filename.clone())
}

fn deterministic_pair_id(front_id: Option<&str>, rear_id: Option<&str>) -> String {
    let mut hasher = DefaultHasher::new();
    front_id.unwrap_or("none").hash(&mut hasher);
    "|".hash(&mut hasher);
    rear_id.unwrap_or("none").hash(&mut hasher);
    format!("pair_{:x}", hasher.finish())
}

fn calculate_confidence(front: &VideoAsset, rear: &VideoAsset, delta_ms: i64, threshold_ms: i64) -> f64 {
    let mut score = 1.0;

    // Time proximity factor (up to 70% of total score)
    if threshold_ms > 0 {
        let time_factor = (threshold_ms - delta_ms).max(0) as f64 / threshold_ms as f64;
        score *= 0.3 + (0.7 * time_factor);
    }

    // Duration similarity factor (penalize if durations differ)
    if let (Some(f_meta), Some(r_meta)) = (&front.metadata, &rear.metadata) {
        let duration_delta = (f_meta.duration_sec - r_meta.duration_sec).abs();
        let duration_factor = if duration_delta < 1.0 {
            1.0
        } else if duration_delta < 5.0 {
            0.9
        } else if duration_delta < 10.0 {
            0.7
        } else {
            0.5
        };
        score *= duration_factor;
    }

    score.clamp(0.0, 1.0)
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
            metadata: None,
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
