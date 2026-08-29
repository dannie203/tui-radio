/**
 * 32-Band ISO Equalizer Presets & Curve Generator
 */

export const EQ_PRESETS = {
  FLAT: {
    id: 'FLAT',
    name: '🎚️ Flat Reference (0 dB)',
    description: 'Clean uncolored studio monitor frequency response',
    gains: Array(32).fill(0)
  },
  MEGA_BASS: {
    id: 'MEGA_BASS',
    name: '🔊 Mega Bass Club (+7dB Lows)',
    description: 'Deep analog sub-bass and kick thump punch',
    // 32 bands: boost 20Hz - 160Hz
    gains: [
      7.0, 7.0, 6.5, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
      0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
      0.5, 1.0, 1.5, 2.0, 2.5, 2.5, 2.0, 1.5, 1.0, 0.5, 0.0, 0.0
    ]
  },
  VOCAL_CLEAR: {
    id: 'VOCAL_CLEAR',
    name: '🎤 Vocal Clarity (+4dB Mid)',
    description: 'Enhanced dialogue and acoustic vocal prominence',
    gains: [
      -2.0, -2.0, -1.5, -1.0, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0,
      2.5, 3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5,
      1.0, 0.5, 0.0, -0.5, -1.0, -1.0, -1.5, -2.0, -2.0, -2.5, -3.0, -3.0
    ]
  },
  ROCK_PUNCH: {
    id: 'ROCK_PUNCH',
    name: '🎸 Rock & Metal Punch (V-Curve)',
    description: 'Aggressive low-end driving guitars and crisp hi-hats',
    gains: [
      5.5, 5.0, 4.5, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -1.5,
      -2.0, -2.0, -1.5, -1.0, 0.0, 1.0, 1.5, 2.0, 2.5, 3.0,
      3.5, 4.0, 4.5, 5.0, 5.0, 4.5, 4.0, 3.5, 3.0, 2.0, 1.0, 0.0
    ]
  },
  LOFI_WARMTH: {
    id: 'LOFI_WARMTH',
    name: '☕ Lo-Fi Tape Warmth (Rolled Highs)',
    description: 'Vintage cassette rolled-off top end with cozy low mids',
    gains: [
      3.0, 3.5, 4.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0,
      0.5, 0.0, 0.0, 0.0, 0.0, -0.5, -1.0, -1.5, -2.0, -3.0,
      -4.0, -5.0, -6.0, -7.0, -8.0, -9.0, -10.0, -11.0, -12.0, -13.0, -14.0, -15.0
    ]
  },
  CYBER_SYNTH: {
    id: 'CYBER_SYNTH',
    name: '🌆 Cyberpunk Synthwave (Crisp Top/Bottom)',
    description: 'Punchy 80s analog basslines with shimmering retro leads',
    gains: [
      6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 1.5, 0.5, 0.0, 0.0,
      -0.5, -0.5, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5,
      4.0, 4.5, 5.0, 5.5, 6.0, 6.0, 5.5, 5.0, 4.0, 3.0, 2.0, 1.0
    ]
  },
  CLUB_EDM: {
    id: 'CLUB_EDM',
    name: '🎛️ Club EDM & Techno (Sub-Drop)',
    description: 'Sub-bass emphasis with open high-frequency sizzle',
    gains: [
      8.0, 7.5, 7.0, 6.0, 5.0, 3.5, 2.0, 0.5, 0.0, -1.0,
      -1.5, -1.5, -1.0, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0,
      3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.0, 5.5, 4.5, 3.5, 2.0, 1.0
    ]
  }
};

export const EQ_PRESET_KEYS = Object.keys(EQ_PRESETS);

export function getEqPreset(presetId) {
  return EQ_PRESETS[presetId] || EQ_PRESETS.FLAT;
}

export function cycleEqPreset(currentId, delta = 1) {
  const idx = EQ_PRESET_KEYS.indexOf(currentId);
  const nextIdx = (idx + delta + EQ_PRESET_KEYS.length) % EQ_PRESET_KEYS.length;
  return EQ_PRESET_KEYS[nextIdx];
}
