# Changelog

All notable changes to the **BOOMBOX-RS** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.8.11] - 2026-09-13

### Fixed
- **Tokio Runtime block_on Panic on Linux**: Resolved nested runtime panic when triggering desktop notifications via `notify-rust` inside Tokio worker threads. Replaced blocking calls with non-blocking `show_async()` on Linux and safe off-thread blocking tasks on Windows/macOS.
- **TUI Garbled Screen Recovery**: Added `Ctrl + L` shortcut to force an immediate full terminal clear and re-render.
- **Terminal State Protection on Panic**: Installed global panic hook to ensure terminal raw mode and alternate screen are properly restored if any background or main thread panics.

## [3.8.10] - 2026-09-13

### Fixed
- **Windows Double-Keystroke & Release Glitch**: Filtered out `KeyEventKind::Release` events in `main.rs`, eliminating phantom double-stepping on Windows Console and Windows Terminal.
- **Smooth Windowed List Scrolling**: Implemented sliding-window scroll calculation with persistent `scroll_offset` in `AppState` across Browser and History lists. Cursors now move freely within the viewport without triggering jarring full-screen list shifts on every keystroke.
- **Web Pipeline Installer Null Path**: Fixed `Split-Path` parameter binding failure in `install.ps1` when executed via `irm ... | iex`.

## [3.8.9] - 2026-09-12

### Added
- **Automated Windows Dependency Provisioning**: Completely revamped `install.ps1` and added 1-click `install.bat` launcher. Automatically checks and installs Microsoft Visual C++ 2015-2022 Redistributable (x64) if `vcruntime140.dll` is missing, auto-provisions MPV Audio Engine (via `winget` or standalone portable archive), and deploys standalone `yt-dlp.exe` for instant zero-configuration streaming.
- **Automated Linux Package Manager Integration**: Enhanced `install.sh` to auto-detect Linux distribution package managers (`pacman`, `apt`, `dnf`, `zypper`, `apk`, `xbps`) to automatically install `mpv` with interactive TTY support, alongside automatic zero-root standalone `yt-dlp` download to `~/.local/bin`.
- **Local Portable Binary Resolution (`resolve_executable`)**: Implemented dynamic helper discovery across `player.rs`, `stream.rs`, `recorder.rs`, and `artwork.rs`. Boombox now prioritizes `mpv.exe`, `yt-dlp.exe`, and `ffmpeg.exe` placed alongside the binary before falling back to system `PATH`, enabling true plug-and-play portable execution.
- **Cross-Platform Toast Notifications**: Replaced residual Linux-specific `notify-send` subprocess commands in `main.rs` with native cross-platform `notify-rust` toast notifications.

### Changed
- **Windows Terminal & UTF-8 Integration**: Updated Windows batch launchers (`RUN-BOOMBOX.bat`) to enforce `chcp 65001` UTF-8 code page and auto-detect Windows Terminal (`wt.exe`) for pristine retro cyberpunk TrueColor palette rendering.

## [3.8.8] - 2026-09-07

### Added
- **Native Windows WASAPI Loopback Capture**: Integrated direct render loopback audio capture on Windows using the `wasapi` crate, eliminating the need for Stereo Mix or virtual audio cables.
- **Cross-Platform IPC Abstraction**: Replaced raw Unix domain sockets with `interprocess` streams, supporting Unix domain sockets on Linux/macOS and Named Pipes (`\\.\pipe\boombox-rs-mpv`) on Windows.
- **Cross-Platform Notifications**: Replaced external `notify-send` shell command with native `notify-rust` notification library for cross-platform desktop notifications.

### Fixed
- **Direct Sink Monitor Audio Capture on Linux**: Configured `pw-record` with `-P stream.capture.sink=true` (and `parec` fallback with `-d @DEFAULT_MONITOR@`) to capture directly from the active audio sink monitor (speakers/headphones). This resolves the issue where disabling virtual audio sharing in EasyEffects defaulted capture to the microphone input and avoids leaking desktop audio into voice calls (Discord/Vesktop).
- **Subprocess Lifetime Hygiene**: Configured `libc::PR_SET_PDEATHSIG` on spawned recording child processes (`pw-record`/`parec`), ensuring child recording streams terminate immediately if Boombox exits.
- **Cross-Platform Compilation Scopes**: Scoped Linux-specific dependencies (`ksni` D-Bus system tray, `libc::prctl`) to `target_os = "linux"`, providing clean fallback stubs on Windows and macOS.

