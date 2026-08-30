use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackNeuralProfile {
    pub bpm: f64,
    pub camelot_key: String,
    pub energy: f32,
    pub mix_in_sec: f64,
    pub mix_out_sec: f64,
    pub duration_sec: f64,
}

impl Default for TrackNeuralProfile {
    fn default() -> Self {
        Self {
            bpm: 120.0,
            camelot_key: "8A".to_string(),
            energy: 0.5,
            mix_in_sec: 8.0,
            mix_out_sec: 0.0,
            duration_sec: 0.0,
        }
    }
}

impl TrackNeuralProfile {
    /// Duration of a single beat in seconds, derived from the track BPM.
    pub fn beat_sec(&self) -> f64 {
        if self.bpm > 30.0 {
            60.0 / self.bpm
        } else {
            0.5
        }
    }

    /// Snap a time offset to the nearest beat boundary (never negative).
    /// Falling back to `fallback` when BPM is unusable.
    pub fn snap_to_beat(&self, t: f64, fallback: f64) -> f64 {
        let beat = self.beat_sec();
        if !(t.is_finite() && t > 0.0) {
            return fallback;
        }
        let beats = (t / beat).round().max(0.0);
        (beats * beat).max(0.0)
    }
}

pub struct NeuralEngine {
    cache_path: PathBuf,
    profiles: Arc<Mutex<HashMap<String, TrackNeuralProfile>>>,
}

