import { readFile } from 'node:fs/promises';
import { extname } from 'node:path';

const LRCLIB_BASE_URL = 'https://lrclib.net/api';
const REQUEST_HEADERS = { 'User-Agent': 'hiphop-radio-tui/1.0' };

/**
 * Parses LRC formatted text into a sorted array of timed lyric objects.
 * Format: [mm:ss.xx] Lyric text
 *
 * @param {string} lrcContent
 * @returns {Array<{ time: number, text: string }>}
 */
export function parseLrc(lrcContent) {
  if (!lrcContent || typeof lrcContent !== 'string') return [];

  const lines = lrcContent.split(/\r?\n/);
  const result = [];
  const timeRegex = /\[(\d{1,3}):(\d{2})(?:\.(\d{1,3}))?\]/g;

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    // Check if line contains timestamp(s)
    timeRegex.lastIndex = 0;
    const matches = [];
    let match;
    while ((match = timeRegex.exec(trimmed)) !== null) {
      matches.push(match);
    }

    if (matches.length === 0) continue;

    // Extract text after stripping all timestamp tags
    const text = trimmed.replace(timeRegex, '').trim();

    for (const m of matches) {
      const minutes = parseInt(m[1], 10);
      const seconds = parseInt(m[2], 10);
      let ms = 0;
      if (m[3]) {
        const msStr = m[3].padEnd(3, '0').slice(0, 3);
        ms = parseInt(msStr, 10);
      }
      const totalSeconds = Number((minutes * 60 + seconds + ms / 1000).toFixed(3));
      result.push({ time: totalSeconds, text });
    }
  }

  return result.sort((a, b) => a.time - b.time);
}

/**
 * Sanitizes noisy YouTube video or stream titles to extract clean Artist and Track Title.
 * E.g. "Eminem - Lose Yourself (Official Video) [HD]" -> { artist: "Eminem", title: "Lose Yourself" }
 *
 * @param {string} rawTitle
 * @param {string} rawArtist
 * @returns {{ artist: string, title: string, cleanTitle: string }}
 */
