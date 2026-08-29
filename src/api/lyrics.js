import { readFile } from 'node:fs/promises';
import { extname } from 'node:path';

const LRCLIB_BASE_URL = 'https://lrclib.net/api';
const REQUEST_HEADERS = { 'User-Agent': 'hiphop-radio-tui/1.0' };

/**
 * Parses inline word-level timestamps in enhanced LRC lines like <00:12.34>Word
 *
 * @param {string} rawLine
 * @param {number} lineStartTime
 * @returns {Array<{ time: number, text: string }>|null}
 */
export function parseEnhancedWords(rawLine, lineStartTime) {
  if (!rawLine || typeof rawLine !== 'string') return null;
  const wordTagRegex = /<(?:\d{1,3}:)?\d{2}(?:\.\d{1,3})?>/g;
  if (!wordTagRegex.test(rawLine)) return null;

  wordTagRegex.lastIndex = 0;
  const tokens = [];
  const parts = rawLine.split(/(<(?:\d{1,3}:)?\d{2}(?:\.\d{1,3})?>)/g).filter(Boolean);

  let currentWordTime = lineStartTime;
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    const match = /^<(\d{1,3}:)?(\d{2})(?:\.(\d{1,3}))?>$/.exec(part);
    if (match) {
      const minutes = match[1] ? parseInt(match[1].replace(':', ''), 10) : 0;
      const seconds = parseInt(match[2], 10);
      let ms = 0;
      if (match[3]) {
        const msStr = match[3].padEnd(3, '0').slice(0, 3);
        ms = parseInt(msStr, 10);
      }
      currentWordTime = Number((minutes * 60 + seconds + ms / 1000).toFixed(3));
    } else {
      const cleanWord = part.trim();
      if (cleanWord) {
        tokens.push({ time: currentWordTime, text: part });
      }
    }
  }

  return tokens.length > 0 ? tokens : null;
}

/**
 * Parses LRC formatted text into a sorted array of timed lyric objects.
 * Format: [mm:ss.xx] Lyric text
 *
 * @param {string} lrcContent
 * @returns {Array<{ time: number, text: string, words?: Array<{ time: number, text: string }> }>}
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

    const rawLineBody = trimmed.replace(timeRegex, '').trim();
    const cleanText = rawLineBody.replace(/<(?:\d{1,3}:)?\d{2}(?:\.\d{1,3})?>/g, '').trim();

    for (const m of matches) {
      const minutes = parseInt(m[1], 10);
      const seconds = parseInt(m[2], 10);
      let ms = 0;
      if (m[3]) {
        const msStr = m[3].padEnd(3, '0').slice(0, 3);
        ms = parseInt(msStr, 10);
      }
      const totalSeconds = Number((minutes * 60 + seconds + ms / 1000).toFixed(3));
      const words = parseEnhancedWords(rawLineBody, totalSeconds);
      const item = { time: totalSeconds, text: cleanText };
      if (words && words.length > 0) {
        item.words = words;
      }
      result.push(item);
    }
  }

  return result.sort((a, b) => a.time - b.time);
}

export const MATRIX_GLYPHS = '01#@$%&*+=~?<>{}[]X0Z79¥§ΔΨΩ░▒▓█';

/**
 * Scrambles a single word into Matrix / cyber glyphs, preserving punctuation.
 *
 * @param {string} word
 * @param {number} [tick=0]
 * @returns {string} Scrambled word
 */
