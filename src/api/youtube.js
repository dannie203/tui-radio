import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const CONFIG_DIR = join(homedir(), '.config', 'hiphop-tui');
const COOKIES_FILE = join(CONFIG_DIR, 'cookies.txt');

/**
 * Normalizes any stream or platform URL (YouTube, SoundCloud, Bandcamp, Mixcloud, Direct Audio).
 *
 * @param {string} input
 * @returns {string}
 */
export function normalizeMediaInput(input) {
  let raw = (input || '').trim();
  if (!raw) return '';

  // Prefixes
  if (raw.startsWith('yt:')) {
    const id = raw.slice(3).trim();
    if (id.startsWith('http')) raw = id;
    else return `https://www.youtube.com/watch?v=${id}`;
  }
  if (raw.startsWith('sc:')) {
    const id = raw.slice(3).trim();
    if (id.startsWith('http')) raw = id;
    else return `https://soundcloud.com/${id}`;
  }
  if (raw.startsWith('bc:')) {
    const id = raw.slice(3).trim();
    if (id.startsWith('http')) raw = id;
    else if (id.includes('.bandcamp.com')) raw = `https://${id}`;
    else if (id.includes('/')) {
      const parts = id.split('/');
      return `https://${parts[0]}.bandcamp.com/${parts.slice(1).join('/')}`;
    } else {
      return `https://${id}.bandcamp.com`;
    }
  }

  // Prepend https:// if domain is provided without protocol
  if (/^(music\.)?youtube\.com\//i.test(raw) || /^youtu\.be\//i.test(raw) || /^www\.youtube\.com\//i.test(raw) ||
      /^(www\.)?soundcloud\.com\//i.test(raw) || /^(www\.)?mixcloud\.com\//i.test(raw) || /^[a-z0-9-]+\.bandcamp\.com\//i.test(raw) ||
      /^bandcamp\.com\//i.test(raw)) {
    raw = `https://${raw}`;
  }

  // Standardize music.youtube.com to youtube.com for bulletproof streaming
  if (/^https?:\/\/music\.youtube\.com\//i.test(raw)) {
    raw = raw.replace(/^https?:\/\/music\.youtube\.com\//i, 'https://www.youtube.com/');
  }

  return raw;
}

export const normalizeYouTubeInput = normalizeMediaInput;

/**
 * Detects whether a string is a direct audio stream URL (.mp3, .flac, .m3u8, .aac, .ogg, .opus, .wav).
 */
export function isDirectAudioStream(url) {
  if (!url || typeof url !== 'string') return false;
  return /^https?:\/\/.*\.(mp3|flac|m3u8|aac|ogg|opus|wav)(\?.*)?$/i.test(url.trim());
}

/**
 * Detects the platform metadata and format badges from URL and extractor info.
 */
export function detectPlatform(url = '', entry = {}) {
  const target = (url || entry.webpage_url || entry.url || '').toLowerCase();
  const extractor = (entry.extractor || entry.extractor_key || '').toLowerCase();

  if (extractor.includes('soundcloud') || target.includes('soundcloud.com')) {
    return {
      type: 'soundcloud',
      prefix: 'sc',
      format: 'MP3',
      codec: 'MP3',
      platform: 'SoundCloud',
      bitrate: 192
    };
  }
  if (extractor.includes('bandcamp') || target.includes('bandcamp.com')) {
    return {
      type: 'bandcamp',
      prefix: 'bc',
      format: 'FLAC',
      codec: 'FLAC',
      platform: 'Bandcamp',
      bitrate: 320
    };
  }
  if (extractor.includes('mixcloud') || target.includes('mixcloud.com')) {
    return {
      type: 'mixcloud',
      prefix: 'mc',
      format: 'AAC',
      codec: 'AAC',
      platform: 'Mixcloud',
      bitrate: 128
    };
  }
  if (isDirectAudioStream(target)) {
    return {
      type: 'stream',
      prefix: 'web',
      format: 'STREAM',
      codec: 'DIRECT',
      platform: 'Web Stream',
      bitrate: 256
    };
  }
  return {
    type: 'youtube',
    prefix: 'yt',
    format: 'OPUS',
    codec: 'OPUS',
    platform: 'YouTube',
    bitrate: 160
  };
}

/**
 * Extracts a single YouTube 11-char video ID from any URL format.
 */
export function extractVideoId(url) {
  if (!url) return null;
  const match = url.match(/[?&]v=([a-zA-Z0-9_-]{11})/);
  if (match) return match[1];
  const shortMatch = url.match(/youtu\.be\/([a-zA-Z0-9_-]{11})/);
  if (shortMatch) return shortMatch[1];
  const embedMatch = url.match(/embed\/([a-zA-Z0-9_-]{11})/);
  if (embedMatch) return embedMatch[1];
  return null;
}

/**
 * Converts raw yt-dlp stderr messages into clean, user-friendly status strings.
 */
export function formatYouTubeError(errMessage) {
  const msg = String(errMessage || '');
  if (msg.includes('The playlist does not exist') || msg.includes('HTTP Error 400') || msg.includes('This playlist is private')) {
    return 'Playlist unavailable or private on streaming provider';
  }
  if (msg.includes('Video unavailable') || msg.includes('Track unavailable') || msg.includes('Private video')) {
    return 'Track is unavailable or private on streaming provider';
  }
  if (msg.includes('Sign in to confirm') || msg.includes('bot')) {
    return 'Streaming provider requires authentication (export cookies.txt)';
  }
  return msg.replace(/^ERROR:\s*(\[[^\]]+\]\s*)?/i, '').split('\n')[0].slice(0, 50);
}

