use crate::state::types::{
    AppMode, DolbyMode, EqPreset, RecordFormat, StereoMode, TapeType, VisualizerSpeed,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub music_dir: Option<String>,
    pub default_mode: Option<String>,
    pub volume_step: u32,
    pub notifications: bool,
    pub auto_save_session: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            music_dir: Some("~/Music".to_string()),
            default_mode: Some("local".to_string()),
            volume_step: 5,
            notifications: true,
            auto_save_session: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub default_volume: u32,
    pub stereo_mode: String,
    pub dolby_mode: String,
    pub tape_type: String,
    pub eq_preset: String,
    pub bass_boost: bool,
    pub record_format: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            default_volume: 80,
            stereo_mode: "stereo".to_string(),
            dolby_mode: "dolby_b".to_string(),
            tape_type: "type_ii".to_string(),
            eq_preset: "flat".to_string(),
            bass_boost: false,
            record_format: "opus".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub visualizer_speed: String,
    #[serde(default = "default_spectrum_color_mode")]
    pub spectrum_color_mode: String,
    pub spectrum_custom_color: Option<String>,
    pub matrix_scramble: bool,
    pub lyrics_offset: f64,
}

fn default_spectrum_color_mode() -> String {
    "rgb_cycle".to_string()
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "omarchy".to_string(),
            visualizer_speed: "snappy".to_string(),
            spectrum_color_mode: "rgb_cycle".to_string(),
            spectrum_custom_color: None,
            matrix_scramble: true,
            lyrics_offset: 0.0,
        }
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into())).join(".config"))
        .join("boombox")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

const DEFAULT_CONFIG_TEMPLATE: &str = r##"# ==============================================================================
#  📼 BOOMBOX RX-505 — High-Fidelity Retro Cyberpunk Audio Player
#  User Configuration File: ~/.config/boombox/config.toml
# ==============================================================================

[general]
# Music directory to scan for local tracks, FLAC albums, and audio files
# Leave unset or set to "~/Music"
music_dir = "~/Music"

# Default starting mode when opening Boombox: "local", "radio", "queue", "youtube"
default_mode = "local"

# Volume step percentage for volume up / down (keys: +, -, [, ])
volume_step = 5

# Desktop notifications for track and station changes
notifications = true

# Automatically save and restore session (last track, volume, playlist)
auto_save_session = true


[audio]
# Initial playback volume (0 - 100)
default_volume = 80

# Stereo Soundstage Engine: "stereo", "mono", "wide3d"
stereo_mode = "stereo"

# Retro Cassette Dolby Noise Reduction: "off", "dolby_b", "dolby_c", "dolby_s"
dolby_mode = "dolby_b"

# Magnetic Cassette Tape Bias Formula: "type_i" (Ferric), "type_ii" (Chrome), "type_iv" (Metal)
tape_type = "type_ii"

# 32-Band ISO Equalizer Preset:
# Options: "flat", "megabass", "vocal", "rock", "lofi", "cyberpunk", "club"
eq_preset = "flat"

# Dynamic Hardware Bass-Boost emulation
bass_boost = false

# Recording audio format for cassette mixtape rips: "opus", "flac", "mp3", "m4a"
record_format = "opus"


[ui]
# Visual color theme: "omarchy", "cyberpunk", "retro_amber", "neon_synth", "matrix"
theme = "omarchy"

# Spectrum Visualizer FPS / Responsiveness: "snappy", "standard", "liquid"
visualizer_speed = "snappy"

# 32-Band ISO Chroma Equalizer Spectrum Color Mode:
# Options: "rgb_cycle" (Dynamic 60FPS Fluid RGB Wave), "static_iso" (Fixed Rainbow), "gradient" (Multi-level LED), "cyberpunk" (Neon Wave), "fire_ice" (Flame Wave), "matrix" (Phosphor Green), "amber" (Vintage Gold), "theme" (Accent)
spectrum_color_mode = "rgb_cycle"

# Optional Custom Hex color for spectrum (e.g. "#00ffcc" or "#ff007f")
# spectrum_custom_color = "#00ffcc"

# Enable cyberpunk matrix rain title scrambling effect on standby
matrix_scramble = true

# Synced lyrics display time offset in seconds (e.g. 0.0, -0.5, 0.5)
lyrics_offset = 0.0
"##;

