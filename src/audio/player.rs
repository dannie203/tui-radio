#[cfg(unix)]
use interprocess::local_socket::GenericFilePath;
#[cfg(windows)]
use interprocess::local_socket::GenericNamespaced;
use interprocess::local_socket::{prelude::*, Stream as IpcStream};
use interprocess::TryClone;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(unix)]
pub const MPV_SOCKET: &str = "/tmp/boombox-rs-mpv.sock";

#[cfg(windows)]
pub const MPV_SOCKET: &str = r"\\.\pipe\boombox-rs-mpv";

fn connect_to_mpv(socket_path: &str) -> std::io::Result<IpcStream> {
    #[cfg(unix)]
    {
        let name = socket_path.to_fs_name::<GenericFilePath>()?;
        IpcStream::connect(name)
    }
    #[cfg(windows)]
    {
        let _ = socket_path;
        let name = "boombox-rs-mpv".to_ns_name::<GenericNamespaced>()?;
        IpcStream::connect(name)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlayerMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub codec: Option<String>,
    pub bit_depth: Option<u32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub time_pos: f64,
    pub duration: f64,
    pub percent_pos: f64,
    pub volume: u32,
    pub eof: bool,
    pub metadata: PlayerMetadata,
}

pub struct MpvPlayer {
    socket_path: String,
    process: Arc<Mutex<Option<Child>>>,
    stream: Arc<Mutex<Option<IpcStream>>>,
    request_id: AtomicU64,
    pub status: Arc<Mutex<PlayerStatus>>,
    running: Arc<AtomicBool>,
    master_volume: Arc<Mutex<u32>>,
    current_af: Arc<Mutex<String>>,
}

/// Resolves an external helper executable path.
/// Checks next to the running boombox executable first, then falls back to PATH.
pub fn resolve_executable(name: &str) -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            #[cfg(windows)]
            let file_name = if name.ends_with(".exe") {
                name.to_string()
            } else {
                format!("{}.exe", name)
            };
            #[cfg(not(windows))]
            let file_name = name.to_string();

            let candidate = parent.join(&file_name);
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from(name)
}

