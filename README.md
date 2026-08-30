# BOOMBOX-RS

> A high-performance retro cyberpunk cassette deck music player, local Hi-Res audio explorer, multi-platform streaming engine, and worldwide radio deck written in pure Rust.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-purple.svg)]()

---

## Core Features

- **RX-505 Cassette Deck Simulation**: Vintage single-bay cassette mechanism with animated dual spools, tape bias formulation (Type I Normal, Type II High/CrO2, Type IV Metal), Dolby Noise Reduction (B, C, S), and real-time transport status indicators.
- **CRT Phosphor Monitor & Real-Time Oscilloscope**: Dual-beam stereo oscilloscope rendering real-time audio waveforms over coordinate reticle grids with hardware telemetry (sweep frequency, volts/div, trigger sync) and dynamic viewport auto-scaling.
- **32-Band ISO Equalizer & Dual VU Meters**: Real-time FFT audio spectrum analyzer (20Hz — 20kHz) with ballistic peak tracking, calibrated dB scales, and 8 dynamic visualizer color palettes.
- **Hi-Fi Local Audio Crates**: Hierarchical album and track browser with audio format and metadata identification (FLAC, MP3, WAV, OPUS, AAC, M4A, OGG).
- **Curated Worldwide Radio**: Direct streaming access to curated global radio stations categorized across multiple genres (Lo-Fi, Jazz, Synthwave, Hip-Hop, Rock, Electronic, Ambient, Classical).
- **Universal Stream Player & Queue Expansion**: Seamless stream playback and playlist resolution for YouTube, YouTube Music, SoundCloud, Bandcamp, and direct media URLs.
- **Synchronized Karaoke Lyrics**: Real-time line-by-line synced lyrics via LRCLIB with local caching, sub-millisecond offset calibration, and Matrix cipher decryption mode.
- **TrueColor Terminal Cover Artwork**: 24-bit ANSI half-block artwork renderer for embedded album pictures, YouTube thumbnails, and online cover art.
- **DSP Audio Equalization & Tape Recording**: Live parametric equalizer curves, analog bass boost, and stream recording to OPUS, MP3, FLAC, or M4A.
- **Desktop Tray Integration**: System tray integration with media controls and playback metadata.

---

## Keybindings

| Key | Action | Description |
| :--- | :--- | :--- |
| **`Space`** | Play / Pause | Toggle audio playback |
| **`s`** | Stop | Stop playback and reset track position |
| **`n` / `p`** | Next / Previous | Skip to next or previous track in list/queue |
| **`[` / `]`** | Seek / Sync Offset | Seek ±10s in Deck view, or adjust Lyrics timing ±0.25s in Lyrics view |
| **`{` / `}`** | Fine Sync Offset | Adjust Lyrics timing offset by ±1.0s |
| **`0`** | Reset Sync Offset | Reset Lyrics offset to `0.0s` |
| **`Shift+S`** | Matrix Scramble | Toggle Matrix cipher text decryption effect in Lyrics view |
| **`+` / `-`** | Volume | Adjust audio volume |
| **`b`** | Mega Bass | Toggle analog sub-harmonic low-end boost |
| **`d`** | Dolby NR | Cycle Dolby Noise Reduction (Off, Dolby B, Dolby C, Dolby S) |
| **`e`** | EQ Preset | Cycle Equalizer curves (Flat, Mega Bass, Vocal, Rock, Lo-Fi, Synth, EDM) |
| **`t`** | Theme | Cycle color themes |
| **`r`** | Repeat Mode | Cycle repeat mode (Off, Repeat Track, Repeat All) |
| **`z`** | Shuffle | Toggle playlist shuffle |
| **`1` - `4`** | Mode Select | `1`: Local Library, `2`: Radio Stations, `3`: Queue, `4`: Streams |
| **`Tab`** | Cycle Mode | Switch between Local, Radio, Queue, and Streams views |
| **`g`** | Cycle Genre | Filter radio stations by genre |
| **`l`** | Lyrics View | Toggle synchronized karaoke lyrics view |
| **`w`** | Artwork View | Toggle high-resolution album cover artwork view |
| **`H`** | History | Open smart playback history modal (deduplicated recency list) |
| **`u`** | Stream Search | Universal stream URL resolver and online search (YouTube, SoundCloud, Spotify) |
| **`/`** | Live Filter | Zero-latency in-memory filter across tracks, artists, stations, and history |
| **`m`** | Favorite | Toggle star / favorite flag on selected track |
| **`M`** | Mixtapes | Open mixtape playlist manager |
| **`R`** | Record | Start recording audio stream to local library |
| **`Ctrl+R`** | Cancel Record | Abort active recording |
| **`o`** | Settings | Open settings dashboard (Autoplay, Equalizer, Dolby, Tape, Theme) |
| **`F5`** | Hot-Reload | Reload configuration and theme without dropping playback |
| **`?`** | Help | Show shortcut and reference modal |
| **`q`** | Quit | Exit application |

