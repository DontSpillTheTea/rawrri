use std::{
    fs::OpenOptions,
    io::Write,
    process::{Child, Command, Stdio},
    sync::Mutex,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackSnapshot {
    pub active_pair_id: Option<String>,
    pub is_playing: bool,
    pub playhead_sec: f64,
    pub front_loaded: bool,
    pub rear_loaded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PlaybackState {
    pub active_pair_id: Option<String>,
    pub is_playing: bool,
    pub playhead_sec: f64,
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
}

impl PlaybackManager {
    pub fn load_pair(
        &mut self,
        pair_id: String,
        front_path: Option<String>,
        rear_path: Option<String>,
    ) -> Result<PlaybackSnapshot, String> {
        self.shutdown_players();
        self.state.active_pair_id = Some(pair_id.clone());
        self.state.playhead_sec = 0.0;
        self.state.is_playing = false;
        println!(
            "playback.load_pair pair_id={} front_present={} rear_present={}",
            pair_id,
            front_path.is_some(),
            rear_path.is_some()
        );

        if let Some(path) = front_path {
            let mut player = PlayerInstance::spawn("front")?;
            player.load_file(&path)?;
            player.set_paused(true)?;
            player.seek(0.0)?;
            self.front = Some(player);
        }

        if let Some(path) = rear_path {
            let mut player = PlayerInstance::spawn("rear")?;
            player.load_file(&path)?;
            player.set_paused(true)?;
            player.seek(0.0)?;
            self.rear = Some(player);
        }

        Ok(self.snapshot())
    }

    pub fn set_playing(&mut self, is_playing: bool) -> Result<PlaybackSnapshot, String> {
        self.state.is_playing = is_playing;
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
            front_loaded: self.front.is_some(),
            rear_loaded: self.rear.is_some(),
        }
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
    }
}

impl PlayerInstance {
    fn spawn(side: &str) -> Result<Self, String> {
        let ipc_path = named_pipe_path(side);
        let title = format!("rawrii {side}");
        let args = vec![
            "--idle=yes".to_string(),
            "--force-window=yes".to_string(),
            "--pause".to_string(),
            "--keep-open=yes".to_string(),
            "--no-terminal".to_string(),
            format!("--title={title}"),
            format!("--input-ipc-server={ipc_path}"),
        ];
        println!("playback.spawn side={} command=mpv {:?}", side, args);

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
