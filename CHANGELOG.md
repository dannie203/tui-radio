# 📜 Changelog

All notable changes to the **BOOMBOX-TUI** project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
