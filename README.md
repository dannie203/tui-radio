# 📼 BOOMBOX RX-505 // Retro Cyberpunk TUI Audio Deck

[![Node.js](https://img.shields.io/badge/Node.js-v20+-green.svg)](https://nodejs.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![MPV Engine](https://img.shields.io/badge/Audio-MPV%20IPC-purple.svg)](https://mpv.io/)

A high-fidelity, retro cassette-styled Terminal User Interface (TUI) music player and radio explorer. Built with **Node.js**, **Blessed**, **MPV IPC**, and a hardware-modeled **DSP visualizer engine**.

```text
 ┌─────────────────────────────────────────────────────────────┐
 │ [A] RETRO STEREO DECK        DOLBY-B        Type-II CrO2    │
 │   (  |  ) ══════ [   BOOMBOX RX-505 ] ══════ (  |  )        │
 │       ● REC        ▶ PLAY        ❚❚ PAUSE        ▲ STOP     │
 └─────────────────────────────────────────────────────────────┘
 COUNTER : [01:23 / 03:45]               SOUNDSTAGE: STEREO
 TITLE   : Shook Ones, Pt. II
 ARTIST  : Mobb Deep [The Infamous]
 STREAM  : Loud Records / RCA
 PROG    : [■■■■■■■■□□□□□□□□□□□□□□□□] 01:23 / 03:45 (37%)
 DSP     : STEREO │ DOLBY-B │ TYPE-II │ MEGA BASS +7dB
 STATUS  : [ PLAYING ]  CODEC: [FLAC 24/96k]  VOL: [========  ] 80%
```

---

## ⚡ Features

- **📼 Crates // Local Music Library**:
  - Hierarchical crate navigation: **Artists → Albums → Tracks → Playlists**.
  - Lossless passthrough for Hi-Res audio (FLAC 24-bit/192kHz, WAV, ALAC, AIFF, MP3, AAC, OPUS, OGG).
  - Embedded cover artwork extraction rendered in high-resolution ANSI half-block pixel graphics (`▀`).
- **📻 Radio Stations Receiver**:
  - 290+ curated stations powered by the Radio-Browser API with instant offline fallback.
  - Curated genre presets: *Boom-Bap, 90s Rap, Lo-Fi, Underground, Classic, Favorites*.
  - Continuous animated green phosphor stream scanner for live infinite broadcasts.
- **🎤 Live Synced Karaoke Lyrics**:
  - Real-time synced scrolling lyrics powered by [LRCLIB](https://lrclib.net/) and local `.lrc` sidecar files.
  - Interactive sync timing calibration offset adjust (`<` / `>`).
- **📺 YouTube & YouTube Music Streaming**:
  - Instant direct stream extraction via `yt-dlp` for single tracks, playlists, and albums.
  - Interactive search mode (`/`) filtering out non-music noise and memes.
- **🎛 Vintage DSP Hardware Emulation**:
  - **Soundstage**: `STEREO`, `MONO`, `3D WIDE` stereo widening.
  - **Dolby Noise Reduction**: `DOLBY-B`, `DOLBY-C`, `DOLBY-S`, `OFF`.
  - **Tape Bias Formulations**: `TYPE-I` (Ferric), `TYPE-II` (CrO2 Chrome), `TYPE-IV` (Metal).
  - **Mega Bass Boost EQ**: Dual-stage low-end saturation (+7dB @ 60Hz, +4dB @ 125Hz).
- **📊 Real-time Spectrum & VU Needles**:
  - Asymmetric dual-rate EMA smoothed stereo VU needle meters with decibel readout (`-30dB` to `+3dB`).
  - 10-band ISO standard graphic equalizer spectrum (`31Hz` to `16kHz`) with peak hold and gravitational decay.
  - Zero per-frame memory allocations and Worker thread telemetry via `SharedArrayBuffer`.

---

## 📦 System Requirements

- **Node.js**: `v20.0.0` or higher
- **MPV**: `mpv` installed and available on system `$PATH` (with PipeWire, PulseAudio, or ALSA)
- **yt-dlp**: Required for YouTube streaming (`sudo pacman -S yt-dlp` / `brew install yt-dlp` / `apt install yt-dlp`)

---

## 🚀 Installation & Running

### 1. Clone the repository
```bash
git clone https://github.com/dannie203/hiphop-radio-tui.git
cd hiphop-radio-tui
```

### 2. Install dependencies
```bash
npm install
```

### 3. Launch the player
```bash
npm start
```

### 4. Optional: Global CLI link
```bash
npm link
# Run from anywhere in your terminal:
hiphop-radio
```

---

## 🎮 Keybindings & Hardware Controls

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
| **`1` - `4`** | Jump directly to Deck Mode / Crates category |
| **`/`** | Open search / crate filter dial |
| **`Y` / `U`** | Open YouTube URL & Stream Loader modal |
| **`L`** | Toggle **Live Synced Phosphor Lyrics** display |
| **`W`** | Toggle **Full-Res Album Cover Artwork** view |
| **`S`** | Cycle Soundstage (*Stereo / Mono / 3D Wide*) |
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
| **`O`** | Rescan local music directory (`~/Music`) |
| **`Q`** | Eject tape & Quit player |

---

## 🧪 Testing

Run the automated test suite with Node's native test runner:
```bash
npm test
```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