impl AppConfig {
    pub fn load_or_create() -> Self {
        let dir = get_config_dir();
        let _ = fs::create_dir_all(&dir);
        let path = get_config_path();

        if !path.exists() {
            let _ = fs::write(&path, DEFAULT_CONFIG_TEMPLATE);
            return Self::default();
        }

        if let Ok(content) = fs::read_to_string(&path) {
            match toml::from_str::<AppConfig>(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("⚠️ Warning: Error parsing config.toml: {}. Using default values.", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    pub fn resolved_music_dir(&self) -> PathBuf {
        if let Some(ref dir_str) = self.general.music_dir {
            if dir_str.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    return home.join(&dir_str[2..]);
                }
            }
            return PathBuf::from(dir_str);
        }
        dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn get_app_mode(&self) -> AppMode {
        match self.general.default_mode.as_deref().unwrap_or("local").to_lowercase().as_str() {
            "radio" | "stations" => AppMode::RadioStations,
            "queue" | "playlist" => AppMode::Queue,
            "youtube" | "streams" => AppMode::YoutubeMusic,
            _ => AppMode::LocalTracks,
        }
    }

    pub fn get_stereo_mode(&self) -> StereoMode {
        match self.audio.stereo_mode.to_lowercase().as_str() {
            "mono" => StereoMode::Mono,
            "wide" | "wide3d" | "3d" => StereoMode::Wide3D,
            _ => StereoMode::Stereo,
        }
    }

    pub fn get_dolby_mode(&self) -> DolbyMode {
        match self.audio.dolby_mode.to_lowercase().as_str() {
            "off" | "none" => DolbyMode::Off,
            "dolby_c" | "c" => DolbyMode::DolbyC,
            "dolby_s" | "s" => DolbyMode::DolbyS,
            _ => DolbyMode::DolbyB,
        }
    }

    pub fn get_tape_type(&self) -> TapeType {
        match self.audio.tape_type.to_lowercase().as_str() {
            "type_i" | "type1" | "ferric" | "normal" => TapeType::TypeI,
            "type_iv" | "type4" | "metal" => TapeType::TypeIV,
            _ => TapeType::TypeII,
        }
    }

    pub fn get_eq_preset(&self) -> EqPreset {
        match self.audio.eq_preset.to_lowercase().as_str() {
            "bass" | "megabass" | "bass_boost" => EqPreset::MegaBass,
            "vocal" | "vocalclear" => EqPreset::VocalClear,
            "rock" | "rockpunch" => EqPreset::RockPunch,
            "lofi" | "lofiwarmth" => EqPreset::LofiWarmth,
            "cyberpunk" | "cybersynth" => EqPreset::CyberSynth,
            "club" | "clubedm" => EqPreset::ClubEdm,
            _ => EqPreset::Flat,
        }
    }

    pub fn get_record_format(&self) -> RecordFormat {
        match self.audio.record_format.to_lowercase().as_str() {
            "flac" => RecordFormat::Flac,
            "mp3" => RecordFormat::Mp3,
            "m4a" => RecordFormat::M4a,
            _ => RecordFormat::Opus,
        }
    }

    pub fn get_visualizer_speed(&self) -> VisualizerSpeed {
        match self.ui.visualizer_speed.to_lowercase().as_str() {
            "standard" | "normal" => VisualizerSpeed::Standard,
            "liquid" | "smooth" => VisualizerSpeed::SmoothLiquid,
            _ => VisualizerSpeed::UltraSnappy,
        }
    }

    pub fn get_spectrum_color_mode(&self) -> crate::state::types::SpectrumColorMode {
        use crate::state::types::SpectrumColorMode;
        match self.ui.spectrum_color_mode.to_lowercase().as_str() {
            "rgb" | "rgb_cycle" | "cycle" | "wave" | "chroma_wave" | "dynamic" => SpectrumColorMode::RgbCycle,
            "static_iso" | "static_rainbow" | "static" => SpectrumColorMode::ChromaRainbow,
            "vertical" | "led" | "gradient" => SpectrumColorMode::VerticalGradient,
            "cyberpunk" | "neon" => SpectrumColorMode::CyberpunkNeon,
            "matrix" | "phosphor" => SpectrumColorMode::MatrixPhosphor,
            "amber" | "gold" | "vintage" => SpectrumColorMode::AmberVintage,
            "fire_ice" | "fire" | "ice" => SpectrumColorMode::FireAndIce,
            "theme" | "accent" | "custom" => SpectrumColorMode::ThemeAccent,
            _ => SpectrumColorMode::RgbCycle,
        }
    }
}