## [3.8.7] - 2026-09-06

### Fixed
- **Stream URL Security Hardening**: `resolve_stream_item()` now rejects untrusted schemes (`file://`, `gopher://`, `smb://`, `dict://`, etc.) before any item enters the playback queue, and `is_search_query()` refuses any payload containing `://`. Combined with the existing SSRF block against cloud metadata endpoints, external stream resolution is now safe by construction.
- **Subprocess Zombie Prevention**: `MpvPlayer` now tracks its child process through an `Arc<Mutex<Option<Child>>>`; the previous process is killed and fully reaped before every IPC reconnection, eliminating leaked zombie processes during `mpv` restart cycles.
- **History Metadata At Rest**: `HistoryEntry` now persists `format`, `bitrate`, `sample_rate`, and `bit_depth` with `#[serde(default)]` back-compatibility, so `history_to_media_item()` returns full metadata in O(1) without any blocking disk probe on the main event thread.
- **Accurate Decoder Bit Depth**: Replaced the fragile `contains()` heuristic in `audio-params` parsing with an exhaustive format-match table (`u8`→8, `s16`→16, `s24`→24, `s32`→32, `double`→64), keeping a safe fallback for extended encodings.
- **Precise PipeWire Sample Rate**: Clock-rate detection now queries `pw-metadata -n settings 0 clock.rate` directly (sub-10ms, deterministic) instead of parsing the verbose `pw-dump` JSON blob (`pw-dump` retained only as a fallback).
- **Stream Metadata Integrity**: Spawned stream `MediaItem`s no longer fake `48kHz/16-bit`; the correct decoder parameters arrive live from `mpv` IPC via `audio-params`.
- **Hardcoded Path Removal**: The fallback radio list is now located relative to the executable and the config directory, and the tray icon fallback resolves `$HOME` dynamically instead of assuming a hardcoded username.

### Changed
- Introduced a shared `strip_stream_prefixes()` helper to eliminate duplicated URL-normalization logic across stream resolution paths.
- Static `.lrc` parsing regexes are compiled once via `std::sync::LazyLock` instead of per-call.
- Clarified the Mixtape removal status message to `"Removed last track '{}' from Mixtape"` when the final track is deleted.
- Restructured the crate into a library + binary layout (`src/lib.rs`) exposing `api`, `audio`, `state`, and `ui` modules, enabling future unit-testability and reuse.

### Added
- GitHub Actions CI workflow (`ubuntu-latest`, `windows-latest`) running `cargo check`, `cargo test`, and release builds on push and pull requests.
- Convenience cargo aliases (`cargo ci`, `cargo lint`) for fast local verification.

---

## [3.8.6] - 2026-08-31

### Fixed
- **Boombox Toggle Window Behavior**: Rewrote `boombox-toggle` so `SUPER + M` (and the launcher icon) no longer impossible to trust. Previously it could spawn duplicate instances or leave the boombox fullscreen / floating, which covered the adjacent window instead of splitting beside it. The toggle now:
  - Matches the real boombox window precisely (app-id `org.omarchy.boombox`, exact title `BOOMBOX RX-505`, or the parent terminal of a live process) so unrelated windows can never hijack it.
  - Never spawns a duplicate instance when one is already running.
  - Brings the boombox onto the current workspace as a normal tiled window — exiting fullscreen and floating as needed — so it splits beside whatever is already open instead of covering it. Hides to the scratchpad only when it is an ordinary tiled window in view.

---

## [3.8.5] - 2026-08-31

### Changed
- **Cassette Spool Animation Speed**: Calibrated cassette tape spool rotation to a realistic vintage speed (~90 RPM / 6 FPS tick) for a relaxing and smooth visual experience.

---

## [3.8.4] - 2026-08-31

### Added
- **In-App Background Update Notification**: Automatic non-blocking GitHub release checker with header brand badges, real-time statusline alerts, desktop notifications, and Settings dashboard integration.

---

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