---

## Requirements

- **Backend**: [mpv](https://mpv.io) (must be installed and available in system PATH)
- **Optional**:
  - `yt-dlp`: For YouTube streams and playlist expansion
  - `ffmpeg`: For stream recording and audio format transcoding

---

## 📦 Installation Options

### 🪟 Windows

* **Option 1: Windows Installer Package (`.msi`) — (Recommended)**
  * Download [**`Boombox-3.8.3-x86_64.msi`**](https://github.com/dannie203/tui-radio/releases/latest/download/Boombox-3.8.3-x86_64.msi).
  * Double-click to install. Automatically configures PATH, creates Desktop & Start Menu shortcuts, and registers in Windows Settings / Control Panel with 1-click clean uninstall.
* **Option 2: Standalone Portable Edition (`.zip`)**
  * Download [**`boombox-rs-windows-x86_64.zip`**](https://github.com/dannie203/tui-radio/releases/latest/download/boombox-rs-windows-x86_64.zip).
  * Extract anywhere and double-click `RUN-BOOMBOX.bat` or `boombox-rs.exe`. Zero installation or admin rights required.
* **Option 3: 1-Line PowerShell Installer**
  ```powershell
  irm https://raw.githubusercontent.com/dannie203/tui-radio/main/install.ps1 | iex
  ```

---

### 🐧 Linux & macOS

* **Option 1: 1-Line Automated Script**
  ```bash
  curl -sSL https://raw.githubusercontent.com/dannie203/tui-radio/main/install.sh | bash
  ```
  *Automatically downloads latest release, installs `boombox` to `~/.local/bin`, installs desktop entry and HiColor icons.*
* **Option 2: Manual Binary (`.tar.gz`)**
  * Download [**`boombox-rs-linux-x86_64.tar.gz`**](https://github.com/dannie203/tui-radio/releases/latest/download/boombox-rs-linux-x86_64.tar.gz), extract and copy `boombox-rs` to your `$PATH`.*

---

## 🛠️ Build from Source

### Linux

```bash
git clone https://github.com/dannie203/tui-radio.git
cd tui-radio
cargo build --release
./target/release/boombox-rs
```

### Windows

```powershell
git clone https://github.com/dannie203/tui-radio.git
cd tui-radio
cargo build --release
.\target\release\boombox-rs.exe
```

---

## Configuration

Configuration is automatically created at `~/.config/boombox/config.toml` (Linux) or `%APPDATA%\boombox\config.toml` (Windows):

```toml
[general]
music_dir = "~/Music"
volume_step = 5
notifications = true

[ui]
theme = "cyberpunk"
spectrum_color_mode = "rgb_cycle"
lyrics_offset = 0.0
matrix_scramble = false

[audio]
eq_preset = "flat"
record_format = "opus"
```

---

## License

Distributed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See [LICENSE](LICENSE) for details.
