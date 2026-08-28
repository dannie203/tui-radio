import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const CONFIG_DIR = join(homedir(), '.config', 'hiphop-tui');
const COOKIES_FILE = join(CONFIG_DIR, 'cookies.txt');

/**
 * Normalizes YouTube and YouTube Music URLs, handles protocol omissions,
 * and standardizes music.youtube.com domains for universal streaming compatibility.
 *
 * @param {string} input
 * @returns {string}
 */
export function normalizeYouTubeInput(input) {
  let raw = (input || '').trim();
  if (!raw) return '';

  // If user pasted "yt:VIDEO_ID"
  if (raw.startsWith('yt:')) {
    const id = raw.slice(3).trim();
    if (id.startsWith('http')) raw = id;
    else return `https://www.youtube.com/watch?v=${id}`;
  }

  // Prepend https:// if domain is provided without protocol
  if (/^(music\.)?youtube\.com\//i.test(raw) || /^youtu\.be\//i.test(raw) || /^www\.youtube\.com\//i.test(raw)) {
    raw = `https://${raw}`;
  }

  // Standardize music.youtube.com to youtube.com for bulletproof yt-dlp & mpv streaming
  if (/^https?:\/\/music\.youtube\.com\//i.test(raw)) {
    raw = raw.replace(/^https?:\/\/music\.youtube\.com\//i, 'https://www.youtube.com/');
  }

  return raw;
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
    return 'Playlist unavailable or private on YouTube';
  }
  if (msg.includes('Video unavailable') || msg.includes('Private video')) {
    return 'Video is unavailable or private on YouTube';
  }
  if (msg.includes('Sign in to confirm') || msg.includes('bot')) {
    return 'YouTube requires authentication (export cookies.txt)';
  }
  return msg.replace(/^ERROR:\s*(\[[^\]]+\]\s*)?/i, '').split('\n')[0].slice(0, 50);
}

function executeResolve(query) {
  return new Promise((resolve, reject) => {
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
      reject(new Error('Timed out resolving YouTube link (15s)'));
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
        const id = entry.id || entry.url || `yt-${idx}`;
        const videoUrl = entry.webpage_url || entry.url || (entry.id ? `https://www.youtube.com/watch?v=${entry.id}` : query);
        const title = entry.title || 'YouTube Stream';
        const artist = entry.channel || entry.uploader || 'YouTube';
        const duration = typeof entry.duration === 'number' ? entry.duration : 0;

        return {
          id: `yt:${id}`,
          type: 'youtube',
          name: title,
          title,
          artist,
          album: playlistTitle || 'YouTube Live / Stream',
          trackNo: idx + 1,
          duration,
          url: videoUrl,
          path: videoUrl,
          format: 'OPUS',
          codec: 'OPUS',
          bitrate: 160,
          country: 'WEB',
          tags: `youtube, web, stream, ${artist}`
        };
      });

      resolve({
        isPlaylist: tracks.length > 1,
        title: playlistTitle || (tracks.length === 1 ? tracks[0].title : 'YouTube Playlist'),
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
 * Resolves a YouTube or YouTube Music URL, Playlist URL, or search query using yt-dlp.
 * Returns normalized track object(s) ready for Queue and MPV playback.
 */
export async function resolveMedia(input) {
  const normalized = normalizeYouTubeInput(input);
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
 * Searches specifically for Music tracks on YouTube, filtering out non-music content,
 * long podcasts, and short memes.
 *
 * @param {string} query
 * @param {number} [limit=15]
 * @returns {Promise<Array<object>>}
 */
export function searchMusic(query, limit = 15) {
  return new Promise((resolve, reject) => {
    const raw = (query || '').trim();
    if (!raw) return resolve([]);

    // Check if searching for a full album/mixtape
    const isLongForm = /\b(album|mixtape|full album|mix|compilation|live at)\b/i.test(raw);
    const searchTarget = `ytsearch${limit * 2}:${raw}`;

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
      reject(new Error('Timed out searching YouTube Music (15s)'));
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

      for (const line of lines) {
        try {
          const entry = JSON.parse(line);
          const dur = typeof entry.duration === 'number' ? entry.duration : 0;

          // Filter out short audio memes (< 30s) or long podcasts (> 12 mins) unless searching for albums
          if (!isLongForm) {
            if (dur > 0 && (dur < 30 || dur > 720)) continue;
          }

          const rawTitle = entry.title || 'Unknown Track';
          const rawChannel = entry.channel || entry.uploader || 'YouTube';
          const isTopic = rawChannel.includes('- Topic') || rawChannel.includes('VEVO') || entry.channel_is_verified;

          candidates.push({
            id: `yt:${entry.id || entry.url}`,
            type: 'youtube',
            name: rawTitle,
            title: rawTitle,
            artist: rawChannel.replace(/\s*-\s*Topic$/i, '').replace(/VEVO$/i, '').trim() || 'YouTube',
            channel: rawChannel,
            isTopic: Boolean(isTopic),
            album: isTopic ? 'Official Release' : 'YouTube Music',
            trackNo: candidates.length + 1,
            duration: dur,
            url: entry.webpage_url || entry.url || (entry.id ? `https://www.youtube.com/watch?v=${entry.id}` : ''),
            path: entry.webpage_url || entry.url || (entry.id ? `https://www.youtube.com/watch?v=${entry.id}` : ''),
            format: 'OPUS',
            codec: 'OPUS',
            bitrate: 160,
            country: 'WEB',
            tags: `youtube, music, ${rawChannel}`
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

