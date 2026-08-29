import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { findActiveLyricIndex } from '../api/lyrics.js';
import {
  CONFIG_DIR,
  loadConfig,
  saveConfig,
  DEFAULT_CONFIG,
  SETTINGS_SECTIONS
} from './config.js';
import { loadSession, saveSession } from './session.js';

const FAVORITES_FILE = join(CONFIG_DIR, 'favorites.json');

export const MODES = ['LOCAL TRACKS', 'RADIO STATIONS', 'QUEUE', 'YOUTUBE MUSIC'];
export const NAV_ROOT_LEVELS = ['ARTISTS', 'ALBUMS', 'PLAYLISTS', 'ALL TRACKS'];

export const GENRE_FILTERS = [
  'ALL',
  'FAVORITES',
  'BOOM-BAP',
  '90s RAP',
  'LO-FI',
  'UNDERGROUND',
  'CLASSIC'
];

export const STEREO_MODES = ['STEREO', 'MONO', '3D WIDE'];
export const DOLBY_MODES = ['DOLBY-B', 'DOLBY-C', 'DOLBY-S', 'OFF'];
export const TAPE_TYPES = ['TYPE-II', 'TYPE-I', 'TYPE-IV'];

export class Store {
  constructor(initialConfig = DEFAULT_CONFIG) {
    this.config = structuredClone ? structuredClone(initialConfig || DEFAULT_CONFIG) : JSON.parse(JSON.stringify(initialConfig || DEFAULT_CONFIG));
    this.state = {
      mode: 'LOCAL TRACKS',
      library: {
        artists: {},
        albums: {},
        tracksById: {},
        playlists: {},
        allTrackIds: []
      },
      nav: {
        level: 'ARTISTS', // 'ARTISTS' | 'ALBUMS' | 'PLAYLISTS' | 'TRACKS' | 'ALL TRACKS'
        selectedArtist: null,
        selectedAlbumKey: null,
        selectedPlaylist: null,
        history: []
      },
      localViewItems: [],
      localTracks: [],
      filteredLocalTracks: [],
      stations: [],
      filteredStations: [],
      favorites: [],
      queue: [],
      queueIndex: -1,
      selectedIndex: 0,
      query: '',
      genreFilter: 'ALL',
      musicDir: this.config.library?.musicDir || join(homedir(), 'Music'),
      current: null,
      playing: false,
      paused: false,
      volume: this.config.dsp?.volume ?? 80,
      timePos: 0,
      duration: 0,
      percentPos: 0,
      metadata: 'Nothing playing',
      status: 'Ready',
      source: '',
      tapeCounter: '00:00',
      shuffle: false,
      repeat: 'off', // 'off' | 'all' | 'one'
      stereoMode: this.config.dsp?.stereoMode || 'STEREO',
      dolbyMode: this.config.dsp?.dolbyMode || 'DOLBY-B',
      tapeType: this.config.dsp?.tapeType || 'TYPE-II',
      bassBoost: Boolean(this.config.dsp?.bassBoost),
      config: this.config,
      settingsVisible: false,
      settingsSelectedIndex: 0,
      settingsSections: SETTINGS_SECTIONS,
      lyrics: null,
      lyricsStatus: 'idle', // 'idle' | 'loading' | 'found' | 'unavailable' | 'error'
      lyricsTrackId: null,
      lyricsVisible: false,
      lyricsScrollOffset: 0,
      lyricsSyncOffset: this.config.lyrics?.syncOffset || 0,
      activeLyricIndex: -1,
      youtubeResults: [],
      youtubeQuery: '',
      youtubeLoading: false
    };
    this.listeners = new Set();
  }

