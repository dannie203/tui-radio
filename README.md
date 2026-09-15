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

## ⌨️ Keybindings

### 🎵 Audio Playback & Transport
| Key | Action | Description |
| :--- | :--- | :--- |
| **`Space`** | Play / Pause | Toggle audio playback |
| **`s`** | Stop | Stop playback and reset track position |
| **`n` / `p`** | Next / Previous | Skip to next or previous track in list/queue |
| **`+` / `-`** | Volume ±5% | Adjust audio output volume |
| **`[` / `]`** | Seek ±10s | Seek backward/forward 10 seconds (or adjust Lyrics offset in Lyrics view) |
| **`{` / `}`** | Fine Lyrics Offset | Adjust lyrics sync calibration by ±1.0s (Lyrics view) |
| **`0`** | Reset Lyrics Offset | Reset lyrics timing calibration back to `0.0s` |
| **`Shift+S`** | Matrix Cipher | Toggle real-time Matrix decryption cipher effect on lyrics |

### 🧭 Navigation & Library Exploration
| Key | Action | Description |
| :--- | :--- | :--- |
| **`1` — `4`** | Mode Select | `1`: Local Crates, `2`: Radio Stations, `3`: Queue, `4`: Online Streams |
| **`Tab`** | Cycle Modes | Seamlessly cycle through all 4 audio modes |
| **`j` / `k`** *(or `↓` / `↑`)* | Move Cursor | Move selection up / down in any list |
| **`Enter`** | Drill In / Play | Play selected track or enter Album / Crate view |
| **`h` / `Backspace`** *(or `←`)* | Drill Out / Back | Return from Album view back to Albums overview |
| **`v`** | View Style | Toggle Local Library view (`Albums & Crates` ↔ `All Tracks Flat`) |
| **`g`** | Genre Filter | Cycle radio station genre filter (Lo-Fi, Synthwave, Jazz, Rock, etc.) |

### 🗂️ Queue, Favorites & Playlists
| Key | Action | Description |
| :--- | :--- | :--- |
| **`a`** | Add to Queue | Enqueue selected track or entire album into playback queue |
| **`x`** | Remove from Queue | Remove highlighted item from the queue |
| **`c`** | Clear Queue | Clear all pending tracks from the queue |
| **`m`** | Favorite | Toggle favorite star (`★`) on current or selected track |
| **`M`** *(Shift+M)* | Mixtapes | Open / Close **Mixtape Manager** (`Enter` to play, `a` to add, `x` to del) |
| **`H`** *(Shift+H)* | History | Open / Close **Playback History** (`Enter` to play, `a` to queue) |

### 🎛️ DSP Equalization & Tape Recording
| Key | Action | Description |
| :--- | :--- | :--- |
| **`b`** | Mega Bass | Toggle analog sub-harmonic +7dB low-end boost |
| **`d`** | Dolby NR | Cycle hardware Dolby Noise Reduction (Off, Dolby B, Dolby C, Dolby S) |
| **`e`** | 32-Band EQ | Cycle ISO Equalizer presets (Flat, Rock, Vocal, Lo-Fi, Synth, EDM) |
| **`t`** | Theme | Cycle cyberpunk CRT color palettes |
| **`r`** | Repeat Mode | Cycle repeat state (Off, Repeat Track, Repeat All) |
| **`z`** | Shuffle | Toggle random playback shuffle |
| **`R`** *(Shift+R)* | Tape Record | Start / Stop stream recording directly to audio file |
| **`Ctrl+R`** | Cancel Record | Abort active audio recording immediately |

### 🔍 Search, Views & System
| Key | Action | Description |
| :--- | :--- | :--- |
| **`/`** | Live Filter | Real-time fuzzy filter across tracks, artists, and radio stations |
| **`u`** | Universal Search | Universal stream URL resolver (YouTube, Spotify, SoundCloud, direct URLs) |
| **`l`** | Lyrics Deck | Toggle live synchronized karaoke lyrics display |
| **`w`** | Artwork Deck | Toggle 24-bit TrueColor album cover art display |
| **`o`** | Settings | Open interactive configuration dashboard |
| **`?`** | Help | Toggle complete on-screen keyboard shortcut reference |
| **`F5`** | Hot-Reload | Re-read configuration and reload app in-place without stopping audio |
| **`q`** | Quit | Exit Boombox cleanly |

---

## Requirements

- **Backend**: [mpv](https://mpv.io) (must be installed and available in system PATH)
- **Optional**:
  - `yt-dlp`: For YouTube streams and playlist expansion
  - `ffmpeg`: For stream recording and audio format transcoding

---

## 📦 Installation Options

### 🪟 Windows

* **Option 1: 1-Line Automated PowerShell Installer — (Recommended)**
  * Open PowerShell and run:
  ```powershell
  irm https://raw.githubusercontent.com/dannie203/tui-radio/main/install.ps1 | iex
  ```
  * *Automatically verifies/installs required dependencies (`mpv`, `yt-dlp`, VC++ 2015-2022 Runtime), configures UTF-8 terminal launchers, registers user PATH, and creates Desktop + Start Menu shortcuts.*

* **Option 2: Standalone Portable Edition (`.zip`)**
  * Download [**`boombox-rs-windows-x86_64.zip`**](https://github.com/dannie203/tui-radio/releases/latest/download/boombox-rs-windows-x86_64.zip).
  * Extract anywhere and double-click `install.bat` (to auto-install dependencies) or `RUN-BOOMBOX.bat` (to launch portable). Zero admin rights required.

* **Option 3: Windows Installer Package (`.msi`)**
  * Download [**`Boombox-3.8.9-x86_64.msi`**](https://github.com/dannie203/tui-radio/releases/latest/download/Boombox-3.8.9-x86_64.msi).
  * Double-click to install. Automatically configures PATH, creates Desktop & Start Menu shortcuts, and registers in Windows Settings / Control Panel.

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
