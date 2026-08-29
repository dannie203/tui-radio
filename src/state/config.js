import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

export const CONFIG_DIR = join(homedir(), '.config', 'hiphop-tui');
export const CONFIG_FILE = join(CONFIG_DIR, 'config.json');

export const COLOR_THEMES = [
  { id: 'RGB_CHROMA', name: '🌈 RGB Chroma Wave (Keyboard Cycle)' },
  { id: 'AMBER_GOLD', name: '📻 Vintage Amber Gold (Nakamichi Hi-Fi)' },
  { id: 'GREEN_PHOSPHOR', name: '📟 Cyber Phosphor Green (CRT Bay)' },
  { id: 'CYAN_NEON', name: '🌆 Neon Synthwave (Cyberpunk Cyan/Pink)' },
  { id: 'MONOCHROME', name: '🧊 Monochrome Ice (Silver Studio)' }
];

export const PEAK_HOLD_OPTIONS = [
  { id: 1100, name: 'Long Hold (1.1s - Studio)' },
  { id: 2000, name: 'Ultra Hold (2.0s - Hi-Fi)' },
  { id: 800, name: 'Medium (800ms)' },
  { id: 300, name: 'Snappy (300ms - Fast)' }
];

export const PEAK_DECAY_OPTIONS = [
  { id: 38, name: 'Smooth Float (38 u/s)' },
  { id: 20, name: 'Slow Drift (20 u/s)' },
  { id: 85, name: 'Instant Fall (85 u/s)' }
];

export const BAND_WIDTH_OPTIONS = [
  { id: 'auto', name: 'Auto-Fit (Responsive 3/2/1)' },
  { id: 3, name: 'Wide (3 Chars)' },
  { id: 2, name: 'Standard (2 Chars)' },
  { id: 1, name: 'Compact (1 Char)' }
];

export const DEFAULT_CONFIG = {
  visualizer: {
    colorTheme: 'RGB_CHROMA',
    bandWidth: 'auto',
    peakHoldMs: 1100,
    peakDecayRate: 38
  },
  dsp: {
    stereoMode: 'STEREO',
    dolbyMode: 'DOLBY-B',
    tapeType: 'TYPE-II',
    bassBoost: false,
    vocalMode: 'OFF',
    volume: 80
  },
  lyrics: {
    autoFetch: true,
    syncOffset: 0
  },
  library: {
    musicDir: join(homedir(), 'Music'),
    scanOnStartup: true
  }
};