function executeResolve(query) {
  return new Promise((resolve, reject) => {
    // Check direct audio stream shortcut
    if (isDirectAudioStream(query)) {
      const filename = query.split('/').pop().split('?')[0] || 'Audio Stream';
      const name = decodeURIComponent(filename.replace(/\.[a-z0-9]+$/i, ''));
      return resolve({
        isPlaylist: false,
        title: name,
        tracks: [{
          id: `web:${query}`,
          type: 'stream',
          name,
          title: name,
          artist: 'Direct Stream',
          album: 'Web Audio Broadcast',
          trackNo: 1,
          duration: 0,
          url: query,
          path: query,
          format: 'STREAM',
          codec: 'DIRECT',
          bitrate: 256,
          country: 'WEB',
          tags: 'stream, audio, direct'
        }]
      });
    }

    const isUrl = /^https?:\/\//i.test(query);
    const target = isUrl ? query : `ytsearch1:${query}`;

    const args = [
      '--dump-json',
      '--flat-playlist',
      '--no-warnings',
      '--skip-download'
    ];

    if (existsSync(COOKIES_FILE)) {
      args.push('--cookies', COOKIES_FILE);
    }

    args.push(target);

    const proc = spawn('yt-dlp', args);

    let stdout = '';
    let stderr = '';

    const timeout = setTimeout(() => {
      proc.kill('SIGKILL');
      reject(new Error('Timed out resolving stream link (15s)'));
    }, 15000);

    proc.stdout.on('data', (d) => { stdout += d.toString(); });
    proc.stderr.on('data', (d) => { stderr += d.toString(); });

    proc.on('close', (code) => {
      clearTimeout(timeout);
      if (code !== 0 && !stdout.trim()) {
        return reject(new Error(stderr.trim() || `yt-dlp exited with code ${code}`));
      }

      const lines = stdout.trim().split('\n').filter(Boolean);
      const rawEntries = [];
      let playlistTitle = null;

      for (const line of lines) {
        try {
          const parsed = JSON.parse(line);
          if (parsed._type === 'playlist' && Array.isArray(parsed.entries)) {
            playlistTitle = parsed.title || playlistTitle;
            rawEntries.push(...parsed.entries);
          } else {
            if (parsed.playlist_title) playlistTitle = parsed.playlist_title;
            rawEntries.push(parsed);
          }
        } catch {
          // ignore corrupted JSON line
        }
      }

      if (rawEntries.length === 0) {
        return reject(new Error('No streamable tracks found for query'));
      }

      const tracks = rawEntries.map((entry, idx) => {
        const platform = detectPlatform(query, entry);
        const id = entry.id || entry.url || `${platform.prefix}-${idx}`;
        const streamUrl = entry.webpage_url || entry.url || (entry.id && platform.type === 'youtube' ? `https://www.youtube.com/watch?v=${entry.id}` : query);
        const title = entry.title || `${platform.platform} Stream`;
        const artist = entry.channel || entry.uploader || entry.artist || platform.platform;
        const duration = typeof entry.duration === 'number' ? entry.duration : 0;

        return {
          id: `${platform.prefix}:${id}`,
          type: platform.type,
          name: title,
          title,
          artist,
          album: playlistTitle || `${platform.platform} Stream`,
          trackNo: idx + 1,
          duration,
          url: streamUrl,
          path: streamUrl,
          format: platform.format,
          codec: platform.codec,
          bitrate: platform.bitrate,
          country: 'WEB',
          tags: `${platform.type}, web, stream, ${artist}`
        };
      });

      resolve({
        isPlaylist: tracks.length > 1,
        title: playlistTitle || (tracks.length === 1 ? tracks[0].title : 'Online Playlist'),
        tracks
      });
    });

    proc.on('error', (err) => {
      clearTimeout(timeout);
      reject(new Error(`Failed to execute yt-dlp: ${err.message}`));
    });
  });
}

/**
 * Resolves any media URL (YouTube, SoundCloud, Bandcamp, Mixcloud, Direct Audio Stream)
 * Returns normalized track object(s) ready for Queue and MPV playback.
 */
