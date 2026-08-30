use crate::state::types::{DolbyMode, EqPreset, StereoMode, TapeType};

#[derive(Debug, Clone)]
pub struct EqPresetData {
    pub id: EqPreset,
    /// 32 ISO-band gains in dB (20Hz → 20kHz)
    pub gains: [f32; 32],
}

pub fn all_presets() -> [EqPresetData; 7] {
    [
        EqPresetData {
            id: EqPreset::Flat,
            gains: [0.0; 32],
        },
        EqPresetData {
            id: EqPreset::MegaBass,
            gains: [
                7.0, 7.0, 6.5, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.5, 1.0, 1.5, 2.0, 2.5, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0,
            ],
        },
        EqPresetData {
            id: EqPreset::VocalClear,
            gains: [
                -2.0, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0,
                2.5, 3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5,
                1.0, 0.5, 0.0, -0.5, -1.0, -1.0, -1.5, -2.0, -2.0, -2.5, -3.0, -3.0,
            ],
        },
        EqPresetData {
            id: EqPreset::RockPunch,
            gains: [
                5.5, 5.0, 4.5, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -1.5,
                -2.0, -2.0, -1.5, -1.0, 0.0, 1.0, 1.5, 2.0, 2.5, 3.0,
                3.5, 4.0, 4.5, 5.0, 5.0, 4.5, 4.0, 3.5, 3.0, 2.0, 1.0, 0.0,
            ],
        },
        EqPresetData {
            id: EqPreset::LofiWarmth,
            gains: [
                3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0,
                0.5, 0.0, 0.0, 0.0, 0.0, -0.5, -1.0, -1.5, -2.0, -3.0,
                -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, -12.0, -13.0, -14.0, -15.0,
            ],
        },
        EqPresetData {
            id: EqPreset::CyberSynth,
            gains: [
                6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 1.5, 0.5, 0.0, 0.0,
                -0.5, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5,
                4.0, 4.5, 5.0, 5.5, 6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 2.0, 1.0,
            ],
        },
        EqPresetData {
            id: EqPreset::ClubEdm,
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

/// Computes the combined 32-band EQ gain profile including Preset, Mega Bass, Dolby NR, and Tape Bias.
pub fn compute_total_gains(
    preset: EqPreset,
    bass_boost: bool,
    dolby: DolbyMode,
    tape: TapeType,
) -> [f32; 32] {
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
        DolbyMode::DolbyB => {
            // High-hiss cut above 5kHz
            for i in 24..32 {
                combined[i] -= 3.0;
            }
        }
        DolbyMode::DolbyC => {
            // Wide filter above 2.5kHz
            for i in 21..32 {
                combined[i] -= 5.0;
            }
        }
        DolbyMode::DolbyS => {
            // Studio master tape warm curve
            for i in 0..8 {
                combined[i] += 1.0;
            }
            for i in 24..32 {
                combined[i] -= 2.0;
            }
        }
        DolbyMode::Off => {}
    }

    match tape {
        TapeType::TypeI => {
            // Normal Fe: warm low-mids, gentle tape high roll-off
            for i in 6..14 {
                combined[i] += 1.5;
            }
            for i in 28..32 {
                combined[i] -= 1.5;
            }
        }
        TapeType::TypeII => {
            // Chrome CrO2: 70µs high-bias crispness
            for i in 25..31 {
                combined[i] += 2.0;
            }
        }
        TapeType::TypeIV => {
            // Metal: extended hi-res spectrum punch
            for i in 0..6 {
                combined[i] += 1.0;
            }
            for i in 27..32 {
                combined[i] += 1.2;
            }
        }
    }

    combined
}

/// Builds an FFmpeg / MPV audio filter string for real-time DSP (Equalizer, Stereo Soundstage, Tape, Dolby).
pub fn build_mpv_af_string(
    preset: EqPreset,
    bass_boost: bool,
    dolby: DolbyMode,
    stereo: StereoMode,
    tape: TapeType,
) -> String {
    let combined = compute_total_gains(preset, bass_boost, dolby, tape);
    let mut lavfi_filters = Vec::new();

    // 1. Equalizer Filter (firequalizer)
    let is_flat = combined.iter().all(|&g| g.abs() < 0.05);
    if !is_flat {
        let mut entries = String::new();
        for (i, &freq) in ISO_FREQUENCIES.iter().enumerate() {
            let gain = combined[i].clamp(-24.0, 18.0);
            if !entries.is_empty() {
                entries.push(';');
            }
            entries.push_str(&format!("entry({:.1},{:.1})", freq, gain));
        }
        lavfi_filters.push(format!("firequalizer=gain_entry='{}'", entries));
    }

    // 2. Soundstage / Spatial Stereo Filter
    match stereo {
        StereoMode::Mono => {
            lavfi_filters.push("pan=mono|c0=0.5*c0+0.5*c1".to_string());
        }
        StereoMode::Wide3D => {
            lavfi_filters.push("extrastereo=m=2.2".to_string());
        }
        StereoMode::Stereo => {}
    }

    if lavfi_filters.is_empty() {
        String::new()
    } else {
        format!("lavfi=[{}]", lavfi_filters.join(","))
    }
}