impl NeuralEngine {
    pub fn new() -> Self {
        let cache_dir = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("boombox");
        let _ = fs::create_dir_all(&cache_dir);
        let cache_path = cache_dir.join("neural_profiles.json");

        let profiles = if cache_path.exists() {
            fs::read_to_string(&cache_path)
                .ok()
                .and_then(|data| serde_json::from_str::<HashMap<String, TrackNeuralProfile>>(&data).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            cache_path,
            profiles: Arc::new(Mutex::new(profiles)),
        }
    }

    pub fn get_profile(&self, track_path: &str) -> Option<TrackNeuralProfile> {
        let guard = self.profiles.lock().unwrap();
        guard.get(track_path).cloned()
    }

    /// Re-reads the on-disk cache. Call after an external/library neural scan
    /// has updated `neural_profiles.json` so the in-memory map picks it up.
    pub fn reload(&self) {
        let profiles = if self.cache_path.exists() {
            fs::read_to_string(&self.cache_path)
                .ok()
                .and_then(|data| serde_json::from_str::<HashMap<String, TrackNeuralProfile>>(&data).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        *self.profiles.lock().unwrap() = profiles;
    }

    /// Number of cached neural profiles currently loaded.
    pub fn profile_count(&self) -> usize {
        self.profiles.lock().unwrap().len()
    }

    pub fn insert_profile(&self, track_path: String, profile: TrackNeuralProfile) {
        let mut guard = self.profiles.lock().unwrap();
        guard.insert(track_path, profile);
        self.save_cache_locked(&guard);
    }

    fn save_cache_locked(&self, map: &HashMap<String, TrackNeuralProfile>) {
        if let Ok(json) = serde_json::to_string_pretty(map) {
            let _ = fs::write(&self.cache_path, json);
        }
    }

    /// Check if two Camelot keys are harmonically compatible (Same key, Relative major/minor, or +-1 step)
    pub fn is_harmonic_match(key_a: &str, key_b: &str) -> bool {
        if key_a == key_b {
            return true;
        }
        let parse_camelot = |k: &str| -> Option<(u32, char)> {
            if k.len() < 2 {
                return None;
            }
            let num = k[..k.len() - 1].parse::<u32>().ok()?;
            let letter = k.chars().last()?.to_ascii_uppercase();
            if (1..=12).contains(&num) && (letter == 'A' || letter == 'B') {
                Some((num, letter))
            } else {
                None
            }
        };

        let (num_a, letter_a) = match parse_camelot(key_a) {
            Some(v) => v,
            None => return false,
        };
        let (num_b, letter_b) = match parse_camelot(key_b) {
            Some(v) => v,
            None => return false,
        };

        // Same number, different letter (Relative Major <-> Minor, e.g. 8A <-> 8B)
        if num_a == num_b {
            return true;
        }

        // Same letter, adjacent numbers (+-1 step on the wheel, e.g. 8A <-> 7A, 9A)
        if letter_a == letter_b {
            let diff = (num_a as i32 - num_b as i32).abs();
            if diff == 1 || diff == 11 {
                return true;
            }
        }

        false
    }

    /// Generate an Apple Music-style Intelligent DJ Transition Plan between two tracks
    pub fn plan_transition(
        &self,
        track_a: &TrackNeuralProfile,
        track_b: &TrackNeuralProfile,
        base_fade_dur: f64,
        mode: crate::state::types::AutomixMode,
    ) -> TransitionPlan {
        use crate::state::types::AutomixMode;

        let bpm_diff = (track_a.bpm - track_b.bpm).abs();
        let key_match = Self::is_harmonic_match(&track_a.camelot_key, &track_b.camelot_key);

        // 1. Determine Transition Archetype based on selected mode & neural analysis
        let strategy = match mode {
            AutomixMode::NeuralBassSwap => DjTransitionStrategy::BassSwap,
            AutomixMode::NeuralEchoOut => DjTransitionStrategy::EchoOutDrop,
            AutomixMode::NeuralFilterSweep => DjTransitionStrategy::FilterSweep,
            AutomixMode::EqualPower | AutomixMode::SmoothExponential | AutomixMode::LinearRamp => {
                DjTransitionStrategy::BassSwap
            }
            AutomixMode::Disabled => DjTransitionStrategy::BreakdownCut,
            AutomixMode::NeuralAuto => {
                if bpm_diff > 8.0 {
                    // Large tempo gap: use Echo-Out Reverb Drop to avoid chaotic trainwrecking
                    DjTransitionStrategy::EchoOutDrop
                } else if bpm_diff <= 8.0 && key_match && track_a.energy > 0.4 && track_b.energy > 0.4 {
                    // Harmonic match with steady beats: use Pro Bass-Swap
                    DjTransitionStrategy::BassSwap
                } else if track_a.energy <= 0.4 || track_b.energy <= 0.4 {
                    // Ambient / Acoustic / Low energy: smooth spectral filter sweep
                    DjTransitionStrategy::FilterSweep
                } else {
                    DjTransitionStrategy::BassSwap
                }
            }
        };

        // 2. Calculate Exact Musical Overlap (in Bars & Seconds)
        let bar_sec_a = track_a.beat_sec() * 4.0;
        let overlap_bars: u32 = match strategy {
            DjTransitionStrategy::EchoOutDrop => 2, // Quick 2-bar echo release
            DjTransitionStrategy::BassSwap => {
                if base_fade_dur >= 10.0 {
                    8
                } else if base_fade_dur >= 5.0 {
                    4
                } else {
                    2
                }
            }
            DjTransitionStrategy::FilterSweep => 4,
            DjTransitionStrategy::BreakdownCut => 1,
        };

        let duration_sec = match mode {
            AutomixMode::EqualPower | AutomixMode::SmoothExponential | AutomixMode::LinearRamp => {
                base_fade_dur.clamp(1.0, 30.0)
            }
            _ => (overlap_bars as f64 * bar_sec_a).clamp(2.0, 16.0),
        };

        let bass_swap_sec = duration_sec * 0.5; // Swap bass at exact midpoint downbeat

        // 3. Tempo Stretch Ratio (only if tempo difference is within +-6%)
        let tempo_sync_ratio = if bpm_diff <= 8.0 && track_b.bpm > 30.0 {
            track_a.bpm / track_b.bpm
        } else {
            1.0
        };

        // 4. Snap mix points to musical beats
        let mix_in_sec = track_b.snap_to_beat(track_b.mix_in_sec, 0.0);
        let mix_out_sec = if track_a.duration_sec > duration_sec {
            track_a.duration_sec - duration_sec
        } else {
            (track_a.duration_sec - 4.0).max(0.0)
        };

        TransitionPlan {
            strategy,
            duration_sec,
            overlap_bars,
            tempo_sync_ratio,
            bass_swap_sec,
            mix_in_sec,
            mix_out_sec,
            key_match,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DjTransitionStrategy {
    /// Equal-power blend with high-pass sweep on Deck A and instant Bass swap at the downbeat
    BassSwap,
    /// Gentle multi-band filter sweep (Treble in -> Mid in -> Bass in)
    FilterSweep,
    /// 1/2 Beat Echo & Reverb tail on Deck A + punchy Downbeat drop on Deck B
    EchoOutDrop,
    /// Direct phrase cut on downbeat
    BreakdownCut,
}

impl DjTransitionStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            DjTransitionStrategy::BassSwap => "AI BASS-SWAP (Pro DJ)",
            DjTransitionStrategy::FilterSweep => "AI FILTER SWEEP (Smooth)",
            DjTransitionStrategy::EchoOutDrop => "AI ECHO-OUT DROP (BPM Sync)",
            DjTransitionStrategy::BreakdownCut => "AI DOWNBEAT CUT",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransitionPlan {
    pub strategy: DjTransitionStrategy,
    pub duration_sec: f64,
    pub overlap_bars: u32,
    pub tempo_sync_ratio: f64,
    pub bass_swap_sec: f64,
    pub mix_in_sec: f64,
    pub mix_out_sec: f64,
    pub key_match: bool,
}

