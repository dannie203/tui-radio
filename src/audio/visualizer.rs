use crate::audio::capture::LiveAudioData;
use std::time::Instant;

pub const NUM_BANDS: usize = 32;

/// Standard ISO 266 / IEC 61260 1/3-Octave Nominal Center Frequencies (20Hz - 20kHz)
pub const ISO_32_BANDS: [f32; 32] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0,
    125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0,
    800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0,
    5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 18000.0, 20000.0,
];

pub struct VisualizerEngine {
    smoothed_bands: [f32; 32],
    peak_bands: [f32; 32],
    peak_timers: [Instant; 32],
    smoothed_vu_l: f32,
    smoothed_vu_r: f32,
    peak_vu_l: f32,
    peak_vu_r: f32,
    peak_timer_l: Instant,
    peak_timer_r: Instant,
    attack_alpha: f32,
    release_alpha: f32,
    peak_hold_secs: f32,
    peak_decay_rate: f32,
    phase: f32,
    last_update: Instant,
}

impl VisualizerEngine {
    pub fn new(_fft_size: usize) -> Self {
        let now = Instant::now();
        Self {
            smoothed_bands: [0.0; 32],
            peak_bands: [0.0; 32],
            peak_timers: [now; 32],
            smoothed_vu_l: 0.0,
            smoothed_vu_r: 0.0,
            peak_vu_l: 0.0,
            peak_vu_r: 0.0,
            peak_timer_l: now,
            peak_timer_r: now,
            attack_alpha: 0.92,
            release_alpha: 0.32,
            peak_hold_secs: 0.75,
            peak_decay_rate: 58.0,
            phase: 0.0,
            last_update: now,
        }
    }

    pub fn set_ballistics(&mut self, attack: f32, release: f32) {
        self.attack_alpha = attack;
        self.release_alpha = release;
    }

