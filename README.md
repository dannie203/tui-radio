# 📼 BOOMBOX-RS (v3.3.0)

> **BOOMBOX RX-505** — A high-performance retro cyberpunk cassette deck music player, local Hi-Res audio explorer, multi-platform streaming engine, and worldwide radio deck written in **pure Rust**.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/Version-3.3.0-amber.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Wayland%20%7C%20X11-purple.svg)]()

---

## ⚡ Highlights & New in v3.3.0

- 🦀 **100% Pure Rust Port**: Blazing-fast startup, sub-millisecond I/O, minimal resource footprint (~20MB RAM vs 150MB+ in Node.js), and zero garbage collection pauses.
- 💽 **High-Res TrueColor Half-Block Artwork (`w`)**: 24-bit ANSI Half-Block (`▀`) terminal cover art renderer with 2x vertical resolution. Supports embedded FLAC/MP3/M4A pictures, YouTube & YouTube Music CDN thumbnails, `yt-dlp` stream covers (SoundCloud, Bandcamp, Spotify), and Apple iTunes (600x600 HD) fallback.
- 🌈 **32-Band ISO Equalizer with 60 FPS Fluid RGB Chroma Wave**: Real-time PCM audio FFT spectral analyzer with ballistic peak physics and 8 dynamic visualizer color palettes (`rgb_cycle`, `chroma_rainbow`, `vertical_gradient`, `cyberpunk_neon`, `fire_and_ice`, `matrix_phosphor`, `amber_vintage`, `theme_accent`).
- 🎤 **Sub-Millisecond Synced Karaoke Lyrics (`l`)**: Smart multi-source LRCLIB search flow with automated local `.lrc` caching alongside audio files, manual timing offset controls (`[` / `]` / `{` / `}`), and Matrix Cipher text decryption mode (`Shift+S`).
- 📺 **Universal Stream Queue Expansion (`u`)**: Paste YouTube, YouTube Music, SoundCloud, Bandcamp, or Qobuz links to auto-expand entire playlists directly into the queue.
- 🔴 **Tape Recorder & Stream Ripper (`R` / `Ctrl+R`)**: Rip streams and live radio broadcasts directly to `~/Music/Boombox Recordings` with configurable encoding formats (`OPUS`, `MP3`, `FLAC`, `M4A`).
- ⚡ **Instant Hot-Reload (`F5`)**: Reload app, theme, and configuration on the fly via dedicated keybinding or Unix signals (`SIGUSR1`, `SIGHUP`) without interrupting playback.
- 🖥️ **StatusNotifierItem (SNI) Desktop Tray**: Native KSNI FreeDesktop tray icon with AppMenu controls and live metadata.

---

## 🕹️ Keybindings Cheat Sheet

| Key | Action | Description |
| :--- | :--- | :--- |
| **`Space`** | Play / Pause | Toggle audio playback |
| **`s`** | Stop | Stop playback and reset position |
| **`n` / `p`** | Next / Prev | Skip to next or previous track in list/queue |
| **`[` / `]`** | Seek ±10s / Sync Offset | Seek ±10s in Deck view, or adjust Lyrics timing ±0.25s in Lyrics view |
| **`{` / `}`** | Fine Sync Offset | Adjust Lyrics timing offset by ±1.0s |
| **`0`** | Reset Sync Offset | Reset Lyrics offset to `0.0s` |
| **`Shift+S`** | Matrix Scramble | Toggle Matrix cipher text decryption effect in Lyrics view |
| **`+` / `-`** | Volume Up / Down | Adjust audio volume by configured step |
| **`b`** | Mega Bass | Toggle +7dB analog sub-harmonic low-end boost |
| **`d`** | Dolby Mode | Cycle Dolby Noise Reduction filter (Off, Dolby B, Dolby C, Dolby S) |
| **`e`** | EQ Profile | Cycle 32-band ISO Equalizer curves (Flat, Mega Bass, Vocal, Rock, Lo-Fi, Synth, EDM) |
| **`t`** | Theme | Cycle 8 retro cyberpunk & Hi-Fi color themes |
| **`r`** | Repeat Mode | Cycle repeat mode (Off, Repeat Track, Repeat All) |
| **`z`** | Shuffle | Toggle playlist shuffle on/off |
| **`1` - `4`** | Switch Mode | `1`: Local Library, `2`: Radio Stations, `3`: Queue, `4`: YouTube Streams |
| **`Tab`** | Cycle Mode | Cycle between Local, Radio, Queue, and Streams views |
| **`g`** | Cycle Genre | Paginate radio genres (Lofi, Jazz, Synthwave, Hip-Hop, Rock, EDM, Ambient, Classical) |
| **`l`** | Lyrics View | Toggle live synced karaoke lyrics screen |
| **`w`** | Artwork View | Toggle high-resolution album cover artwork & thumbnail screen |
| **`u`** | Stream URL | Open modal to paste and expand YouTube / SoundCloud / Stream links |
| **`/`** | Search Filter | Interactive fuzzy search across tracks, artists, and stations |
| **`m`** | Favorite | Toggle star / favorite on selected track |
| **`M`** | Mixtapes | Open custom mixtape playlist manager |
| **`R`** | Record | Start recording current track/stream to `~/Music/Boombox Recordings` |
| **`Ctrl+R`** | Cancel Record | Instantly abort active tape recording |
| **`o`** | Settings | Open comprehensive settings dashboard |
| **`F5`** | Hot-Reload | Instantly hot-reload application and configuration without audio drop |
| **`?`** | Help Modal | Show quick reference and keybindings modal |
| **`q`** | Quit | Exit Boombox |

---

## 📦 Requirements

- **Linux** (Wayland / Hyprland / Sway or X11)
- **mpv**: Audio playback backend daemon
- **yt-dlp** *(optional)*: For YouTube streaming & playlist expansion
- **ffmpeg** *(optional)*: For tape recording and stream ripping

---

## 🚀 Installation & Build

### From Source (Cargo)

```bash
git clone https://github.com/dannie203/tui-radio.git
cd tui-radio
cargo build --release
install -m 755 target/release/boombox-rs ~/.local/bin/boombox-rs
install -m 755 boombox-toggle ~/.local/bin/boombox-toggle
```

### Launching

```bash
# Launch Boombox interactive TUI
boombox-rs

# Or toggle as a scratchpad in Hyprland / Sway
boombox-toggle
```

---

## ⚙️ Configuration

Boombox automatically generates its configuration file at `~/.config/boombox/config.toml` on first run:

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

## 📜 License

Distributed under the **GNU General Public License v3.0 or later** (GPL-3.0-or-later). See [LICENSE](LICENSE) for details.