export function matrixScrambleWord(word, tick = 0) {
  if (!word || typeof word !== 'string') return '';
  let res = '';
  for (let i = 0; i < word.length; i++) {
    const ch = word[i];
    if (/[.,!?:;"'()\[\]\s\-\/\\]/.test(ch)) {
      res += ch;
    } else {
      const rand = Math.floor(Math.abs(Math.sin(i * 9.1 + (tick || 0) * 0.35 + ch.charCodeAt(0) * 3.7) * 1000));
      res += MATRIX_GLYPHS[rand % MATRIX_GLYPHS.length];
    }
  }
  return res;
}

/**
 * Scrambles an entire line into Matrix code, preserving whitespace and punctuation.
 *
 * @param {string} text
 * @param {number} [tick=0]
 * @returns {string} Scrambled line
 */
export function scrambleLine(text, tick = 0) {
  if (!text || typeof text !== 'string') return '';
  const tokens = text.match(/\S+|\s+/g) || [text];
  return tokens.map((token, idx) => {
    if (/^\s+$/.test(token)) return token;
    return matrixScrambleWord(token, tick + idx * 7);
  }).join('');
}

/**
 * Formats a lyric line where upcoming words are scrambled in Matrix code,
 * and as playback reaches each word, it un-matrixes into the clean lyrical text.
 *
 * @param {object} item Timed lyric item { time: number, text: string, words?: Array<{ time: number, text: string }> }
 * @param {number} currentTime Current playback position in seconds
 * @param {object|null} [nextItem] Next lyric item for duration estimation
 * @param {number} [totalDuration=0] Total track duration in seconds
 * @param {object} [options] Styling options
 * @returns {string} Formatted markup string
 */
export function formatKaraokeText(item, currentTime, nextItem = null, totalDuration = 0, options = {}) {
  if (!item || !item.text) return '';

  const safeText = String(item.text).replace(/[{}]/g, '').trim();
  if (!safeText) return '';

  const sungColor = options.sungColor || '#33ff33';
  const activeWordColor = options.activeWordColor || '#ffd24d';
  const peakWordColor = options.peakWordColor || '#ffffff';
  const cipherColor = options.cipherColor || '#475466';
  const tick = options.tick || Math.floor((currentTime || 0) * 30);

  const startTime = item.time || 0;

  // Calculate estimated line singing duration
  let lineDuration;
  if (nextItem && nextItem.time > startTime) {
    const rawGap = nextItem.time - startTime;
    const estimated = Math.max(2.5, Math.min(8.0, safeText.length * 0.18 + 0.6));
    lineDuration = rawGap <= 8.0 ? rawGap : Math.min(rawGap, estimated);
  } else if (totalDuration > startTime) {
    const rem = totalDuration - startTime;
    const estimated = Math.max(3.0, safeText.length * 0.18 + 0.6);
    lineDuration = Math.min(rem, estimated);
  } else {
    lineDuration = Math.max(3.0, safeText.length * 0.18 + 0.6);
  }

  // If line hasn't started yet: entire line is scrambled matrix code
  if (currentTime < startTime) {
    return `{${cipherColor}-fg}${scrambleLine(safeText, tick)}{/${cipherColor}-fg}`;
  }

  // If line has finished singing: entire line is fully un-matrixed & sung (green)
  if (currentTime >= startTime + lineDuration) {
    return `{bold}{${sungColor}-fg}${safeText}{/${sungColor}-fg}{/bold}`;
  }

  // Tokenize into words and whitespace delimiters
  const tokens = safeText.match(/\S+|\s+/g) || [safeText];
  const wordTokens = tokens.filter((t) => /\S/.test(t));
  const totalWords = Math.max(1, wordTokens.length);

  // If enhanced word-level timestamps exist
  if (Array.isArray(item.words) && item.words.length > 0) {
    let result = '';
    for (let w = 0; w < item.words.length; w++) {
      const wordObj = item.words[w];
      const wordText = String(wordObj.text || '').replace(/[{}]/g, '');
      if (!wordText) continue;

      const wStart = wordObj.time;
      let wEnd;
      if (w < item.words.length - 1) {
        wEnd = item.words[w + 1].time;
      } else {
        wEnd = startTime + lineDuration;
      }
      const wDuration = Math.max(0.1, wEnd - wStart);
      const wElapsed = currentTime - wStart;

      if (wElapsed <= 0) {
        // Upcoming word -> scrambled matrix code
        const trailing = wordText.endsWith(' ') ? ' ' : '';
        result += `{${cipherColor}-fg}${matrixScrambleWord(wordText.trimEnd(), tick + w * 7)}${trailing}{/${cipherColor}-fg}`;
      } else if (wElapsed >= wDuration) {
        // Fully un-matrixed sung word
        result += `{bold}{${sungColor}-fg}${wordText}{/${sungColor}-fg}{/bold}`;
      } else {
        // Active word: actively un-matrixing
        const wProgress = Math.min(1.0, Math.max(0, wElapsed / wDuration));
        const trailing = wordText.endsWith(' ') ? ' ' : '';
        if (wProgress < 0.35) {
          const matrixWord = matrixScrambleWord(wordText.trimEnd(), tick + w * 3);
          result += `{bold}{${activeWordColor}-fg}${matrixWord}${trailing}{/${activeWordColor}-fg}{/bold}`;
        } else {
          result += `{bold}{${peakWordColor}-fg}${wordText}{/${peakWordColor}-fg}{/bold}`;
        }
      }
    }
    return result;
  }

  // Standard line-level LRC: word-by-word un-matrix progression
  const elapsed = currentTime - startTime;
  const lineProgress = Math.min(1.0, Math.max(0, elapsed / lineDuration));

  let wordIndex = 0;
  let result = '';

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    if (/^\s+$/.test(token)) {
      result += token;
      continue;
    }

    const currentWordIdx = wordIndex++;
    const wordStartProgress = currentWordIdx / totalWords;
    const wordEndProgress = (currentWordIdx + 1) / totalWords;
    const wordDurationProgress = 1 / totalWords;

    if (lineProgress >= wordEndProgress) {
      // Word is un-matrixed & sung (green)
      result += `{bold}{${sungColor}-fg}${token}{/${sungColor}-fg}{/bold}`;
    } else if (lineProgress <= wordStartProgress) {
      // Word is still scrambled matrix code
      result += `{${cipherColor}-fg}${matrixScrambleWord(token, tick + currentWordIdx * 7)}{/${cipherColor}-fg}`;
    } else {
      // Word is actively un-matrixing in real-time
      const wordProgress = (lineProgress - wordStartProgress) / wordDurationProgress;
      if (wordProgress < 0.35) {
        // Matrix glitch phase (amber)
        const matrixWord = matrixScrambleWord(token, tick + currentWordIdx * 3);
        result += `{bold}{${activeWordColor}-fg}${matrixWord}{/${activeWordColor}-fg}{/bold}`;
      } else {
        // Un-matrix lock phase: snaps into crystal clear word (white)
        result += `{bold}{${peakWordColor}-fg}${token}{/${peakWordColor}-fg}{/bold}`;
      }
    }
  }

  return result;
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
