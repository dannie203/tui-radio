import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { Store } from '../src/state/store.js';

describe('Store Lyrics State Management', () => {
  test('initializes with default lyrics state', () => {
    const store = new Store();
    assert.equal(store.state.lyrics, null);
    assert.equal(store.state.lyricsStatus, 'idle');
    assert.equal(store.state.lyricsTrackId, null);
    assert.equal(store.state.lyricsVisible, false);
    assert.equal(store.state.lyricsScrollOffset, 0);
    assert.equal(store.state.lyricsSyncOffset, 0);
    assert.equal(store.state.activeLyricIndex, -1);
  });

  test('setLyrics updates state and status to found', () => {
    const store = new Store();
    let emitted = false;
    store.subscribe(() => { emitted = true; });

    const mockLyrics = {
      synced: [{ time: 10, text: 'Hello' }],
      plain: 'Hello',
      source: 'lrclib',
      isSynced: true
    };

    store.setLyrics('track-123', mockLyrics);

    assert.equal(emitted, true);
    assert.equal(store.state.lyricsTrackId, 'track-123');
    assert.equal(store.state.lyricsStatus, 'found');
    assert.deepEqual(store.state.lyrics, mockLyrics);
    assert.equal(store.state.activeLyricIndex, -1);
  });

  test('setLyricsStatus changes status and clears lyrics when unavailable', () => {
    const store = new Store();
    store.setLyrics('track-123', { plain: 'Song' });
    assert.equal(store.state.lyricsStatus, 'found');

    store.setLyricsStatus('unavailable', 'track-123');
    assert.equal(store.state.lyricsStatus, 'unavailable');
    assert.equal(store.state.lyrics, null);
  });

  test('clearLyrics resets all lyrics properties', () => {
    const store = new Store();
    store.setLyrics('track-123', { plain: 'Words' });
    store.scrollLyrics(5);
    store.adjustLyricsSyncOffset(1.5);

    store.clearLyrics();
    assert.equal(store.state.lyrics, null);
    assert.equal(store.state.lyricsStatus, 'idle');
    assert.equal(store.state.lyricsTrackId, null);
    assert.equal(store.state.lyricsScrollOffset, 0);
    assert.equal(store.state.lyricsSyncOffset, 0);
    assert.equal(store.state.activeLyricIndex, -1);
  });

  test('toggleLyrics toggles visibility and resets scroll offset', () => {
    const store = new Store();
    assert.equal(store.state.lyricsVisible, false);

    const v1 = store.toggleLyrics();
    assert.equal(v1, true);
    assert.equal(store.state.lyricsVisible, true);

    store.scrollLyrics(10);
    assert.equal(store.state.lyricsScrollOffset, 10);

    const v2 = store.toggleLyrics();
    assert.equal(v2, false);
    assert.equal(store.state.lyricsVisible, false);

    store.toggleLyrics(true);
    assert.equal(store.state.lyricsVisible, true);
    assert.equal(store.state.lyricsScrollOffset, 0);
  });

  test('adjustLyricsSyncOffset updates sync offset cleanly', () => {
    const store = new Store();
    store.adjustLyricsSyncOffset(0.5);
    assert.equal(store.state.lyricsSyncOffset, 0.5);

    store.adjustLyricsSyncOffset(-1.0);
    assert.equal(store.state.lyricsSyncOffset, -0.5);
  });

  test('updateActiveLyric tracks time with sync offset', () => {
    const store = new Store();
    const mockLyrics = {
      synced: [
        { time: 10, text: 'First line' },
        { time: 20, text: 'Second line' },
        { time: 30, text: 'Third line' }
      ],
      isSynced: true
    };

    store.setLyrics('track-1', mockLyrics);

    assert.equal(store.updateActiveLyric(5), -1);
    assert.equal(store.state.activeLyricIndex, -1);

    assert.equal(store.updateActiveLyric(12), 0);
    assert.equal(store.state.activeLyricIndex, 0);

    assert.equal(store.updateActiveLyric(22), 1);
    assert.equal(store.state.activeLyricIndex, 1);

    // Apply offset of -5s (time 22s effectively becomes 17s -> line 0)
    store.adjustLyricsSyncOffset(-5);
    assert.equal(store.updateActiveLyric(22), 0);
    assert.equal(store.state.activeLyricIndex, 0);
  });
});
