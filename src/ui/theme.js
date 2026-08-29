/**
 * Theme Engine for BOOMBOX-TUI
 * Supporting modern popular Linux palettes (Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula, Matrix, Amber)
 */

export const THEMES = {
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
  return THEMES[themeId] || THEMES.AMBER_GOLD;
}

export function cycleTheme(currentId, delta = 1) {
  const idx = THEME_KEYS.indexOf(currentId);
  const nextIdx = (idx + delta + THEME_KEYS.length) % THEME_KEYS.length;
  return THEME_KEYS[nextIdx];
}
