# 📼 BOOMBOX RX-505 // Retro Cyberpunk TUI Audio Deck

[![Node.js](https://img.shields.io/badge/Node.js-v20+-green.svg?style=flat-square&logo=node.js)](https://nodejs.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg?style=flat-square)](LICENSE)
[![Audio Engine](https://img.shields.io/badge/Audio-MPV%20IPC%20%7C%20PipeWire-purple.svg?style=flat-square&logo=linux)](https://mpv.io/)
[![Interface](https://img.shields.io/badge/Interface-Blessed%20TUI-cyan.svg?style=flat-square)](https://github.com/chjj/blessed)
[![Desktop Integration](https://img.shields.io/badge/Desktop-MPRIS2%20%7C%20SNI%20Tray-orange.svg?style=flat-square&logo=archlinux)](https://specifications.freedesktop.org/mpris-spec/latest/)
[![Tests](https://img.shields.io/badge/Tests-72%2F72%20Passed-brightgreen.svg?style=flat-square)](https://nodejs.org/api/test.html)

A high-fidelity, retro cassette-styled Terminal User Interface (TUI) music player, audio visualizer, and radio explorer. Built with **Node.js**, **Blessed**, **MPV IPC**, and a hardware-modeled **32-Band ISO Graphic Equalizer DSP Engine**.

```text
 ┌─────────────────────────────────────────────────────────────┐
 │ [A] RETRO STEREO DECK        DOLBY-B        Type-II CrO2    │
 │   (  |  ) ══════ [   BOOMBOX RX-505 ] ══════ (  |  )        │
 │       ● REC        ▶ PLAY        ❚❚ PAUSE        ▲ STOP     │
 └─────────────────────────────────────────────────────────────┘
 COUNTER : [01:23 / 03:45]               SOUNDSTAGE: ✦ 3D WIDE
 TITLE   : Shook Ones, Pt. II
 ARTIST  : Mobb Deep [The Infamous]
 STREAM  : Loud Records / RCA
 PROG    : [■■■■■■■■□□□□□□□□□□□□□□□□] 01:23 / 03:45 (37%)
 DSP     : 3D WIDE │ DOLBY-B │ TYPE-II │ MEGA BASS +7dB
 STATUS  : [ PLAYING ]  CODEC: [FLAC 24/96k]  VOL: [========  ] 80%
```

---

## ⚡ Key Highlights & Features

### 📊 1. 32-Band ISO 266 Equalizer & Dual VU Ballistics Engine
- **32 ISO Standard 1/3-Octave Bands**: Spans from **20Hz to 20kHz** (`20, 25, 31.5, ..., 16k, 18k, 20k`).
- **ISO 226 Equal-Loudness & Pink-Noise Tilt**: Psychoacoustic slope compensation (`0dB` @ 20Hz → `+34dB` @ 20kHz) ensures high frequencies (cymbals, hi-hats, air harmonics) are dynamic and responsive.
- **RMS + Peak Transient Detection**: `65% RMS + 35% Peak` blend captures rapid percussion hits without energy dilution.
- **High-Res Dual VU Needle Meters**: Asymmetric dual-rate EMA smoothing with calibrated decibel scale (`-30dB` to `+3dB`).
- **5 Aesthetic Themes**:
  - 🌈 **RGB Chroma Wave** (Dynamic 360° chromatic sweep)
  - 🟡 **Vintage Amber Gold** (Hi-Fi 1980s cassette deck)
  - 🟢 **Cyber Phosphor Green** (Classic CRT oscilloscope)
  - 🟣 **Neon Synthwave** (Outrun Magenta & Cyan)
  - ⚪ **Monochrome Ice** (Minimalist Slate & White)
- **Zero GC Churn**: Thread telemetry over `SharedArrayBuffer` with precomputed LUT tables.

---

### 🎛 2. Open-Air 3D Soundstage & Hardware DSP Emulation
- **✦ 3D WIDE (Open-Air Free-Field Spatializer)**:
  - **Zero Feedback Delay Reverb**: Eliminates boxy, muddy room echoes.
  - **Mid/Side Matrix Separation**: Keeps center Lead Vocal, Kick, and Bass 100% dry and punchy while widening the stereo ambiance (`slev=1.45`, `base=0.35`).
  - **Air Presence Lift**: High-shelf boost (`+1.5dB @ 12kHz`) for open-air acoustic clarity.
- **STEREO & MONO Soundstages**: True center-summed dual mono and bit-perfect clean stereo passthrough.
- **Mega Bass Boost EQ**: Dual-stage saturation (`+7dB @ 60Hz`, `+4dB @ 125Hz`).
- **Dolby Noise Reduction**: `DOLBY-B`, `DOLBY-C`, `DOLBY-S`, `OFF`.
- **Tape Bias Formulations**: `TYPE-I` (Ferric), `TYPE-II` (CrO2 Chrome), `TYPE-IV` (Metal).

---

### 💻 3. Linux Desktop System Tray & MPRIS2 Integration
- **StatusNotifierItem (SNI Tray)**:
  - Appears natively on Linux panels (Waybar, Quickshell, Polybar, KDE, GNOME).
  - Hover tooltip displays active track, artist, and playback status.
  - Right-click context menu: Play/Pause, Next/Prev, 3D Soundstage, Mega Bass, Volume, Focus TUI, Quit.
- **MPRIS2 D-Bus Controller (`org.mpris.MediaPlayer2.hiphop_radio`)**:
  - Full support for Linux lockscreens, top bar media widgets, and hardware media keys (`Fn + Play/Pause/Next/Prev`).
- **Headless / Daemon Mode**:
  - Run in the background without terminal: `hiphop-radio --tray` or `hiphop-radio --daemon`.
- **Desktop Notifications**: Instant notifications via `notify-send` on track change.

---

### 🎤 4. Matrix Word-by-Word Karaoke & Synced Lyrics
- **Matrix De-scrambler Animation**: Future lyrics lines appear encrypted in green Matrix katakana glyphs (`ｦｱｳｴｵｶｷｹｺ...`) and decrypt word-by-word as sung.
- **Multi-Source Fetching**: Powered by [LRCLIB](https://lrclib.net/) and local `.lrc` sidecar files.
- **Timing Offset Calibration**: Real-time sync adjustment (`<` / `>`) saved per track.

---

### 📼 5. Hierarchical Crates & Multi-Source Library
- **Crates Explorer**: Browse by **Artists → Albums → Tracks → Playlists**.
- **Lossless Codec Detection**: Real-time format badges: `[FLAC 24/192k]`, `[FLAC 16/44.1k]`, `[MP3 320k]`, `[OPUS 160k]`, `[AAC+ 128k]`.
- **Album Cover Art View (`W`)**: High-resolution ANSI pixel art renderer (`▀`).
- **Radio Stations Receiver**: 290+ curated stations with genre presets (*Boom-Bap, 90s Rap, Lo-Fi, Underground, Classic, Favorites*).
- **YouTube & YouTube Music Loader**: Direct audio stream extraction with yt-dlp.

---

## 📦 System Requirements

- **Linux / macOS**: Modern terminal emulator with 256-color or Truecolor support (Kitty, Alacritty, Foot, Ghostty, WezTerm, iTerm2).
- **Node.js**: `v20.0.0` or higher.
- **MPV**: `mpv` installed and available on `$PATH` (PipeWire, PulseAudio, or ALSA).
- **yt-dlp**: Required for YouTube stream playback (`sudo pacman -S yt-dlp` / `apt install yt-dlp` / `brew install yt-dlp`).
- **Optional (for Tray & MPRIS2 on Linux)**: `python3` with `python-gobject` (`sudo pacman -S python-gobject`).

---

## 🚀 Installation & Usage

### 1. Clone & Install
```bash
git clone https://github.com/dannie203/hiphop-radio-tui.git
cd hiphop-radio-tui
npm install
```

### 2. Launch the Player
```bash
# Standard TUI mode
npm start

# Or run in Background Tray / Daemon mode (no terminal UI)
node bin/index.js --tray
```

### 3. Global Installation
```bash
npm link

# Launch anywhere:
hiphop-radio

# Launch minimized to tray:
hiphop-radio --tray
```

---

## 🎮 Keybindings & Controls

| Key | Action |
| :--- | :--- |
| **`↵` / `→` / `l`** | **Dive In** (Open Artist / Album / Play track) |
| **`⎋` / `←` / `h`** | **Back Up** (Return to parent crate level) |
| **`j` / `k`** or **`↓` / `↑`** | Navigate list selection |
| **`PgUp` / `PgDn`** | Fast scroll list (8 items) |
| **`␣` (Space)** | Pause / Resume playback |
| **`N` / `P`** | Next / Previous track |
| **`Shift+←` / `Shift+→`** | Seek backward / forward 10 seconds |
| **`+` / `-`** | Volume ±5% |
| **`Tab` / `M`** | Cycle Deck Mode (*Local Tracks / Radio / Queue / YouTube*) |
| **`1` - `4`** | Jump directly to Deck Mode (*1: Local, 2: Radio, 3: Queue, 4: YouTube*) |
| **`/`** | Open search dial / crate filter |
| **`Y` / `U`** | Open YouTube URL & Stream Loader modal |
| **`L`** | Toggle **Live Synced Matrix Lyrics** view |
| **`W`** | Toggle **Full-Res Album Cover Artwork** view |
| **`O`** | Open **Settings & Preferences Panel** (Theme, Visualizer, DSP defaults) |
| **`S`** | Cycle Soundstage (*✦ 3D WIDE / MONO / STEREO*) |
| **`D`** | Cycle Dolby Noise Reduction (*DOLBY-B / C / S / OFF*) |
| **`T`** | Cycle Tape Bias (*Type-II CrO2 / Type-I Fe / Type-IV Metal*) |
| **`B`** | Toggle **Mega Bass Boost** EQ |
| **`G`** | Cycle Radio Genre filter (*Boom-Bap / Lo-Fi / 90s Rap / Underground / Classic / Favs*) |
| **`A`** | Add track / album / artist to Queue |
| **`C`** | Clear playback Queue |
| **`X` / `Delete`** | Remove item from Queue |
| **`Z`** | Toggle Shuffle mode |
| **`R`** | Toggle Repeat mode (*Off / All / One*) |
| **`F`** | Star / Favorite active track or station |
| **`Q`** | Eject tape & Quit player |

---

## ⚙️ Configuration & Customization

Settings are automatically saved and persistent in `~/.config/hiphop-tui/config.json`:

```json
{
  "visualizer": {
    "colorTheme": "RGB_CHROMA",
    "bandWidth": "auto",
    "peakHoldMs": 600,
    "peakDecayRate": 40
  },
  "dsp": {
    "defaultStereoMode": "3D WIDE",
    "defaultDolbyMode": "DOLBY-B",
    "defaultTapeType": "TYPE-II",
    "bassBoost": false
  },
  "library": {
    "musicDir": "~/Music",
    "scanOnStartup": true
  }
}
```

---

## 🧪 Automated Testing

Run the full automated test suite with Node's native test runner:

```bash
npm test
```

```text
ℹ tests 72
ℹ suites 24
ℹ pass 72
ℹ fail 0
```

---

## 📄 License & Legal Notice

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)** - see the [LICENSE](LICENSE) file for details.

### ⚖️ Copyleft Protection
- You are free to run, study, share, and modify this software.
- Any modified or derived versions **must remain open-source under the GNU GPL v3.0** and retain original author attribution.
- Packaging this software as closed-source proprietary software for commercial sale is strictly prohibited by the GPL-3.0 license.

### 🛡️ Disclaimer & Fair Use
- **BOOMBOX RX-505** is an open-source client-side audio player and terminal user interface. It **does not host, store, or distribute** any copyrighted audio or media files on its own servers.
- Online streams, radio broadcasts, and synced lyrics are resolved dynamically from public third-party services (Radio-Browser, LRCLIB, YouTube, SoundCloud, Bandcamp) as requested by the end user.
- All trademarks, logos, and artist materials belong to their respective copyright holders.
