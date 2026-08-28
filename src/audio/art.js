import { readFile, readdir } from 'node:fs/promises';
import { dirname, join, extname } from 'node:path';
import { parseFile } from 'music-metadata';
import jpeg from 'jpeg-js';
import { PNG } from 'pngjs';

const MAX_ART_CACHE = 40;
const artCache = new Map();

function setArtCache(key, value) {
  if (artCache.has(key)) artCache.delete(key);
  else if (artCache.size >= MAX_ART_CACHE) {
    const oldestKey = artCache.keys().next().value;
    artCache.delete(oldestKey);
  }
  artCache.set(key, value);
}

function rgbToHex(r, g, b) {
  return '#' + [r, g, b].map((x) => {
    const hex = Math.min(255, Math.max(0, Math.round(x))).toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  }).join('');
}

/**
 * Decode image buffer (JPEG or PNG) to RGBA raw pixel array
 */
function decodeImage(buffer, format) {
  try {
    const isPng = format?.includes('png') || (buffer[0] === 0x89 && buffer[1] === 0x50);
    if (isPng) {
      const png = PNG.sync.read(buffer);
      return { width: png.width, height: png.height, data: png.data };
    }
    // Default to JPEG
    const decoded = jpeg.decode(buffer, { useTArray: true });
    return { width: decoded.width, height: decoded.height, data: decoded.data };
  } catch {
    return null;
  }
}

/**
 * Find cover file in track directory (cover.jpg, folder.jpg, front.png, etc.)
 */
async function findLocalFolderCover(trackPath) {
  try {
    const dir = dirname(trackPath);
    const entries = await readdir(dir);
    const coverNames = new Set(['cover.jpg', 'cover.jpeg', 'cover.png', 'folder.jpg', 'folder.png', 'front.jpg', 'front.png', 'album.jpg']);
    for (const file of entries) {
      if (coverNames.has(file.toLowerCase())) {
        const buf = await readFile(join(dir, file));
        return { data: buf, format: extname(file).slice(1) };
      }
    }
  } catch {}
  return null;
}

/**
 * Extract album artwork from file metadata or directory
 */
export async function extractArtworkBuffer(trackPath) {
  if (!trackPath) return null;
  if (artCache.has(trackPath)) {
    const cached = artCache.get(trackPath);
    // Refresh LRU position
    artCache.delete(trackPath);
    artCache.set(trackPath, cached);
    return cached;
  }

  try {
    const meta = await parseFile(trackPath, { skipCovers: false });
    if (meta.common.picture?.[0]) {
      const pic = meta.common.picture[0];
      const res = { data: pic.data, format: pic.format };
      setArtCache(trackPath, res);
      return res;
    }
  } catch {}

  // Fallback to local directory folder cover
  const localCover = await findLocalFolderCover(trackPath);
  if (localCover) {
    setArtCache(trackPath, localCover);
    return localCover;
  }

  setArtCache(trackPath, null);
  return null;
}

/**
 * Render image buffer into high-resolution ANSI half-block characters (▀)
 */
export function renderHalfBlockArt(imageBuffer, format, targetWidth = 26, targetHeight = 13) {
  const decoded = decodeImage(imageBuffer, format);
  if (!decoded) return null;

  const lines = [];
  const srcW = decoded.width;
  const srcH = decoded.height;

  for (let y = 0; y < targetHeight; y++) {
    let line = '';
    for (let x = 0; x < targetWidth; x++) {
      const srcX = Math.floor((x / targetWidth) * srcW);
      const srcYTop = Math.floor(((y * 2) / (targetHeight * 2)) * srcH);
      const srcYBot = Math.floor(((y * 2 + 1) / (targetHeight * 2)) * srcH);

      const idxTop = (srcYTop * srcW + srcX) * 4;
      const idxBot = (srcYBot * srcW + srcX) * 4;

      const fgHex = rgbToHex(decoded.data[idxTop], decoded.data[idxTop + 1], decoded.data[idxTop + 2]);
      const bgHex = rgbToHex(decoded.data[idxBot], decoded.data[idxBot + 1], decoded.data[idxBot + 2]);

      line += `{${fgHex}-fg}{${bgHex}-bg}▀{/${bgHex}-bg}{/${fgHex}-fg}`;
    }
    lines.push(line);
  }

  return lines.join('\n');
}

/**
 * Generate ASCII artwork for fallback or phosphor monitors
 */
export function renderAsciiArt(imageBuffer, format, targetWidth = 26, targetHeight = 13) {
  const decoded = decodeImage(imageBuffer, format);
  if (!decoded) return null;

  const chars = [' ', '░', '▒', '▓', '█'];
  const lines = [];

  for (let y = 0; y < targetHeight; y++) {
    let line = '';
    for (let x = 0; x < targetWidth; x++) {
      const srcX = Math.floor((x / targetWidth) * decoded.width);
      const srcY = Math.floor((y / targetHeight) * decoded.height);
      const idx = (srcY * decoded.width + srcX) * 4;
      const r = decoded.data[idx];
      const g = decoded.data[idx + 1];
      const b = decoded.data[idx + 2];
      const brightness = (r * 0.299 + g * 0.587 + b * 0.114) / 255;
      const charIdx = Math.min(chars.length - 1, Math.floor(brightness * chars.length));
      line += chars[charIdx];
    }
    lines.push(line);
  }

  return lines.join('\n');
}
