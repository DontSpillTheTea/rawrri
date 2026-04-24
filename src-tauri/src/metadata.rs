use std::process::Command;
use serde::Deserialize;
use crate::models::MediaMetadata;

#[derive(Debug, Deserialize)]
struct FFProbeOutput {
    streams: Vec<FFProbeStream>,
    format: FFProbeFormat,
}

#[derive(Debug, Deserialize)]
struct FFProbeStream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FFProbeFormat {
    duration: Option<String>,
    tags: Option<FFProbeTags>,
}

#[derive(Debug, Deserialize)]
struct FFProbeTags {
    creation_time: Option<String>,
}

pub fn extract_metadata(path: &str) -> Result<MediaMetadata, String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .map_err(|e| format!("failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        return Err(format!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let ffprobe_data: FFProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("failed to parse ffprobe output: {}", e))?;

    let video_stream = ffprobe_data.streams.iter().find(|s| s.codec_type == "video");
    let audio_stream = ffprobe_data.streams.iter().find(|s| s.codec_type == "audio");

    let duration_sec = ffprobe_data.format.duration
        .as_ref()
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| video_stream.and_then(|s| s.duration.as_ref()?.parse::<f64>().ok()))
        .unwrap_or(0.0);

    let creation_time = ffprobe_data.format.tags.and_then(|t| t.creation_time);

    Ok(MediaMetadata {
        duration_sec,
        creation_time,
        width: video_stream.and_then(|s| s.width).unwrap_or(0),
        height: video_stream.and_then(|s| s.height).unwrap_or(0),
        codec: video_stream.and_then(|s| s.codec_name.clone()).unwrap_or_else(|| "unknown".to_string()),
        has_audio: audio_stream.is_some(),
        is_corrupt: duration_sec == 0.0,
        stream_count: ffprobe_data.streams.len(),
    })
}
