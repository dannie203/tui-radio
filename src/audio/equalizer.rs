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

pub fn get_preset(id: EqPreset) -> EqPresetData {
    all_presets()
        .into_iter()
        .find(|p| p.id == id)
        .unwrap_or(all_presets()[0].clone())
}

pub fn preset_gains(id: EqPreset) -> [f32; 32] {
    get_preset(id).gains
}

/// Converts a 32-band dB gain curve into a linear multiplier for the spectrum.
/// 0dB → 1.0, +12dB → ~4.0, -12dB → ~0.25 (clamped to sensible range).
pub fn gain_to_multiplier(db: f32) -> f32 {
    (db / 20.0).exp2().clamp(0.2, 4.0)
}