impl MpvPlayer {
    pub fn new() -> Self {
        let socket_path = MPV_SOCKET.to_string();
        #[cfg(unix)]
        if Path::new(&socket_path).exists() {
            let _ = fs::remove_file(&socket_path);
        }

        let child = Command::new(resolve_executable("mpv"))
            .arg("--no-video")
            .arg("--idle=yes")
            .arg(format!("--input-ipc-server={}", socket_path))
            .arg("--really-quiet")
            .arg("--audio-display=no")
            .arg("--gapless-audio=yes")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();

        let stream_arc = Arc::new(Mutex::new(None));
        let status_arc = Arc::new(Mutex::new(PlayerStatus {
            volume: 80,
            ..Default::default()
        }));
        let running_arc = Arc::new(AtomicBool::new(true));

        let player = Self {
            socket_path: socket_path.clone(),
            process: Arc::new(Mutex::new(child)),
            stream: stream_arc,
            request_id: AtomicU64::new(1),
            status: status_arc,
            running: running_arc,
            master_volume: Arc::new(Mutex::new(80)),
            current_af: Arc::new(Mutex::new(String::new())),
        };

        // Retry connection up to 50 times (1.5 seconds) until MPV is listening
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(30));
            if let Ok(s) = connect_to_mpv(&socket_path) {
                let _ = s.set_nonblocking(false);
                if let Ok(reader_stream) = s.try_clone() {
                    *player.stream.lock().unwrap_or_else(|e| e.into_inner()) = Some(s);
                    player.spawn_reader_thread(reader_stream);
                    player.observe_properties();
                    break;
                }
            }
        }

        player
    }

    fn observe_properties(&self) {
        let props = [
            ("time-pos", 1),
            ("duration", 2),
            ("percent-pos", 3),
            ("pause", 4),
            ("media-title", 5),
            ("metadata", 6),
            ("audio-codec-name", 7),
            ("audio-params", 8),
            ("audio-bitrate", 10),
        ];

        for (prop, id) in props {
            self.send_json_command_raw(serde_json::json!(["observe_property", id, prop]));
        }
        let vol = *self.master_volume.lock().unwrap_or_else(|e| e.into_inner());
        self.send_json_command_raw(serde_json::json!(["set_property", "volume", vol]));
    }

    fn reconnect(&self) {
        // Clean up previous child process if still running to prevent zombies
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut old_child) = guard.take() {
                let _ = old_child.kill();
                let _ = old_child.wait();
            }
        }

        let socket_path = self.socket_path.clone();
        #[cfg(unix)]
        if Path::new(&socket_path).exists() {
            let _ = fs::remove_file(&socket_path);
        }

        let child = Command::new(resolve_executable("mpv"))
            .arg("--no-video")
            .arg("--idle=yes")
            .arg(format!("--input-ipc-server={}", socket_path))
            .arg("--really-quiet")
            .arg("--audio-display=no")
            .arg("--gapless-audio=yes")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok();

        let mut process_guard = self.process.lock().unwrap_or_else(|e| e.into_inner());
        *process_guard = child;
        drop(process_guard);

        for _ in 0..20 {
            thread::sleep(Duration::from_millis(15));
            if let Ok(mut guard) = self.process.lock() {
                if let Some(ref mut child) = *guard {
                    if let Ok(Some(_)) = child.try_wait() {
                        break;
                    }
                }
            }
            if let Ok(s) = connect_to_mpv(&socket_path) {
                let _ = s.set_nonblocking(false);
                if let Ok(reader_stream) = s.try_clone() {
                    let mut guard = self.stream.lock().unwrap_or_else(|e| e.into_inner());
                    *guard = Some(s);
                    self.spawn_reader_thread(reader_stream);
                    self.observe_properties();
                    let af = self.current_af.lock().unwrap_or_else(|e| e.into_inner()).clone();
                    if !af.is_empty() {
                        self.send_json_command_raw(serde_json::json!(["set_property", "af", af]));
                    }
                    break;
                }
            }
        }
    }

    fn spawn_reader_thread(&self, stream: IpcStream) {
        let status_clone = Arc::clone(&self.status);
        let running_clone = Arc::clone(&self.running);

        let _ = thread::Builder::new()
            .name("boombox-mpv-ipc".to_string())
            .spawn(move || {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();

            while running_clone.load(Ordering::Relaxed) {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break, // Socket closed
                    Ok(_) => {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                            let mut st = match status_clone.lock() {
                                Ok(g) => g,
                                Err(e) => e.into_inner(),
                            };

                            if let Some(event) = val.get("event").and_then(|e| e.as_str()) {
                                match event {
                                    "playback-restart" | "file-loaded" => {
                                        st.is_playing = true;
                                        st.eof = false;
                                    }
                                    "end-file" => {
                                        let reason = val.get("reason").and_then(|r| r.as_str()).unwrap_or("");
                                        if reason == "eof" {
                                            st.eof = true;
                                        }
                                        st.is_playing = false;
                                        st.time_pos = 0.0;
                                        st.percent_pos = 0.0;
                                    }
                                    _ => {}
                                }
                            }

                            if val.get("event").and_then(|e| e.as_str()) == Some("property-change") {
                                if let (Some(name), data) = (
                                    val.get("name").and_then(|n| n.as_str()),
                                    val.get("data"),
                                ) {
                                    match name {
                                        "time-pos" => {
                                            if let Some(num) = data.and_then(|d| d.as_f64()) {
                                                st.time_pos = num;
                                                st.is_playing = true;
                                            }
                                        }
                                        "duration" => {
                                            if let Some(num) = data.and_then(|d| d.as_f64()) {
                                                st.duration = num;
                                            }
                                        }
                                        "percent-pos" => {
                                            if let Some(num) = data.and_then(|d| d.as_f64()) {
                                                st.percent_pos = num;
                                            }
                                        }
                                        "pause" => {
                                            if let Some(b) = data.and_then(|d| d.as_bool()) {
                                                st.is_paused = b;
                                            }
                                        }
                                        "media-title" => {
                                            if let Some(t) = data.and_then(|d| d.as_str()) {
                                                st.metadata.title = Some(t.to_string());
                                            }
                                        }
                                        "audio-codec-name" => {
                                            if let Some(c) = data.and_then(|d| d.as_str()) {
                                                st.metadata.codec = Some(c.to_uppercase());
                                            }
                                        }
                                        "audio-params" => {
                                            if let Some(obj) = data.and_then(|d| d.as_object()) {
                                                if let Some(sr) = obj.get("samplerate").and_then(|v| v.as_u64()) {
                                                    st.metadata.sample_rate = Some(sr as u32);
                                                }
                                                if let Some(ch) = obj.get("channel-count").and_then(|v| v.as_u64()) {
                                                    st.metadata.channels = Some(if ch == 2 { "Stereo".to_string() } else { format!("{} Ch", ch) });
                                                }
                                                if let Some(fmt) = obj.get("format").and_then(|v| v.as_str()) {
                                                    let lower = fmt.to_ascii_lowercase();
                                                    st.metadata.bit_depth = match lower.as_str() {
                                                        "u8" | "s8" => Some(8),
                                                        "s16" | "u16" | "s16p" => Some(16),
                                                        "s24" | "u24" | "s24p" => Some(24),
                                                        "s32" | "u32" | "s32p" | "float" | "floatp" => Some(32),
                                                        "double" | "doublep" => Some(64),
                                                        _ => {
                                                            if lower.contains("16") {
                                                                Some(16)
                                                            } else if lower.contains("24") {
                                                                Some(24)
                                                            } else if lower.contains("32") {
                                                                Some(32)
                                                            } else {
                                                                None
                                                            }
                                                        }
                                                    };
                                                }
                                            }
                                        }
                                        "audio-bitrate" => {
                                            if let Some(br) = data.and_then(|d| d.as_f64()) {
                                                st.metadata.bitrate = Some((br / 1000.0).round() as u32);
                                            }
                                        }
                                        "metadata" => {
                                            if let Some(obj) = data.and_then(|d| d.as_object()) {
                                                if let Some(t) = obj.get("title").or_else(|| obj.get("TITLE")).and_then(|v| v.as_str()) {
                                                    st.metadata.title = Some(t.to_string());
                                                }
                                                if let Some(a) = obj.get("artist").or_else(|| obj.get("ARTIST")).and_then(|v| v.as_str()) {
                                                    st.metadata.artist = Some(a.to_string());
                                                }
                                                if let Some(alb) = obj.get("album").or_else(|| obj.get("ALBUM")).and_then(|v| v.as_str()) {
                                                    st.metadata.album = Some(alb.to_string());
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    pub fn send_json_command(&self, cmd: serde_json::Value) {
        if !self.send_json_command_raw(cmd.clone()) {
            self.reconnect();
            let _ = self.send_json_command_raw(cmd);
        }
    }

    fn send_json_command_raw(&self, cmd: serde_json::Value) -> bool {
        let mut guard = self.stream.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut s) = *guard {
                let id = self.request_id.fetch_add(1, Ordering::SeqCst);
                let payload = if cmd.is_array() {
                    serde_json::json!({
                        "command": cmd,
                        "request_id": id
                    })
                } else {
                    cmd
                };
                let mut cmd_str = payload.to_string();
                cmd_str.push('\n');
            if s.write_all(cmd_str.as_bytes()).is_ok() && s.flush().is_ok() {
                return true;
            }
        }
        false
    }

    pub fn apply_audio_filter(&self, af_str: &str) {
        *self.current_af.lock().unwrap_or_else(|e| e.into_inner()) = af_str.to_string();
        if af_str.is_empty() {
            self.send_json_command(serde_json::json!(["set_property", "af", ""]));
        } else {
            self.send_json_command(serde_json::json!(["set_property", "af", af_str]));
        }
    }

    pub fn play(&self, url: &str) {
        let is_remote = url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ytdl://");
        if is_remote && !crate::api::stream::is_safe_stream_url(url) {
            let mut st = self.status.lock().unwrap_or_else(|e| e.into_inner());
            st.is_playing = false;
            st.is_paused = false;
            st.metadata.title = Some("⚠️ Blocked unsafe stream URL".to_string());
            return;
        }

        {
            let mut st = self.status.lock().unwrap_or_else(|e| e.into_inner());
            st.is_playing = true;
            st.is_paused = false;
            st.time_pos = 0.0;
            st.metadata = PlayerMetadata::default();
        }
        let vol = *self.master_volume.lock().unwrap_or_else(|e| e.into_inner());
        self.send_json_command(serde_json::json!(["set_property", "volume", vol]));
        let af = self.current_af.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if !af.is_empty() {
            self.send_json_command(serde_json::json!(["set_property", "af", af]));
        }
        self.send_json_command(serde_json::json!(["loadfile", url, "replace"]));
        self.send_json_command(serde_json::json!(["set_property", "pause", false]));
    }

    pub fn toggle_pause(&self) {
        let new_paused = {
            let mut st = self.status.lock().unwrap_or_else(|e| e.into_inner());
            st.is_paused = !st.is_paused;
            st.is_paused
        };
        self.send_json_command(serde_json::json!(["set_property", "pause", new_paused]));
    }

    pub fn stop(&self) {
        self.send_json_command(serde_json::json!(["stop"]));
        let mut st = self.status.lock().unwrap_or_else(|e| e.into_inner());
        st.is_playing = false;
        st.is_paused = false;
        st.eof = false;
        st.time_pos = 0.0;
    }

    pub fn seek(&self, seconds: f64) {
        self.send_json_command(serde_json::json!(["seek", seconds, "relative"]));
    }

    pub fn set_volume(&self, delta: i32) {
        let mut master = self.master_volume.lock().unwrap_or_else(|e| e.into_inner());
        let new_vol = (*master as i32 + delta).clamp(0, 100) as u32;
        *master = new_vol;
        self.send_json_command(serde_json::json!(["set_property", "volume", new_vol]));
    }

    pub fn get_status(&self) -> PlayerStatus {
        let mut guard = match self.status.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };
        let mut status = guard.clone();
        status.volume = match self.master_volume.lock() {
            Ok(g) => *g,
            Err(e) => *e.into_inner(),
        };
        // Clear eof pulse so auto-advance only fires once per track completion
        guard.eof = false;
        status
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.stop();
        if let Ok(mut guard) = self.process.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }
    }
}
