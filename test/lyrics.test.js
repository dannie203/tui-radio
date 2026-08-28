import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseLrc,
  cleanTitleAndArtist,
  findActiveLyricIndex,
  fetchLyrics
} from '../src/api/lyrics.js';

describe('Lyrics API & LRC Parser', () => {
  describe('parseLrc', () => {
    test('parses standard 2-digit centisecond timestamps', () => {
      const lrc = `
        [00:12.34] First line of song
        [01:05.50] Second line of song
        [02:30.00] Third line of song
      `;
      const result = parseLrc(lrc);
      assert.equal(result.length, 3);
      assert.deepEqual(result[0], { time: 12.34, text: 'First line of song' });
      assert.deepEqual(result[1], { time: 65.5, text: 'Second line of song' });
      assert.deepEqual(result[2], { time: 150, text: 'Third line of song' });
    });

    test('parses 3-digit millisecond timestamps and multiple timestamps per line', () => {
      const lrc = `
        [00:05.250] Intro beat
        [00:15.00][00:25.00] Repeated chorus hook
      `;
      const result = parseLrc(lrc);
      assert.equal(result.length, 3);
      assert.equal(result[0].time, 5.25);
      assert.equal(result[0].text, 'Intro beat');
      assert.equal(result[1].time, 15);
      assert.equal(result[1].text, 'Repeated chorus hook');
      assert.equal(result[2].time, 25);
      assert.equal(result[2].text, 'Repeated chorus hook');
    });

    test('ignores metadata tags and empty lines', () => {
      const lrc = `
        [ti:N.Y. State of Mind]
        [ar:Nas]
        [al:Illmatic]
        [00:10.00] Straight out the dungeons of rap
      `;
      const result = parseLrc(lrc);
      assert.equal(result.length, 1);
      assert.equal(result[0].text, 'Straight out the dungeons of rap');
      assert.equal(result[0].time, 10);
    });

    test('returns empty array for invalid input', () => {
      assert.deepEqual(parseLrc(''), []);
      assert.deepEqual(parseLrc(null), []);
      assert.deepEqual(parseLrc('Just plain text with no timestamps'), []);
    });
  });

  describe('cleanTitleAndArtist', () => {
    test('cleans YouTube video junk and brackets', () => {
      const cleaned = cleanTitleAndArtist(
        'Eminem - Lose Yourself (Official Music Video) [HD]',
        'EminemVEVO'
      );
      assert.equal(cleaned.artist, 'Eminem');
      assert.equal(cleaned.title, 'Lose Yourself');
      assert.equal(cleaned.cleanTitle, 'Lose Yourself');
    });

    test('strips Topic suffixes and features', () => {
      const cleaned = cleanTitleAndArtist(
        'Nas - N.Y. State of Mind (Audio) ft. DJ Premier',
        'Nas - Topic'
      );
      assert.equal(cleaned.artist, 'Nas');
      assert.equal(cleaned.cleanTitle, 'N.Y. State of Mind');
    });

    test('handles em-dash separated title with generic artist', () => {
      const cleaned = cleanTitleAndArtist(
        '2Pac – Ambitionz Az A Ridah [Explicit Audio]',
        'YouTube'
      );
      assert.equal(cleaned.artist, '2Pac');
      assert.equal(cleaned.title, 'Ambitionz Az A Ridah');
    });
  });

  describe('findActiveLyricIndex', () => {
    const synced = [
      { time: 10.0, text: 'Line 1' },
      { time: 20.0, text: 'Line 2' },
      { time: 35.5, text: 'Line 3' },
      { time: 50.0, text: 'Line 4' }
    ];

    test('returns -1 before the first timestamp', () => {
      assert.equal(findActiveLyricIndex(synced, 0), -1);
      assert.equal(findActiveLyricIndex(synced, 9.99), -1);
    });

    test('returns exact index when timestamp matches or between timestamps', () => {
      assert.equal(findActiveLyricIndex(synced, 10.0), 0);
      assert.equal(findActiveLyricIndex(synced, 15.0), 0);
      assert.equal(findActiveLyricIndex(synced, 20.0), 1);
      assert.equal(findActiveLyricIndex(synced, 34.0), 1);
      assert.equal(findActiveLyricIndex(synced, 35.5), 2);
    });

    test('returns last index for times beyond the last line', () => {
      assert.equal(findActiveLyricIndex(synced, 50.0), 3);
      assert.equal(findActiveLyricIndex(synced, 120.0), 3);
    });

    test('handles empty or missing array', () => {
      assert.equal(findActiveLyricIndex([], 10), -1);
      assert.equal(findActiveLyricIndex(null, 10), -1);
    });
  });

  describe('fetchLyrics', () => {
    test('successfully fetches and normalizes synced lyrics from LRCLIB get', async () => {
      const mockFetch = async (url) => {
        assert.match(url, /lrclib\.net\/api\/get/);
        return {
          ok: true,
          status: 200,
          json: async () => ({
            trackName: 'Shook Ones, Pt. II',
            artistName: 'Mobb Deep',
            syncedLyrics: '[00:10.00] Word up son, word up\n[00:15.00] To all the killers and a hundred dollar billers',
            plainLyrics: 'Word up son, word up\nTo all the killers and a hundred dollar billers'
          })
        };
      };

      const result = await fetchLyrics({
        artist: 'Mobb Deep',
        title: 'Shook Ones, Pt. II',
        fetchImpl: mockFetch
      });

      assert.equal(result.isSynced, true);
      assert.equal(result.source, 'lrclib');
      assert.equal(result.synced.length, 2);
      assert.equal(result.synced[0].time, 10);
      assert.equal(result.synced[0].text, 'Word up son, word up');
      assert.equal(result.artistName, 'Mobb Deep');
    });

    test('falls back to search when /api/get returns 404', async () => {
      const mockFetch = async (url) => {
        if (url.includes('/api/get')) {
          return { ok: false, status: 404 };
        }
        if (url.includes('/api/search')) {
          return {
            ok: true,
            status: 200,
            json: async () => [
              {
                trackName: 'Juicy',
                artistName: 'The Notorious B.I.G.',
                syncedLyrics: '[00:08.50] It was all a dream',
                plainLyrics: 'It was all a dream'
              }
            ]
          };
        }
        throw new Error('Unexpected URL');
      };

      const result = await fetchLyrics({
        artist: 'The Notorious B.I.G.',
        title: 'Juicy (Official Audio)',
        fetchImpl: mockFetch
      });

      assert.equal(result.isSynced, true);
      assert.equal(result.synced.length, 1);
      assert.equal(result.synced[0].text, 'It was all a dream');
    });

    test('handles missing lyrics gracefully with source none', async () => {
      const mockFetch = async () => ({
        ok: false,
        status: 404
      });

      const result = await fetchLyrics({
        artist: 'NonExistentArtist',
        title: 'NonExistentSong12345',
        fetchImpl: mockFetch
      });

      assert.equal(result.isSynced, false);
      assert.equal(result.source, 'none');
      assert.deepEqual(result.synced, []);
      assert.equal(result.plain, '');
    });

    test('handles network errors gracefully without crashing', async () => {
      const mockFetch = async () => {
        throw new Error('Connection refused');
      };

      const result = await fetchLyrics({
        artist: 'Wu-Tang Clan',
        title: 'C.R.E.A.M.',
        fetchImpl: mockFetch
      });

      assert.equal(result.isSynced, false);
      assert.equal(result.source, 'none');
      assert.equal(result.error, 'Connection refused');
    });

    test('re-throws abort signal when request is cancelled', async () => {
      const controller = new AbortController();
      controller.abort();

      const mockFetch = async (_url, { signal }) => {
        if (signal?.aborted) {
          const err = new Error('The operation was aborted');
          err.name = 'AbortError';
          throw err;
        }
        return { ok: true, json: async () => ({}) };
      };

      await assert.rejects(
        () => fetchLyrics({
          artist: 'Tupac',
          title: 'Changes',
          signal: controller.signal,
          fetchImpl: mockFetch
        }),
        { name: 'AbortError' }
      );
    });
  });
});
