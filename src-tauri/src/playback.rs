use std::{
    env,
    io::{BufRead, BufReader},
    fs::OpenOptions,
    io::Write,
    process::{Child, Command, Stdio},
    sync::mpsc,
    sync::Mutex,
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSnapshot {
    pub active_pair_id: Option<String>,
    pub is_playing: bool,
    pub playhead_sec: f64,
    pub pair_duration_sec: Option<f64>,
    pub front_time_sec: Option<f64>,
    pub rear_time_sec: Option<f64>,
    pub front_duration_sec: Option<f64>,
    pub rear_duration_sec: Option<f64>,
    pub sync_delta_sec: Option<f64>,
    pub front_loaded: bool,
    pub rear_loaded: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PlaybackState {
    pub active_pair_id: Option<String>,
    pub is_playing: bool,
    pub playhead_sec: f64,
    pub pair_duration_sec: Option<f64>,
    pub front_time_sec: Option<f64>,
    pub rear_time_sec: Option<f64>,
    pub front_duration_sec: Option<f64>,
    pub rear_duration_sec: Option<f64>,
    pub sync_delta_sec: Option<f64>,
    pub last_error: Option<String>,
    pub play_started_at: Option<Instant>,
    pub playhead_started_sec: f64,
}

#[derive(Default)]
pub struct PlaybackManager {
    front: Option<PlayerInstance>,
    rear: Option<PlayerInstance>,
    state: PlaybackState,
}

#[derive(Default)]
pub struct PlaybackController {
    pub manager: Mutex<PlaybackManager>,
}

struct PlayerInstance {
    side: String,
    ipc_path: String,
    child: Child,
    next_request_id: u64,
}

impl PlaybackManager {
    pub fn load_pair(
        &mut self,
        pair_id: String,
        front_path: Option<String>,
        rear_path: Option<String>,
        front_wid: Option<u64>,
        rear_wid: Option<u64>,
    ) -> Result<PlaybackSnapshot, String> {
        self.shutdown_players();
        self.state.active_pair_id = Some(pair_id.clone());
        self.state.playhead_sec = 0.0;
        self.state.is_playing = false;
        self.state.play_started_at = None;
        self.state.playhead_started_sec = 0.0;
        self.state.front_time_sec = None;
        self.state.rear_time_sec = None;
        self.state.front_duration_sec = None;
        self.state.rear_duration_sec = None;
        self.state.pair_duration_sec = None;
        self.state.sync_delta_sec = None;
        self.state.last_error = None;
        println!(
            "playback.load_pair pair_id={} front_present={} rear_present={} front_wid={:?} rear_wid={:?}",
            pair_id,
            front_path.is_some(),
            rear_path.is_some(),
            front_wid,
            rear_wid
        );

        if let Some(path) = front_path {
            let mut player = PlayerInstance::spawn("front", front_wid)?;
            player.load_file(&path)?;
            player.set_paused(true)?;
            player.seek(0.0)?;
            self.front = Some(player);
        }

        if let Some(path) = rear_path {
            let mut player = PlayerInstance::spawn("rear", rear_wid)?;
            player.load_file(&path)?;
            player.set_paused(true)?;
            player.seek(0.0)?;
            self.rear = Some(player);
        }

        Ok(self.snapshot())
    }

    pub fn set_playing(&mut self, is_playing: bool) -> Result<PlaybackSnapshot, String> {
        self.update_estimated_playhead();
        self.state.is_playing = is_playing;
        if is_playing {
            self.state.play_started_at = Some(Instant::now());
            self.state.playhead_started_sec = self.state.playhead_sec;
        } else {
            self.state.play_started_at = None;
            self.state.playhead_started_sec = self.state.playhead_sec;
        }
        println!("playback.set_playing is_playing={}", is_playing);

        if let Some(front) = self.front.as_mut() {
            front.set_paused(!is_playing)?;
        }
        if let Some(rear) = self.rear.as_mut() {
            rear.set_paused(!is_playing)?;
        }

        Ok(self.snapshot())
    }

    pub fn toggle_playing(&mut self) -> Result<PlaybackSnapshot, String> {
        let next = !self.state.is_playing;
        self.set_playing(next)
    }

    pub fn seek_to(&mut self, playhead_sec: f64) -> Result<PlaybackSnapshot, String> {
        let clamped = playhead_sec.max(0.0);
        self.state.playhead_sec = clamped;
        self.state.playhead_started_sec = clamped;
        self.state.play_started_at = if self.state.is_playing {
            Some(Instant::now())
        } else {
            None
        };
        self.state.front_time_sec = Some(clamped);
        self.state.rear_time_sec = Some(clamped);
        println!("playback.seek_to playhead_sec={:.3}", clamped);

        if let Some(front) = self.front.as_mut() {
            front.seek(clamped)?;
        }
        if let Some(rear) = self.rear.as_mut() {
            rear.seek(clamped)?;
        }

        Ok(self.snapshot())
    }

    pub fn stop(&mut self) -> PlaybackSnapshot {
        println!("playback.stop");
        self.shutdown_players();
        self.state = PlaybackState::default();
        self.snapshot()
    }

    pub fn snapshot(&self) -> PlaybackSnapshot {
        PlaybackSnapshot {
            active_pair_id: self.state.active_pair_id.clone(),
            is_playing: self.state.is_playing,
            playhead_sec: self.state.playhead_sec,
            pair_duration_sec: self.state.pair_duration_sec,
            front_time_sec: self.state.front_time_sec,
            rear_time_sec: self.state.rear_time_sec,
            front_duration_sec: self.state.front_duration_sec,
            rear_duration_sec: self.state.rear_duration_sec,
            sync_delta_sec: self.state.sync_delta_sec,
            front_loaded: self.front.is_some(),
            rear_loaded: self.rear.is_some(),
            last_error: self.state.last_error.clone(),
        }
    }

    pub fn refresh_state(&mut self) -> PlaybackSnapshot {
        self.update_estimated_playhead();

        let mut first_error: Option<String> = None;

        let front_time = match self.front.as_mut() {
            Some(front) => match front.get_property_f64("time-pos") {
                Ok(value) => value,
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    None
                }
            },
            None => None,
        };
        let rear_time = match self.rear.as_mut() {
            Some(rear) => match rear.get_property_f64("time-pos") {
                Ok(value) => value,
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    None
                }
            },
            None => None,
        };
        let front_duration = match self.front.as_mut() {
            Some(front) => match front.get_property_f64("duration") {
                Ok(value) => value,
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    None
                }
            },
            None => None,
        };
        let rear_duration = match self.rear.as_mut() {
            Some(rear) => match rear.get_property_f64("duration") {
                Ok(value) => value,
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                    None
                }
            },
            None => None,
        };

        self.state.front_time_sec = front_time.or(self.state.front_time_sec);
        self.state.rear_time_sec = rear_time.or(self.state.rear_time_sec);
        self.state.front_duration_sec = front_duration.or(self.state.front_duration_sec);
        self.state.rear_duration_sec = rear_duration.or(self.state.rear_duration_sec);
        self.state.pair_duration_sec = match (self.state.front_duration_sec, self.state.rear_duration_sec) {
            (Some(front), Some(rear)) => Some(front.max(rear)),
            (Some(front), None) => Some(front),
            (None, Some(rear)) => Some(rear),
            (None, None) => None,
        };

        self.state.sync_delta_sec = match (self.state.front_time_sec, self.state.rear_time_sec) {
            (Some(front), Some(rear)) => Some(rear - front),
            _ => None,
        };

        self.state.playhead_sec = match (self.state.front_time_sec, self.state.rear_time_sec) {
            (Some(front), Some(rear)) => (front + rear) / 2.0,
            (Some(front), None) => front,
            (None, Some(rear)) => rear,
            (None, None) => self.state.playhead_sec,
        };

        self.state.last_error = first_error;
        self.snapshot()
    }

    fn shutdown_players(&mut self) {
        if let Some(front) = self.front.as_mut() {
            front.shutdown();
        }
        if let Some(rear) = self.rear.as_mut() {
            rear.shutdown();
        }
        self.front = None;
        self.rear = None;
        self.state.is_playing = false;
        self.state.play_started_at = None;
        self.state.playhead_started_sec = 0.0;
    }

    fn update_estimated_playhead(&mut self) {
        if !self.state.is_playing {
            return;
        }
        let Some(started_at) = self.state.play_started_at else {
            return;
        };
        let elapsed = Instant::now().duration_since(started_at).as_secs_f64();
        self.state.playhead_sec = self.state.playhead_started_sec + elapsed;
    }
}