    /// Process real-time PCM audio spectrum and VU meters from live capture
    pub fn update_with_live_audio(
        &mut self,
        live: &LiveAudioData,
        is_playing: bool,
        is_paused: bool,
        bass_boost: bool,
        eq_gains: [f32; 32],
    ) -> ([f32; 32], [f32; 32], f32, f32, f32, f32) {
        let now = Instant::now();
        let dt = (now.duration_since(self.last_update).as_secs_f32()).clamp(0.001, 0.1);
        self.last_update = now;

        if !is_playing || is_paused {
            // Smooth natural decay to noise floor
            for i in 0..32 {
                self.smoothed_bands[i] = (self.smoothed_bands[i] - self.peak_decay_rate * dt * 2.2).max(0.0);
                self.peak_bands[i] = (self.peak_bands[i] - self.peak_decay_rate * dt * 1.5).max(self.smoothed_bands[i]);
            }
            self.smoothed_vu_l = (self.smoothed_vu_l - 60.0 * dt).max(0.0);
            self.smoothed_vu_r = (self.smoothed_vu_r - 60.0 * dt).max(0.0);
            self.peak_vu_l = (self.peak_vu_l - 40.0 * dt).max(self.smoothed_vu_l);
            self.peak_vu_r = (self.peak_vu_r - 40.0 * dt).max(self.smoothed_vu_r);
            return (
                self.smoothed_bands,
                self.peak_bands,
                self.smoothed_vu_l,
                self.smoothed_vu_r,
                self.peak_vu_l,
                self.peak_vu_r,
            );
        }

        let is_live_active = now.duration_since(live.last_update).as_millis() < 600;
        let mut raw_bands;
        let raw_vu_l;
        let raw_vu_r;

        if is_live_active {
            // 1. Use REAL live audio data from PipeWire / PulseAudio!
            raw_bands = live.raw_bands;
            raw_vu_l = live.raw_vu_left;
            raw_vu_r = live.raw_vu_right;

            // Apply Bass Boost multiplier on live low bands (< 100Hz)
            if bass_boost {
                for b in 0..8 {
                    raw_bands[b] = (raw_bands[b] * 1.35).min(100.0);
                }
            }
        } else {
            // 2. Organic fallback simulation if audio stream is silent/initializing
            self.phase += dt * 4.2;
            let p = self.phase;
            let bass_mul = if bass_boost { 1.35 } else { 1.0 };

            let sub_bass = ((p * 2.4).sin().abs()).powf(1.5) * 62.0 * bass_mul;
            let kick_transient = ((p * 4.8).sin().max(0.0)).powf(3.0) * 96.0 * bass_mul;
            let bass_warmth = ((p * 3.4 + 0.8).sin() * 0.5 + 0.5) * 78.0 * bass_mul;
            let vocal_mid = ((p * 5.8).sin() * 0.5 + 0.5) * 68.0;
            let snare_crack = ((p * 2.4 + 1.5).sin().max(0.0)).powf(3.5) * 84.0;
            let presence_lead = ((p * 4.8 + 1.2).cos() * 0.5 + 0.5) * 64.0;
            let hihat_tick = ((p * 9.2).sin().abs()).powf(2.2) * 76.0;
            let air_shimmer = ((p * 7.4 + 2.4).cos() * 0.5 + 0.5) * 58.0;

            raw_bands = [0.0f32; 32];
            for b in 0..32 {
                let fb = b as f32;
                let val = if b < 3 {
                    sub_bass * (0.75 + (fb / 3.0) * 0.25) + kick_transient * 0.35
                } else if b < 7 {
                    let center = 1.0 - ((fb - 5.0).abs() / 3.0);
                    kick_transient * center.max(0.55) + sub_bass * 0.3
                } else if b < 12 {
                    let center = 1.0 - ((fb - 9.5).abs() / 3.5);
                    bass_warmth * center.max(0.5) + kick_transient * 0.25
                } else if b < 18 {
                    let center = 1.0 - ((fb - 15.0).abs() / 4.0);
                    vocal_mid * center.max(0.45) + snare_crack * 0.45
                } else if b < 23 {
                    let center = 1.0 - ((fb - 20.5).abs() / 3.5);
                    presence_lead * center.max(0.4) + snare_crack * 0.4
                } else if b < 28 {
                    let rise = (fb - 23.0) / 5.0;
                    hihat_tick * 0.65 + air_shimmer * (0.4 + rise * 0.4)
                } else {
                    let air_taper = 1.0 - ((fb - 28.0) / 4.0) * 0.2;
                    air_shimmer * air_taper + hihat_tick * 0.4
                };
                let noise = ((p * 15.7 + fb * 2.1).sin() * 0.5 + 0.5) * 12.0;
                raw_bands[b] = (val + noise).clamp(8.0, 98.0);
            }

            raw_vu_l = (raw_bands[4] * 0.35 + raw_bands[9] * 0.25 + raw_bands[16] * 0.25 + raw_bands[25] * 0.15).clamp(10.0, 98.0);
            raw_vu_r = (raw_bands[5] * 0.32 + raw_bands[10] * 0.26 + raw_bands[17] * 0.24 + raw_bands[26] * 0.18).clamp(10.0, 98.0);
        }

        // Apply 32-band Equalizer Preset Gain Curves (dB → linear multiplier)
        for i in 0..32 {
            let mult = 1.0 + (eq_gains[i] / 20.0);
            raw_bands[i] = (raw_bands[i] * mult.clamp(0.4, 2.2)).clamp(0.0, 100.0);
        }

        // Dual-Rate EMA Ballistics
        for i in 0..32 {
            let target = raw_bands[i];
            let current = self.smoothed_bands[i];
            let alpha = if target >= current { self.attack_alpha } else { self.release_alpha };
            self.smoothed_bands[i] = alpha * target + (1.0 - alpha) * current;

            if self.smoothed_bands[i] >= self.peak_bands[i] {
                self.peak_bands[i] = self.smoothed_bands[i];
                self.peak_timers[i] = now;
            } else if now.duration_since(self.peak_timers[i]).as_secs_f32() > self.peak_hold_secs {
                self.peak_bands[i] = (self.peak_bands[i] - self.peak_decay_rate * dt).max(self.smoothed_bands[i]);
            }
        }

        // Stereo VU Meter Ballistics
        let alpha_l = if raw_vu_l >= self.smoothed_vu_l { self.attack_alpha } else { self.release_alpha };
        self.smoothed_vu_l = alpha_l * raw_vu_l + (1.0 - alpha_l) * self.smoothed_vu_l;

        let alpha_r = if raw_vu_r >= self.smoothed_vu_r { self.attack_alpha } else { self.release_alpha };
        self.smoothed_vu_r = alpha_r * raw_vu_r + (1.0 - alpha_r) * self.smoothed_vu_r;

        if self.smoothed_vu_l >= self.peak_vu_l {
            self.peak_vu_l = self.smoothed_vu_l;
            self.peak_timer_l = now;
        } else if now.duration_since(self.peak_timer_l).as_secs_f32() > self.peak_hold_secs {
            self.peak_vu_l = (self.peak_vu_l - self.peak_decay_rate * dt).max(self.smoothed_vu_l);
        }

        if self.smoothed_vu_r >= self.peak_vu_r {
            self.peak_vu_r = self.smoothed_vu_r;
            self.peak_timer_r = now;
        } else if now.duration_since(self.peak_timer_r).as_secs_f32() > self.peak_hold_secs {
            self.peak_vu_r = (self.peak_vu_r - self.peak_decay_rate * dt).max(self.smoothed_vu_r);
        }

        (
            self.smoothed_bands,
            self.peak_bands,
            self.smoothed_vu_l,
            self.smoothed_vu_r,
            self.peak_vu_l,
            self.peak_vu_r,
        )
    }
}
