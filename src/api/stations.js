import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const API_URL = 'https://de1.api.radio-browser.info/json/stations/bytag/hiphop';
const FALLBACK_URL = fileURLToPath(new URL('../../data/fallback.json', import.meta.url));
const REQUEST_HEADERS = { 'User-Agent': 'hiphop-radio-tui/1.0' };

function normalizeStation(station) {
  let codec = String(station.codec || '').trim().toUpperCase();
  if (!codec) {
    const url = (station.url_resolved || station.url || '').toLowerCase();
    if (url.includes('.aac') || url.includes('/aac') || url.includes('aac=')) codec = 'AAC';
    else if (url.includes('.ogg') || url.includes('/ogg') || url.includes('ogg=')) codec = 'OGG';
    else if (url.includes('.opus') || url.includes('/opus') || url.includes('opus=')) codec = 'OPUS';
    else if (url.includes('.flac') || url.includes('/flac') || url.includes('flac=')) codec = 'FLAC';
    else if (url.includes('.m3u8') || url.includes('/hls')) codec = 'HLS/AAC';
    else codec = 'MP3';
  }

  return {
    id: station.stationuuid || station.id || station.url,
    type: 'radio',
    name: String(station.name || 'Unnamed station').trim(),
    country: String(station.country || station.countrycode || 'Unknown').trim(),
    tags: String(station.tags || 'hip-hop').trim(),
    bitrate: Number(station.bitrate) || 0,
    codec,
    url: station.url_resolved || station.url,
    homepage: station.homepage || ''
  };
}

export function deduplicateStations(stations) {
  const seenUrls = new Set();
  const stationMap = new Map();

  for (const station of stations) {
    if (!station || !station.url || !station.name) continue;

    // Remove trailing slashes and normalize URL to prevent duplicate stream endpoints
    const cleanUrl = String(station.url).trim().toLowerCase().replace(/\/+$/, '');
    if (seenUrls.has(cleanUrl)) continue;
    seenUrls.add(cleanUrl);

    // Key by normalized name and country
    const key = `${station.name.trim().toLowerCase()}::${station.country.trim().toLowerCase()}`;
    const existing = stationMap.get(key);

    if (!existing) {
      stationMap.set(key, station);
    } else {
      // Prefer the stream with higher bitrate or HTTPS
      const existingBitrate = existing.bitrate || 0;
      const currentBitrate = station.bitrate || 0;

      if (currentBitrate > existingBitrate) {
        stationMap.set(key, station);
      } else if (currentBitrate === existingBitrate && station.url.startsWith('https://') && !existing.url.startsWith('https://')) {
        stationMap.set(key, station);
      }
    }
  }

  return Array.from(stationMap.values()).sort((left, right) => left.name.localeCompare(right.name));
}

async function loadFallback() {
  const data = JSON.parse(await readFile(FALLBACK_URL, 'utf8'));
  const normalized = data.map(normalizeStation).filter((station) => station.url);
  return deduplicateStations(normalized);
}

export async function fetchStations({ signal, fetchImpl = globalThis.fetch } = {}) {
  try {
    const response = await fetchImpl(API_URL, { signal, headers: REQUEST_HEADERS });
    if (!response.ok) throw new Error(`Radio-Browser returned ${response.status}`);
    const rawStations = (await response.json())
      .map(normalizeStation)
      .filter((station) => station.url && station.name);

    const stations = deduplicateStations(rawStations);
    if (stations.length > 0) return { stations, source: 'Radio-Browser' };
    throw new Error('Radio-Browser returned no stations');
  } catch (error) {
    return { stations: await loadFallback(), source: 'local fallback', error };
  }
}
