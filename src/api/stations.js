import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const API_SERVERS = [
  'https://all.api.radio-browser.info',
  'https://de1.api.radio-browser.info',
  'https://nl1.api.radio-browser.info',
  'https://at1.api.radio-browser.info'
];

const FALLBACK_URL = fileURLToPath(new URL('../../data/fallback.json', import.meta.url));
const REQUEST_HEADERS = { 'User-Agent': 'boombox-tui/2.2' };

const BLOCKED_PATTERNS = [
  /\brfa\b/i,
  /radio\s*free\s*asia/i,
  /\bbolsa\b/i,
  /\brfi\b/i,
  /\bvoa\b/i,
  /\bbbc\b/i,
  /viet\s*tan/i,
  /viettan/i,
  /dan\s*lam\s*bao/i,
  /sbtn/i,
  /calitoday/i,
  /channel\s*today/i,
  /saigon\s*nho/i,
  /little\s*saigon/i,
  /saigon\s*houston/i,
  /houston/i,
  /atlanta/i,
  /washington/i,
  /nguoi\s*viet\s*daily/i,
  /tieng\s*nuoc\s*toi/i,
  /que\s*huong\s*radio/i,
  /radio\s*chan\s*troi\s*moi/i,
  /vietnam\s*exile/i,
  /chinh\s*phu\s*quoc\s*gia/i,
  /vietnamese\s*national/i,
  /viet\s*radio\s*1560/i,
  /\brva\b/i,
  /radio\s*veritas/i,
  /\bcri\b/i,
  /china\s*radio/i,
  /kc\s*radio/i,
  /\bkali\b/i,
  /replay\s*news/i,
  /abdulbasit/i,
  /quran/i,
  /digital\s*radio\s*hongkong/i,
  /l0ves0n9scafe/i
];

export function isBlockedStation(station) {
  if (!station) return true;
  const target = `${station.name || ''} ${station.tags || ''} ${station.homepage || ''} ${station.url || ''}`.toLowerCase();
  return BLOCKED_PATTERNS.some((pattern) => pattern.test(target));
}

function normalizeStation(station) {
  if (!station || isBlockedStation(station)) return null;

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
    countryCode: String(station.countrycode || '').trim().toUpperCase(),
    tags: String(station.tags || 'international,radio').trim(),
    bitrate: Number(station.bitrate) || 0,
    codec,
    url: station.url_resolved || station.url,
    homepage: station.homepage || '',
    favicon: station.favicon || '',
    votes: Number(station.votes) || 0
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
  const normalized = data.map(normalizeStation).filter(Boolean).filter((station) => station.url);
  return deduplicateStations(normalized);
}

/**
 * Fetch international worldwide radio stations from Radio-Browser API mirrors
 */
export async function fetchStations({ signal, fetchImpl = globalThis.fetch, query, tag, country, limit = 600 } = {}) {
  const server = API_SERVERS[0];

  // Specific single query/tag/country endpoint if specified
  if (query) {
    const endpoint = `${server}/json/stations/byname/${encodeURIComponent(query)}?limit=${limit}`;
    try {
      const res = await fetchImpl(endpoint, { signal, headers: REQUEST_HEADERS });
      if (res.ok) {
        const raw = (await res.json()).map(normalizeStation).filter(Boolean).filter((s) => s.url && s.name);
        return { stations: deduplicateStations(raw), source: 'Radio-Browser' };
      }
    } catch {}
  }

  if (tag) {
    const endpoint = `${server}/json/stations/bytag/${encodeURIComponent(tag)}?limit=${limit}`;
    try {
      const res = await fetchImpl(endpoint, { signal, headers: REQUEST_HEADERS });
      if (res.ok) {
        const raw = (await res.json()).map(normalizeStation).filter(Boolean).filter((s) => s.url && s.name);
        return { stations: deduplicateStations(raw), source: 'Radio-Browser' };
      }
    } catch {}
  }

  // Worldwide multi-genre collection endpoints
  const endpoints = [
    `${server}/json/stations/topvote/150`,
    `${server}/json/stations/topclick/150`,
    `${server}/json/stations/bytag/lofi?limit=60`,
    `${server}/json/stations/bytag/synthwave?limit=60`,
    `${server}/json/stations/bytag/jazz?limit=60`,
    `${server}/json/stations/bytag/hiphop?limit=60`,
    `${server}/json/stations/bytag/rock?limit=60`,
    `${server}/json/stations/bytag/electronic?limit=60`,
    `${server}/json/stations/bytag/classical?limit=60`,
    `${server}/json/stations/bytag/pop?limit=60`,
    `${server}/json/stations/bycountry/Vietnam?limit=40`,
    `${server}/json/stations/bycountry/Japan?limit=40`
  ];

  try {
    const results = await Promise.allSettled(
      endpoints.map((url) =>
        fetchImpl(url, { signal, headers: REQUEST_HEADERS })
          .then((res) => (res.ok ? res.json() : []))
          .catch(() => [])
      )
    );

    const allRaw = [];
    for (const r of results) {
      if (r.status === 'fulfilled' && Array.isArray(r.value)) {
        allRaw.push(...r.value);
      }
    }

    if (allRaw.length > 0) {
      const normalized = allRaw.map(normalizeStation).filter(Boolean).filter((s) => s.url && s.name);
      const stations = deduplicateStations(normalized);
      if (stations.length > 0) {
        return { stations, source: 'Radio-Browser (Worldwide)' };
      }
    }

    throw new Error('No international stations returned from Radio-Browser');
  } catch (error) {
    return { stations: await loadFallback(), source: 'local fallback', error };
  }
}
