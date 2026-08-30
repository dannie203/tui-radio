use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    LocalTracks,
    RadioStations,
    Queue,
    YoutubeMusic,
}

impl AppMode {
    pub const ALL: [AppMode; 4] = [
        AppMode::LocalTracks,
        AppMode::RadioStations,
        AppMode::Queue,
        AppMode::YoutubeMusic,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            AppMode::LocalTracks => "LOCAL CRATES",
            AppMode::RadioStations => "RADIO STATIONS",
            AppMode::Queue => "PLAYLIST QUEUE",
            AppMode::YoutubeMusic => "YOUTUBE & STREAMS",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            AppMode::LocalTracks => "Local Crates",
            AppMode::RadioStations => "Radio Stations",
            AppMode::Queue => "Queue",
            AppMode::YoutubeMusic => "Streams",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            AppMode::LocalTracks => 0,
            AppMode::RadioStations => 1,
            AppMode::Queue => 2,
            AppMode::YoutubeMusic => 3,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx % 4 {
            0 => AppMode::LocalTracks,
            1 => AppMode::RadioStations,
            2 => AppMode::Queue,
            _ => AppMode::YoutubeMusic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalViewLevel {
    Albums,
    Tracks,
    AllTracks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenreFilter {
    All,
    Favorites,
    LoFi,
    Synthwave,
    Jazz,
    HipHop,
    Rock,
    Electronic,
    Classical,
    Pop,
    Vietnam,
    Japan,
    GlobalTop,
}

impl GenreFilter {
    pub const ALL: [GenreFilter; 13] = [
        GenreFilter::All,
        GenreFilter::Favorites,
        GenreFilter::LoFi,
        GenreFilter::Synthwave,
        GenreFilter::Jazz,
        GenreFilter::HipHop,
        GenreFilter::Rock,
        GenreFilter::Electronic,
        GenreFilter::Classical,
        GenreFilter::Pop,
        GenreFilter::Vietnam,
        GenreFilter::Japan,
        GenreFilter::GlobalTop,
    ];

    pub fn cycle(&self, delta: i32) -> Self {
        let idx = GenreFilter::ALL.iter().position(|g| g == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(GenreFilter::ALL.len() as i32) as usize;
        GenreFilter::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            GenreFilter::All => "ALL GENRES",
            GenreFilter::Favorites => "★ FAVORITES",
            GenreFilter::LoFi => "LO-FI & BEATS",
            GenreFilter::Synthwave => "SYNTHWAVE & RETRO",
            GenreFilter::Jazz => "JAZZ & BLUES",
            GenreFilter::HipHop => "HIP-HOP & BOOM-BAP",
            GenreFilter::Rock => "ROCK & INDIE",
            GenreFilter::Electronic => "ELECTRONIC & EDM",
            GenreFilter::Classical => "CLASSICAL & AMBIENT",
            GenreFilter::Pop => "POP & TOP 40",
            GenreFilter::Vietnam => "VIETNAM RADIO",
            GenreFilter::Japan => "JAPAN & ANIME",
            GenreFilter::GlobalTop => "GLOBAL TOP 100",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualizerSpeed {
    UltraSnappy,
    Standard,
    SmoothLiquid,
}

impl VisualizerSpeed {
    pub const ALL: [VisualizerSpeed; 3] = [
        VisualizerSpeed::UltraSnappy,
        VisualizerSpeed::Standard,
        VisualizerSpeed::SmoothLiquid,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = VisualizerSpeed::ALL.iter().position(|v| v == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(VisualizerSpeed::ALL.len() as i32) as usize;
        VisualizerSpeed::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            VisualizerSpeed::UltraSnappy => "⚡ Ultra-Snappy (0.92/0.32)",
            VisualizerSpeed::Standard => "🎛️ Studio Standard (0.76/0.18)",
            VisualizerSpeed::SmoothLiquid => "🌊 Liquid Smooth (0.50/0.10)",
        }
    }

    pub fn alphas(&self) -> (f32, f32) {
        match self {
            VisualizerSpeed::UltraSnappy => (0.92, 0.32),
            VisualizerSpeed::Standard => (0.76, 0.18),
            VisualizerSpeed::SmoothLiquid => (0.50, 0.10),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpectrumColorMode {
    RgbCycle,
    ChromaRainbow,
    VerticalGradient,
    CyberpunkNeon,
    FireAndIce,
    MatrixPhosphor,
    AmberVintage,
    ThemeAccent,
}

impl SpectrumColorMode {
    pub const ALL: [SpectrumColorMode; 8] = [
        SpectrumColorMode::RgbCycle,
        SpectrumColorMode::ChromaRainbow,
        SpectrumColorMode::VerticalGradient,
        SpectrumColorMode::CyberpunkNeon,
        SpectrumColorMode::FireAndIce,
        SpectrumColorMode::MatrixPhosphor,
        SpectrumColorMode::AmberVintage,
        SpectrumColorMode::ThemeAccent,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = SpectrumColorMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(SpectrumColorMode::ALL.len() as i32) as usize;
        SpectrumColorMode::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SpectrumColorMode::RgbCycle => "🌈 Dynamic RGB Wave (Fluid Cycle)",
            SpectrumColorMode::ChromaRainbow => "🎨 Static 32-Band ISO Rainbow",
            SpectrumColorMode::VerticalGradient => "📊 Multi-Level Vertical LED",
            SpectrumColorMode::CyberpunkNeon => "🌆 Neon Cyberpunk Pulse Wave",
            SpectrumColorMode::FireAndIce => "🔥 Fire & Ice Dynamic Wave",
            SpectrumColorMode::MatrixPhosphor => "📟 Retro Phosphor Matrix Green",
            SpectrumColorMode::AmberVintage => "📻 Vintage Hi-Fi Amber Gold",
            SpectrumColorMode::ThemeAccent => "✨ Theme Primary Accent",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlbumGroup {
    pub name: String,
    pub artist: String,
    pub year: Option<String>,
    pub format: String,
    pub tracks: Vec<MediaItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Deck,
    Lyrics,
    Artwork,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    None,
    Help,
    UrlInput,
    Search,
    Settings,
    Mixtape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StereoMode {
    Stereo,
    Mono,
    Wide3D,
}

impl StereoMode {
    pub const ALL: [StereoMode; 3] = [
        StereoMode::Stereo,
        StereoMode::Mono,
        StereoMode::Wide3D,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = StereoMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(StereoMode::ALL.len() as i32) as usize;
        StereoMode::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            StereoMode::Stereo => "● STEREO",
            StereoMode::Mono => "◉ MONO",
            StereoMode::Wide3D => "✦ 3D WIDE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DolbyMode {
    Off,
    DolbyB,
    DolbyC,
    DolbyS,
}

impl DolbyMode {
    pub const ALL: [DolbyMode; 4] = [
        DolbyMode::Off,
        DolbyMode::DolbyB,
        DolbyMode::DolbyC,
        DolbyMode::DolbyS,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = DolbyMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(DolbyMode::ALL.len() as i32) as usize;
        DolbyMode::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            DolbyMode::Off => "DOLBY: OFF",
            DolbyMode::DolbyB => "DOLBY-B (High-Hiss Cut)",
            DolbyMode::DolbyC => "DOLBY-C (Wide Filter)",
            DolbyMode::DolbyS => "DOLBY-S (Studio Master)",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            DolbyMode::Off => "DOLBY OFF",
            DolbyMode::DolbyB => "DOLBY-B",
            DolbyMode::DolbyC => "DOLBY-C",
            DolbyMode::DolbyS => "DOLBY-S",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TapeType {
    TypeI,
    TypeII,
    TypeIV,
}

impl TapeType {
    pub const ALL: [TapeType; 3] = [
        TapeType::TypeI,
        TapeType::TypeII,
        TapeType::TypeIV,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = TapeType::ALL.iter().position(|m| m == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(TapeType::ALL.len() as i32) as usize;
        TapeType::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            TapeType::TypeI => "Type-I Normal Fe",
            TapeType::TypeII => "Type-II CrO2 High Bias",
            TapeType::TypeIV => "Type-IV Metal Hi-End",
        }
    }

    pub fn short_label(&self) -> &'static str {
        match self {
            TapeType::TypeI => "TYPE-I (Fe)",
            TapeType::TypeII => "TYPE-II (CrO2)",
            TapeType::TypeIV => "TYPE-IV (Metal)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

impl RepeatMode {
    pub const ALL: [RepeatMode; 3] = [
        RepeatMode::Off,
        RepeatMode::All,
        RepeatMode::One,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = RepeatMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(RepeatMode::ALL.len() as i32) as usize;
        RepeatMode::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            RepeatMode::Off => "OFF",
            RepeatMode::One => "ONE",
            RepeatMode::All => "ALL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqPreset {
    Flat,
    MegaBass,
    VocalClear,
    RockPunch,
    LofiWarmth,
    CyberSynth,
    ClubEdm,
}

impl EqPreset {
    pub const ALL: [EqPreset; 7] = [
        EqPreset::Flat,
        EqPreset::MegaBass,
        EqPreset::VocalClear,
        EqPreset::RockPunch,
        EqPreset::LofiWarmth,
        EqPreset::CyberSynth,
        EqPreset::ClubEdm,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = EqPreset::ALL.iter().position(|e| e == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(EqPreset::ALL.len() as i32) as usize;
        EqPreset::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            EqPreset::Flat => "FLAT (Reference)",
            EqPreset::MegaBass => "MEGA BASS (+7dB)",
            EqPreset::VocalClear => "VOCAL CLARITY",
            EqPreset::RockPunch => "ROCK & METAL",
            EqPreset::LofiWarmth => "LO-FI WARMTH",
            EqPreset::CyberSynth => "CYBERPUNK SYNTH",
            EqPreset::ClubEdm => "EDM CLUB DROP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordFormat {
    Opus,
    Mp3,
    Flac,
    M4a,
}

impl RecordFormat {
    pub const ALL: [RecordFormat; 4] = [
        RecordFormat::Opus,
        RecordFormat::Mp3,
        RecordFormat::Flac,
        RecordFormat::M4a,
    ];

    pub fn cycle(&self) -> Self {
        self.cycle_dir(1)
    }

    pub fn cycle_dir(&self, delta: i32) -> Self {
        let idx = RecordFormat::ALL.iter().position(|r| r == self).unwrap_or(0);
        let next = (idx as i32 + delta).rem_euclid(RecordFormat::ALL.len() as i32) as usize;
        RecordFormat::ALL[next]
    }

    pub fn label(&self) -> &'static str {
        match self {
            RecordFormat::Opus => "OPUS (Zero-Loss Native)",
            RecordFormat::Mp3 => "MP3 (320k CBR)",
            RecordFormat::Flac => "FLAC (Lossless Master)",
            RecordFormat::M4a => "AAC / M4A (256k)",
        }
    }

    pub fn ext(&self) -> &'static str {
        match self {
            RecordFormat::Opus => "opus",
            RecordFormat::Mp3 => "mp3",
            RecordFormat::Flac => "flac",
            RecordFormat::M4a => "m4a",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub url: String,
    pub duration: f64,
    pub format: Option<String>,
    pub bitrate: Option<u32>,
    pub is_radio: bool,
    pub is_youtube: bool,
    pub is_favorite: bool,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub track_no: Option<u32>,
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub bit_depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedLyricLine {
    pub time: f64,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct AudioTelemetry {
    pub vu_left: f32,
    pub vu_right: f32,
    pub peak_left: f32,
    pub peak_right: f32,
    pub eq_bands: [f32; 32],
    pub eq_peaks: [f32; 32],
    pub spool_frame: usize,
    pub time_pos: f64,
    pub duration: f64,
    pub percent_pos: f64,
    pub tape_counter: String,
    pub audio_codec: String,
    pub audio_bit_depth: u32,
    pub audio_bitrate: u32,
    pub audio_sample_rate: u32,
    pub audio_channels: String,
    pub is_live: bool,
}

impl Default for AudioTelemetry {
    fn default() -> Self {
        Self {
            vu_left: 0.0,
            vu_right: 0.0,
            peak_left: 0.0,
            peak_right: 0.0,
            eq_bands: [0.0; 32],
            eq_peaks: [0.0; 32],
            spool_frame: 0,
            time_pos: 0.0,
            duration: 0.0,
            percent_pos: 0.0,
            tape_counter: "00:00".to_string(),
            audio_codec: "STANDBY".to_string(),
            audio_bit_depth: 16,
            audio_bitrate: 0,
            audio_sample_rate: 48000,
            audio_channels: "Stereo".to_string(),
            is_live: false,
        }
    }
}


