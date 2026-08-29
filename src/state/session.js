import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { CONFIG_DIR } from './config.js';

export const SESSION_FILE = join(CONFIG_DIR, 'session.json');

export async function loadSession() {
  try {
    const raw = await readFile(SESSION_FILE, 'utf8');
    const data = JSON.parse(raw);
    return data && typeof data === 'object' ? data : null;
  } catch {
    return null;
  }
}

export async function saveSession(state) {
  if (!state) return;
  try {
    const session = {
      mode: state.mode,
      nav: {
        level: state.nav?.level || 'ARTISTS',
        selectedArtist: state.nav?.selectedArtist || null,
        selectedAlbumKey: state.nav?.selectedAlbumKey || null,
        selectedPlaylist: state.nav?.selectedPlaylist || null
      },
      current: state.current ? {
        id: state.current.id,
        title: state.current.title || state.current.name,
        name: state.current.name || state.current.title,
        artist: state.current.artist,
        album: state.current.album,
        path: state.current.path,
        url: state.current.url,
        type: state.current.type,
        duration: state.current.duration,
        codec: state.current.codec,
        bitrate: state.current.bitrate,
        sampleRate: state.current.sampleRate
      } : null,
      timePos: state.timePos || 0,
      queue: state.queue || [],
      queueIndex: state.queueIndex ?? -1,
      selectedIndex: state.selectedIndex || 0,
      volume: state.volume ?? 80,
      shuffle: Boolean(state.shuffle),
      repeat: state.repeat || 'off',
      genreFilter: state.genreFilter || 'ALL',
      stereoMode: state.stereoMode || 'STEREO',
      dolbyMode: state.dolbyMode || 'DOLBY-B',
      tapeType: state.tapeType || 'TYPE-II',
      bassBoost: Boolean(state.bassBoost),
      lyricsVisible: Boolean(state.lyricsVisible),
      lyricsSyncOffset: state.lyricsSyncOffset || 0
    };
    await mkdir(CONFIG_DIR, { recursive: true });
    await writeFile(SESSION_FILE, JSON.stringify(session, null, 2), 'utf8');
  } catch {}
}
