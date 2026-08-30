use crate::state::types::CrossfadeCurve;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const DECK_A_SOCKET: &str = "/tmp/boombox-rs-deck-a.sock";
pub const DECK_B_SOCKET: &str = "/tmp/boombox-rs-deck-b.sock";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckId {
    DeckA,
    DeckB,
}

impl DeckId {
    pub fn name(&self) -> &'static str {
        match self {
            DeckId::DeckA => "DECK-A",
            DeckId::DeckB => "DECK-B",
        }
    }

    pub fn other(&self) -> Self {
        match self {
            DeckId::DeckA => DeckId::DeckB,
            DeckId::DeckB => DeckId::DeckA,
        }
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
    pub metadata: PlayerMetadata,
    pub active_deck: String,
    pub is_crossfading: bool,
    pub crossfade_progress: f32,
}

/// Represents an individual physical cassette deck (MPV instance)
pub struct SingleDeck {
    id: DeckId,
    socket_path: String,
    process: Option<Child>,
    stream: Arc<Mutex<Option<UnixStream>>>,
    request_id: AtomicU64,
    pub status: Arc<Mutex<PlayerStatus>>,
    running: Arc<AtomicBool>,
}

impl SingleDeck {
    pub fn new(id: DeckId, socket_path: &str) -> Self {
        let deck = Self {
            id,
            socket_path: socket_path.to_string(),
            process: None,
            stream: Arc::new(Mutex::new(None)),
            request_id: AtomicU64::new(1),
            status: Arc::new(Mutex::new(PlayerStatus {
                volume: 80,
                active_deck: id.name().to_string(),
                ..Default::default()
            })),
            running: Arc::new(AtomicBool::new(true)),
        };
        deck
    }

