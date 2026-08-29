use crate::audio::visualizer::ISO_32_BANDS;
use rustfft::{num_complex::Complex, FftPlanner};
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct LiveAudioData {
    pub raw_bands: [f32; 32],
    pub raw_vu_left: f32,
    pub raw_vu_right: f32,
    pub sample_rate: u32,
    pub last_update: Instant,
}

impl Default for LiveAudioData {
    fn default() -> Self {
        Self {
            raw_bands: [0.0; 32],
            raw_vu_left: 0.0,
            raw_vu_right: 0.0,
            sample_rate: 48000,
            last_update: Instant::now(),
        }
    }
}

pub struct AudioCaptureEngine {
    data: Arc<Mutex<LiveAudioData>>,
    running: Arc<AtomicBool>,
}

impl AudioCaptureEngine {
    pub fn new() -> Self {
        let data = Arc::new(Mutex::new(LiveAudioData::default()));
        let running = Arc::new(AtomicBool::new(true));

        let data_clone = Arc::clone(&data);
        let running_clone = Arc::clone(&running);

        thread::Builder::new()
            .name("boombox-audio-capture".to_string())
            .spawn(move || {
                run_capture_loop(data_clone, running_clone);
            })
            .expect("Failed to spawn audio capture thread");

        Self { data, running }
    }

    pub fn get_live_data(&self) -> LiveAudioData {
        self.data.lock().unwrap().clone()
    }
}