export function cleanTitleAndArtist(rawTitle = '', rawArtist = '') {
  let titleStr = String(rawTitle || '').trim();
  let artistStr = String(rawArtist || '').trim();

  // Clean common channel suffixes
  artistStr = artistStr
    .replace(/\s*-\s*Topic$/i, '')
    .replace(/VEVO$/i, '')
    .trim();

  // Regex pattern for video tags
  const tagPattern = /\s*[\(\[](?:official\s*(?:video|audio|music\s*video|lyric\s*video|visualizer|hd|4k|remastered|version|remix)?|audio|lyrics?|lyric|visualizer|mv|hq|hd|4k|explicit(?:\s*audio)?|clean|remastered|prod\.[^\)\]]+)[\)\]]/gi;

  titleStr = titleStr.replace(tagPattern, '').replace(tagPattern, '').replace(/["'|]/g, '').trim();

  // Check if title itself contains "Artist - Title" or "Artist – Title"
  if (titleStr.includes(' - ') || titleStr.includes(' – ')) {
    const sep = titleStr.includes(' - ') ? ' - ' : ' – ';
    const parts = titleStr.split(sep);
    const candidateArtist = parts[0].trim();
    let candidateTitle = parts.slice(1).join(sep).trim();
    candidateTitle = candidateTitle.replace(tagPattern, '').replace(tagPattern, '').trim();

    const isGenericArtist = !artistStr ||
      artistStr.toLowerCase() === 'youtube' ||
      artistStr.toLowerCase() === 'unknown artist' ||
      artistStr.toLowerCase() === 'live broadcast';

    if (isGenericArtist || candidateArtist.toLowerCase() === artistStr.toLowerCase()) {
      artistStr = candidateArtist;
      titleStr = candidateTitle;
    }
  }

  // Strip feature tags like "ft. XYZ" or "feat. XYZ" from search title for higher lookup hit-rates
  const cleanTitle = titleStr.replace(/\s*(?:\(ft\.|\(feat\.|\(featuring|ft\.|feat\.|featuring)\s+.*$/i, '').trim();

  return {
    artist: artistStr,
    title: titleStr,
    cleanTitle: cleanTitle || titleStr
  };
}

/**
 * Finds the index of the active lyric line using binary search.
 *
 * @param {Array<{ time: number, text: string }>} syncedLyrics
 * @param {number} currentTime
 * @returns {number} Index of current active lyric line, or -1 if before start or empty
 */
export function findActiveLyricIndex(syncedLyrics, currentTime) {
  if (!syncedLyrics || syncedLyrics.length === 0) return -1;
  if (currentTime < syncedLyrics[0].time) return -1;

  let low = 0;
  let high = syncedLyrics.length - 1;
  let bestIndex = -1;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (syncedLyrics[mid].time <= currentTime) {
      bestIndex = mid;
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }

  return bestIndex;
}

/**
 * Fetches lyrics with local sidecar and LRCLIB integration.
 * Normalizes plain and synced lyrics into a unified response shape.
 *
 * @param {object} params
 * @param {string} [params.artist]
 * @param {string} [params.title]
 * @param {string} [params.album]
 * @param {number} [params.duration]
 * @param {string} [params.path]
 * @param {AbortSignal} [params.signal]
 * @param {typeof fetch} [params.fetchImpl]
 * @returns {Promise<{
 *   synced: Array<{ time: number, text: string }>,
 *   plain: string,
 *   source: string,
 *   isSynced: boolean,
 *   trackName: string,
 *   artistName: string,
 *   duration: number,
 *   error?: string
 * }>}
 */
export async function fetchLyrics({
  artist = '',
  title = '',
  album = '',
  duration = 0,
  path = '',
  signal,
  fetchImpl = globalThis.fetch
} = {}) {
  const cleaned = cleanTitleAndArtist(title, artist);
  const searchTitle = cleaned.cleanTitle || cleaned.title;
  const searchArtist = cleaned.artist;

  // 1. Check for local .lrc sidecar file if path is a local file
  if (path && typeof path === 'string' && path.startsWith('/')) {
    try {
      const ext = extname(path);
      if (ext) {
        const lrcPath = path.slice(0, -ext.length) + '.lrc';
        const rawLrc = await readFile(lrcPath, 'utf8');
        const synced = parseLrc(rawLrc);
        return {
          synced,
          plain: rawLrc,
          source: 'local-file',
          isSynced: synced.length > 0,
          trackName: cleaned.title || title,
          artistName: cleaned.artist || artist,
          duration: duration || 0
        };
      }
    } catch {
      // Local .lrc does not exist or unreadable, continue to LRCLIB
    }
  }

  if (!searchTitle) {
    return {
      synced: [],
      plain: '',
      source: 'none',
      isSynced: false,
      trackName: '',
      artistName: '',
      duration: 0
    };
  }

  // 2. Query LRCLIB API
  try {
    const params = new URLSearchParams();
    params.set('track_name', searchTitle);
    if (searchArtist) params.set('artist_name', searchArtist);
    if (album && album !== 'Singles / Unknown Album' && !album.startsWith('YouTube')) {
      params.set('album_name', album);
    }
    if (duration > 0) {
      params.set('duration', String(Math.round(duration)));
    }

    let response = await fetchImpl(`${LRCLIB_BASE_URL}/get?${params.toString()}`, {
      signal,
      headers: REQUEST_HEADERS
    });

    let data = null;

    if (response.ok) {
      data = await response.json();
    } else if (response.status === 404) {
      // Fallback to search endpoint with looser query
      const searchQuery = [searchArtist, searchTitle].filter(Boolean).join(' ');
      const searchUrl = `${LRCLIB_BASE_URL}/search?q=${encodeURIComponent(searchQuery)}`;
      const searchResponse = await fetchImpl(searchUrl, { signal, headers: REQUEST_HEADERS });
      if (searchResponse.ok) {
        const list = await searchResponse.json();
        if (Array.isArray(list) && list.length > 0) {
          // Prioritize entry with synced lyrics or closest duration
          data = list.find((item) => item.syncedLyrics) || list[0];
        }
      }
    }

    if (data && (data.syncedLyrics || data.plainLyrics)) {
      const synced = data.syncedLyrics ? parseLrc(data.syncedLyrics) : [];
      return {
        synced,
        plain: data.plainLyrics || data.syncedLyrics || '',
        source: 'lrclib',
        isSynced: synced.length > 0,
        trackName: data.trackName || cleaned.title || title,
        artistName: data.artistName || cleaned.artist || artist,
        duration: data.duration || duration || 0
      };
    }

    return {
      synced: [],
      plain: '',
      source: 'none',
      isSynced: false,
      trackName: cleaned.title || title,
      artistName: cleaned.artist || artist,
      duration: duration || 0
    };
  } catch (err) {
    if (signal?.aborted) {
      throw err;
    }
    return {
      synced: [],
      plain: '',
      source: 'none',
      isSynced: false,
      trackName: cleaned.title || title,
      artistName: cleaned.artist || artist,
      duration: duration || 0,
      error: err.message
    };
  }
}