export const SETTINGS_SECTIONS = [
  {
    id: 'dsp.vocalMode',
    section: '🎤 VOCAL & KARAOKE DSP',
    label: 'Vocal / Karaoke Mode',
    options: ['OFF', 'KARAOKE', 'ACAPELLA', 'VOICE_BOOST'],
    labels: ['OFF (Original Mix)', '🎤 KARAOKE (Vocal Cut)', '🎙 ACAPELLA (Vocal Isolate)', '🔊 VOICE BOOST (Clear Voice)'],
    get: (cfg) => cfg?.dsp?.vocalMode ?? 'OFF',
    set: (cfg, val) => { if (!cfg.dsp) cfg.dsp = {}; cfg.dsp.vocalMode = val; }
  },
  {
    id: 'visualizer.colorTheme',
    section: '🌈 VISUALIZER & SPECTRUM',
    label: 'Visualizer Theme',
    options: ['RGB_CHROMA', 'AMBER_GOLD', 'GREEN_PHOSPHOR', 'CYAN_NEON', 'MONOCHROME'],
    labels: ['🌈 RGB Chroma Wave', '📻 Amber Gold (Hi-Fi)', '📟 Phosphor Green', '🌆 Neon Synthwave', '🧊 Monochrome Ice'],
    get: (cfg) => cfg?.visualizer?.colorTheme ?? 'RGB_CHROMA',
    set: (cfg, val) => { if (!cfg.visualizer) cfg.visualizer = {}; cfg.visualizer.colorTheme = val; }
  },
  {
    id: 'visualizer.bandWidth',
    section: '🌈 VISUALIZER & SPECTRUM',
    label: 'Band Width',
    options: ['auto', 3, 2, 1],
    labels: ['Auto-Fit (3/2/1)', 'Wide (3)', 'Standard (2)', 'Compact (1)'],
    get: (cfg) => cfg?.visualizer?.bandWidth ?? 'auto',
    set: (cfg, val) => { if (!cfg.visualizer) cfg.visualizer = {}; cfg.visualizer.bandWidth = val; }
  },
  {
    id: 'visualizer.peakHoldMs',
    section: '🌈 VISUALIZER & SPECTRUM',
    label: 'Peak Hold',
    options: [1100, 2000, 800, 300],
    labels: ['Long (1.1s)', 'Ultra (2.0s)', 'Medium (800ms)', 'Snappy (300ms)'],
    get: (cfg) => cfg?.visualizer?.peakHoldMs ?? 1100,
    set: (cfg, val) => { if (!cfg.visualizer) cfg.visualizer = {}; cfg.visualizer.peakHoldMs = val; }
  },
  {
    id: 'visualizer.peakDecayRate',
    section: '🌈 VISUALIZER & SPECTRUM',
    label: 'Peak Falloff',
    options: [38, 20, 85],
    labels: ['Smooth (38 u/s)', 'Slow (20 u/s)', 'Fast (85 u/s)'],
    get: (cfg) => cfg?.visualizer?.peakDecayRate ?? 38,
    set: (cfg, val) => { if (!cfg.visualizer) cfg.visualizer = {}; cfg.visualizer.peakDecayRate = val; }
  },
  {
    id: 'dsp.stereoMode',
    section: '🎛 DSP HARDWARE DEFAULTS',
    label: 'Soundstage Field',
    options: ['STEREO', 'MONO', '3D WIDE'],
    labels: ['● STEREO', '◉ MONO', '✦ 3D WIDE'],
    get: (cfg) => cfg?.dsp?.stereoMode ?? 'STEREO',
    set: (cfg, val) => { if (!cfg.dsp) cfg.dsp = {}; cfg.dsp.stereoMode = val; }
  },
  {
    id: 'dsp.dolbyMode',
    section: '🎛 DSP HARDWARE DEFAULTS',
    label: 'Dolby NR Mode',
    options: ['DOLBY-B', 'DOLBY-C', 'DOLBY-S', 'OFF'],
    labels: ['DOLBY-B (Std)', 'DOLBY-C (High)', 'DOLBY-S (Spectral)', 'OFF (Bypass)'],
    get: (cfg) => cfg?.dsp?.dolbyMode ?? 'DOLBY-B',
    set: (cfg, val) => { if (!cfg.dsp) cfg.dsp = {}; cfg.dsp.dolbyMode = val; }
  },
  {
    id: 'dsp.tapeType',
    section: '🎛 DSP HARDWARE DEFAULTS',
    label: 'Tape Bias Type',
    options: ['TYPE-II', 'TYPE-I', 'TYPE-IV'],
    labels: ['Type-II (CrO2)', 'Type-I (Ferric)', 'Type-IV (Metal)'],
    get: (cfg) => cfg?.dsp?.tapeType ?? 'TYPE-II',
    set: (cfg, val) => { if (!cfg.dsp) cfg.dsp = {}; cfg.dsp.tapeType = val; }
  },
  {
    id: 'dsp.bassBoost',
    section: '🎛 DSP HARDWARE DEFAULTS',
    label: 'Mega Bass Boost',
    options: [false, true],
    labels: ['Disabled', '🔊 +7dB @ 60Hz ON'],
    get: (cfg) => Boolean(cfg?.dsp?.bassBoost),
    set: (cfg, val) => { if (!cfg.dsp) cfg.dsp = {}; cfg.dsp.bassBoost = val; }
  },
  {
    id: 'lyrics.autoFetch',
    section: '🎤 LIVE LYRICS & SYNC',
    label: 'Auto-Fetch Lyrics',
    options: [true, false],
    labels: ['Enabled (LRCLIB)', 'Disabled'],
    get: (cfg) => cfg?.lyrics?.autoFetch !== false,
    set: (cfg, val) => { if (!cfg.lyrics) cfg.lyrics = {}; cfg.lyrics.autoFetch = val; }
  },
  {
    id: 'library.scanOnStartup',
    section: '📁 MUSIC CRATES & LIBRARY',
    label: 'Scan on Launch',
    options: [true, false],
    labels: ['Enabled', 'Disabled'],
    get: (cfg) => cfg?.library?.scanOnStartup !== false,
    set: (cfg, val) => { if (!cfg.library) cfg.library = {}; cfg.library.scanOnStartup = val; }
  }
];

/**
 * Deep merge default config with user overrides to ensure schema safety
 */
export function mergeConfig(defaults, overrides) {
  if (!overrides || typeof overrides !== 'object') return { ...defaults };
  const merged = { ...defaults };
  for (const key of Object.keys(defaults)) {
    if (overrides[key] !== undefined) {
      if (typeof defaults[key] === 'object' && defaults[key] !== null && !Array.isArray(defaults[key])) {
        merged[key] = mergeConfig(defaults[key], overrides[key]);
      } else {
        merged[key] = overrides[key];
      }
    }
  }
  return merged;
}

/**
 * Load persistent settings from disk with safe fallback
 */
export async function loadConfig() {
  try {
    const raw = await readFile(CONFIG_FILE, 'utf8');
    const parsed = JSON.parse(raw);
    return mergeConfig(DEFAULT_CONFIG, parsed);
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Save current settings to disk
 */
export async function saveConfig(config) {
  try {
    await mkdir(CONFIG_DIR, { recursive: true });
    await writeFile(CONFIG_FILE, JSON.stringify(config, null, 2), 'utf8');
    return true;
  } catch {
    return false;
  }
}
