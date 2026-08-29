# BOOMBOX RX-505

A retro-inspired terminal music player for local libraries, radio stations, and online media sources. It combines a keyboard-driven interface with MPV-powered playback for a lightweight desktop-like experience in the shell.

## Overview

This project is built for users who want to listen to music without leaving the terminal. It supports local media browsing, internet radio, queue control, basic metadata handling, and a few desktop integrations where available.

The interface is intentionally styled like a cassette deck, but the project remains practical: fast to launch, easy to control, and built around a simple local-first workflow.

## Key features

- Local music playback and library browsing
- Internet radio station support
- YouTube / YouTube Music stream loading
- Queue, repeat, shuffle, and playback controls
- Keyboard-first terminal interface
- Lyrics support using LRCLIB and local `.lrc` files
- Tray and MPRIS integration on compatible Linux setups
- Basic visualizer and themed UI elements

## Requirements

- Node.js 20+
- MPV installed and available in `PATH`
- Optional: `yt-dlp` for YouTube playback
- Optional: Python GObject for tray/MPRIS support on Linux

## Installation

```bash
git clone https://github.com/dannie203/tui-radio.git
cd tui-radio
npm install
```

## Run the app

Start the TUI:

```bash
npm start
```

Run in tray/daemon mode:

```bash
node bin/index.js --tray
```

Or install globally:

```bash
npm link
hiphop-radio
```

## Common controls

The app is designed primarily for keyboard use.

- Arrow keys or `j` / `k` to move through items
- Enter / Right Arrow to open or play
- Left Arrow / Backspace to go back
- Space to pause / resume
- `N` / `P` for next / previous
- `+` / `-` to adjust volume
- Tab / `M` to switch deck mode
- `/` to open search
- `L` to toggle lyrics
- `W` to toggle artwork
- `Q` to quit

## Configuration

The app stores user preferences in the config directory. On Linux this is commonly:

```bash
~/.config/hiphop-tui/config.json
```

## Project structure

```text
bin/
  index.js
src/
  api/
  audio/
  desktop/
  state/
  ui/
test/
```

Main areas:

- `bin/index.js`: app entry point
- `src/audio/`: playback, library, visuals, metadata
- `src/api/`: stations, lyrics, YouTube integration
- `src/state/`: configuration and app state
- `src/ui/`: terminal layout and rendering
- `test/`: automated tests

## Testing

```bash
npm test
```

## License

This project is licensed under the GNU GPL v3.0 or later. See [LICENSE](LICENSE) for details.

## Notes

This is a terminal-first music client intended for Linux and macOS users who prefer a lightweight, keyboard-centric workflow. It depends on external playback tooling such as MPV and may optionally use `yt-dlp` and desktop integrations for a fuller experience.
