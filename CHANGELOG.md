# Changelog

All notable changes to the **BOOMBOX-RS** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.8.3] - 2026-08-31

### Fixed
- **Terminal Safety & Teardown**: Guaranteed raw mode disable and screen restore via RAII drop guard across all error returns and panics.
- **Async Concurrency**: Removed synchronous mutex lock held across `.await` points in tape recorder to avoid async thread contention.
- **MPV IPC Self-Healing**: Added automatic background process re-spawning and IPC socket reconnection when MPV exits or drops connection.
- **List Viewport Scrolling**: Fixed scroll offset calculation in library and radio browsers for seamless auto-scrolling on large libraries.
- **SSRF & Arg Injection Protection**: Hardened external `yt-dlp` tool invocations with strict `--` parameter delimiters.
- **Decoupled Radio Genre Filtering**: Isolated radio genre switching from history queries to avoid inadvertent history list changes.
- **Portability**: Replaced machine-specific hardcoded script path with dynamic `$HOME` lookup.

### Added
- **Live Broadcast Telemetry**: Activated `[● LIVE]` status LED on Cassette Deck and dynamic live broadcast indicator on CRT Phosphor Monitor.

---

## [3.8.2] - 2026-08-30

### Added
- Streaming Autoplay engine powered by YouTube Algorithmic Radio Mix (`RD<VIDEO_ID>`) with background zero-delay prefetching.
- Local privacy-first, zero-tracking playback history (`~/.config/boombox-tui/history.json`) with interactive modal (`Shift+H`).
- Smart Upsert deduplication: tracks are recorded uniquely, refreshing `last_played_at` and incrementing `play_count`.
- Dedicated Streaming Autoplay toggle in settings dashboard (`o`).

### Changed
- Clarified single-responsibility search architecture: `/` is dedicated exclusively to instant in-memory live filtering, while `u` handles universal external online streaming and search.

---

## [3.8.1] - 2026-08-30

### Added
- Universal online keyword search across YouTube, SoundCloud, and Spotify with platform prefixes (`yt:`, `sc:`, `sp:`).
- Deep search filtering in local crates with automatic flat-view switching and full track/album metadata matching.
- Process identity and thread naming for performance monitors (`btop`, `htop`, `ps`) via Linux `prctl(PR_SET_NAME)`.
- Middle-click play/pause and mouse wheel volume scroll controls on system tray icon.

### Fixed
- Delayed tray icon removal by explicitly unregistering StatusNotifierItem via D-Bus on application teardown.
- Fallback placeholder icons in desktop tray menu by binding local hicolor theme paths.
- Search filter persistence issue when clearing input or pressing Escape.

---

## [3.8.0] - 2026-08-30

### Added
- Real-time 2D dual-beam CRT phosphor oscilloscope waveform renderer with coordinate reticle grid and hardware telemetry.
- Dynamic responsive layout auto-scaling for right pane based on terminal viewport height and width.
- Windows cross-compilation support targeting `x86_64-pc-windows-msvc` using named pipe IPC communication.
- Bidirectional navigation support for equalizer presets and hardware audio modes.

### Changed
- Phosphor monitor expanded dynamically from a fixed 8-line height to viewport-proportional sizing.
- Progress bar and audio telemetry labels scale horizontally up to 80 columns on wide terminals.
- Equalizer spectrum bars and dual VU meters scale vertically to fill available viewport space without bottom margin gaps.

### Fixed
- Line clipping issue where system status and recording indicators were truncated on compact terminal viewports.
- Cross-platform build failures caused by Unix-specific signal handlers and file locking mechanisms.

---

## [3.7.0] - 2026-08-30

### Added
- Single-deck MPV IPC controller with native gapless playback and automated error recovery.
- Classic retro RX-505 single-bay cassette deck simulation with animated spools and transport indicators.

### Changed
- Streamlined codebase into a 100% pure Rust audio engine without external Python runtime dependencies.
- Reduced memory footprint to ~20MB and optimized startup time.

### Removed
- Experimental neural engine, ONNX model dependencies, and Python virtual environment scripts.

---

## [3.3.0] - 2026-08-29

### Added
- 24-bit TrueColor ANSI half-block cover art renderer supporting embedded tags, web thumbnails, and online CDN sources.
- 32-Band ISO Equalizer spectrum analyzer with real-time FFT ballistics and 8 color palettes.
- Synchronized karaoke lyrics via LRCLIB with local `.lrc` caching, sub-millisecond offset calibration, and cipher mode.
- Dedicated hot-reload keybinding (`F5`) and Unix signal handling (`SIGUSR1`, `SIGHUP`).

---

## [3.2.0] - 2026-08-28

### Added
- Universal stream resolution for YouTube, YouTube Music, SoundCloud, Bandcamp, and direct media URLs.
- Automatic playlist expansion into the playback queue using `yt-dlp`.

---

## [3.1.0] - 2026-08-27

### Added
- Stream recording engine supporting OPUS, MP3, FLAC, and M4A formats.
- Mixtape custom playlist manager with persistent JSON storage.
- 7-curve 32-band ISO Equalizer preset profiles (Flat, Mega Bass, Vocal, Rock, Lo-Fi, Synth, EDM).

---

## [3.0.0] - 2026-08-25

### Added
- Initial Rust release of Boombox TUI audio player and radio explorer.
- Terminal user interface powered by Ratatui and Crossterm.
- Local library scanner for FLAC, MP3, WAV, OPUS, AAC, and OGG audio files.
- Curated worldwide internet radio station browser.