    pub fn start(&mut self) {
        if Path::new(&self.socket_path).exists() {
            let _ = fs::remove_file(&self.socket_path);
        }

        let child = Command::new("mpv")
            .arg("--no-video")
            .arg("--idle=yes")
            .arg("--really-quiet")
            .arg("--gapless-audio=yes")
            .arg(format!("--input-ipc-server={}", self.socket_path))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if let Ok(child_proc) = child {
            self.process = Some(child_proc);

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
                                                    "s24" | "s24p" | "s32" | "s32p" => 24,
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

    pub fn play(&self, url: &str, start_pos: f64) {
        {
            let mut st = self.status.lock().unwrap();
            st.is_playing = true;
            st.is_paused = false;
            st.time_pos = start_pos;
        }
        self.send_json_command(serde_json::json!(["loadfile", url, "replace"]));
        if start_pos > 0.0 {
            // Give the demuxer a moment to load before seeking, otherwise the
            // seek can race the load and land at a wrong (jarring) position.
            thread::sleep(Duration::from_millis(180));
            self.send_json_command(serde_json::json!(["seek", start_pos, "absolute"]));
        }
        self.send_json_command(serde_json::json!(["set_property", "pause", false]));
    }

    pub fn set_volume_exact(&self, volume: u32) {
        {
            let mut st = self.status.lock().unwrap();
            st.volume = volume;
        }
        self.send_json_command(serde_json::json!(["set_property", "volume", volume]));
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
}

impl Drop for SingleDeck {
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

/// Cyberpunk Retro Cassette Dual-Deck Audio Engine
pub struct MpvPlayer {
    deck_a: Arc<SingleDeck>,
    deck_b: Arc<SingleDeck>,
    active_deck: Arc<Mutex<DeckId>>,
    master_volume: Arc<Mutex<u32>>,
    is_crossfading: Arc<AtomicBool>,
    crossfade_progress: Arc<Mutex<f32>>,
}

impl MpvPlayer {
    pub fn new() -> Self {
        let mut d_a = SingleDeck::new(DeckId::DeckA, DECK_A_SOCKET);
        d_a.start();
        let mut d_b = SingleDeck::new(DeckId::DeckB, DECK_B_SOCKET);
        d_b.start();

        Self {
            deck_a: Arc::new(d_a),
            deck_b: Arc::new(d_b),
            active_deck: Arc::new(Mutex::new(DeckId::DeckA)),
            master_volume: Arc::new(Mutex::new(80)),
            is_crossfading: Arc::new(AtomicBool::new(false)),
            crossfade_progress: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn active_deck_ref(&self) -> Arc<SingleDeck> {
        let current = *self.active_deck.lock().unwrap();
        match current {
            DeckId::DeckA => Arc::clone(&self.deck_a),
            DeckId::DeckB => Arc::clone(&self.deck_b),
        }
    }

    pub fn inactive_deck_ref(&self) -> Arc<SingleDeck> {
        let current = *self.active_deck.lock().unwrap();
        match current {
            DeckId::DeckA => Arc::clone(&self.deck_b),
            DeckId::DeckB => Arc::clone(&self.deck_a),
        }
    }

    pub fn play(&self, url: &str) {
        self.is_crossfading.store(false, Ordering::SeqCst);
        let active = self.active_deck_ref();
        let vol = *self.master_volume.lock().unwrap();
        active.set_volume_exact(vol);
        active.play(url, 0.0);

        // Ensure other deck is silent
        self.inactive_deck_ref().stop();
    }

    /// Trigger seamless Smart Crossfade / AI-DJ transition from current deck to target deck
    pub fn start_crossfade(
        &self,
        next_url: &str,
        duration_sec: f64,
        curve: CrossfadeCurve,
        mix_in_sec: f64,
        strategy: crate::audio::neural::DjTransitionStrategy,
        tempo_sync_ratio: f64,
    ) {
        use crate::audio::neural::DjTransitionStrategy;

        if self.is_crossfading.load(Ordering::SeqCst) {
            return;
        }

        self.is_crossfading.store(true, Ordering::SeqCst);
        let outgoing_deck = self.active_deck_ref();
        let incoming_deck = self.inactive_deck_ref();
        let next_deck_id = self.active_deck.lock().unwrap().other();

        let base_vol = *self.master_volume.lock().unwrap();
        let is_xfading_flag = Arc::clone(&self.is_crossfading);
        let progress_state = Arc::clone(&self.crossfade_progress);
        let active_deck_state = Arc::clone(&self.active_deck);

        // Start incoming deck with 0 initial volume and synced tempo speed
        incoming_deck.set_volume_exact(0);
        if (tempo_sync_ratio - 1.0).abs() > 0.005 {
            incoming_deck.send_json_command(serde_json::json!(["set_property", "speed", tempo_sync_ratio]));
        }
        incoming_deck.play(next_url, mix_in_sec);

        // Initial Filter Setup based on Neural DJ Strategy
        match strategy {
            DjTransitionStrategy::BassSwap => {
                // Incoming deck starts with Bass Cutoff (>280Hz) to prevent vocal/bass collision
                incoming_deck.send_json_command(serde_json::json!(["af", "set", "lavfi=[highpass=f=280]"]));
            }
            DjTransitionStrategy::FilterSweep => {
                incoming_deck.send_json_command(serde_json::json!(["af", "set", "lavfi=[lowpass=f=1500]"]));
            }
            DjTransitionStrategy::EchoOutDrop => {
                // Outgoing deck gets 1/2 beat Reverb & Echo Tail
                outgoing_deck.send_json_command(serde_json::json!(["af", "set", "lavfi=[aecho=0.8:0.88:120:0.4]"]));
            }
            DjTransitionStrategy::BreakdownCut => {}
        }

        thread::spawn(move || {
            let total_steps = (duration_sec * 20.0).max(10.0) as usize; // 50ms tick
            let step_delay = Duration::from_millis(50);
            let mut bass_swapped = false;

            for step in 0..=total_steps {
                if !is_xfading_flag.load(Ordering::SeqCst) {
                    break;
                }

                let t = (step as f64) / (total_steps as f64); // 0.0 -> 1.0
                *progress_state.lock().unwrap() = t as f32;

                // 1. Dynamic DSP Filter Modulation
                match strategy {
                    DjTransitionStrategy::BassSwap => {
                        if t < 0.5 {
                            // First Half: Smoothly roll off outgoing sub-bass (30Hz -> 220Hz)
                            let cutoff = (30.0 + t * 380.0).round() as u32;
                            outgoing_deck.send_json_command(serde_json::json!(["af", "set", format!("lavfi=[highpass=f={}]", cutoff)]));
                        } else if !bass_swapped {
                            // ⚡ MIDPOINT DOWNBEAT SNAP: BASS SWAP!
                            // Instant kill of outgoing bass + Full low-end release on incoming deck
                            outgoing_deck.send_json_command(serde_json::json!(["af", "set", "lavfi=[highpass=f=550]"]));
                            incoming_deck.send_json_command(serde_json::json!(["af", "set", ""])); // Full punchy bass
                            bass_swapped = true;
                        }
                    }
                    DjTransitionStrategy::FilterSweep => {
                        let hp_cutoff = (30.0 + t * 650.0).round() as u32;
                        let lp_cutoff = (1500.0 + t * 18500.0).round() as u32;
                        outgoing_deck.send_json_command(serde_json::json!(["af", "set", format!("lavfi=[highpass=f={}]", hp_cutoff)]));
                        incoming_deck.send_json_command(serde_json::json!(["af", "set", format!("lavfi=[lowpass=f={}]", lp_cutoff)]));
                    }
                    DjTransitionStrategy::EchoOutDrop => {
                        // Quick exponential drop into echo tail
                    }
                    DjTransitionStrategy::BreakdownCut => {}
                }

                // 2. Volume Envelope Curves
                let (gain_out, gain_in) = match strategy {
                    DjTransitionStrategy::EchoOutDrop => {
                        // Fast decay for outgoing deck while reverb tail sustains
                        let out = (1.0 - t).powi(2);
                        let in_gain = (t * std::f64::consts::FRAC_PI_2).sin();
                        (out, in_gain)
                    }
                    _ => match curve {
                        CrossfadeCurve::EqualPower => {
                            let angle = t * std::f64::consts::FRAC_PI_2;
                            (angle.cos(), angle.sin())
                        }
                        CrossfadeCurve::SmoothExponential => {
                            let smooth = t * t * (3.0 - 2.0 * t);
                            (1.0 - smooth, smooth)
                        }
                        CrossfadeCurve::Linear => {
                            (1.0 - t, t)
                        }
                    },
                };

                let vol_out = ((base_vol as f64) * gain_out).round().clamp(0.0, 100.0) as u32;
                let vol_in = ((base_vol as f64) * gain_in).round().clamp(0.0, 100.0) as u32;

                outgoing_deck.set_volume_exact(vol_out);
                incoming_deck.set_volume_exact(vol_in);

                thread::sleep(step_delay);
            }

            // Finalize transition & Clear DSP Filters
            outgoing_deck.stop();
            outgoing_deck.send_json_command(serde_json::json!(["af", "set", ""]));
            outgoing_deck.set_volume_exact(base_vol);

            incoming_deck.send_json_command(serde_json::json!(["af", "set", ""]));
            incoming_deck.send_json_command(serde_json::json!(["set_property", "speed", 1.0])); // Restore natural tempo
            incoming_deck.set_volume_exact(base_vol);

            *active_deck_state.lock().unwrap() = next_deck_id;
            *progress_state.lock().unwrap() = 0.0;
            is_xfading_flag.store(false, Ordering::SeqCst);
        });
    }

    pub fn toggle_pause(&self) {
        let active = self.active_deck_ref();
        let mut st = active.status.lock().unwrap();
        st.is_paused = !st.is_paused;
        active.send_json_command(serde_json::json!(["cycle", "pause"]));
    }

    pub fn stop(&self) {
        self.is_crossfading.store(false, Ordering::SeqCst);
        self.deck_a.stop();
        self.deck_b.stop();
    }

    pub fn seek(&self, seconds: f64) {
        let active = self.active_deck_ref();
        active.send_json_command(serde_json::json!(["seek", seconds, "relative"]));
    }

    pub fn set_volume(&self, delta: i32) {
        let mut master = self.master_volume.lock().unwrap();
        let new_vol = (*master as i32 + delta).clamp(0, 100) as u32;
        *master = new_vol;

        if !self.is_crossfading.load(Ordering::SeqCst) {
            let active = self.active_deck_ref();
            active.set_volume_exact(new_vol);
        }
    }

    pub fn get_status(&self) -> PlayerStatus {
        let active_deck_id = *self.active_deck.lock().unwrap();
        let mut status = self.active_deck_ref().status.lock().unwrap().clone();
        status.active_deck = active_deck_id.name().to_string();
        status.volume = *self.master_volume.lock().unwrap();
        status.is_crossfading = self.is_crossfading.load(Ordering::SeqCst);
        status.crossfade_progress = *self.crossfade_progress.lock().unwrap();
        status
    }
}