export async function resolveMedia(input) {
  const normalized = normalizeMediaInput(input);
  if (!normalized) throw new Error('Empty link or search query');

  try {
    return await executeResolve(normalized);
  } catch (err) {
    // If it failed on a URL that contains a video ID, fallback to resolving the single video
    const videoId = extractVideoId(normalized);
    if (videoId && (normalized.includes('&list=') || normalized.includes('?list='))) {
      try {
        return await executeResolve(`https://www.youtube.com/watch?v=${videoId}`);
      } catch {}
    }
    throw new Error(formatYouTubeError(err.message));
  }
}

/**
 * Searches for Music tracks across YouTube, SoundCloud, and streaming providers.
 *
 * @param {string} query
 * @param {number} [limit=15]
 * @returns {Promise<Array<object>>}
 */
export function searchMusic(query, limit = 15) {
  return new Promise((resolve, reject) => {
    const raw = (query || '').trim();
    if (!raw) return resolve([]);

    // Check platform search prefixes
    let searchTarget;
    let platformType = 'youtube';

    if (raw.startsWith('sc:') || raw.startsWith('soundcloud:')) {
      const clean = raw.replace(/^(sc:|soundcloud:)/i, '').trim();
      searchTarget = `scsearch${limit}:${clean}`;
      platformType = 'soundcloud';
    } else if (raw.startsWith('yt:') || raw.startsWith('youtube:')) {
      const clean = raw.replace(/^(yt:|youtube:)/i, '').trim();
      searchTarget = `ytsearch${limit * 2}:${clean}`;
      platformType = 'youtube';
    } else {
      const isLongForm = /\b(album|mixtape|full album|mix|compilation|live at)\b/i.test(raw);
      searchTarget = `ytsearch${limit * 2}:${raw}`;
      platformType = 'youtube';
    }

    const args = [
      '--dump-json',
      '--flat-playlist',
      '--no-warnings',
      '--skip-download'
    ];

    if (existsSync(COOKIES_FILE)) {
      args.push('--cookies', COOKIES_FILE);
    }

    args.push(searchTarget);

    const proc = spawn('yt-dlp', args);

    let stdout = '';
    let stderr = '';

    const timeout = setTimeout(() => {
      proc.kill('SIGKILL');
      reject(new Error('Timed out searching online music streams (15s)'));
    }, 15000);

    proc.stdout.on('data', (d) => { stdout += d.toString(); });
    proc.stderr.on('data', (d) => { stderr += d.toString(); });

    proc.on('close', (code) => {
      clearTimeout(timeout);
      if (code !== 0 && !stdout.trim()) {
        return reject(new Error(stderr.trim() || `yt-dlp search exited with code ${code}`));
      }

      const lines = stdout.trim().split('\n').filter(Boolean);
      const candidates = [];
      const isLongForm = /\b(album|mixtape|full album|mix|compilation|live at)\b/i.test(raw);

      for (const line of lines) {
        try {
          const entry = JSON.parse(line);
          const dur = typeof entry.duration === 'number' ? entry.duration : 0;

          // Filter out short audio memes (< 30s) or long podcasts (> 12 mins) unless searching for albums
          if (!isLongForm && platformType === 'youtube') {
            if (dur > 0 && (dur < 30 || dur > 720)) continue;
          }

          const platform = detectPlatform('', entry);
          const rawTitle = entry.title || 'Unknown Track';
          const rawChannel = entry.channel || entry.uploader || entry.artist || platform.platform;
          const isTopic = rawChannel.includes('- Topic') || rawChannel.includes('VEVO') || entry.channel_is_verified;

          candidates.push({
            id: `${platform.prefix}:${entry.id || entry.url}`,
            type: platform.type,
            name: rawTitle,
            title: rawTitle,
            artist: rawChannel.replace(/\s*-\s*Topic$/i, '').replace(/VEVO$/i, '').trim() || platform.platform,
            channel: rawChannel,
            isTopic: Boolean(isTopic),
            album: isTopic ? 'Official Release' : `${platform.platform} Music`,
            trackNo: candidates.length + 1,
            duration: dur,
            url: entry.webpage_url || entry.url || (entry.id && platform.type === 'youtube' ? `https://www.youtube.com/watch?v=${entry.id}` : ''),
            path: entry.webpage_url || entry.url || (entry.id && platform.type === 'youtube' ? `https://www.youtube.com/watch?v=${entry.id}` : ''),
            format: platform.format,
            codec: platform.codec,
            bitrate: platform.bitrate,
            country: 'WEB',
            tags: `${platform.type}, music, ${rawChannel}`
          });
        } catch {
          // ignore corrupted JSON line
        }
      }

      // Prioritize official Topic and VEVO tracks
      candidates.sort((a, b) => {
        if (a.isTopic && !b.isTopic) return -1;
        if (!a.isTopic && b.isTopic) return 1;
        return 0;
      });

      resolve(candidates.slice(0, limit));
    });

    proc.on('error', (err) => {
      clearTimeout(timeout);
      reject(new Error(`Failed to execute yt-dlp: ${err.message}`));
    });
  });
}