impl PlayerInstance {
    fn spawn(side: &str, wid: Option<u64>) -> Result<Self, String> {
        let ipc_path = named_pipe_path(side);
        let title = format!("rawrii {side}");
        let debug_no_config = env::var("RAWRII_MPV_NO_CONFIG")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let mut args = vec![
            "--idle=yes".to_string(),
            "--pause".to_string(),
            "--keep-open=yes".to_string(),
            "--no-terminal".to_string(),
            format!("--title={title}"),
            format!("--input-ipc-server={ipc_path}"),
        ];
        if debug_no_config {
            // Debug experiment path to rule out user mpv.conf interference.
            args.push("--no-config".to_string());
        }
        if let Some(wid) = wid {
            args.push(format!("--wid={wid}"));
        } else if cfg!(debug_assertions) {
            println!(
                "playback.spawn side={} embedding unavailable, using dev fallback external window",
                side
            );
            args.push("--force-window=yes".to_string());
        } else {
            return Err(format!(
                "mpv embedding failed for {} and release fallback is disabled",
                side
            ));
        }
        println!(
            "playback.spawn side={} command=mpv debug_no_config={} args={:?}",
            side, debug_no_config, args
        );

        let child = Command::new("mpv")
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                format!(
                    "Failed to spawn mpv for {}: {}. Ensure mpv is installed and available in PATH.",
                    side, err
                )
            })?;

        Ok(Self {
            side: side.to_string(),
            ipc_path,
            child,
            next_request_id: 1,
        })
    }

    fn load_file(&mut self, path: &str) -> Result<(), String> {
        println!("playback.loadfile side={} path={}", self.side, path);
        self.send_command(json!({
            "command": ["loadfile", path, "replace"]
        }))
    }

    fn set_paused(&mut self, paused: bool) -> Result<(), String> {
        self.send_command(json!({
            "command": ["set_property", "pause", paused]
        }))
    }

    fn seek(&mut self, time_pos: f64) -> Result<(), String> {
        self.send_command(json!({
            "command": ["set_property", "time-pos", time_pos]
        }))
    }

    fn send_command(&mut self, command: serde_json::Value) -> Result<(), String> {
        let serialized = serde_json::to_string(&command)
            .map_err(|err| format!("Failed to serialize mpv command: {}", err))?;
        let payload = format!("{serialized}\n");

        let mut last_err: Option<String> = None;
        for _ in 0..50 {
            match OpenOptions::new().write(true).open(&self.ipc_path) {
                Ok(mut stream) => {
                    stream
                        .write_all(payload.as_bytes())
                        .map_err(|err| format!("Failed writing to mpv IPC {}: {}", self.ipc_path, err))?;
                    return Ok(());
                }
                Err(err) => {
                    last_err = Some(err.to_string());
                    sleep(Duration::from_millis(20));
                }
            }
        }

        Err(format!(
            "Failed to connect to mpv IPC for {} at {}: {}",
            self.side,
            self.ipc_path,
            last_err.unwrap_or_else(|| "unknown error".to_string())
        ))
    }

    fn get_property_f64(&mut self, property: &str) -> Result<Option<f64>, String> {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        let command = json!({
            "command": ["get_property", property],
            "request_id": request_id
        });
        let response = self.send_command_and_wait(command, request_id)?;
        let error = response.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
        if error != "success" {
            return Ok(None);
        }
        Ok(response.get("data").and_then(|value| value.as_f64()))
    }

    fn send_command_and_wait(
        &self,
        command: serde_json::Value,
        request_id: u64,
    ) -> Result<serde_json::Value, String> {
        let serialized = serde_json::to_string(&command)
            .map_err(|err| format!("Failed to serialize mpv command: {}", err))?;
        let payload = format!("{serialized}\n");
        let ipc_path = self.ipc_path.clone();
        let side = self.side.clone();
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(move || {
            let outcome: Result<serde_json::Value, String> = (|| {
                let mut stream = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&ipc_path)
                    .map_err(|err| format!("Failed to open mpv IPC {}: {}", ipc_path, err))?;
                stream
                    .write_all(payload.as_bytes())
                    .map_err(|err| format!("Failed writing to mpv IPC {}: {}", ipc_path, err))?;

                let mut reader = BufReader::new(stream);
                for _ in 0..64 {
                    let mut line = String::new();
                    let read = reader
                        .read_line(&mut line)
                        .map_err(|err| format!("Failed reading mpv IPC {}: {}", ipc_path, err))?;
                    if read == 0 {
                        break;
                    }
                    let value: serde_json::Value = match serde_json::from_str(line.trim()) {
                        Ok(value) => value,
                        Err(_) => continue,
                    };
                    let response_id = value.get("request_id").and_then(|id| id.as_u64());
                    if response_id == Some(request_id) {
                        return Ok(value);
                    }
                }
                Err(format!(
                    "No matching response for request_id={} on {}",
                    request_id, ipc_path
                ))
            })();
            let _ = tx.send(outcome);
        });

        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(result) => result,
            Err(_) => Err(format!(
                "Timed out waiting for mpv IPC response side={} request_id={}",
                side, request_id
            )),
        }
    }

    fn shutdown(&mut self) {
        println!("playback.shutdown side={}", self.side);
        let _ = self.send_command(json!({
            "command": ["quit"]
        }));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn named_pipe_path(side: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(r"\\.\pipe\rawrii_{}_{}", side, millis)
}
