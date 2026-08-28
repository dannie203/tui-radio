import { readdir, stat, readFile } from 'node:fs/promises';
import { homedir, cpus } from 'node:os';
import { join, extname, basename, relative } from 'node:path';
import { parseFile } from 'music-metadata';

const AUDIO_EXTENSIONS = new Set(['.flac', '.mp3', '.wav', '.m4a', '.ogg', '.opus', '.aac', '.wma', '.alac', '.aiff']);
const PLAYLIST_EXTENSIONS = new Set(['.m3u', '.m3u8', '.json', '.pls']);

const yieldTick = () => new Promise((resolve) => setImmediate(resolve));

/**
 * Throttled worker pool to prevent saturating I/O or blocking the Event Loop
 */
async function asyncPool(limit, items, iteratorFn, onProgress) {
  const results = [];
  const executing = new Set();
  let completed = 0;

  for (const item of items) {
    const p = Promise.resolve().then(() => iteratorFn(item));
    results.push(p);
    executing.add(p);

    const clean = () => {
      executing.delete(p);
      completed++;
      if (onProgress) onProgress(completed, items.length);
    };
    p.then(clean, clean);

    if (executing.size >= limit) {
      await Promise.race(executing);
      await yieldTick();
    }
  }
  return Promise.all(results);
}

/**
 * Fallback metadata cleaner based on filename heuristics
 */
export function cleanTrackTitle(filename) {
  const nameWithoutExt = basename(filename, extname(filename));
  const cleanNumber = nameWithoutExt.replace(/^(\d+[\s.-]+)/, '');
  if (cleanNumber.includes(' - ')) {
    const parts = cleanNumber.split(' - ');
    return {
      artist: parts[0].trim(),
      title: parts.slice(1).join(' - ').trim(),
      trackNo: 1
    };
  }
  return {
    artist: 'Unknown Artist',
    title: cleanNumber.trim() || nameWithoutExt,
    trackNo: 1
  };
}

/**
 * Extract audio tags and technical audio properties
 */
export async function extractMetadata(fullPath) {
  const ext = extname(fullPath).toLowerCase();
  const formatExt = ext.slice(1).toUpperCase();

  try {
    const [fileStat, meta] = await Promise.all([
      stat(fullPath),
      parseFile(fullPath, { skipCovers: true, duration: true })
    ]);

    const common = meta.common || {};
    const format = meta.format || {};
    const fallback = cleanTrackTitle(fullPath);

    const title = (common.title && common.title.trim()) || fallback.title;
    const artist = (common.artist && common.artist.trim()) || (common.albumartist && common.albumartist.trim()) || fallback.artist;
    const albumArtist = (common.albumartist && common.albumartist.trim()) || artist;
    const album = (common.album && common.album.trim()) || 'Singles / Unknown Album';
    const trackNo = common.track?.no || fallback.trackNo || 1;
    const diskNo = common.disk?.no || 1;
    const year = common.year || (common.date ? parseInt(common.date, 10) : null) || null;
    const genre = (common.genre && common.genre[0]) || 'Hip-Hop / Urban';
    const duration = format.duration || 0;
    const bitrate = format.bitrate ? Math.round(format.bitrate / 1000) : 320;
    const sampleRate = format.sampleRate || 44100;
    const bitsPerSample = format.bitsPerSample || 16;
    const lossless = format.lossless ?? (formatExt === 'FLAC' || formatExt === 'WAV' || formatExt === 'ALAC');
    const embeddedLyrics = Array.isArray(common.lyrics) ? common.lyrics.join('\n') : (common.lyrics || null);

    return {
      id: `local:${fullPath}`,
      type: 'local',
      name: title,
      title,
      artist,
      albumArtist,
      album,
      trackNo,
      diskNo,
      year,
      genre,
      duration,
      path: fullPath,
      url: fullPath,
      format: formatExt,
      bitrate,
      sampleRate,
      bitsPerSample,
      lossless,
      size: fileStat.size,
      lyrics: embeddedLyrics,
      lyricsSource: embeddedLyrics ? 'embedded' : null,
      country: 'LOCAL',
      tags: `local, ${ext.slice(1)}, ${album}, ${artist}, ${genre}`
    };
  } catch {
    // Fallback if tag parsing fails or corrupted tags
    try {
      const fileStat = await stat(fullPath);
      const fallback = cleanTrackTitle(fullPath);
      return {
        id: `local:${fullPath}`,
        type: 'local',
        name: fallback.title,
        title: fallback.title,
        artist: fallback.artist,
        albumArtist: fallback.artist,
        album: 'Singles / Unknown Album',
        trackNo: 1,
        diskNo: 1,
        year: null,
        genre: 'Hip-Hop',
        duration: 0,
        path: fullPath,
        url: fullPath,
        format: formatExt,
        bitrate: 320,
        sampleRate: 44100,
        bitsPerSample: 16,
        lossless: formatExt === 'FLAC',
        size: fileStat.size,
        country: 'LOCAL',
        tags: `local, ${ext.slice(1)}`
      };
    } catch {
      return null;
    }
  }
}

/**
 * Parse .m3u / .m3u8 / .pls / .json playlist files
 */
