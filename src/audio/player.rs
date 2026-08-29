use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const MPV_SOCKET_PATH: &str = "/tmp/boombox-rs-mpv.sock";

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
    pub metadata: PlayerMetadata,
}

pub struct MpvPlayer {
    process: Option<Child>,
    socket_path: String,
    stream: Arc<Mutex<Option<UnixStream>>>,
    request_id: AtomicU64,
    status: Arc<Mutex<PlayerStatus>>,
    running: Arc<AtomicBool>,
}

impl MpvPlayer {
    pub fn new() -> Self {
        let mut player = Self {
            process: None,
            socket_path: MPV_SOCKET_PATH.to_string(),
            stream: Arc::new(Mutex::new(None)),
            request_id: AtomicU64::new(1),
            status: Arc::new(Mutex::new(PlayerStatus {
                volume: 80,
                ..Default::default()
            })),
            running: Arc::new(AtomicBool::new(true)),
        };
        player.start_engine();
        player
    }

    pub fn start_engine(&mut self) {
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let child = Command::new("mpv")
            .arg("--no-video")
            .arg("--idle=yes")
            .arg("--really-quiet")
            .arg(format!("--input-ipc-server={}", self.socket_path))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if let Ok(child_proc) = child {
            self.process = Some(child_proc);

            // Wait for socket to become ready
            for _ in 0..50 {
                thread::sleep(Duration::from_millis(30));
                if let Ok(s) = UnixStream::connect(&self.socket_path) {
                    let _ = s.set_nonblocking(false);
                    if let Ok(reader_stream) = s.try_clone() {
                        *self.stream.lock().unwrap() = Some(s);
                        self.spawn_reader_thread(reader_stream);
                        self.observe_properties();
                        break;
                    }
                }
            }
        }
    }

    fn observe_properties(&self) {
        self.send_json_command(serde_json::json!(["observe_property", 1, "time-pos"]));
        self.send_json_command(serde_json::json!(["observe_property", 2, "duration"]));
        self.send_json_command(serde_json::json!(["observe_property", 3, "percent-pos"]));
        self.send_json_command(serde_json::json!(["observe_property", 4, "pause"]));
        self.send_json_command(serde_json::json!(["observe_property", 5, "metadata"]));
        self.send_json_command(serde_json::json!(["observe_property", 6, "audio-codec-name"]));
        self.send_json_command(serde_json::json!(["observe_property", 7, "audio-bitrate"]));
        self.send_json_command(serde_json::json!(["observe_property", 8, "audio-params"]));
        self.send_json_command(serde_json::json!(["set_property", "volume", 80]));
    }

    fn spawn_reader_thread(&self, stream: UnixStream) {
        let status = Arc::clone(&self.status);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                if let Ok(l) = line {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&l) {
                        if let Some(event) = val.get("event").and_then(|v| v.as_str()) {
                            if event == "property-change" {
                                let name = val.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let data = val.get("data");

                                let mut st = status.lock().unwrap();
                                match name {
                                    "time-pos" => {
                                        if let Some(num) = data.and_then(|v| v.as_f64()) {
                                            st.time_pos = num;
                                            st.is_playing = true;
                                        }
                                    }
                                    "duration" => {
                                        if let Some(num) = data.and_then(|v| v.as_f64()) {
                                            st.duration = num;
                                        }
                                    }
                                    "percent-pos" => {
                                        if let Some(num) = data.and_then(|v| v.as_f64()) {
                                            st.percent_pos = num;
                                        }
                                    }
                                    "pause" => {
                                        if let Some(b) = data.and_then(|v| v.as_bool()) {
                                            st.is_paused = b;
                                        }
                                    }
                                    "audio-codec-name" => {
                                        if let Some(s) = data.and_then(|v| v.as_str()) {
                                            st.metadata.codec = Some(s.to_uppercase());
                                        }
                                    }
                                    "audio-bitrate" => {
                                        if let Some(num) = data.and_then(|v| v.as_u64()) {
                                            st.metadata.bitrate = Some((num / 1000) as u32);
                                        }
                                    }
                                    "audio-params" => {
                                        if let Some(obj) = data.and_then(|v| v.as_object()) {
                                            if let Some(sr) = obj.get("samplerate").and_then(|v| v.as_u64()) {
                                                st.metadata.sample_rate = Some(sr as u32);
                                            }
                                            if let Some(fmt) = obj.get("format").and_then(|v| v.as_str()) {
                                                let bits = match fmt {
                                                    "s16" | "s16p" => 16,
                                                    "s24" | "s24p" => 24,
                                                    "s32" | "s32p" => 24, // ALSA/FLAC 24-in-32 bit container
                                                    "float" | "floatp" => 32,
                                                    _ => 16,
                                                };
                                                st.metadata.bit_depth = Some(bits);
                                            }
                                            if let Some(ch) = obj.get("channels").and_then(|v| v.as_str()) {
                                                st.metadata.channels = Some(ch.to_string());
                                            }
                                        }
                                    }
                                    "metadata" => {
                                        if let Some(obj) = data.and_then(|v| v.as_object()) {
                                            if let Some(t) = obj.get("icy-title").or_else(|| obj.get("title")).and_then(|v| v.as_str()) {
                                                st.metadata.title = Some(t.to_string());
                                            }
                                            if let Some(a) = obj.get("artist").or_else(|| obj.get("Artist")).and_then(|v| v.as_str()) {
                                                st.metadata.artist = Some(a.to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn send_json_command(&self, cmd: serde_json::Value) {
        let mut guard = self.stream.lock().unwrap();
        if let Some(ref mut stream) = *guard {
            let id = self.request_id.fetch_add(1, Ordering::SeqCst);
            let cmd_json = serde_json::json!({
                "command": cmd,
                "request_id": id
            });
            let _ = writeln!(stream, "{}", cmd_json);
            let _ = stream.flush();
        }
    }

    pub fn play(&self, url: &str) {
        {
            let mut st = self.status.lock().unwrap();
            st.is_playing = true;
            st.is_paused = false;
            st.time_pos = 0.0;
        }
        self.send_json_command(serde_json::json!(["loadfile", url, "replace"]));
        self.send_json_command(serde_json::json!(["set_property", "pause", false]));
    }

    pub fn toggle_pause(&self) {
        let mut st = self.status.lock().unwrap();
        st.is_paused = !st.is_paused;
        self.send_json_command(serde_json::json!(["cycle", "pause"]));
    }

    pub fn stop(&self) {
        {
            let mut st = self.status.lock().unwrap();
            st.is_playing = false;
            st.is_paused = false;
            st.time_pos = 0.0;
        }
        self.send_json_command(serde_json::json!(["stop"]));
    }

    pub fn seek(&self, seconds: f64) {
        self.send_json_command(serde_json::json!(["seek", seconds, "relative"]));
    }

    pub fn set_volume(&self, delta: i32) {
        let mut st = self.status.lock().unwrap();
        let new_vol = (st.volume as i32 + delta).clamp(0, 100) as u32;
        st.volume = new_vol;
        self.send_json_command(serde_json::json!(["set_property", "volume", new_vol]));
    }

    pub fn get_status(&self) -> PlayerStatus {
        self.status.lock().unwrap().clone()
    }
}

impl Drop for MpvPlayer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.stop();
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }
    }
}
