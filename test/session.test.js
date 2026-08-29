import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { Store } from '../src/state/store.js';
import { saveSession, loadSession } from '../src/state/session.js';

describe('Session Persistence & Restoration Engine', () => {
  test('saves and restores full playback session state', async () => {
    const store = new Store();
    store.state.mode = 'QUEUE';
    store.state.volume = 92;
    store.state.stereoMode = '3D WIDE';
    store.state.bassBoost = true;
    store.state.shuffle = true;
    store.state.repeat = 'all';
    store.state.current = {
      id: 'local:test:song1',
      title: 'Session Track 1',
      artist: 'Retro Artist',
      album: 'Tape Vol 1',
      duration: 210,
      type: 'local'
    };
    store.state.timePos = 65.5;
    store.state.queue = [
      { id: '1', title: 'Q1', type: 'local' },
      { id: '2', title: 'Q2', type: 'local' }
    ];
    store.state.queueIndex = 0;

    await store.saveSession();

    const loaded = await loadSession();
    assert.ok(loaded);
    assert.equal(loaded.mode, 'QUEUE');
    assert.equal(loaded.volume, 92);
    assert.equal(loaded.stereoMode, '3D WIDE');
    assert.equal(loaded.bassBoost, true);
    assert.equal(loaded.shuffle, true);
    assert.equal(loaded.repeat, 'all');
    assert.equal(loaded.current.title, 'Session Track 1');
    assert.equal(loaded.timePos, 65.5);
    assert.equal(loaded.queue.length, 2);

    // Test Store restore
    const freshStore = new Store();
    await freshStore.loadSession();
    assert.equal(freshStore.state.mode, 'QUEUE');
    assert.equal(freshStore.state.volume, 92);
    assert.equal(freshStore.state.stereoMode, '3D WIDE');
    assert.equal(freshStore.state.bassBoost, true);
    assert.equal(freshStore.state.current.title, 'Session Track 1');
    assert.equal(freshStore.state.timePos, 65.5);
    assert.equal(freshStore.state.queue.length, 2);
  });
});
