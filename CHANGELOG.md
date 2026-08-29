# 📜 Changelog (Boombox-RS)

All notable changes to the **BOOMBOX-RS** Rust port will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.3.0] - 2026-08-30

### 🚀 Added
- **💽 High-Resolution Album Cover Artwork & Multi-Platform Thumbnails** (`w`):
  - TrueColor 24-bit Half-Block (`▀`) terminal artwork renderer delivering 2x vertical resolution in TUI.
  - Native embedded picture extraction from `FLAC`, `MP3`, `M4A`, and `OGG` tags using `lofty` with directory cover image scan (`cover.jpg`, `folder.jpg`, `artwork.jpg`).
  - High-speed direct thumbnail fetching for YouTube / YouTube Music videos & streams via YouTube CDN (`maxresdefault.jpg` / `hqdefault.jpg`).
  - Universal thumbnail extraction for online streaming sources (SoundCloud, Bandcamp, Spotify, Vimeo, Twitch, Bilibili) using `yt-dlp`.
  - Apple iTunes Search API (600x600 HD) fallback search.
- **🌈 Dynamic 60FPS Fluid RGB Chroma Spectrum Visualizer**:
  - Introduced `SpectrumColorMode` enum with 8 palettes (`rgb_cycle`, `chroma_rainbow`, `vertical_gradient`, `cyberpunk_neon`, `fire_and_ice`, `matrix_phosphor`, `amber_vintage`, `theme_accent`).
  - Animated 60 FPS fluid rainbow wave cycle running across 32 ISO EQ frequency bands.
  - Configurable via Settings modal (`o`) and persisted in `~/.config/boombox/config.toml`.
- **⚡ Hot-Reload Keybinding (`F5`)**:
  - Added dedicated `F5` hotkey and Unix signals (`SIGUSR1`, `SIGHUP`) to instantly reload the app, UI, and configuration without stopping audio.
- **🎯 Smart Synced Lyrics Search & Local Caching**:
  - Upgraded LRCLIB search flow to prioritize true `syncedLyrics` across full search results before falling back to plain lyrics.
  - Automated local `.lrc` file caching alongside audio files in `~/Music` for 0ms instant loading on future playback.
  - Fixed 3-digit millisecond timestamp decimal parsing and added standard `[offset:ms]` LRC tag support.
  - Interactive lyrics timing offset controls (`[` / `]` for ±0.25s, `{` / `}` for ±1.0s, `0` to reset).

---

## [3.2.0] - 2026-08-30

### 🚀 Added
- **Universal Stream Queue Expansion** (`u` → paste link, then `Ctrl+A` or `Enter`): Pasting a supported stream link (YouTube, YouTube Music, SoundCloud, BandLab, Qobuz, Deezer, Tidal, Bandcamp, Apple Music, ...) now resolves it via `yt-dlp --flat-playlist` into its real track list and populates the entire queue (deduplicated). Pressing `Enter` auto-plays the first expanded track; `Ctrl+A` just queues. Plain single-track / non-supported links keep the original one-track behavior.

### 🔄 Changed
- Renamed `src/api/youtube.rs` → `src/api/stream.rs` as a universal stream resolver (`api::stream`). `resolve_youtube_queue` → `resolve_stream_queue`, `enqueue_youtube_url` → `enqueue_stream_url`. Detect and label arbitrary yt-dlp-backed sources generically, so new domains (e.g. BandLab, Qobuz) need only a KNOWN_SOURCES entry.

---

## [3.1.0] - 2026-08-30

### 🚀 Added (Synced from BOOMBOX-TUI v2.4.1)
- **Tape Recorder / Stream Ripper** (`R` / `Ctrl+R`): Record current YouTube / SoundCloud / Bandcamp streams via `yt-dlp` and live radio streams via `ffmpeg` straight to `~/Music/Boombox Recordings`, with desktop notifications and instant cancel (`Ctrl+R`). Zero-touch safeguard preserves existing local Hi-Res files.
- **Configurable Recording Formats** (`[O] → f`): Choose `OPUS` native 0-loss, `MP3` 320k, `FLAC` lossless, or `M4A` 256k.
- **Mixtape & Custom Playlist Manager** (`M`): Create, manage, delete, and persist mixtapes to `~/.config/boombox-tui/mixtapes.json` across sessions. Add current track with `Enter`, add currently-playing with `a`, removes with `d`.
- **32-Band Explorer EQ Presets** (`E`): Full 7-preset ISO curve generator (`FLAT`, `MEGA_BASS`, `VOCAL_CLEAR`, `ROCK_PUNCH`, `LOFI_WARMTH`, `CYBER_SYNTH`, `CLUB_EDM`) with the live spectral visualizer now honoring the active gain curve.
- **Live Recording Status**: `[● REC]` badge in header, EQ/Rec status in the Phosphor LCD monitor, and recorder/format state in the Settings modal.

### 🔄 Changed
- Updated `EqPreset` from a 6-value DSP set to the full 7-preset 32-band ISO curve set matching BOOMBOX-TUI.

---

## [3.0.0] - 2026-08-30
- Initial Rust port of the BOOMBOX-TUI cassette deck music player and radio explorer.
