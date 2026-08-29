import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { MixtapeManager } from '../src/audio/mixtape.js';

describe('Mixtape & Custom Playlist Manager', () => {
  test('initializes with default Favorites Mixtape and performs CRUD operations', async () => {
    const manager = new MixtapeManager();
    await manager.init();

    const initial = manager.getMixtapes();
    assert.ok(initial.length >= 1);
    assert.ok(initial.some((m) => m.name.includes('Favorites')));

    // Create a new custom mixtape
    const custom = await manager.createMixtape('🔥 Cyber Night Drive');
    assert.ok(custom.id);
    assert.equal(custom.name, '🔥 Cyber Night Drive');
    assert.equal(custom.tracks.length, 0);

    // Add tracks from multi-sources
    const added1 = await manager.addTrackToMixtape(custom.id, {
      title: 'Resonance',
      artist: 'HOME',
      url: 'https://soundcloud.com/home-2001/resonance'
    });
    assert.equal(added1, true);

    const added2 = await manager.addTrackToMixtape(custom.id, {
      title: 'Midnight City',
      artist: 'M83',
      url: 'https://youtube.com/watch?v=123'
    });
    assert.equal(added2, true);

    // Deduplication check: adding same URL again should return false
    const duplicate = await manager.addTrackToMixtape(custom.id, {
      title: 'Resonance Duplicate',
      artist: 'HOME',
      url: 'https://soundcloud.com/home-2001/resonance'
    });
    assert.equal(duplicate, false);

    // Check count
    const updated = manager.getMixtape(custom.id);
    assert.equal(updated.tracks.length, 2);

    // Remove track
    const removed = await manager.removeTrackFromMixtape(custom.id, 0);
    assert.equal(removed, true);
    assert.equal(manager.getMixtape(custom.id).tracks.length, 1);

    // Delete mixtape
    const deleted = await manager.deleteMixtape(custom.id);
    assert.equal(deleted, true);
    assert.equal(manager.getMixtape(custom.id), null);
  });
});
