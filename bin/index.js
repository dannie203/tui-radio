#!/usr/bin/env node
import { Store } from '../src/state/store.js';
import { fetchStations } from '../src/api/stations.js';
import { resolveMedia, searchMusic } from '../src/api/youtube.js';
import { fetchLyrics, cleanTitleAndArtist } from '../src/api/lyrics.js';
import { scanDirectory } from '../src/audio/library.js';
import { MpvPlayer } from '../src/audio/player.js';
import { createLayout } from '../src/ui/layout.js';
import { TrayManager } from '../src/desktop/tray_manager.js';
import { clearPidFile, isProcessAlive, readPidFile, writePidFile } from '../src/desktop/instance_manager.js';

const PID_FILE = '/tmp/hiphop-tui.pid';

async function lockInstance() {
  const isDaemonMode = process.argv.includes('--daemon') || process.argv.includes('--tray') || process.argv.includes('--minimized') || process.argv.includes('-d');
  const existingPid = readPidFile(PID_FILE);

  if (existingPid && existingPid !== process.pid && isProcessAlive(existingPid)) {
    if (isDaemonMode) {
      process.exit(0);
    }
    try {
      process.kill(existingPid, 'SIGTERM');
      await new Promise((r) => setTimeout(r, 200));
    } catch {}
  }

  writePidFile(PID_FILE, process.pid);
}

const store = new Store();
const player = new MpvPlayer();
let layout;
let tray;
let shuttingDown = false;
let currentLyricsController = null;
let currentLyricsReqId = 0;
let lastRadioLyricsKey = '';

function setStatus(status) {
  store.update({ status });
}

async function shutdown(exitCode = 0) {
  if (shuttingDown) return;
  shuttingDown = true;
  currentLyricsController?.abort();
  currentLyricsController = null;
  try { await store.saveSession(); } catch {}
  tray?.destroy();
  if (layout?.animTimer) clearInterval(layout.animTimer);
  layout?.screen?.destroy();
  clearPidFile(PID_FILE);
  try { await player.close(); }
  finally { process.exit(exitCode); }
}

function restoreUi() {
  if (layout) {
    try {
      layout.screen?.show();
      layout.screen?.focus();
      layout.screen?.render();
    } catch {}
    return;
  }

  try {
    layout = createLayout(store, actions, player);
    if (layout?.screen) {
      layout.screen.show();
      layout.screen.focus();
      layout.screen.render();
    }
  } catch (error) {
    setStatus(`UI restore failed: ${error.message}`);
  }
}

function detachToBackground() {
  if (layout) {
    if (layout.animTimer) clearInterval(layout.animTimer);
    try {
      layout.screen?.program?.input?.pause();
      layout.screen?.destroy();
    } catch {}
    layout = null;
  }
  process.stdout.on('error', () => {});
  process.stderr.on('error', () => {});
  try { process.stdin.pause(); } catch {}

  store.saveSession();
  tray?.sendDesktopNotification(
    'Minimized to System Tray',
    store.state.current?.title || 'BOOMBOX RX-505',
    'Music continues in background (Controlled via Tray & Media Keys)'
  );
}

process.on('exit', () => { if (shuttingDown) player.close(); });
process.on('SIGINT', () => { shutdown(0); }); // Ctrl+C explicitly shuts down
process.on('SIGTERM', () => { shutdown(0); });
process.on('SIGUSR1', () => {
  if (!layout) {
    restoreUi();
    return;
  }

  try {
    layout.screen?.show();
    layout.screen?.focus();
    layout.screen?.render();
  } catch {}
});
process.on('SIGHUP', () => {
  // Super+W / Window Close sends SIGHUP: detach and continue playing in background!
  detachToBackground();
});
process.on('uncaughtException', (error) => {
  console.error(error);
  shutdown(1);
});

