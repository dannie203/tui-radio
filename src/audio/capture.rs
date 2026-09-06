use crate::audio::visualizer::ISO_32_BANDS;
use rustfft::{num_complex::Complex, FftPlanner};
#[cfg(target_os = "linux")]
use std::io::Read;
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "linux")]
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
    pub raw_wave_left: [f32; 512],
    pub raw_wave_right: [f32; 512],
}

impl Default for LiveAudioData {
    fn default() -> Self {
        Self {
            raw_bands: [0.0; 32],
            raw_vu_left: 0.0,
            raw_vu_right: 0.0,
            sample_rate: 48000,
            last_update: Instant::now(),
            raw_wave_left: [0.0; 512],
            raw_wave_right: [0.0; 512],
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
        self.data.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Drop for AudioCaptureEngine {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Shared real-time DSP calculation (RMS dB VU, 32-band FFT, Oscilloscope waveform)
pub fn process_pcm_frame(
    byte_buffer: &[u8],
    fft_size: usize,
    current_sample_rate: u32,
    hann_window: &[f32],
    fft: &dyn rustfft::Fft<f32>,
    fft_buffer: &mut [Complex<f32>],
    bin_width: f32,
    factor_lower: f32,
    factor_upper: f32,
    data: &Arc<Mutex<LiveAudioData>>,
) {
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
    fft.process(fft_buffer);

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

    // 3. Extract Real-Time PCM Audio Waveform for Oscilloscope
    let mut raw_wave_l = [0.0f32; 512];
    let mut raw_wave_r = [0.0f32; 512];
    let start_wave = fft_size.saturating_sub(512);
    for (w_idx, src_i) in (start_wave..fft_size).enumerate() {
        let offset = src_i * 4;
        raw_wave_l[w_idx] = i16::from_le_bytes([byte_buffer[offset], byte_buffer[offset + 1]]) as f32 / 32768.0;
        raw_wave_r[w_idx] = i16::from_le_bytes([byte_buffer[offset + 2], byte_buffer[offset + 3]]) as f32 / 32768.0;
    }

    // 4. Store Live Telemetry
    if let Ok(mut guard) = data.lock() {
        guard.raw_bands = raw_bands;
        guard.raw_vu_left = raw_vu_l;
        guard.raw_vu_right = raw_vu_r;
        guard.sample_rate = current_sample_rate;
        guard.last_update = Instant::now();
        guard.raw_wave_left = raw_wave_l;
        guard.raw_wave_right = raw_wave_r;
    }
}

// ==============================================================================
//  LINUX BACKEND: PipeWire (pw-record) & PulseAudio (parec)
// ==============================================================================
#[cfg(target_os = "linux")]
pub fn detect_pipewire_sample_rate() -> u32 {
    // 1. Fast path: query PipeWire global settings metadata directly (<10ms)
    if let Ok(output) = Command::new("pw-metadata")
        .args(["-n", "settings", "0", "clock.rate"])
        .output()
    {
        if let Ok(text) = String::from_utf8(output.stdout) {
            for line in text.lines() {
                if line.contains("clock.rate") {
                    if let Some(pos) = line.find("value:'") {
                        let sub = &line[pos + 7..];
                        let digits: String = sub.chars().take_while(|c| c.is_ascii_digit()).collect();
                        if let Ok(rate) = digits.parse::<u32>() {
                            if (22050..=384000).contains(&rate) {
                                return rate;
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Fallback: pw-dump
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

#[cfg(target_os = "linux")]
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
        let mut pw_cmd = Command::new("pw-record");
        pw_cmd
            .args([
                "-P", "stream.capture.sink=true",
                "--rate", &rate_str,
                "--channels", "2",
                "--format", "s16",
                "--raw", "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        unsafe {
            pw_cmd.pre_exec(|| {
                libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                Ok(())
            });
        }

        let child: Option<Child> = pw_cmd.spawn().ok().or_else(|| {
            let parec_arg = format!("--rate={}", rate_str);
            let mut parec_cmd = Command::new("parec");
            parec_cmd
                .args([
                    "-d", "@DEFAULT_MONITOR@",
                    &parec_arg,
                    "--channels=2",
                    "--format=s16le",
                    "--raw",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());
            unsafe {
                parec_cmd.pre_exec(|| {
                    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
                    Ok(())
                });
            }
            parec_cmd.spawn().ok()
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
                            process_pcm_frame(
                                &byte_buffer,
                                fft_size,
                                current_sample_rate,
                                &hann_window,
                                fft.as_ref(),
                                &mut fft_buffer,
                                bin_width,
                                factor_lower,
                                factor_upper,
                                &data,
                            );
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

// ==============================================================================
//  WINDOWS BACKEND: WASAPI Loopback Capture (Desktop Audio Stream)
// ==============================================================================
#[cfg(windows)]
fn run_capture_loop(data: Arc<Mutex<LiveAudioData>>, running: Arc<AtomicBool>) {
    let _ = wasapi::initialize_mta();
    let current_sample_rate = 48000u32;
    let fft_size = 2048;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut hann_window = vec![0.0f32; fft_size];
    for i in 0..fft_size {
        hann_window[i] = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos());
    }

    let mut fft_buffer = vec![Complex { re: 0.0f32, im: 0.0f32 }; fft_size];
    let bin_width = current_sample_rate as f32 / fft_size as f32;
    let factor_lower = 2.0f32.powf(-1.0 / 6.0);
    let factor_upper = 2.0f32.powf(1.0 / 6.0);
    let mut byte_buffer = vec![0u8; fft_size * 4];

    while running.load(Ordering::Relaxed) {
        let capture_res: Result<(), Box<dyn std::error::Error>> = (|| {
            let enumerator = wasapi::DeviceEnumerator::new()?;
            let device = enumerator.get_default_device(&wasapi::Direction::Render)?;
            let mut audio_client = device.get_iaudioclient()?;
            let desired_format = wasapi::WaveFormat::new(
                16,
                16,
                &wasapi::SampleType::Int,
                current_sample_rate as usize,
                2,
                None,
            );
            let (def_time, _) = audio_client.get_device_period().unwrap_or((100000, 100000));
            let mode = wasapi::StreamMode::PollingShared {
                autoconvert: true,
                buffer_duration_hns: def_time,
            };
            audio_client.initialize_client(&desired_format, &wasapi::Direction::Capture, &mode)?;
            let capture_client = audio_client.get_audiocaptureclient()?;
            audio_client.start_stream()?;

            let mut sample_queue = std::collections::VecDeque::<u8>::with_capacity(fft_size * 8);

            while running.load(Ordering::Relaxed) {
                let _ = capture_client.read_from_device_to_deque(&mut sample_queue);
                if sample_queue.len() >= fft_size * 4 {
                    for b in byte_buffer.iter_mut() {
                        *b = sample_queue.pop_front().unwrap_or(0);
                    }
                    process_pcm_frame(
                        &byte_buffer,
                        fft_size,
                        current_sample_rate,
                        &hann_window,
                        fft.as_ref(),
                        &mut fft_buffer,
                        bin_width,
                        factor_lower,
                        factor_upper,
                        &data,
                    );
                } else {
                    thread::sleep(Duration::from_millis(15));
                }
            }
            Ok(())
        })();

        if capture_res.is_err() {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

// ==============================================================================
//  FALLBACK BACKEND (macOS & Other POSIX): Graceful simulation fallback
// ==============================================================================
#[cfg(all(unix, not(target_os = "linux")))]
fn run_capture_loop(_data: Arc<Mutex<LiveAudioData>>, running: Arc<AtomicBool>) {
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));
    }
}
