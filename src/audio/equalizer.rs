use crate::state::types::EqPreset;

pub const NUM_BANDS: usize = 32;

#[derive(Debug, Clone)]
pub struct EqPresetData {
    pub id: EqPreset,
    pub name: &'static str,
    pub description: &'static str,
    /// 32 ISO-band gains in dB (20Hz → 20kHz)
    pub gains: [f32; 32],
}

pub fn all_presets() -> [EqPresetData; 7] {
    [
        EqPresetData {
            id: EqPreset::Flat,
            name: "🎚️ Flat Reference (0 dB)",
            description: "Clean uncolored studio monitor frequency response",
            gains: [0.0; 32],
        },
        EqPresetData {
            id: EqPreset::MegaBass,
            name: "🔊 Mega Bass Club (+7dB Lows)",
            description: "Deep analog sub-bass and kick thump punch",
            gains: [
                7.0, 7.0, 6.5, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.5, 1.0, 1.5, 2.0, 2.5, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0,
            ],
        },
        EqPresetData {
            id: EqPreset::VocalClear,
            name: "🎤 Vocal Clarity (+4dB Mid)",
            description: "Enhanced dialogue and acoustic vocal prominence",
            gains: [
                -2.0, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0,
                2.5, 3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5,
                1.0, 0.5, 0.0, -0.5, -1.0, -1.0, -1.5, -2.0, -2.0, -2.5, -3.0, -3.0,
            ],
        },
        EqPresetData {
            id: EqPreset::RockPunch,
            name: "🎸 Rock & Metal Punch (V-Curve)",
            description: "Aggressive low-end driving guitars and crisp hi-hats",
            gains: [
                5.5, 5.0, 4.5, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -1.5,
                -2.0, -2.0, -1.5, -1.0, 0.0, 1.0, 1.5, 2.0, 2.5, 3.0,
                3.5, 4.0, 4.5, 5.0, 5.0, 4.5, 4.0, 3.5, 3.0, 2.0, 1.0, 0.0,
            ],
        },
        EqPresetData {
            id: EqPreset::LofiWarmth,
            name: "☕ Lo-Fi Tape Warmth (Rolled Highs)",
            description: "Vintage cassette rolled-off top end with cozy low mids",
            gains: [
                3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0,
                0.5, 0.0, 0.0, 0.0, 0.0, -0.5, -1.0, -1.5, -2.0, -3.0,
                -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, -12.0, -13.0, -14.0, -15.0,
            ],
        },
        EqPresetData {
            id: EqPreset::CyberSynth,
            name: "🌆 Cyberpunk Synthwave (Crisp Top/Bottom)",
            description: "Punchy 80s analog basslines with shimmering retro leads",
            gains: [
                6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 1.5, 0.5, 0.0, 0.0,
                -0.5, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5,
                4.0, 4.5, 5.0, 5.5, 6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 2.0, 1.0,
            ],
        },
        EqPresetData {
            id: EqPreset::ClubEdm,
            name: "🎛️ Club EDM & Techno (Sub-Drop)",
            description: "Sub-bass emphasis with open high-frequency sizzle",
            gains: [
                8.0, 7.5, 7.0, 6.0, 5.0, 3.5, 2.0, 0.5, 0.0, -1.0,
                -1.5, -1.5, -1.0, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0,
                3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.0, 5.5, 4.5, 3.5, 2.0, 1.0,
            ],
        },
    ]
}

pub const ISO_FREQUENCIES: [f32; 32] = [
    20.0, 25.0, 31.5, 40.0, 50.0, 63.0, 80.0, 100.0,
    125.0, 160.0, 200.0, 250.0, 315.0, 400.0, 500.0, 630.0,
    800.0, 1000.0, 1250.0, 1600.0, 2000.0, 2500.0, 3150.0, 4000.0,
    5000.0, 6300.0, 8000.0, 10000.0, 12500.0, 16000.0, 18000.0, 20000.0,
];

pub fn get_preset(id: EqPreset) -> EqPresetData {
    all_presets()
        .into_iter()
        .find(|p| p.id == id)
        .unwrap_or(all_presets()[0].clone())
}

pub fn preset_gains(id: EqPreset) -> [f32; 32] {
    get_preset(id).gains
}

/// Builds an FFmpeg `firequalizer` audio filter string for MPV real-time DSP.
pub fn build_mpv_af_string(
    preset: EqPreset,
    bass_boost: bool,
    dolby: crate::state::types::DolbyMode,
) -> String {
    let base_gains = preset_gains(preset);
    let mut combined = base_gains;

    if bass_boost {
        // Add punchy Mega Bass analog curve (+7.5dB sub, tapering to 0 at 250Hz)
        let bass_tapers = [7.5, 7.5, 7.0, 6.5, 5.5, 4.5, 3.0, 2.0, 1.0, 0.5, 0.0];
        for (i, &b) in bass_tapers.iter().enumerate() {
            if i < combined.len() {
                combined[i] += b;
            }
        }
    }

    match dolby {
        crate::state::types::DolbyMode::DolbyB => {
            // High-hiss cut above 5kHz
            for i in 24..32 {
                combined[i] -= 3.0;
            }
        }
        crate::state::types::DolbyMode::DolbyC => {
            // Wide filter above 2.5kHz
            for i in 21..32 {
                combined[i] -= 5.0;
            }
        }
        crate::state::types::DolbyMode::DolbyS => {
            // Studio master tape warm curve
            for i in 0..8 {
                combined[i] += 1.0;
            }
            for i in 24..32 {
                combined[i] -= 2.0;
            }
        }
        crate::state::types::DolbyMode::Off => {}
    }

    // Check if flat 0dB everywhere
    let is_flat = combined.iter().all(|&g| g.abs() < 0.05);
    if is_flat {
        return String::new();
    }

    // Build firequalizer gain_entry string
    let mut entries = String::new();
    for (i, &freq) in ISO_FREQUENCIES.iter().enumerate() {
        let gain = combined[i].clamp(-24.0, 18.0);
        if !entries.is_empty() {
            entries.push(';');
        }
        entries.push_str(&format!("entry({:.1},{:.1})", freq, gain));
    }

    format!("lavfi=[firequalizer=gain_entry='{}']", entries)
}

/// Converts a 32-band dB gain curve into a linear multiplier for the spectrum.
/// 0dB → 1.0, +12dB → ~4.0, -12dB → ~0.25 (clamped to sensible range).
pub fn gain_to_multiplier(db: f32) -> f32 {
    (db / 20.0).exp2().clamp(0.2, 4.0)
}