  subscribe(listener) {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  emit() {
    for (const listener of this.listeners) listener(this.state);
  }

  update(patch) {
    Object.assign(this.state, patch);
    this.emit();
  }

  async loadFavorites() {
    try {
      const data = JSON.parse(await readFile(FAVORITES_FILE, 'utf8'));
      this.state.favorites = Array.isArray(data) ? data : [];
      this.applyFilter();
    } catch (error) {
      if (error.code !== 'ENOENT') this.update({ status: `Favorites unavailable: ${error.message}` });
    }
  }

  async toggleFavorite(item) {
    if (!item) return;
    const track = item.raw || item;
    const exists = this.state.favorites.some((fav) => fav.id === track.id);
    const favorites = exists
      ? this.state.favorites.filter((fav) => fav.id !== track.id)
      : [...this.state.favorites, track];
    await mkdir(CONFIG_DIR, { recursive: true });
    await writeFile(FAVORITES_FILE, JSON.stringify(favorites, null, 2));
    this.state.favorites = favorites;
    const label = (track.title || track.name || 'Item').slice(0, 24);
    this.state.status = exists
      ? `Ejected "${label}" from Favorites ★`
      : `Loaded "${label}" into Favorites ★`;
    this.applyFilter();
  }

  setMode(mode) {
    if (!MODES.includes(mode)) return;
    this.state.mode = mode;
    this.state.selectedIndex = 0;
    this.state.query = '';
    this.state.status = `Deck Mode: [ ${mode} ]`;
    this.applyFilter();
  }

  cycleMode(delta = 1) {
    const currentIndex = Math.max(0, MODES.indexOf(this.state.mode));
    const nextIndex = (currentIndex + delta + MODES.length) % MODES.length;
    this.setMode(MODES[nextIndex]);
  }

  setLibrary(library, tracks = []) {
    this.state.library = library || {
      artists: {},
      albums: {},
      tracksById: {},
      playlists: {},
      allTrackIds: []
    };
    this.state.localTracks = tracks.length ? tracks : Object.values(this.state.library.tracksById);
    if (this.state.mode === 'LOCAL TRACKS' && this.state.localTracks.length === 0 && this.state.stations.length > 0) {
      this.state.mode = 'RADIO STATIONS';
    }
    this.applyFilter();
  }

  setLocalTracks(tracks) {
    this.state.localTracks = tracks;
    this.applyFilter();
  }

  setStations(stations, source) {
    this.state.stations = stations;
    this.state.source = source;
    this.applyFilter();
  }

  setGenre(genre) {
    if (!GENRE_FILTERS.includes(genre)) return;
    this.state.genreFilter = genre;
    this.state.selectedIndex = 0;
    this.state.status = `Genre Filter: [ ${genre} ]`;
    this.applyFilter();
  }

  cycleGenre(delta = 1) {
    const currentIndex = Math.max(0, GENRE_FILTERS.indexOf(this.state.genreFilter));
    const nextIndex = (currentIndex + delta + GENRE_FILTERS.length) % GENRE_FILTERS.length;
    this.setGenre(GENRE_FILTERS[nextIndex]);
  }

  setNavLevel(level) {
    if (!NAV_ROOT_LEVELS.includes(level)) return;
    this.state.nav.level = level;
    this.state.nav.selectedArtist = null;
    this.state.nav.selectedAlbumKey = null;
    this.state.nav.selectedPlaylist = null;
    this.state.nav.history = [];
    this.state.selectedIndex = 0;
    this.state.query = '';
    this.state.status = `Crates View: [ ${level} ]`;
    this.applyFilter();
  }

  cycleNavLevel(delta = 1) {
    const currentRoot = NAV_ROOT_LEVELS.includes(this.state.nav.level) ? this.state.nav.level : 'ARTISTS';
    const currentIdx = NAV_ROOT_LEVELS.indexOf(currentRoot);
    const nextIdx = (currentIdx + delta + NAV_ROOT_LEVELS.length) % NAV_ROOT_LEVELS.length;
    this.setNavLevel(NAV_ROOT_LEVELS[nextIdx]);
  }

  drillDown() {
    if (this.state.mode !== 'LOCAL TRACKS') return null;

    const list = this.getActiveList();
    const item = list[this.state.selectedIndex];
    if (!item) return null;

    if (item.type === 'artist') {
      this.state.nav.history.push({
        level: this.state.nav.level,
        selectedArtist: this.state.nav.selectedArtist,
        selectedAlbumKey: this.state.nav.selectedAlbumKey,
        selectedPlaylist: this.state.nav.selectedPlaylist,
        selectedIndex: this.state.selectedIndex
      });
      this.state.nav.level = 'ALBUMS';
      this.state.nav.selectedArtist = item.name;
      this.state.selectedIndex = 0;
      this.state.query = '';
      this.state.status = `Artist: [ ${item.name} ] (${item.albumCount} albums)`;
      this.applyFilter();
      return { action: 'navigate', level: 'ALBUMS' };
    }

    if (item.type === 'album') {
      this.state.nav.history.push({
        level: this.state.nav.level,
        selectedArtist: this.state.nav.selectedArtist,
        selectedAlbumKey: this.state.nav.selectedAlbumKey,
        selectedPlaylist: this.state.nav.selectedPlaylist,
        selectedIndex: this.state.selectedIndex
      });
      this.state.nav.level = 'TRACKS';
      this.state.nav.selectedAlbumKey = item.key;
      this.state.selectedIndex = 0;
      this.state.query = '';
      this.state.status = `Album: [ ${item.title} ] (${item.trackCount} tracks)`;
      this.applyFilter();
      return { action: 'navigate', level: 'TRACKS' };
    }

    if (item.type === 'playlist') {
      this.state.nav.history.push({
        level: this.state.nav.level,
        selectedArtist: this.state.nav.selectedArtist,
        selectedAlbumKey: this.state.nav.selectedAlbumKey,
        selectedPlaylist: this.state.nav.selectedPlaylist,
        selectedIndex: this.state.selectedIndex
      });
      this.state.nav.level = 'TRACKS';
      this.state.nav.selectedPlaylist = item.name;
      this.state.selectedIndex = 0;
      this.state.query = '';
      this.state.status = `Playlist: [ ${item.name} ] (${item.trackCount} tracks)`;
      this.applyFilter();
      return { action: 'navigate', level: 'TRACKS' };
    }

    if (item.type === 'track') {
      return { action: 'play', track: item.raw || item };
    }

    return null;
  }

  drillUp() {
    if (this.state.mode !== 'LOCAL TRACKS') return false;

    if (this.state.nav.history.length > 0) {
      const prev = this.state.nav.history.pop();
      this.state.nav.level = prev.level;
      this.state.nav.selectedArtist = prev.selectedArtist;
      this.state.nav.selectedAlbumKey = prev.selectedAlbumKey;
      this.state.nav.selectedPlaylist = prev.selectedPlaylist;
      this.applyFilter();
      const list = this.getActiveList();
      this.state.selectedIndex = Math.min(prev.selectedIndex, Math.max(0, list.length - 1));
      this.emit();
      return true;
    }

    if (this.state.nav.level !== 'ARTISTS') {
      this.setNavLevel('ARTISTS');
      return true;
    }

    return false;
  }

  cycleStereoMode(delta = 1) {
    let currentMode = this.state.stereoMode;
    if (currentMode === 'STEREO-3D' || currentMode === '3D' || currentMode === 'WIDE') currentMode = '3D WIDE';
    const currentIndex = Math.max(0, STEREO_MODES.indexOf(currentMode));
    const nextIndex = (currentIndex + delta + STEREO_MODES.length) % STEREO_MODES.length;
    this.state.stereoMode = STEREO_MODES[nextIndex];
    this.state.status = `Stereo Mode: [ ${this.state.stereoMode} ]`;
    if (this.config.dsp) this.config.dsp.stereoMode = this.state.stereoMode;
    this.emit();
    return this.state.stereoMode;
  }

  cycleDolbyMode(delta = 1) {
    const currentIndex = Math.max(0, DOLBY_MODES.indexOf(this.state.dolbyMode));
    const nextIndex = (currentIndex + delta + DOLBY_MODES.length) % DOLBY_MODES.length;
    this.state.dolbyMode = DOLBY_MODES[nextIndex];
    this.state.status = `Dolby NR: [ ${this.state.dolbyMode} ]`;
    this.emit();
    return this.state.dolbyMode;
  }

  cycleTapeType(delta = 1) {
    const currentIndex = Math.max(0, TAPE_TYPES.indexOf(this.state.tapeType));
    const nextIndex = (currentIndex + delta + TAPE_TYPES.length) % TAPE_TYPES.length;
    this.state.tapeType = TAPE_TYPES[nextIndex];
    this.state.status = `Tape Bias: [ ${this.state.tapeType} ]`;
    this.emit();
    return this.state.tapeType;
  }

  toggleBassBoost() {
    this.state.bassBoost = !this.state.bassBoost;
    this.state.status = `Mega Bass: ${this.state.bassBoost ? 'BOOST [🔊]' : 'FLAT'}`;
    this.emit();
    return this.state.bassBoost;
  }

  toggleShuffle() {
    this.state.shuffle = !this.state.shuffle;
    this.state.status = `Shuffle: ${this.state.shuffle ? 'ON [🔀]' : 'OFF'}`;
    this.emit();
  }

  toggleRepeat() {
    const modes = ['off', 'all', 'one'];
    const nextIdx = (modes.indexOf(this.state.repeat) + 1) % modes.length;
    this.state.repeat = modes[nextIdx];
    this.state.status = `Repeat: ${this.state.repeat.toUpperCase()} [🔁]`;
    this.emit();
  }

  addToQueue(item) {
    const target = item || this.selected();
    if (!target) return;

    if (target.type === 'track') {
      const track = target.raw || target;
      this.state.queue.push(track);
      this.state.status = `Added "${(track.title || track.name).slice(0, 20)}" to Queue [${this.state.queue.length}]`;
    } else if (target.type === 'album') {
      const album = this.state.library.albums[target.key];
      if (album?.trackIds) {
        for (const tid of album.trackIds) {
          const t = this.state.library.tracksById[tid];
          if (t) this.state.queue.push(t);
        }
        this.state.status = `Added Album "${target.title}" (${album.trackIds.length} tracks) to Queue`;
      }
    } else if (target.type === 'artist') {
      const artist = this.state.library.artists[target.name];
      if (artist?.albums) {
        let addedCount = 0;
        for (const aKey of artist.albums) {
          const alb = this.state.library.albums[aKey];
          if (alb?.trackIds) {
            for (const tid of alb.trackIds) {
              const t = this.state.library.tracksById[tid];
              if (t) { this.state.queue.push(t); addedCount++; }
            }
          }
        }
        this.state.status = `Added Artist "${target.name}" (${addedCount} tracks) to Queue`;
      }
    } else if (target.type === 'playlist') {
      const pl = this.state.library.playlists[target.name];
      if (pl?.trackIds) {
        for (const tid of pl.trackIds) {
          const t = this.state.library.tracksById[tid];
          if (t) this.state.queue.push(t);
        }
        this.state.status = `Added Playlist "${target.name}" (${pl.trackIds.length} tracks) to Queue`;
      }
    } else {
      // Direct track / station object
      this.state.queue.push(target);
      this.state.status = `Added "${(target.title || target.name).slice(0, 20)}" to Queue [${this.state.queue.length}]`;
    }

    this.emit();
  }

  removeFromQueue(index) {
    if (index >= 0 && index < this.state.queue.length) {
      const removed = this.state.queue.splice(index, 1)[0];
      this.state.status = `Removed "${(removed.title || removed.name).slice(0, 20)}" from Queue`;
      this.state.selectedIndex = Math.min(this.state.selectedIndex, Math.max(0, this.state.queue.length - 1));
      this.emit();
    }
  }

  clearQueue() {
    this.state.queue = [];
    this.state.queueIndex = -1;
    this.state.status = 'Queue cleared';
    this.emit();
  }

  filter(query) {
    this.state.query = query;
    this.state.selectedIndex = 0;
    this.applyFilter();
  }

  applyFilter() {
    const q = (this.state.query || '').trim().toLowerCase();
    const lib = this.state.library || { artists: {}, albums: {}, tracksById: {}, playlists: {} };

    // 1. Build Hierarchical View Items for LOCAL TRACKS
    const nav = this.state.nav;
    let viewItems = [];

    if (nav.level === 'ARTISTS') {
      const artists = Object.values(lib.artists || {}).sort((a, b) => a.name.localeCompare(b.name));
      viewItems = artists.map((art) => ({
        type: 'artist',
        id: `artist:${art.name}`,
        name: art.name,
        albumCount: art.albums?.length || 0,
        trackCount: art.trackCount || 0,
        raw: art
      }));
      if (q) {
        viewItems = viewItems.filter((i) => i.name.toLowerCase().includes(q));
      }
    } else if (nav.level === 'ALBUMS') {
      let albumKeys = nav.selectedArtist
        ? lib.artists[nav.selectedArtist]?.albums || []
        : Object.keys(lib.albums || {});

      viewItems = albumKeys.map((key) => {
        const alb = lib.albums[key] || { title: 'Unknown', artist: 'Unknown', trackIds: [] };
        return {
          type: 'album',
          id: `album:${key}`,
          key,
          title: alb.title,
          artist: alb.artist,
          year: alb.year,
          genre: alb.genre,
          format: alb.format,
          lossless: alb.lossless,
          trackCount: alb.trackIds?.length || 0,
          raw: alb
        };
      });
      if (q) {
        viewItems = viewItems.filter((i) =>
          i.title.toLowerCase().includes(q) || i.artist.toLowerCase().includes(q)
        );
      }
    } else if (nav.level === 'PLAYLISTS') {
      viewItems = Object.values(lib.playlists || {}).map((pl) => ({
        type: 'playlist',
        id: pl.id || `playlist:${pl.name}`,
        name: pl.name,
        trackCount: pl.trackIds?.length || 0,
        raw: pl
      }));
      if (q) {
        viewItems = viewItems.filter((i) => i.name.toLowerCase().includes(q));
      }
    } else if (nav.level === 'TRACKS' || nav.level === 'ALL TRACKS') {
      let trackIds = [];
      if (nav.selectedAlbumKey) {
        trackIds = lib.albums[nav.selectedAlbumKey]?.trackIds || [];
      } else if (nav.selectedPlaylist) {
        trackIds = lib.playlists[nav.selectedPlaylist]?.trackIds || [];
      } else if (nav.selectedArtist) {
        const artist = lib.artists[nav.selectedArtist];
        if (artist) {
          for (const aKey of artist.albums) {
            const alb = lib.albums[aKey];
            if (alb) trackIds.push(...alb.trackIds);
          }
        }
      } else {
        trackIds = lib.allTrackIds?.length ? lib.allTrackIds : Object.keys(lib.tracksById || {});
      }

      viewItems = trackIds.map((id) => {
        const t = lib.tracksById[id] || { id, title: 'Unknown', artist: 'Unknown' };
        return {
          type: 'track',
          id: t.id,
          title: t.title || t.name,
          artist: t.artist,
          album: t.album,
          trackNo: t.trackNo || 1,
          duration: t.duration || 0,
          format: t.format || 'FLAC',
          bitrate: t.bitrate || 320,
          lossless: t.lossless,
          path: t.path,
          url: t.url || t.path,
          raw: t
        };
      });

      if (q) {
        viewItems = viewItems.filter((i) =>
          [i.title, i.artist, i.album].some((val) => String(val || '').toLowerCase().includes(q))
        );
      }
    }

    this.state.localViewItems = viewItems;

    // Filter flat local tracks (for compatibility)
    let tracks = this.state.localTracks;
    if (q) {
      tracks = tracks.filter((track) =>
        [track.title, track.artist, track.album, track.tags].some((val) =>
          String(val || '').toLowerCase().includes(q)
        )
      );
    }
    this.state.filteredLocalTracks = tracks;

    // 2. Filter radio stations
    const genre = this.state.genreFilter;
    let stations = this.state.stations;

    if (genre === 'FAVORITES') {
      const favIds = new Set(this.state.favorites.map((f) => f.id));
      stations = stations.filter((station) => favIds.has(station.id));
    } else if (genre === 'BOOM-BAP') {
      stations = stations.filter((station) => {
        const text = `${station.name} ${station.tags}`.toLowerCase();
        return text.includes('boom') || text.includes('bap') || text.includes('golden') || text.includes('underground');
      });
    } else if (genre === 'LO-FI') {
      stations = stations.filter((station) => {
        const text = `${station.name} ${station.tags}`.toLowerCase();
        return text.includes('lofi') || text.includes('lo-fi') || text.includes('chill') || text.includes('beats');
      });
    } else if (genre === '90s RAP') {
      stations = stations.filter((station) => {
        const text = `${station.name} ${station.tags}`.toLowerCase();
        return text.includes('90s') || text.includes('west coast') || text.includes('east coast') || text.includes('rap');
      });
    } else if (genre === 'UNDERGROUND') {
      stations = stations.filter((station) => {
        const text = `${station.name} ${station.tags}`.toLowerCase();
        return text.includes('underground') || text.includes('indie') || text.includes('drill') || text.includes('grime');
      });
    } else if (genre === 'CLASSIC') {
      stations = stations.filter((station) => {
        const text = `${station.name} ${station.tags}`.toLowerCase();
        return text.includes('classic') || text.includes('old school') || text.includes('80s') || text.includes('golden');
      });
    }

    if (q) {
      stations = stations.filter((station) =>
        [station.name, station.country, station.tags].some((val) =>
          String(val || '').toLowerCase().includes(q)
        )
      );
    }
    this.state.filteredStations = stations;

    // Constrain selected index
    const activeList = this.getActiveList();
    this.state.selectedIndex = Math.min(
      Math.max(0, this.state.selectedIndex),
      Math.max(0, activeList.length - 1)
    );
    this.emit();
  }

  getActiveList() {
    if (this.state.mode === 'LOCAL TRACKS') return this.state.localViewItems;
    if (this.state.mode === 'RADIO STATIONS') return this.state.filteredStations;
    if (this.state.mode === 'QUEUE') return this.state.queue;
    if (this.state.mode === 'YOUTUBE MUSIC') return this.state.youtubeResults;
    return [];
  }

  moveSelection(delta) {
    const list = this.getActiveList();
    if (!list.length) return;
    this.update({ selectedIndex: (this.state.selectedIndex + delta + list.length) % list.length });
  }

  selected() {
    const list = this.getActiveList();
    return list[this.state.selectedIndex];
  }

  getNextItem() {
    if (this.state.repeat === 'one' && this.state.current) {
      return this.state.current;
    }

    // Check queue first
    if (this.state.queue.length > 0) {
      if (this.state.current && this.state.queueIndex === -1) {
        const currId = this.state.current.id;
        const found = this.state.queue.findIndex((t) => t.id === currId || (t.url && t.url === this.state.current.url));
        if (found !== -1) this.state.queueIndex = found;
      }
      if (this.state.shuffle) {
        const nextIdx = Math.floor(Math.random() * this.state.queue.length);
        this.state.queueIndex = nextIdx;
        return this.state.queue[nextIdx];
      }
      if (this.state.queueIndex < this.state.queue.length - 1) {
        this.state.queueIndex++;
        return this.state.queue[this.state.queueIndex];
      }
      if (this.state.repeat === 'all') {
        this.state.queueIndex = 0;
        return this.state.queue[0];
      }
      return null;
    }

    if (this.state.mode === 'YOUTUBE MUSIC') {
      const list = this.state.youtubeResults;
      if (!list.length) return null;

      const currId = this.state.current?.id;
      let currIdx = list.findIndex((t) => t.id === currId);
      if (currIdx === -1) currIdx = this.state.selectedIndex;

      if (this.state.shuffle) {
        const randomIdx = Math.floor(Math.random() * list.length);
        this.state.selectedIndex = randomIdx;
        return list[randomIdx];
      }
      if (currIdx < list.length - 1) {
        this.state.selectedIndex = currIdx + 1;
        return list[currIdx + 1];
      }
      if (this.state.repeat === 'all') {
        this.state.selectedIndex = 0;
        return list[0];
      }
      return null;
    }

    // If currently inside an album/playlist in LOCAL TRACKS
    if (this.state.mode === 'LOCAL TRACKS') {
      const activeTracks = this.state.localViewItems.filter((i) => i.type === 'track');
      if (activeTracks.length > 0) {
        const currId = this.state.current?.id;
        const currIdx = activeTracks.findIndex((t) => t.id === currId);

        if (this.state.shuffle) {
          const randomIdx = Math.floor(Math.random() * activeTracks.length);
          return activeTracks[randomIdx].raw || activeTracks[randomIdx];
        }

        if (currIdx >= 0 && currIdx < activeTracks.length - 1) {
          return activeTracks[currIdx + 1].raw || activeTracks[currIdx + 1];
        }

        if (this.state.repeat === 'all') {
          return activeTracks[0].raw || activeTracks[0];
        }
      }

      // Fallback to library allTracks
      const allTracks = this.state.localTracks;
      if (!allTracks.length) return null;

      if (this.state.shuffle) {
        return allTracks[Math.floor(Math.random() * allTracks.length)];
      }
      const currIdx = allTracks.findIndex((t) => t.id === this.state.current?.id);
      if (currIdx >= 0 && currIdx < allTracks.length - 1) return allTracks[currIdx + 1];
      if (this.state.repeat === 'all') return allTracks[0];
      return null;
    }

    // Radio stations navigation
    const list = this.state.filteredStations;
    if (!list.length) return null;

    if (this.state.shuffle) {
      const randomIdx = Math.floor(Math.random() * list.length);
      this.state.selectedIndex = randomIdx;
      return list[randomIdx];
    }

    if (this.state.selectedIndex < list.length - 1) {
      this.state.selectedIndex++;
      return list[this.state.selectedIndex];
    }

    if (this.state.repeat === 'all') {
      this.state.selectedIndex = 0;
      return list[0];
    }

    return null;
  }

  getPrevItem() {
    if (this.state.queue.length > 0 && this.state.queueIndex > 0) {
      this.state.queueIndex--;
      return this.state.queue[this.state.queueIndex];
    }

    if (this.state.mode === 'YOUTUBE MUSIC') {
      const list = this.state.youtubeResults;
      if (!list.length) return null;
      if (this.state.selectedIndex > 0) {
        this.state.selectedIndex--;
        return list[this.state.selectedIndex];
      }
      this.state.selectedIndex = list.length - 1;
      return list[this.state.selectedIndex];
    }

    if (this.state.mode === 'LOCAL TRACKS') {
      const activeTracks = this.state.localViewItems.filter((i) => i.type === 'track');
      if (activeTracks.length > 0) {
        const currId = this.state.current?.id;
        const currIdx = activeTracks.findIndex((t) => t.id === currId);
        if (currIdx > 0) return activeTracks[currIdx - 1].raw || activeTracks[currIdx - 1];
        return activeTracks[activeTracks.length - 1].raw || activeTracks[activeTracks.length - 1];
      }
    }

    const list = this.state.mode === 'LOCAL TRACKS' ? this.state.filteredLocalTracks : this.state.filteredStations;
    if (!list.length) return null;

    if (this.state.selectedIndex > 0) {
      this.state.selectedIndex--;
      return list[this.state.selectedIndex];
    }

    this.state.selectedIndex = list.length - 1;
    return list[this.state.selectedIndex];
  }

  setYouTubeResults(results, query) {
    this.state.youtubeResults = results;
    this.state.youtubeQuery = query;
    this.state.youtubeLoading = false;
    this.state.selectedIndex = 0;
    this.state.status = `YouTube Music: ${results.length} tracks found for "${query}"`;
    this.emit();
  }

  setYouTubeLoading(loading, query) {
    this.state.youtubeLoading = loading;
    if (query) this.state.youtubeQuery = query;
    this.state.status = loading ? `Searching YouTube Music for "${query}"...` : this.state.status;
    this.emit();
  }

  setLyrics(trackId, lyrics) {
    this.state.lyrics = lyrics;
    this.state.lyricsTrackId = trackId;
    this.state.lyricsStatus = (lyrics && (lyrics.synced?.length > 0 || lyrics.plain)) ? 'found' : 'unavailable';
    this.state.lyricsScrollOffset = 0;
    this.state.lyricsSyncOffset = 0;
    this.state.activeLyricIndex = -1;
    this.emit();
  }

  setLyricsStatus(status, trackId = this.state.lyricsTrackId) {
    this.state.lyricsStatus = status;
    this.state.lyricsTrackId = trackId;
    if (status !== 'found' && status !== 'loading') {
      this.state.lyrics = null;
    }
    this.emit();
  }

  clearLyrics() {
    this.state.lyrics = null;
    this.state.lyricsStatus = 'idle';
    this.state.lyricsTrackId = null;
    this.state.lyricsScrollOffset = 0;
    this.state.lyricsSyncOffset = 0;
    this.state.activeLyricIndex = -1;
    this.emit();
  }

  toggleLyrics(forceState) {
    this.state.lyricsVisible = forceState !== undefined ? Boolean(forceState) : !this.state.lyricsVisible;
    if (this.state.lyricsVisible) {
      this.state.lyricsScrollOffset = 0;
    }
    this.state.status = this.state.lyricsVisible ? 'Deck View: [ 🎤 LIVE LYRICS DISPLAY ]' : 'Deck View: [ CASSETTE DECK ]';
    this.emit();
    return this.state.lyricsVisible;
  }

  scrollLyrics(delta) {
    this.state.lyricsScrollOffset += delta;
    this.emit();
  }

  adjustLyricsSyncOffset(delta) {
    this.state.lyricsSyncOffset = Number((this.state.lyricsSyncOffset + delta).toFixed(2));
    this.state.status = `Lyrics Sync Offset: ${this.state.lyricsSyncOffset >= 0 ? '+' : ''}${this.state.lyricsSyncOffset}s`;
    this.emit();
  }

  updateActiveLyric(timePos) {
    if (!this.state.lyrics?.synced || this.state.lyrics.synced.length === 0) {
      if (this.state.activeLyricIndex !== -1) {
        this.state.activeLyricIndex = -1;
        this.emit();
      }
      return -1;
    }

    const targetTime = Math.max(0, (timePos || 0) + this.state.lyricsSyncOffset);
    const index = findActiveLyricIndex(this.state.lyrics.synced, targetTime);
    if (this.state.activeLyricIndex !== index) {
      this.state.activeLyricIndex = index;
      this.emit();
    }
    return index;
  }

  async initSettings() {
    const loaded = await loadConfig();
    this.config = loaded;
    this.state.config = loaded;
    this.state.musicDir = loaded.library?.musicDir || this.state.musicDir;
    this.state.stereoMode = loaded.dsp?.stereoMode || this.state.stereoMode;
    this.state.dolbyMode = loaded.dsp?.dolbyMode || this.state.dolbyMode;
    this.state.tapeType = loaded.dsp?.tapeType || this.state.tapeType;
    this.state.bassBoost = Boolean(loaded.dsp?.bassBoost);
    this.state.lyricsSyncOffset = loaded.lyrics?.syncOffset || this.state.lyricsSyncOffset;
    this.emit();
    return this.config;
  }

  toggleSettings(forceState) {
    this.state.settingsVisible = forceState !== undefined ? Boolean(forceState) : !this.state.settingsVisible;
    if (this.state.settingsVisible) {
      this.state.status = 'Deck Menu: [ ⚙ SETTINGS & CONFIGURATION PANEL ]';
    } else {
      this.state.status = 'Deck View: [ CASSETTE DECK ]';
    }
    this.emit();
    return this.state.settingsVisible;
  }

  moveSettingsSelection(delta) {
    const total = this.state.settingsSections.length;
    if (total === 0) return;
    this.state.settingsSelectedIndex = (this.state.settingsSelectedIndex + delta + total) % total;
    this.emit();
  }

  cycleSettingValue(delta = 1) {
    const currentSection = this.state.settingsSections[this.state.settingsSelectedIndex];
    if (!currentSection) return null;

    const currentVal = currentSection.get(this.config);
    const idx = currentSection.options.indexOf(currentVal);
    const nextIdx = (idx + delta + currentSection.options.length) % currentSection.options.length;
    const nextVal = currentSection.options[nextIdx];

    currentSection.set(this.config, nextVal);
    this.state.config = { ...this.config };

    // Mirror to state if relevant
    if (currentSection.id === 'dsp.stereoMode') this.state.stereoMode = nextVal;
    if (currentSection.id === 'dsp.dolbyMode') this.state.dolbyMode = nextVal;
    if (currentSection.id === 'dsp.tapeType') this.state.tapeType = nextVal;
    if (currentSection.id === 'dsp.bassBoost') this.state.bassBoost = nextVal;

    this.saveCurrentConfig();
    this.state.status = `Updated: ${currentSection.label} ➔ ${currentSection.labels[nextIdx]}`;
    this.emit();
    return { section: currentSection, value: nextVal };
  }

  async saveCurrentConfig() {
    await saveConfig(this.config);
  }

  async loadSession() {
    try {
      const session = await loadSession();
      if (!session) return;
      if (session.mode && MODES.includes(session.mode)) this.state.mode = session.mode;
      if (session.nav) {
        this.state.nav.level = session.nav.level || this.state.nav.level;
        this.state.nav.selectedArtist = session.nav.selectedArtist || null;
        this.state.nav.selectedAlbumKey = session.nav.selectedAlbumKey || null;
        this.state.nav.selectedPlaylist = session.nav.selectedPlaylist || null;
      }
      if (session.queue && Array.isArray(session.queue)) {
        this.state.queue = session.queue;
        this.state.queueIndex = session.queueIndex ?? -1;
      }
      if (session.volume !== undefined) this.state.volume = session.volume;
      if (session.shuffle !== undefined) this.state.shuffle = session.shuffle;
      if (session.repeat) this.state.repeat = session.repeat;
      if (session.genreFilter && GENRE_FILTERS.includes(session.genreFilter)) this.state.genreFilter = session.genreFilter;
      if (session.stereoMode) this.state.stereoMode = session.stereoMode;
      if (session.dolbyMode) this.state.dolbyMode = session.dolbyMode;
      if (session.tapeType) this.state.tapeType = session.tapeType;
      if (session.bassBoost !== undefined) this.state.bassBoost = session.bassBoost;
      if (session.selectedIndex !== undefined) this.state.selectedIndex = session.selectedIndex;
      if (session.lyricsSyncOffset !== undefined) this.state.lyricsSyncOffset = session.lyricsSyncOffset;
      if (session.current) {
        this.state.current = session.current;
        this.state.timePos = session.timePos || 0;
        this.state.duration = session.current.duration || 0;
        const mins = Math.floor(this.state.timePos / 60);
        const secs = Math.floor(this.state.timePos % 60);
        this.state.tapeCounter = `${mins}:${String(secs).padStart(2, '0')}`;
        this.state.status = `Session Restored: [ ${session.current.title || session.current.name} ]`;
      }
      this.applyFilter();
    } catch {}
  }

  async saveSession() {
    await saveSession(this.state);
  }
}

