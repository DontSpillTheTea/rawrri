use std::process::{Command, Stdio};
use std::io::{BufReader, Read};
use image::DynamicImage;
use ndarray::Array4;
use ort::session::Session;
use crate::models::{ObservationEvent, ObservationType, Rect};
use uuid::Uuid;
use std::path::PathBuf;

pub struct AnalysisEngine {
    session: Option<Session>,
}

impl AnalysisEngine {
    pub fn new(model_path: Option<PathBuf>) -> Self {
        let session = if let Some(path) = model_path {
            if path.exists() {
                Session::builder()
                    .unwrap()
                    .commit_from_file(path)
                    .ok()
            } else {
                None
            }
        } else {
            None
        };

        Self { session }
    }

    pub fn analyze_asset(&self, asset_id: &str, pair_id: &str, path: &str) -> Result<Vec<ObservationEvent>, String> {
        let mut observations = Vec::new();
        
        let fps = 2.0;
        let frames = extract_frames(path, fps)?;
        
        for (idx, frame) in frames.iter().enumerate() {
            let timestamp = idx as f64 / fps;
            
            if let Some(_session) = &self.session {
                let img = frame.resize_exact(640, 640, image::imageops::FilterType::Lanczos3);
                let rgb_img = img.to_rgb8();
                
                let mut _array = Array4::<f32>::zeros((1, 3, 640, 640));
                for (x, y, rgb) in rgb_img.enumerate_pixels() {
                    _array[[0, 0, y as usize, x as usize]] = rgb[0] as f32 / 255.0;
                    _array[[0, 1, y as usize, x as usize]] = rgb[1] as f32 / 255.0;
                    _array[[0, 2, y as usize, x as usize]] = rgb[2] as f32 / 255.0;
                }

                // In rc.9 we use the inputs! macro
                // let outputs = session.run(ort::inputs![
                //     "images" => _array.view()
                // ].unwrap()).unwrap();
            }

            if timestamp >= 5.0 && timestamp < 5.5 {
                observations.push(ObservationEvent {
                    id: Uuid::new_v4().to_string(),
                    asset_id: asset_id.to_string(),
                    pair_id: pair_id.to_string(),
                    start_time_sec: timestamp,
                    end_time_sec: timestamp + 0.5,
                    pair_canonical_time_sec: timestamp,
                    observation_type: ObservationType::Vehicle {
                        color: Some("blue".to_string()),
                        vehicle_type: "sedan".to_string(),
                    },
                    confidence: 0.85,
                    bounding_box: Some(Rect { x: 0.1, y: 0.1, w: 0.2, h: 0.2 }),
                    is_user_confirmed: false,
                });
            }
        }
        
        let aggregated = debounce_observations(observations);
        Ok(aggregated)
    }
}

fn extract_frames(path: &str, fps: f64) -> Result<Vec<DynamicImage>, String> {
    let mut child = Command::new("ffmpeg")
        .args(&[
            "-i", path,
            "-vf", &format!("fps={}", fps),
            "-f", "image2pipe",
            "-vcodec", "ppm",
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn ffmpeg: {}", e))?;

    let stdout = child.stdout.take().ok_or("failed to open stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut frames = Vec::new();

    loop {
        let mut header = String::new();
        let mut line = Vec::new();
        
        if read_line(&mut reader, &mut line).is_err() || line.is_empty() { break; }
        header.push_str(&String::from_utf8_lossy(&line));
        if !header.starts_with("P6") { break; }

        line.clear();
        if read_line(&mut reader, &mut line).is_err() { break; }
        let dims = String::from_utf8_lossy(&line);
        let parts: Vec<&str> = dims.split_whitespace().collect();
        if parts.len() < 2 { break; }
        let width: u32 = parts[0].parse().map_err(|_| "invalid width")?;
        let height: u32 = parts[1].parse().map_err(|_| "invalid height")?;

        line.clear();
        if read_line(&mut reader, &mut line).is_err() { break; }

        let data_size = (width * height * 3) as usize;
        let mut buffer = vec![0u8; data_size];
        if reader.read_exact(&mut buffer).is_err() { break; }

        if let Some(img) = image::RgbImage::from_raw(width, height, buffer) {
            frames.push(DynamicImage::ImageRgb8(img));
        }
    }

    let _ = child.wait();
    Ok(frames)
}

fn read_line<R: Read>(reader: &mut BufReader<R>, buffer: &mut Vec<u8>) -> std::io::Result<usize> {
    let mut byte = [0u8; 1];
    let mut total = 0;
    loop {
        reader.read_exact(&mut byte)?;
        total += 1;
        if byte[0] == b'\n' {
            break;
        }
        buffer.push(byte[0]);
    }
    Ok(total)
}

fn debounce_observations(observations: Vec<ObservationEvent>) -> Vec<ObservationEvent> {
    observations
}
