import { existsSync, readFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

/**
 * Parses simple key = value pairs from TOML color files (Omarchy / Linux themes)
 */
function parseTomlColors(filePath) {
  if (!existsSync(filePath)) return null;
  try {
    const raw = readFileSync(filePath, 'utf8');
    const colors = {};
    for (const line of raw.split('\n')) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#') || !trimmed.includes('=')) continue;
      const [k, v] = trimmed.split('=', 1).map((s) => s.trim());
      const rest = trimmed.slice(trimmed.indexOf('=') + 1).trim().replace(/^["']|["']$/g, '');
      if (rest.startsWith('#') || rest.startsWith('rgb') || rest.startsWith('rgba')) {
        colors[k] = rest;
      }
    }
    return colors;
  } catch {
    return null;
  }
}

/**
 * Detects current Linux desktop / Omarchy theme colors
 */
export function detectSystemTheme() {
  const home = homedir();
  const omarchyStatePath = join(home, '.local', 'state', 'omarchy', 'current', 'theme', 'colors.toml');
  const omarchyConfigPath = join(home, '.config', 'omarchy', 'current', 'theme', 'colors.toml');
  const pywalCachePath = join(home, '.cache', 'wal', 'colors.json');

  // 1. Check Omarchy Dynamic System Theme
  const omarchyColors = parseTomlColors(omarchyStatePath) || parseTomlColors(omarchyConfigPath);
  if (omarchyColors) {
    const accent = omarchyColors.accent || omarchyColors.active_border_color || omarchyColors.cyan || '#7aa2f7';
    const bgDark = omarchyColors.darker_background || omarchyColors.dark_background || omarchyColors.background || '#101315';
    const bgPanel = omarchyColors.background || omarchyColors.dark_background || '#15181b';
    const fg = omarchyColors.foreground || omarchyColors.bright_foreground || '#cacccc';
    const muted = omarchyColors.muted || omarchyColors.dark_foreground || '#4b4e55';
    const borderDim = omarchyColors.selection || omarchyColors.muted || '#343d41';
    const borderFocus = omarchyColors.active_border_color || accent;

    return {
      id: 'SYSTEM_AUTO',
      name: '🖥️ System Auto-Sync (Omarchy / OS Theme)',
      bgDark,
      bgPanel,
      bgLcd: omarchyColors.darker_background || bgDark,
      borderDim,
      borderFocus,
      borderLcd: borderDim,
      amber: accent,
      amberBright: omarchyColors.light_foreground || omarchyColors.bright_yellow || fg,
      amberDim: omarchyColors.blue || omarchyColors.muted || muted,
      greenPhosphor: omarchyColors.green || omarchyColors.bright_green || '#9fa5a9',
      greenDim: borderDim,
      gold: omarchyColors.yellow || omarchyColors.bright_yellow || '#d9dbdc',
      cyanDolby: omarchyColors.cyan || omarchyColors.bright_cyan || accent,
      redLed: omarchyColors.bright_red || omarchyColors.red || '#de6145',
      yellowLed: omarchyColors.yellow || '#d9dbdc',
      cream: fg,
      chrome: omarchyColors.bright_foreground || fg,
      muted
    };
  }

  // 2. Check Pywal System Colors
  if (existsSync(pywalCachePath)) {
    try {
      const walData = JSON.parse(readFileSync(pywalCachePath, 'utf8'));
      if (walData?.colors) {
        const c = walData.colors;
        const special = walData.special || {};
        return {
          id: 'SYSTEM_AUTO',
          name: '🖥️ System Auto-Sync (Pywal / Wal Theme)',
          bgDark: special.background || c.color0 || '#11111b',
          bgPanel: c.color0 || '#181825',
          bgLcd: special.background || '#11111b',
          borderDim: c.color8 || '#45475a',
          borderFocus: c.color4 || c.color6 || '#cba6f7',
          borderLcd: c.color8 || '#45475a',
          amber: c.color3 || c.color4 || '#fabd2f',
          amberBright: c.color11 || c.color15 || '#ffd24d',
          amberDim: c.color8 || '#89b4fa',
          greenPhosphor: c.color2 || c.color10 || '#a6e3a1',
          greenDim: c.color8 || '#313244',
          gold: c.color3 || c.color11 || '#f9e2af',
          cyanDolby: c.color6 || c.color14 || '#89dceb',
          redLed: c.color1 || c.color9 || '#f38ba8',
          yellowLed: c.color3 || '#fab387',
          cream: special.foreground || c.color7 || c.color15 || '#cdd6f4',
          chrome: c.color7 || '#bac2de',
          muted: c.color8 || '#6c7086'
        };
      }
    } catch {}
  }

  // Fallback to Amber Gold
  return null;
}

export const THEMES = {
  SYSTEM_AUTO: {
    id: 'SYSTEM_AUTO',
    name: '🖥️ System Auto-Sync (Omarchy / OS Theme)',
    bgDark: '#101315',
    bgPanel: '#15181b',
    bgLcd: '#0c0e10',
    borderDim: '#343d41',
    borderFocus: '#798186',
    borderLcd: '#22282c',
    amber: '#798186',
    amberBright: '#cacccc',
    amberDim: '#4b4e55',
    greenPhosphor: '#9fa5a9',
    greenDim: '#22282c',
    gold: '#d9dbdc',
    cyanDolby: '#707070',
    redLed: '#de6145',
    yellowLed: '#d9dbdc',
    cream: '#cacccc',
    chrome: '#a5aeb4',
    muted: '#4b4e55'
  },
  AMBER_GOLD: {
    id: 'AMBER_GOLD',
    name: '📻 Vintage Amber Gold (Nakamichi Hi-Fi)',
    bgDark: '#0a0d13',
    bgPanel: '#11151f',
    bgLcd: '#06130a',
    borderDim: '#243348',
    borderFocus: '#ffb000',
    borderLcd: '#1b4d24',
    amber: '#ffb000',
    amberBright: '#ffd24d',
    amberDim: '#996900',
    greenPhosphor: '#33ff33',
    greenDim: '#114a1a',
    gold: '#f5c542',
    cyanDolby: '#00e5ff',
    redLed: '#ff3344',
    yellowLed: '#ffee33',
    cream: '#f3ead8',
    chrome: '#b8c4ce',
    muted: '#6f7e91'
  },
  CATPPUCCIN_MOCHA: {
    id: 'CATPPUCCIN_MOCHA',
    name: '☕ Catppuccin Mocha (Pastel Lavender)',
    bgDark: '#11111b',
    bgPanel: '#181825',
    bgLcd: '#1e1e2e',
    borderDim: '#45475a',
    borderFocus: '#cba6f7',
    borderLcd: '#585b70',
    amber: '#cba6f7', // Mauve
    amberBright: '#f5c2e7', // Pink
    amberDim: '#89b4fa', // Blue
    greenPhosphor: '#a6e3a1', // Green
    greenDim: '#313244',
    gold: '#f9e2af', // Yellow
    cyanDolby: '#89dceb', // Sky
    redLed: '#f38ba8', // Red
    yellowLed: '#fab387', // Peach
    cream: '#cdd6f4', // Text
    chrome: '#bac2de', // Subtext
    muted: '#6c7086' // Overlay
  },
  TOKYO_NIGHT: {
    id: 'TOKYO_NIGHT',
    name: '🌃 Tokyo Night (Neon Storm)',
    bgDark: '#16161e',
    bgPanel: '#1a1b26',
    bgLcd: '#13141c',
    borderDim: '#292e42',
    borderFocus: '#7aa2f7',
    borderLcd: '#414868',
    amber: '#7aa2f7', // Blue
    amberBright: '#7dcfff', // Cyan
    amberDim: '#3d59a1',
    greenPhosphor: '#73daca', // Teal Green
    greenDim: '#1f2335',
    gold: '#e0af68', // Yellow
    cyanDolby: '#2ac3de',
    redLed: '#f7768e', // Red
    yellowLed: '#ff9e64', // Orange
    cream: '#c0caf5',
    chrome: '#a9b1d6',
    muted: '#565f89'
  },
  GRUVBOX_RETRO: {
    id: 'GRUVBOX_RETRO',
    name: '🍂 Gruvbox Retro (Warm Analog)',
    bgDark: '#1d2021',
    bgPanel: '#282828',
    bgLcd: '#141617',
    borderDim: '#504945',
    borderFocus: '#fe8019',
    borderLcd: '#3c3836',
    amber: '#fe8019', // Orange
    amberBright: '#fabd2f', // Yellow
    amberDim: '#af3a03',
    greenPhosphor: '#b8bb26', // Green
    greenDim: '#32302f',
    gold: '#fabd2f',
    cyanDolby: '#8ec07c', // Aqua
    redLed: '#fb4934',
    yellowLed: '#d79921',
    cream: '#ebdbb2',
    chrome: '#d5c4a1',
    muted: '#928374'
  },
  NORD_FROST: {
    id: 'NORD_FROST',
    name: '❄️ Nord Frost (Arctic Ice)',
    bgDark: '#242933',
    bgPanel: '#2e3440',
    bgLcd: '#1e222a',
    borderDim: '#434c5e',
    borderFocus: '#88c0d0',
    borderLcd: '#3b4252',
    amber: '#88c0d0', // Frost Cyan
    amberBright: '#8fbcbb',
    amberDim: '#5e81ac',
    greenPhosphor: '#a3be8c', // Nord Green
    greenDim: '#2e3440',
    gold: '#ebcb8b', // Nord Yellow
    cyanDolby: '#81a1c1',
    redLed: '#bf616a',
    yellowLed: '#d08770',
    cream: '#eceff4',
    chrome: '#e5e9f0',
    muted: '#4c566a'
  },
  DRACULA: {
    id: 'DRACULA',
    name: '🧛 Dracula (Vampire Night)',
    bgDark: '#1e1f29',
    bgPanel: '#282a36',
    bgLcd: '#191a21',
    borderDim: '#44475a',
    borderFocus: '#bd93f9',
    borderLcd: '#6272a4',
    amber: '#bd93f9', // Purple
    amberBright: '#ff79c6', // Pink
    amberDim: '#6272a4',
    greenPhosphor: '#50fa7b', // Green
    greenDim: '#282a36',
    gold: '#f1fa8c', // Yellow
    cyanDolby: '#8be9fd', // Cyan
    redLed: '#ff5555',
    yellowLed: '#ffb86c', // Orange
    cream: '#f8f8f2',
    chrome: '#e6e6e6',
    muted: '#6272a4'
  },
  MATRIX_GREEN: {
    id: 'MATRIX_GREEN',
    name: '📟 Matrix Cyber Green (Hacker CRT)',
    bgDark: '#030a04',
    bgPanel: '#071509',
    bgLcd: '#020603',
    borderDim: '#0d3814',
    borderFocus: '#00ff41',
    borderLcd: '#0b2e10',
    amber: '#00ff41', // Matrix Green
    amberBright: '#33ff66',
    amberDim: '#008f11',
    greenPhosphor: '#00ff41',
    greenDim: '#003b00',
    gold: '#88ff88',
    cyanDolby: '#00e5ff',
    redLed: '#ff3344',
    yellowLed: '#ffff33',
    cream: '#e0ffe4',
    chrome: '#a0e6a8',
    muted: '#286830'
  }
};

export const THEME_LIST = Object.values(THEMES);
export const THEME_KEYS = Object.keys(THEMES);

export function getTheme(themeId) {
  if (themeId === 'SYSTEM_AUTO' || !themeId) {
    const detected = detectSystemTheme();
    if (detected) return detected;
    return THEMES.SYSTEM_AUTO;
  }
  return THEMES[themeId] || THEMES.AMBER_GOLD;
}

export function cycleTheme(currentId, delta = 1) {
  const idx = THEME_KEYS.indexOf(currentId);
  const nextIdx = (idx + delta + THEME_KEYS.length) % THEME_KEYS.length;
  return THEME_KEYS[nextIdx];
}