impl Drop for AudioCaptureEngine {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Detects the active PipeWire / EasyEffects clock sample rate in real-time
pub fn detect_pipewire_sample_rate() -> u32 {
    if let Ok(output) = Command::new("pw-dump").output() {
        if let Ok(text) = String::from_utf8(output.stdout) {
            if let Some(pos) = text.find("\"key\": \"clock.rate\"") {
                let slice = &text[pos..std::cmp::min(pos + 120, text.len())];
                if let Some(val_idx) = slice.find("\"value\":") {
                    let sub = &slice[val_idx + 8..];
                    let digits: String = sub
                        .chars()
                        .skip_while(|c| !c.is_ascii_digit())
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(rate) = digits.parse::<u32>() {
                        if (22050..=384000).contains(&rate) {
                            return rate;
                        }
                    }
                }
            }
        }
    }
    48000 // Linux / PipeWire default clock rate
}

fn run_capture_loop(data: Arc<Mutex<LiveAudioData>>, running: Arc<AtomicBool>) {
    let mut last_rate_check = Instant::now() - Duration::from_secs(10);
    let mut current_sample_rate = 48000u32;

    while running.load(Ordering::Relaxed) {
        // 1. Detect dynamic PipeWire / EasyEffects graph rate
        if last_rate_check.elapsed() > Duration::from_secs(4) {
            current_sample_rate = detect_pipewire_sample_rate();
            last_rate_check = Instant::now();
        }

        // Dynamically scale FFT size to maintain ~20Hz-25Hz bin resolution across 44.1k, 48k, 96k, 192k
        let fft_size = match current_sample_rate {
            r if r <= 48000 => 2048,
            r if r <= 96000 => 4096,
            _ => 8192,
        };

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(fft_size);

        // Compute dynamic Hann Window
        let mut hann_window = vec![0.0f32; fft_size];
        for i in 0..fft_size {
            hann_window[i] = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos());
        }

        let mut fft_buffer = vec![Complex { re: 0.0f32, im: 0.0f32 }; fft_size];
        let mut byte_buffer = vec![0u8; fft_size * 4]; // (S16LE stereo = 4 bytes per frame)

        let rate_str = current_sample_rate.to_string();
        let child: Option<Child> = Command::new("pw-record")
            .args(["--rate", &rate_str, "--channels", "2", "--format", "s16", "--raw", "-"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()
            .or_else(|| {
                let parec_arg = format!("--rate={}", rate_str);
                Command::new("parec")
                    .args([&parec_arg, "--channels=2", "--format=s16le", "--raw"])
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .ok()
            });

        if let Some(mut proc) = child {
            if let Some(ref mut stdout) = proc.stdout {
                let bin_width = current_sample_rate as f32 / fft_size as f32;
                let factor_lower = 2.0f32.powf(-1.0 / 6.0);
                let factor_upper = 2.0f32.powf(1.0 / 6.0);

                while running.load(Ordering::Relaxed) {
                    // Periodic sample rate change detector
                    if last_rate_check.elapsed() > Duration::from_secs(4) {
                        let new_rate = detect_pipewire_sample_rate();
                        last_rate_check = Instant::now();
                        if new_rate != current_sample_rate {
                            current_sample_rate = new_rate;
                            break; // Break loop to re-instantiate FFT and capture stream with new dynamic rate
                        }
                    }

                    match stdout.read_exact(&mut byte_buffer) {
                        Ok(()) => {
                            let mut sum_sq_l = 0.0f32;
                            let mut sum_sq_r = 0.0f32;

                            for i in 0..fft_size {
                                let offset = i * 4;
                                let l_raw = i16::from_le_bytes([byte_buffer[offset], byte_buffer[offset + 1]]) as f32 / 32768.0;
                                let r_raw = i16::from_le_bytes([byte_buffer[offset + 2], byte_buffer[offset + 3]]) as f32 / 32768.0;

                                sum_sq_l += l_raw * l_raw;
                                sum_sq_r += r_raw * r_raw;

                                let mono = (l_raw + r_raw) * 0.5 * hann_window[i];
                                fft_buffer[i] = Complex { re: mono, im: 0.0 };
                            }

                            // 1. Calculate Real RMS dB for Stereo VU
                            let rms_l = (sum_sq_l / fft_size as f32).sqrt();
                            let rms_r = (sum_sq_r / fft_size as f32).sqrt();
                            let db_l = 20.0 * (rms_l.max(1e-5)).log10();
                            let db_r = 20.0 * (rms_r.max(1e-5)).log10();
                            let raw_vu_l = ((db_l + 50.0) * 2.0).clamp(0.0, 100.0);
                            let raw_vu_r = ((db_r + 50.0) * 2.0).clamp(0.0, 100.0);

                            // 2. Compute Real FFT Frequency Decomposition
                            fft.process(&mut fft_buffer);

                            let mut raw_bands = [0.0f32; 32];

                            for (b, &center_freq) in ISO_32_BANDS.iter().enumerate() {
                                let low_freq = center_freq * factor_lower;
                                let high_freq = center_freq * factor_upper;

                                let start_bin = ((low_freq / bin_width).floor() as usize).max(1);
                                let end_bin = ((high_freq / bin_width).ceil() as usize).min(fft_size / 2 - 1);

                                let mut band_sum_sq = 0.0f32;
                                let mut max_mag = 0.0f32;
                                let mut count = 0;

                                for bin in start_bin..=end_bin {
                                    let mag = fft_buffer[bin].norm() * (4.0 / fft_size as f32);
                                    band_sum_sq += mag * mag;
                                    if mag > max_mag {
                                        max_mag = mag;
                                    }
                                    count += 1;
                                }

                                let band_rms = if count > 0 { (band_sum_sq / count as f32).sqrt() } else { 0.0 };
                                let eff_mag = (band_rms * 0.65 + max_mag * 0.35).max(1e-5);
                                let db = 20.0 * eff_mag.log10();

                                // ISO 226 / Pink-noise acoustic slope compensation (+4.5dB/octave)
                                let tilt_db = (b as f32 / 31.0).powf(0.85) * 34.0;
                                let equalized_db = db + tilt_db;
                                let normalized = ((equalized_db + 66.0) * 1.55).clamp(0.0, 100.0);
                                raw_bands[b] = normalized;
                            }

                            // 3. Store Live Telemetry
                            if let Ok(mut guard) = data.lock() {
                                guard.raw_bands = raw_bands;
                                guard.raw_vu_left = raw_vu_l;
                                guard.raw_vu_right = raw_vu_r;
                                guard.sample_rate = current_sample_rate;
                                guard.last_update = Instant::now();
                            }
                        }
                        Err(_) => break, // Pipe closed or buffer error, respawn
                    }
                }
            }
            let _ = proc.kill();
        }

        // Retry backoff
        thread::sleep(Duration::from_millis(300));
    }
}
