import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const BOOMBOX_CONFIG_DIR = join(homedir(), '.config', 'boombox-tui');
const LEGACY_CONFIG_DIR = join(homedir(), '.config', 'hiphop-tui');
const MIXTAPES_FILE = join(BOOMBOX_CONFIG_DIR, 'mixtapes.json');

export class MixtapeManager {
  constructor() {
    this.mixtapes = new Map();
    this.loaded = false;
  }

  async init() {
    if (this.loaded) return this.getMixtapes();

    try {
      if (!existsSync(BOOMBOX_CONFIG_DIR)) {
        await mkdir(BOOMBOX_CONFIG_DIR, { recursive: true });
      }

      let filePath = MIXTAPES_FILE;
      if (!existsSync(filePath) && existsSync(join(LEGACY_CONFIG_DIR, 'mixtapes.json'))) {
        filePath = join(LEGACY_CONFIG_DIR, 'mixtapes.json');
      }

      if (existsSync(filePath)) {
        const raw = await readFile(filePath, 'utf8');
        const list = JSON.parse(raw);
        if (Array.isArray(list)) {
          for (const m of list) {
            if (m && m.id && m.name) {
              this.mixtapes.set(m.id, {
                id: m.id,
                name: m.name,
                createdAt: m.createdAt || new Date().toISOString(),
                tracks: Array.isArray(m.tracks) ? m.tracks : []
              });
            }
          }
        }
      }
    } catch {
      // Graceful fallback on empty
    }

    // Ensure at least one default Favorite Mixtape exists
    if (this.mixtapes.size === 0) {
      const defaultId = 'mixtape:favorites';
      this.mixtapes.set(defaultId, {
        id: defaultId,
        name: '★ Favorites Mixtape',
        createdAt: new Date().toISOString(),
        tracks: []
      });
    }

    this.loaded = true;
    return this.getMixtapes();
  }

  getMixtapes() {
    return Array.from(this.mixtapes.values());
  }

  getMixtape(id) {
    return this.mixtapes.get(id) || null;
  }

  async createMixtape(name) {
    await this.init();
    const cleanName = (name || 'New Mixtape').trim();
    const id = `mixtape:${Date.now()}_${Math.random().toString(36).slice(2, 7)}`;
    const mixtape = {
      id,
      name: cleanName,
      createdAt: new Date().toISOString(),
      tracks: []
    };
    this.mixtapes.set(id, mixtape);
    await this.save();
    return mixtape;
  }

  async deleteMixtape(id) {
    await this.init();
    const deleted = this.mixtapes.delete(id);
    if (deleted) await this.save();
    return deleted;
  }

  async addTrackToMixtape(mixtapeId, track) {
    await this.init();
    const mixtape = this.mixtapes.get(mixtapeId);
    if (!mixtape || !track) return false;

    const normalizedTrack = {
      id: track.id || track.url || `track:${Date.now()}`,
      title: track.title || track.name || 'Unknown Track',
      artist: track.artist || track.author || track.channel || 'Unknown Artist',
      album: track.album || mixtape.name,
      source: track.source || (track.url?.includes('youtube') ? 'youtube' : track.type === 'radio' ? 'radio' : 'local'),
      url: track.url || track.streamUrl || track.path || '',
      duration: track.duration || 0,
      bitrate: track.bitrate || 0,
      codec: track.codec || ''
    };

    // Avoid duplicate URLs inside same mixtape
    const exists = mixtape.tracks.some((t) => t.url && t.url === normalizedTrack.url);
    if (!exists) {
      mixtape.tracks.push(normalizedTrack);
      await this.save();
      return true;
    }
    return false;
  }

  async removeTrackFromMixtape(mixtapeId, trackIndex) {
    await this.init();
    const mixtape = this.mixtapes.get(mixtapeId);
    if (!mixtape || trackIndex < 0 || trackIndex >= mixtape.tracks.length) return false;

    mixtape.tracks.splice(trackIndex, 1);
    await this.save();
    return true;
  }

  async save() {
    try {
      if (!existsSync(BOOMBOX_CONFIG_DIR)) {
        await mkdir(BOOMBOX_CONFIG_DIR, { recursive: true });
      }
      const data = JSON.stringify(Array.from(this.mixtapes.values()), null, 2);
      await writeFile(MIXTAPES_FILE, data, 'utf8');
    } catch {}
  }
}

export const mixtapeManager = new MixtapeManager();
