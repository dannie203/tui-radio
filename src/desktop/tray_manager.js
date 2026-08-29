import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { existsSync } from 'node:fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const SCRIPT_PATH = join(__dirname, 'mpris_tray.py');

export class TrayManager {
  constructor(store, actions, player) {
    this.store = store;
    this.actions = actions;
    this.player = player;
    this.process = null;
    this.lastNotifiedTrackId = null;
    this.active = false;
  }

  start() {
    if (!existsSync(SCRIPT_PATH)) return;

    try {
      this.process = spawn('python3', [SCRIPT_PATH], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      this.active = true;

      this.process.stdout.on('data', (chunk) => {
        const lines = chunk.toString().split('\n');
        for (const line of lines) {
          if (!line.trim()) continue;
          try {
            const msg = JSON.parse(line.trim());
            this.handleAction(msg.action, msg.data);
          } catch (e) {
            // Ignore parse errors from stderr/stdout
          }
        }
      });

      this.process.on('exit', () => {
        this.active = false;
        this.process = null;
      });

      this.process.on('error', () => {
        this.active = false;
        this.process = null;
      });

      // Subscribe to store updates to keep MPRIS/Tray state synchronized
      this.unsubscribe = this.store.subscribe((state) => {
        this.syncState(state);
      });

      // Initial state synchronization
      this.syncState(this.store.state);
    } catch (err) {
      this.active = false;
    }
  }

  syncState(state) {
    if (!this.process || !this.active || this.process.killed) return;

    const activeItem = state.current;
    const title = activeItem?.title || activeItem?.name || state.metadata || 'Nothing playing';
    const artist = activeItem?.artist || (state.mode === 'RADIO STATIONS' ? (activeItem?.name || 'Radio Stream') : 'BOOMBOX RX-505');
    const album = activeItem?.album || activeItem?.country || state.mode || '';

    const payload = {
      type: 'UPDATE',
      state: {
        title,
        artist,
        album,
        playing: Boolean(state.playing),
        paused: Boolean(state.paused),
        volume: state.volume ?? 80,
        duration: state.duration || 0,
        timePos: state.timePos || 0,
        stereoMode: state.stereoMode || 'STEREO',
        bassBoost: Boolean(state.bassBoost),
        dolbyMode: state.dolbyMode || 'DOLBY-B'
      }
    };

    try {
      this.process.stdin.write(JSON.stringify(payload) + '\n');
    } catch {}

    // Send desktop notification when a new track starts playing
    if (state.playing && activeItem && activeItem.id !== this.lastNotifiedTrackId) {
      this.lastNotifiedTrackId = activeItem.id;
      this.sendDesktopNotification(title, artist, album);
    }
  }

  sendDesktopNotification(title, artist, album) {
    try {
      const summary = `🎵 ${title}`;
      const body = `${artist}${album ? ` — ${album}` : ''}`;
      spawn('notify-send', [
        '-a', 'BOOMBOX RX-505',
        '-i', 'audio-player',
        '-h', 'string:category:music',
        summary,
        body
      ], { stdio: 'ignore' });
    } catch {}
  }

  handleAction(action, data) {
    if (!action) return;

    switch (action) {
      case 'play_pause':
        this.actions.togglePause?.();
        break;
      case 'play':
        if (this.store.state.paused) this.actions.togglePause?.();
        break;
      case 'pause':
        if (this.store.state.playing && !this.store.state.paused) this.actions.togglePause?.();
        break;
      case 'stop':
        this.actions.stop?.();
        break;
      case 'next':
        this.actions.next?.();
        break;
      case 'prev':
        this.actions.prev?.();
        break;
      case 'seek':
        if (data !== undefined) this.actions.seek?.(data);
        break;
      case 'volume_up':
        this.actions.volume?.(5);
        break;
      case 'volume_down':
        this.actions.volume?.(-5);
        break;
      case 'cycle_stereo':
        this.actions.cycleStereoMode?.(1);
        break;
      case 'toggle_bass':
        this.actions.toggleBassBoost?.();
        break;
      case 'open_tui':
        // Try to focus terminal window via hyprctl or notification
        try {
          spawn('hyprctl', ['dispatch', 'focuswindow', 'title:NEON//WAVE CYBERPUNK AUDIO TERMINAL'], { stdio: 'ignore' });
        } catch {}
        break;
      case 'quit':
        this.actions.quit?.();
        break;
    }
  }

  destroy() {
    this.unsubscribe?.();
    if (this.process) {
      try {
        this.process.stdin.write(JSON.stringify({ type: 'QUIT' }) + '\n');
        this.process.kill('SIGTERM');
      } catch {}
      this.process = null;
      this.active = false;
    }
  }
}
