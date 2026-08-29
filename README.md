# 📼 BOOMBOX-TUI (v2.4.1)

> **BOOMBOX RX-505** — A retro-cyberpunk cassette music player, Hi-Res local audio explorer, YouTube streamer, and worldwide radio deck for the Linux & Unix terminal.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Node.js Version](https://img.shields.io/badge/Node.js-%3E%3D20-green.svg)](https://nodejs.org)
[![Version](https://img.shields.io/badge/Version-2.4.1-amber.svg)](package.json)

---

## ⚡ Features

- 📼 **Retro Cassette Deck UI**: Dual rotating spools with dynamic tape-pack winding, tape-head bay, smoked cassette window, ANSI half-block album art decoding.
- 🎨 **Linux Desktop Theme Auto-Sync**: Automatic discovery and real-time palette synchronization with Omarchy (`colors.toml`), Pywal, and 7 Linux palettes (Catppuccin Mocha, Tokyo Night, Gruvbox, Nord, Dracula, Matrix Green, Amber Gold).
- 🎚️ **32-Band ISO Equalizer & Sound Presets**: 1-touch profiles (Flat, Mega Bass +7dB, Vocal Clarity, Rock Punch, Lo-Fi Warmth, Cyber Synthwave, Club EDM).
- 🔴 **Tape Recorder & Stream Ripper**: Record live streams to `~/Music/Boombox Recordings` in Native Opus, MP3 320k, FLAC, or M4A with 1-click toggle & instant cancellation (`Shift+R` / `Ctrl+R`).
- ★ **Mixtapes & Custom Playlists**: Create, manage, and save personal mixtapes across local and online tracks.
- 🎵 **Hi-Res Local Audio Player**: High-speed scanner and tag parser for FLAC (16/24/32-bit up to 192kHz), MP3, OPUS, OGG, M4A, WAV with folder hierarchy and artist/album sorting.
- 📻 **Worldwide Radio Explorer**: Integrated Radio Browser API with 30,000+ stations across genres (Lofi, Jazz, Synthwave, Hip-Hop, Classical, Rock, EDM, Ambient) and countries with smart anti-propaganda filtering.
- 📺 **YouTube & Multi-Platform Streaming**: Search YouTube Music directly from the TUI, load tracks, playlists, SoundCloud, Bandcamp, Mixcloud, and direct stream URLs.
- 🎛️ **Hardware DSP & Audio FX**:
  - **3D WIDE**: Open-Air acoustic matrix soundstage expansion.
  - **Mega Bass**: +7dB analog sub-harmonic low-end boost.
  - **Dolby NR Tape Bias**: Analog tape simulation (Dolby B, C, S, Off).
- 📊 **32-Band ISO Visualizer**: Logarithmic FFT spectrum analyzer with asymmetric attack/decay ballistic physics.
- 🎤 **Word-by-Word Synced Karaoke & Lyrics**: LRCLIB synced lyrics engine with Matrix-style word unscrambling and manual sync offset adjustment (`[` / `]`).
- 🖥️ **Desktop & System Tray Integration**: FreeDesktop StatusNotifierItem (SNI) + MPRIS2 Media Controller + Quickshell / EasyEffects-style AppMenu with 22 quick controls.
- 🪟 **Hyprland / Wayland Integration**: Smart workspace focus & scratchpad toggle script (`boombox-toggle`).

---

## 📦 Requirements

- **Node.js**: `>= 20.0.0`
- **mpv**: Audio playback backend engine
- **yt-dlp**: For YouTube/SoundCloud/Bandcamp streaming & recording
- **ffmpeg**: For stream capture & format conversion
- **python-gobject** (optional): For desktop MPRIS2 & System Tray AppMenu on Linux
- **jq** (optional): For Hyprland toggle script

---

## 🚀 Installation

### From Source
```bash
# Clone the repository
git clone https://github.com/dannie203/tui-radio.git
cd tui-radio

# Install dependencies and link globally
npm install
npm link
```

### Arch Linux / Manjaro / Omarchy (PKGBUILD)
```bash
# Build and install locally via makepkg
makepkg -si
```

---

## 🎮 Usage

Start the player:
```bash
boombox
# or aliases:
radio
hiphop-radio
```

Toggle or focus existing window in Hyprland / Sway / Wayland:
```bash
boombox-toggle
```

Run in background tray/daemon mode:
```bash
boombox --tray
```

---

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| `Space` | Play / Pause |
| `↑` / `↓` / `j` / `k` | Navigate lists & tracks |
| `Enter` / `→` | Play selected track / Open folder |
| `←` / `Backspace` | Go back / Parent directory |
| `N` / `P` | Next / Previous track |
| `+` / `-` | Volume up / down (+/- 5%) |
| `Tab` | Cycle deck mode (Local / Radio / YouTube / Queue) |
| `/` | Search current library / YouTube Music |
| `T` / `Ctrl+T` | Cycle Desktop Color Themes (Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula, Matrix...) |
| `E` | Cycle 32-Band Equalizer Sound Presets (Flat, Mega Bass, Vocal, Rock, Lo-Fi, Cyber, EDM) |
| `R` / `Ctrl+R` | Record current stream to `~/Music/Boombox Recordings` (Tape Recorder) |
| `M` / `Ctrl+M` | Add current track to Mixtape / Favorites |
| `L` | Toggle Synced Lyrics & Karaoke |
| `[` / `]` | Adjust Lyrics Sync Offset (±200ms) |
| `O` | Open DSP & Audio Settings Panel |
| `B` | Toggle Mega Bass Boost (+7dB) |
| `S` | Cycle Stereo DSP Mode (Stereo / 3D WIDE / Mono) |
| `D` | Cycle Dolby NR Bias (Off / Dolby-B / Dolby-C / Dolby-S) |
| `W` | Toggle Fullscreen Cassette / Album Artwork |
| `F` | Add / Remove current track from Favorites |
| `A` | Append selected track to Playback Queue |
| `r` | Toggle Repeat mode |
| `Z` | Toggle Shuffle mode |
| `Ctrl+D` / `H` | Detach / Minimize player to System Tray (Background mode) |
| `Esc` / `q` | Close overlay / Quit player |

---

## 🧪 Testing

Run the automated test suite (88 unit tests across 30 suites):
```bash
npm test
```

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See [LICENSE](LICENSE) for details.
