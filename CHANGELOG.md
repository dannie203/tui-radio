# 📜 Changelog (Boombox-RS)

All notable changes to the **BOOMBOX-RS** Rust port will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.8.0] - 2026-08-30

### Added
- Real-time 2D dual-beam CRT phosphor oscilloscope waveform renderer with reticle grid and hardware telemetry.
- Dynamic responsive layout auto-scaling for right pane based on terminal viewport height and width.
- Full vertical space filling for 32-band ISO equalizer spectrum and stereo VU meters.
- Bidirectional navigation support for audio presets and hardware modes.

### Changed
- Phosphor monitor expanded dynamically from fixed height to viewport-proportional sizing.
- Progress bar and audio telemetry labels now scale horizontally up to 80 columns on wide viewports.

### Fixed
- Line clipping issue where system status and recording indicators were hidden on compact viewports.
- Unrendered vertical blank space at the bottom of the visualizer on high-resolution displays.

---

## [3.7.0] - 2026-08-30

### 🚀 Pure-Rust Lightweight Hi-Fi Refactor
- **🦀 100% Pure-Rust Engine**:
  - Completely purged experimental Neural Engine, Python venv, ONNX weights, and DJ automix complexity.
  - Reduced background memory footprint and achieved instantaneous 0.01s boot time.
- **📼 Restored Classic Retro RX-505 Cassette Deck**:
  - Rebuilt the iconic single-bay cassette mechanism with spinning spools, Dolby B/C indicators, Tape bias formula, and transport LEDs.
  - Restored clean 7-line Phosphor LCD Monitor layout.
- **⚡ Single-Deck Rock-Solid MPV Controller**:
  - Streamlined IPC control with automatic reconnection, native gapless audio playback, and pure random shuffle.

---

## [3.6.0] - 2026-08-30

### 🚀 Added & Improved
- **🧠 Full Neural Engine & Apple Music-Style Automix**:
  - Implemented 4 PyTorch Neural Models (`BNEBeatTracker`, `BNECueDetector`, `BNEKeyClassifier`, `BNEDjTransitionPolicy`) with dynamic-axes ONNX Runtime GPU acceleration (`CUDAExecutionProvider` on NVIDIA RTX GPU).
  - 8 selectable Automix / Transition modes (`Shift+X` cycle or Settings `o`): `NeuralAuto`, `NeuralBassSwap`, `NeuralEchoOut`, `NeuralFilterSweep`, `EqualPower`, `SmoothExponential`, `LinearRamp`, `Disabled`.
  - Real-time DSP audio filter modulation in Rust via MPV IPC (`highpass`, `lowpass`, `aecho`, dynamic `speed` tempo stretching).
  - **Bass-Swap**: Rolloff low-end on Deck A and snap 100% punchy bass on Deck B exactly at midpoint downbeat with zero muddy phase cancellation.
  - **Echo-Out Drop**: 1/2 beat reverb tail on outgoing track for large BPM transitions.
- **🔀 AI-DJ Smart Harmonic Shuffle with Anti-Loop History**:
  - Integrated 60-track `played_history` ring buffer preventing 2-track mutual looping.
  - Smart probabilistic Top-K sampling across entire library pool based on Camelot Wheel harmonic compatibility and tempo proximity.
- **🛑 Clean Tray Shutdown & Outro Safeguard**:
  - Fixed Tray menu Quit action to instantly tear down terminal, stop MPV, and exit cleanly without hanging on async tokio tasks.
  - Constrained outro detection window strictly to the final 88%-95% of tracks, preventing premature halfway crossfades.

---

## [3.5.0] - 2026-08-30

### ✨ Improved
- **🎛️ Auto-Mix now uses real neural cues instead of hardcoded guesses**:
  - `analyze_track.py` derives `mix_in_sec` from the Cue Detector's section-class output (first frame that leaves the Intro) and `mix_out_sec` from the start of the Outro / last structural boundary — falling back gracefully when the model is flat. No more fixed `min(15, dur*0.1)` / `dur-16` heuristics. Cache rescanned for all 105 tracks.
- **🥁 Beat-synced crossfade**:
  - The crossfade length is now rounded to a whole number of beats at the outgoing track's BPM, so the fade lands on the beat instead of drifting mid-measure.
  - The incoming track's mix-in cue is snapped to the nearest beat boundary, so the next song comes in *on the beat* rather than jarringly mid-phrase.
- **🧠 Harmonic + BPM-aware track selection**:
  - In Auto-Mix with Smart Cues on, the AI DJ now picks the next track from the pool by scoring Camelot-key harmonic compatibility (`is_harmonic_match`, previously unused) plus BPM proximity — closest tempo that keeps the key in a compatible position. Falls back to queue order otherwise.
- **🔧 Fix mid-song seek-play race**:
  - `SingleDeck::play` now waits briefly for the demuxer after `loadfile` before issuing the cue `seek`, so the incoming deck reliably lands at the intended beat cue instead of a misplaced position.

---

### 🚀 Added
- **🧠 In-App Neural Library Scan (`N`)**:
  - Press `N` to launch the BNE Python pipeline (`neural/scan_library.py` via the bundled `.venv`) in the background. It analyzes every file in `~/Music` through the trained ONNX models (Beat Tracker, Cue Detector, Key Classifier) and writes `~/.config/boombox/neural_profiles.json`.
  - `NeuralEngine` gained `reload()` + `profile_count()`; the on-disk cache is re-read automatically when the scan completes so the AI DJ immediately picks up new BPM / Camelot key / energy / mix cues without restarting.
  - Status bar reflects live progress ("Scanning library...") and a summary when done ("N tracks analyzed & cached"), with a guard so only one scan runs at a time.

---

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