export async function parsePlaylistFile(playlistPath, baseDir) {
  const ext = extname(playlistPath).toLowerCase();
  const playlistName = basename(playlistPath, ext);
  const trackPaths = [];

  try {
    const content = await readFile(playlistPath, 'utf8');
    if (ext === '.json') {
      const data = JSON.parse(content);
      const items = Array.isArray(data) ? data : data.tracks || [];
      for (const item of items) {
        const p = typeof item === 'string' ? item : item.path || item.url;
        if (p) trackPaths.push(p.startsWith('/') ? p : join(baseDir, p));
      }
    } else {
      // M3U / M3U8 / PLS
      const lines = content.split(/\r?\n/);
      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#')) {
          if (trimmed.startsWith('File') && trimmed.includes('=')) {
            // PLS format: File1=/path/to/song.flac
            const p = trimmed.split('=')[1].trim();
            trackPaths.push(p.startsWith('/') ? p : join(baseDir, p));
          } else {
            // M3U format
            trackPaths.push(trimmed.startsWith('/') ? trimmed : join(baseDir, trimmed));
          }
        }
      }
    }
  } catch {
    // Ignore corrupt playlist
  }

  return {
    id: `playlist:${playlistPath}`,
    name: playlistName,
    path: playlistPath,
    trackPaths
  };
}

/**
 * Recursively scan directory and construct hierarchical relational data structure
 */
export async function scanLibrary(baseDir = join(homedir(), 'Music'), onProgress) {
  const audioFiles = [];
  const playlistFiles = [];

  async function walk(currentDir) {
    try {
      const entries = await readdir(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        if (entry.name.startsWith('.')) continue; // skip hidden folders
        const fullPath = join(currentDir, entry.name);

        if (entry.isDirectory()) {
          await walk(fullPath);
        } else if (entry.isFile()) {
          const ext = extname(entry.name).toLowerCase();
          if (AUDIO_EXTENSIONS.has(ext)) {
            audioFiles.push(fullPath);
          } else if (PLAYLIST_EXTENSIONS.has(ext)) {
            playlistFiles.push(fullPath);
          }
        }
      }
    } catch {
      // Directory may not be accessible
    }
  }

  await walk(baseDir);

  if (onProgress) onProgress({ phase: 'discovered', count: audioFiles.length });

  // Parallel non-blocking extraction
  const concurrency = Math.max(4, Math.min(16, (cpus().length || 4) * 2));
  const rawTracks = await asyncPool(
    concurrency,
    audioFiles,
    (file) => extractMetadata(file),
    (done, total) => {
      if (onProgress) onProgress({ phase: 'parsing', done, total });
    }
  );

  const tracks = rawTracks.filter(Boolean);

  // Build relational database in memory
  const library = {
    artists: {},      // artistName -> { name, albums: string[], trackCount: number }
    albums: {},       // albumKey -> { key, title, artist, year, genre, format, lossless, trackIds: string[] }
    tracksById: {},   // trackId -> Track
    playlists: {},    // playlistName -> { id, name, path, trackIds: string[] }
    allTrackIds: []
  };

  for (const track of tracks) {
    library.tracksById[track.id] = track;

    const artistName = track.albumArtist || track.artist || 'Unknown Artist';
    const albumTitle = track.album || 'Singles / Unknown Album';
    const albumKey = `${artistName}::${albumTitle}`;

    // Index Artist
    if (!library.artists[artistName]) {
      library.artists[artistName] = {
        name: artistName,
        albums: new Set(),
        trackCount: 0
      };
    }
    library.artists[artistName].albums.add(albumKey);
    library.artists[artistName].trackCount++;

    // Index Album
    if (!library.albums[albumKey]) {
      library.albums[albumKey] = {
        key: albumKey,
        title: albumTitle,
        artist: artistName,
        year: track.year,
        genre: track.genre,
        format: track.format,
        lossless: track.lossless,
        trackIds: []
      };
    }
    library.albums[albumKey].trackIds.push(track.id);
  }

  // Sort tracks inside each album by Disc Number -> Track Number -> Title
  for (const album of Object.values(library.albums)) {
    album.trackIds.sort((idA, idB) => {
      const a = library.tracksById[idA];
      const b = library.tracksById[idB];
      return (a.diskNo - b.diskNo) || (a.trackNo - b.trackNo) || a.title.localeCompare(b.title);
    });
  }

  // Convert artist album sets to sorted arrays
  for (const artist of Object.values(library.artists)) {
    artist.albums = Array.from(artist.albums).sort((aKey, bKey) => {
      const albA = library.albums[aKey];
      const albB = library.albums[bKey];
      return (albA.year || 0) - (albB.year || 0) || albA.title.localeCompare(albB.title);
    });
  }

  // Construct sorted allTrackIds list (Artist -> Album -> Track)
  const sortedArtists = Object.keys(library.artists).sort((a, b) => a.localeCompare(b));
  for (const artistName of sortedArtists) {
    const artist = library.artists[artistName];
    for (const albumKey of artist.albums) {
      const album = library.albums[albumKey];
      library.allTrackIds.push(...album.trackIds);
    }
  }

  // Process and index custom playlists
  for (const plFile of playlistFiles) {
    const pl = await parsePlaylistFile(plFile, baseDir);
    const validTrackIds = pl.trackPaths
      .map((p) => `local:${p}`)
      .filter((id) => library.tracksById[id]);

    if (validTrackIds.length > 0) {
      library.playlists[pl.name] = {
        id: pl.id,
        name: pl.name,
        path: pl.path,
        trackIds: validTrackIds
      };
    }
  }

  return { library, tracks };
}

/**
 * Backward compatibility wrapper
 */
export async function scanDirectory(baseDir = join(homedir(), 'Music'), onProgress) {
  const { library, tracks } = await scanLibrary(baseDir, onProgress);
  return { library, tracks };
}