async function main() {
  await lockInstance();
  await store.initSettings();
  await store.loadFavorites();

  // Apply loaded visualizer & DSP settings to player
  player.updateBallistics({
    peakHoldMs: store.config.visualizer?.peakHoldMs,
    peakDecayRate: store.config.visualizer?.peakDecayRate
  });
  player.applyDsp({
    stereoMode: store.state.stereoMode,
    dolbyMode: store.state.dolbyMode,
    tapeType: store.state.tapeType,
    bassBoost: store.state.bassBoost
  });

  // Scan local hierarchical music library with live progress feedback
  if (store.config.library?.scanOnStartup !== false) {
    setStatus('Scanning music library and audio metadata...');
    const { library, tracks } = await scanDirectory(store.state.musicDir, (progress) => {
      if (progress.phase === 'parsing') {
        setStatus(`Scanning tags: ${progress.done}/${progress.total} audio files`);
      }
    });

    store.setLibrary(library, tracks);
    if (tracks.length === 0) {
      store.setMode('RADIO STATIONS');
    } else {
      setStatus(`Loaded ${tracks.length} tracks across ${Object.keys(library.artists).length} artists in Crates`);
    }
  } else {
    store.setMode('RADIO STATIONS');
  }

  // Fetch online radio stations in background
  fetchStations().then((result) => {
    store.setStations(result.stations, result.source);
    if (store.state.mode === 'RADIO STATIONS') {
      setStatus(`${result.stations.length} radio stations // ${result.source}`);
    }
  }).catch((err) => {
    setStatus(`Radio offline: ${err.message}`);
  });

  try {
    const startRes = await player.start();
    if (startRes?.connectedToExisting) {
      store.update({ playing: player.state === 'playing' || player.state === 'buffering', paused: player.state === 'paused' });
    }
  } catch (error) {
    setStatus(error.message);
  }

  // Restore previous session (last track, queue, mode, volume, DSP state)
  await store.loadSession();

  function triggerLyricsFetch(item, { isRadio = false } = {}) {
    if (!item) return;
    currentLyricsController?.abort();
    currentLyricsController = null;
    const reqId = ++currentLyricsReqId;
    const controller = new AbortController();
    currentLyricsController = controller;

    store.clearLyrics();
    store.setLyricsStatus('loading', item.id);

    const artist = item.artist || '';
    const title = item.title || item.name || '';
    const album = item.album || '';
    const duration = item.duration || 0;
    const path = item.path || '';

    fetchLyrics({
      artist,
      title,
      album,
      duration,
      path,
      signal: controller.signal
    }).then((result) => {
      if (reqId !== currentLyricsReqId) return; // Discard stale response
      if (result.synced?.length > 0 || result.plain) {
        store.setLyrics(item.id, result);
        if (!isRadio) {
          store.update({ status: `Lyrics found [${result.isSynced ? 'SYNCED' : 'PLAIN'}]` });
        }
      } else {
        store.setLyricsStatus('unavailable', item.id);
      }
    }).catch((err) => {
      if (reqId !== currentLyricsReqId) return;
      if (err.name !== 'AbortError') {
        store.setLyricsStatus('error', item.id);
      }
    });
  }

  const actions = {
    move: (delta) => store.moveSelection(delta),
    drillDown: () => {
      const result = store.drillDown();
      if (result?.action === 'play' && result.track) {
        actions.play(result.track);
      }
    },
    drillUp: () => store.drillUp(),
    setNavLevel: (level) => store.setNavLevel(level),
    cycleNavLevel: (delta) => store.cycleNavLevel(delta),
    play: (itemToPlay) => {
      const selected = store.selected();
      const item = itemToPlay || (selected?.raw || selected);
      if (!item) return;

      // If user pressed play on an artist or album in LOCAL TRACKS, drill down
      if (item.type === 'artist' || item.type === 'album' || item.type === 'playlist') {
        actions.drillDown();
        return;
      }

      // Sync queueIndex if item is in Queue
      if (store.state.queue.length > 0) {
        const qIdx = store.state.queue.findIndex((t) => t.id === item.id || (t.url && t.url === item.url));
        if (qIdx !== -1) {
          store.state.queueIndex = qIdx;
        }
      }

      lastRadioLyricsKey = '';
      player.play(item.url || item.path, item);
      triggerLyricsFetch(item);
      const isLocal = item.type === 'local';
      const label = item.title || item.name || 'Unknown Track';
      const meta = isLocal
        ? `${item.artist || 'Unknown Artist'} - ${item.title || item.name}`
        : item.type === 'youtube'
        ? `${item.artist || 'YouTube'} - ${item.title || item.name}`
        : 'Connecting to live stream...';
      store.update({
        current: item,
        playing: true,
        paused: false,
        metadata: meta,
        status: isLocal ? `Playing "${label}"` : `Streaming "${label}"`
      });
    },
    togglePause: () => {
      if (!store.state.current) {
        actions.play();
        return;
      }
      if (player.state === 'idle' || !player.currentItem) {
        actions.play(store.state.current);
        return;
      }
      player.togglePause();
      const willBePaused = store.state.playing && !store.state.paused;
      store.update({
        status: willBePaused ? 'Paused' : 'Resumed'
      });
    },
    stop: () => {
      player.stop();
      store.update({
        playing: false,
        paused: false,
        status: 'Playback stopped'
      });
    },
    next: () => {
      const nextItem = store.getNextItem();
      if (nextItem) actions.play(nextItem);
      else store.update({ status: 'End of playlist / queue' });
    },
    prev: () => {
      const prevItem = store.getPrevItem();
      if (prevItem) actions.play(prevItem);
    },
    seek: (seconds) => {
      if (store.state.current?.type === 'local' || store.state.current?.type === 'youtube' || store.state.duration > 0) {
        player.seek(seconds, 'relative');
        store.update({ status: `Seek ${seconds > 0 ? '+' : ''}${seconds}s` });
      }
    },
    toggleShuffle: () => store.toggleShuffle(),
    toggleRepeat: () => store.toggleRepeat(),
    cycleStereoMode: (delta = 1) => {
      const mode = store.cycleStereoMode(delta);
      player.applyDsp({ stereoMode: mode });
    },
    setStereoMode: (mode) => {
      const res = store.setStereoMode(mode);
      player.applyDsp({ stereoMode: res });
    },
    cycleDolbyMode: (delta = 1) => {
      const mode = store.cycleDolbyMode(delta);
      player.applyDsp({ dolbyMode: mode });
    },
    cycleTapeType: (delta = 1) => {
      const type = store.cycleTapeType(delta);
      player.applyDsp({ tapeType: type });
    },
    toggleBassBoost: () => {
      const boost = store.toggleBassBoost();
      player.applyDsp({ bassBoost: boost });
    },
    cycleMode: (delta = 1) => store.cycleMode(delta),
    addToQueue: () => {
      const item = store.selected();
      if (item) store.addToQueue(item);
    },
    removeFromQueue: () => {
      if (store.state.mode === 'QUEUE') {
        store.removeFromQueue(store.state.selectedIndex);
      }
    },
    clearQueue: () => store.clearQueue(),
    scanMusicDir: async () => {
      setStatus(`Scanning ${store.state.musicDir}...`);
      const { library, tracks } = await scanDirectory(store.state.musicDir, (progress) => {
        if (progress.phase === 'parsing') {
          setStatus(`Parsing tags: ${progress.done}/${progress.total} tracks`);
        }
      });
      store.setLibrary(library, tracks);
      setStatus(`Discovered ${tracks.length} tracks across ${Object.keys(library.artists).length} artists in ${store.state.musicDir}`);
    },
    loadYouTubeUrl: async (query) => {
      if (!query || !query.trim()) return;
      const raw = query.trim();
      setStatus(`Resolving stream for "${raw.slice(0, 24)}"...`);
      try {
        const res = await resolveMedia(raw);
        if (res.tracks.length === 1) {
          const track = res.tracks[0];
          store.addToQueue(track);
          if (!store.state.playing && !store.state.paused) {
            actions.play(track);
          } else {
            setStatus(`Queued "${track.title.slice(0, 26)}" from YouTube`);
          }
        } else {
          for (const t of res.tracks) {
            store.state.library.tracksById[t.id] = t;
            store.addToQueue(t);
          }
          if (res.title) {
            store.state.library.playlists[res.title] = {
              id: `playlist:yt:${res.title}`,
              name: res.title,
              trackIds: res.tracks.map((t) => t.id)
            };
          }
          setStatus(`Loaded ${res.tracks.length} tracks from "${res.title.slice(0, 22)}" into Queue`);
          if (!store.state.playing && !store.state.paused && res.tracks.length > 0) {
            actions.play(res.tracks[0]);
          }
        }
      } catch (err) {
        setStatus(`YouTube error: ${err.message}`);
      }
    },
    searchYouTube: async (query) => {
      if (!query || !query.trim()) return;
      const raw = query.trim();
      store.setMode('YOUTUBE MUSIC');
      store.setYouTubeLoading(true, raw);
      try {
        const results = await searchMusic(raw, 25);
        store.setYouTubeResults(results, raw);
      } catch (err) {
        store.setYouTubeLoading(false, raw);
        store.update({ status: `YouTube search error: ${err.message}` });
      }
    },
    volume: (delta) => {
      const volume = Math.max(0, Math.min(100, store.state.volume + delta));
      player.setVolume(volume);
      store.update({ volume });
    },
    favorite: async () => {
      const item = store.state.current || store.selected();
      if (item) await store.toggleFavorite(item);
    },
    toggleLyrics: (force) => store.toggleLyrics(force),
    scrollLyrics: (delta) => store.scrollLyrics(delta),
    adjustLyricsSyncOffset: (delta) => store.adjustLyricsSyncOffset(delta),
    toggleSettings: (force) => store.toggleSettings(force),
    moveSettingsSelection: (delta) => store.moveSettingsSelection(delta),
    cycleSettingValue: (delta) => {
      const res = store.cycleSettingValue(delta);
      if (res) {
        if (res.section.id.startsWith('dsp.')) {
          player.applyDsp({
            stereoMode: store.state.stereoMode,
            dolbyMode: store.state.dolbyMode,
            tapeType: store.state.tapeType,
            bassBoost: store.state.bassBoost
          });
        }
        if (res.section.id.startsWith('visualizer.')) {
          player.updateBallistics({
            peakHoldMs: store.config.visualizer.peakHoldMs,
            peakDecayRate: store.config.visualizer.peakDecayRate
          });
        }
      }
      return res;
    },
    quit: () => shutdown(0)
  };

  // Start Desktop Tray & MPRIS2 Media Controller Daemon
  tray = new TrayManager(store, actions, player);
  tray.start();

  const isDaemonMode = process.argv.includes('--daemon') || process.argv.includes('--tray') || process.argv.includes('--minimized') || process.argv.includes('-d');
  if (!isDaemonMode) {
    layout = createLayout(store, actions, player);
  } else {
    // If started in background tray mode, resume last session track or play first track
    const target = store.state.current || store.state.localTracks?.[0] || store.state.stations?.[0];
    if (target) actions.play(target);
  }

  // Periodic session background saver
  const sessionSaveTimer = setInterval(() => {
    store.saveSession();
  }, 15000);
  sessionSaveTimer.unref();

  player.on('state', (state) => {
    store.update({ playing: state === 'playing' || state === 'buffering', paused: state === 'paused' });
  });

  player.on('ended', () => {
    actions.next();
  });

  player.on('metadata', (metadata) => {
    const rawMeta = metadata['icy-title'] || metadata.title || metadata.Artist || '';
    if (rawMeta) {
      const metaStr = String(rawMeta);
      store.update({ metadata: metaStr });

      // If streaming live radio station, attempt best-effort lyrics fetch
      if (store.state.current?.type === 'radio' || store.state.mode === 'RADIO STATIONS') {
        const cleaned = cleanTitleAndArtist(metaStr);
        const radioKey = `${cleaned.artist}::${cleaned.title}`;
        if (cleaned.title && cleaned.artist && cleaned.artist.toLowerCase() !== 'unknown artist' && radioKey !== lastRadioLyricsKey) {
          lastRadioLyricsKey = radioKey;
          triggerLyricsFetch({
            id: `radio:${cleaned.artist}:${cleaned.title}`,
            title: cleaned.title,
            artist: cleaned.artist,
            type: 'radio'
          }, { isRadio: true });
        }
      }
    }
  });

  player.on('error', (error) => {
    store.update({ playing: false, paused: false, status: `Player error: ${error.message}` });
  });
}

main().catch((error) => { console.error(error); shutdown(1); });

