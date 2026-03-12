use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub state: PlaybackState,
    pub position: f64,
    pub duration: f64,
    pub volume: i64,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            state: PlaybackState::Stopped,
            position: 0.0,
            duration: 0.0,
            volume: 80,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PlayerCommand {
    Load(String),
    TogglePause,
    Stop,
    SeekForward(f64),
    SeekBackward(f64),
    SetVolume(i64),
    VolumeUp,
    VolumeDown,
}

#[derive(Debug)]
pub enum PlayerEvent {
    Position(f64),
    Duration(f64),
    Paused(bool),
    Idle,
    TrackEnd,
    Error(String),
}

#[derive(Clone)]
pub struct PlayerSender {
    cmd_tx: mpsc::Sender<String>,
}

impl PlayerSender {
    pub async fn send(&self, cmd: PlayerCommand) -> Result<()> {
        let json = match cmd {
            PlayerCommand::Load(url) => {
                format!(
                    r#"{{"command":["loadfile","{}","replace"]}}"#,
                    escape_json_string(&url)
                )
            }
            PlayerCommand::TogglePause => r#"{"command":["cycle","pause"]}"#.to_string(),
            PlayerCommand::Stop => r#"{"command":["stop"]}"#.to_string(),
            PlayerCommand::SeekForward(secs) => {
                format!(r#"{{"command":["seek","{}","relative"]}}"#, secs)
            }
            PlayerCommand::SeekBackward(secs) => {
                format!(r#"{{"command":["seek","-{}","relative"]}}"#, secs)
            }
            PlayerCommand::SetVolume(vol) => {
                format!(
                    r#"{{"command":["set_property","volume",{}]}}"#,
                    vol.clamp(0, 100)
                )
            }
            PlayerCommand::VolumeUp => r#"{"command":["add","volume","5"]}"#.to_string(),
            PlayerCommand::VolumeDown => r#"{"command":["add","volume","-5"]}"#.to_string(),
        };

        self.cmd_tx
            .send(json)
            .await
            .context("Failed to send command to mpv")?;
        Ok(())
    }
}

pub struct MpvProcess {
    _child: Child,
    pub sender: PlayerSender,
}

impl MpvProcess {
    pub async fn spawn(event_tx: mpsc::Sender<PlayerEvent>) -> Result<Self> {
        let socket_path = socket_path();

        if tokio::fs::try_exists(&socket_path).await.unwrap_or(false) {
            let _ = tokio::fs::remove_file(&socket_path).await;
        }

        let child = Command::new("mpv")
            .arg("--idle")
            .arg("--no-video")
            .arg("--no-terminal")
            .arg("--really-quiet")
            .arg(format!("--input-ipc-server={}", socket_path.display()))
            .arg("--volume=80")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start mpv. Is mpv installed?")?;

        for _ in 0..50 {
            if tokio::fs::try_exists(&socket_path).await.unwrap_or(false) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .context("Failed to connect to mpv IPC socket")?;

        let (reader, writer) = stream.into_split();
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<String>(64);

        tokio::spawn(async move {
            let mut writer = writer;
            while let Some(cmd) = cmd_rx.recv().await {
                let msg = format!("{}\n", cmd);
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let sender = PlayerSender {
            cmd_tx: cmd_tx.clone(),
        };

        let init_sender = cmd_tx.clone();
        tokio::spawn(async move {
            let _ = init_sender
                .send(r#"{"command":["observe_property",1,"time-pos"]}"#.to_string())
                .await;
            let _ = init_sender
                .send(r#"{"command":["observe_property",2,"duration"]}"#.to_string())
                .await;
            let _ = init_sender
                .send(r#"{"command":["observe_property",3,"pause"]}"#.to_string())
                .await;
            let _ = init_sender
                .send(r#"{"command":["observe_property",4,"idle-active"]}"#.to_string())
                .await;
        });

        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(json) = serde_json::from_str::<Value>(&line) {
                    if let Some(event) = parse_mpv_event(&json) {
                        if event_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self {
            _child: child,
            sender,
        })
    }
}

fn parse_mpv_event(json: &Value) -> Option<PlayerEvent> {
    if let Some(event_name) = json.get("event").and_then(|e| e.as_str()) {
        return match event_name {
            "end-file" => {
                let reason = json
                    .get("reason")
                    .and_then(|r| r.as_str())
                    .unwrap_or("");
                if reason == "eof" {
                    Some(PlayerEvent::TrackEnd)
                } else if reason == "error" {
                    let err = json
                        .get("file_error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("unknown error");
                    Some(PlayerEvent::Error(err.to_string()))
                } else {
                    None
                }
            }
            "property-change" => {
                let name = json.get("name").and_then(|n| n.as_str())?;
                match name {
                    "time-pos" => {
                        let pos = json.get("data").and_then(|d| d.as_f64()).unwrap_or(0.0);
                        Some(PlayerEvent::Position(pos))
                    }
                    "duration" => {
                        let dur = json.get("data").and_then(|d| d.as_f64()).unwrap_or(0.0);
                        Some(PlayerEvent::Duration(dur))
                    }
                    "pause" => {
                        let paused =
                            json.get("data").and_then(|d| d.as_bool()).unwrap_or(false);
                        Some(PlayerEvent::Paused(paused))
                    }
                    "idle-active" => {
                        let idle =
                            json.get("data").and_then(|d| d.as_bool()).unwrap_or(false);
                        if idle {
                            Some(PlayerEvent::Idle)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        };
    }
    None
}

fn socket_path() -> PathBuf {
    let dir = std::env::temp_dir();
    dir.join(format!("ytmusic-mpv-{}.sock", std::process::id()))
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
