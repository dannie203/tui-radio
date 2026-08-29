# 📜 Changelog

All notable changes to the **BOOMBOX-TUI** project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.4.1] - 2026-08-30

### 🩹 Fixed
- **UI Animation Loop Theme Scope**: Fixed a `TypeError: Cannot read properties of undefined (reading 'muted')` inside the 30 FPS animation timer by correctly updating the live theme object in scope.
- **Tray Stream EPIPE Guard**: Added standard error handlers to Python D-Bus child process IPC streams (`stdin`, `stdout`, `stderr`) to prevent crashes when the shell/tray process disconnects.
- **Process Robustness**: Added global `unhandledRejection` handler to ensure background network hiccups never terminate playback.

---

## [2.4.0] - 2026-08-30

### 🚀 Added
- **Linux / Omarchy Desktop Theme Auto-Sync (`SYSTEM_AUTO`)**: Automatic discovery and real-time palette synchronization with Omarchy desktop (`colors.toml`), Pywal, and system palettes.
- **Reactive TUI Re-styling Engine**: Instant, zero-flicker re-styling across all TUI panels, cassettes, visualizers, breadcrumbs, modals, and karaoke lyrics.
- **Configurable Recording Formats**: Choose recording export format (`OPUS` native 0-loss, `MP3` 320k, `FLAC` lossless, `M4A`) via settings `[O]`.
- **Instant Cancel & Process Termination for Tape Recorder**: One-touch abort & temporary file cleanup (`Shift+R` / `Ctrl+R`) to instantly stop accidental recordings.
- **Zero-Touch Local File Safeguard**: Strict read-only protection ensuring local Hi-Res FLAC/WAV audio is preserved without re-encoding.
- **Realistic Tape Pack Progress Winding**: Dual spool tape pack dynamically expands and contracts to match playback percentage.

---

## [2.3.0] - 2026-08-30

### 🚀 Added
- **Linux Theme Engine**: Added 7 popular Linux desktop palettes with instant dynamic switching (`AMBER_GOLD`, `CATPPUCCIN_MOCHA`, `TOKYO_NIGHT`, `GRUVBOX_RETRO`, `NORD_FROST`, `DRACULA`, `MATRIX_GREEN`). Shortcut: `T` / `Ctrl+T`.
- **32-Band Equalizer Sound Presets**: Added 1-touch sound profiles (`FLAT`, `MEGA_BASS`, `VOCAL_CLEAR`, `ROCK_PUNCH`, `LOFI_WARMTH`, `CYBER_SYNTH`, `CLUB_EDM`). Shortcut: `E`.
- **Tape Recorder / Stream Ripper**: Record live YouTube, SoundCloud, Bandcamp, and radio streams directly to `~/Music/Boombox Recordings` with full tags and desktop notifications. Shortcut: `R` / `Ctrl+R`.
- **Mixtape & Custom Playlist Manager**: Create, manage, and save personal mixtapes across local and online tracks. Shortcut: `M` / `Ctrl+M`.
- **Official Arch Linux PKGBUILD**: Added `PKGBUILD` for AUR packaging and 1-click installation.

---

## [2.2.1] - 2026-08-30

### 🚀 Added
- **Horizontal Sliding Carousel for Filter Tabs**: Implemented dynamic `renderCarouselTabs` sliding window engine. When cycling through genres (or long tab lists), the active item is always kept visible and centered with styled overflow indicators (`◀` / `▶`) that adapt seamlessly to any terminal width.

---

## [2.2.0] - 2026-08-30

### 🚀 Added
- **Worldwide International Radio Gathering**: Upgraded Radio Browser API integration to query diverse multi-genre categories across the globe (Lofi, Synthwave, Jazz, Hip-Hop, Rock, Electronic, Classical, Pop, Vietnam, Japan, Global Top Voted).
- **International Genre & Country Filters**: Expanded Radio deck filters to include `LO-FI`, `SYNTHWAVE`, `JAZZ`, `HIP-HOP`, `ROCK`, `ELECTRONIC`, `CLASSICAL`, `POP`, `VIETNAM`, `JAPAN`, `GLOBAL TOP`.
- **Multi-Server Resilient Fetching**: Added auto-balancing across `all.api.radio-browser.info`, `de1`, `nl1`, and `at1` mirrors.

---

## [2.1.0] - 2026-08-29

### 🚀 Added
- **In-App Detach & Minimize to Tray**: Added `Ctrl+D`, `Ctrl+H`, and `H` keyboard shortcuts to seamlessly detach / minimize the player to the system tray while music keeps playing in the background.
- **Tray AppMenu Minimize Action**: Added `📦 Hide / Minimize to Tray` option to the D-Bus StatusNotifierItem AppMenu.
- **Desktop Minimize Notification**: Dispatched desktop notification when the player is minimized to background.
- **Help Bar Update**: Added `[^D / H] Detach / Hide` to the bottom TUI control hints.

---

## [2.0.0] - 2026-08-29

### 🚀 Added
- **Complete Rebranding**: Renamed project to `boombox-tui` (`BOOMBOX RX-505 Retro Audio Player`).
- **Full CLI Command Suite**: Provided `boombox`, `boombox-tui`, `boombox-toggle`, `radio`, `hiphop-radio`, `hiphop-radio-toggle`.
- **High-Resolution Vector Logo**: Created `assets/icons/hicolor/scalable/apps/boombox.svg` with 8 standard PNG resolutions (16x16 to 512x512).
- **Dynamic Tray Icons**: Added `boombox-tray.svg`, `boombox-tray-playing.svg` (green wave glow), and `boombox-tray-paused.svg` (amber pause state).
- **FreeDesktop Desktop Entry**: Added `assets/boombox.desktop` with quick actions (*Focus*, *Play/Pause*, *Next*, *Previous*).
- **Hyprland 0.56+ (Omarchy Quattro) Support**: Added native Lua dispatchers (`hl.dsp.focus` / `hl.dsp.window.move`) with smart scratchpad and workspace restoration.
- **Developer Guidelines**: Added `AGENTS.md` enforcing automatic SemVer version increments (`x.y.z`) and changelog tracking for all future modifications.

### 🔄 Changed
- **Tray Activation Logic**: Clicking the Tray Icon or AppMenu now strictly runs in `focus` mode to restore/bring the app window to the front without accidental scratchpad minimization.
- **D-Bus SNI Registration**: Standardized `RegisterStatusNotifierItem` to object path `/StatusNotifierItem` with `IconThemePath` support.
- **Dependency Cleanup**: Removed unused `blessed-contrib` and pruned unused imports.

---

## [1.0.0] - 2026-08-28

### 🎵 Initial Release
- **Cassette Deck UI**: Dual spools animation, smoked tape window, ANSI half-block album art decoder.
- **Local Audio Engine**: High-speed FLAC, MP3, OPUS, OGG, WAV tag parser and library explorer.
- **Radio Explorer**: Radio Browser API integration with genre and country filtering.
- **YouTube Music**: In-TUI music searching and stream resolver.
- **Hardware DSP**: 3D WIDE matrix, Mega Bass (+7dB), Dolby NR tape bias simulation (B/C/S).
- **32-Band Visualizer**: 32 ISO center frequencies spectrum analyzer with ballistic physics.
- **LRCLIB Karaoke**: Word-by-word synced lyrics and Matrix-style decoding.
- **MPRIS2 & Tray**: FreeDesktop StatusNotifierItem and DBusMenu AppMenu.
