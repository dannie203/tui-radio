import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { searchMusic, resolveMedia, normalizeYouTubeInput } from '../src/api/youtube.js';
import { Store, MODES } from '../src/state/store.js';

describe('YouTube Music Search & Resolver', () => {
  test('normalizeYouTubeInput normalizes music.youtube.com and protocol omissions', () => {
    assert.equal(
      normalizeYouTubeInput('https://music.youtube.com/watch?v=aKcZTyojUM8'),
      'https://www.youtube.com/watch?v=aKcZTyojUM8'
    );
    assert.equal(
      normalizeYouTubeInput('music.youtube.com/playlist?list=PL12345'),
      'https://www.youtube.com/playlist?list=PL12345'
    );
    assert.equal(
      normalizeYouTubeInput('youtu.be/aKcZTyojUM8'),
      'https://youtu.be/aKcZTyojUM8'
    );
    assert.equal(
      normalizeYouTubeInput('yt:aKcZTyojUM8'),
      'https://www.youtube.com/watch?v=aKcZTyojUM8'
    );
  });

  test('MODES includes YOUTUBE MUSIC as 4th mode', () => {
    assert.equal(MODES.length, 4);
    assert.equal(MODES[3], 'YOUTUBE MUSIC');
  });

  test('Store initializes and handles YOUTUBE MUSIC mode', () => {
    const store = new Store();
    assert.equal(store.state.youtubeResults.length, 0);
    assert.equal(store.state.youtubeQuery, '');

    const mockTracks = [
      {
        id: 'yt:1',
        title: 'Shook Ones Pt. II',
        artist: 'Mobb Deep',
        duration: 326,
        url: 'https://youtube.com/watch?v=1',
        isTopic: true
      },
      {
        id: 'yt:2',
        title: 'Survival of the Fittest',
        artist: 'Mobb Deep',
        duration: 224,
        url: 'https://youtube.com/watch?v=2',
        isTopic: false
      }
    ];

    store.setMode('YOUTUBE MUSIC');
    store.setYouTubeResults(mockTracks, 'Mobb Deep');

    assert.equal(store.state.mode, 'YOUTUBE MUSIC');
    assert.equal(store.state.youtubeResults.length, 2);
    assert.equal(store.state.youtubeQuery, 'Mobb Deep');

    const activeList = store.getActiveList();
    assert.equal(activeList.length, 2);
    assert.equal(activeList[0].title, 'Shook Ones Pt. II');

    const next = store.getNextItem();
    assert.equal(next.title, 'Survival of the Fittest');
  });

  test('searchMusic returns empty array for blank query', async () => {
    const res = await searchMusic('');
    assert.deepEqual(res, []);
  });
});
