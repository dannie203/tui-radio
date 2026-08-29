import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import {
  parseLrc,
  parseEnhancedWords,
  matrixScrambleWord,
  scrambleLine,
  formatKaraokeText,
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

  describe('parseEnhancedWords', () => {
    test('parses inline word-level timestamps in enhanced LRC lines', () => {
      const line = '<00:10.00>Never <00:10.50>gonna <00:11.00>give <00:11.50>you <00:12.00>up';
      const words = parseEnhancedWords(line, 10.0);
      assert.ok(words);
      assert.equal(words.length, 5);
      assert.equal(words[0].text, 'Never ');
      assert.equal(words[0].time, 10.0);
      assert.equal(words[1].text, 'gonna ');
      assert.equal(words[1].time, 10.5);
      assert.equal(words[4].text, 'up');
      assert.equal(words[4].time, 12.0);
    });

    test('returns null when no word-level timestamps are found', () => {
      assert.equal(parseEnhancedWords('Plain text line', 5.0), null);
      assert.equal(parseEnhancedWords('', 5.0), null);
      assert.equal(parseEnhancedWords(null, 5.0), null);
    });
  });

  describe('matrixScrambleWord & scrambleLine', () => {
    test('scrambles a single word into matrix glyphs while preserving length', () => {
      const word = 'Matrix';
      const scrambled = matrixScrambleWord(word, 5);
      assert.equal(scrambled.length, word.length);
      assert.notEqual(scrambled, word);
    });

    test('preserves punctuation inside or around words', () => {
      const word = "don't!";
      const scrambled = matrixScrambleWord(word, 2);
      assert.equal(scrambled.length, word.length);
      assert.equal(scrambled[3], "'");
      assert.equal(scrambled[5], "!");
    });

    test('scrambles entire lines while preserving whitespace structure', () => {
      const line = 'In the city, uh, you used to drive';
      const scrambled = scrambleLine(line, 10);
      assert.equal(scrambled.length, line.length);
      assert.equal(scrambled[2], ' ');
      assert.equal(scrambled[6], ' ');
      assert.equal(scrambled[11], ',');
      assert.notEqual(scrambled, line);
    });
  });

  describe('formatKaraokeText (Word-by-Word Un-matrixing)', () => {
    test('renders fully scrambled matrix text before start time', () => {
      const item = { time: 10.0, text: 'Hello World' };
      const nextItem = { time: 14.0, text: 'Next line' };
      const res = formatKaraokeText(item, 8.0, nextItem);
      assert.ok(res.startsWith('{#475466-fg}'));
      assert.ok(!res.includes('Hello'));
      assert.ok(!res.includes('World'));
    });

    test('renders completely un-matrixed highlighted text after duration finishes', () => {
      const item = { time: 10.0, text: 'Hello World' };
      const nextItem = { time: 14.0, text: 'Next line' };
      const res = formatKaraokeText(item, 15.0, nextItem);
      assert.equal(res, '{bold}{#33ff33-fg}Hello World{/#33ff33-fg}{/bold}');
    });

    test('un-matrixes word-by-word as song progresses', () => {
      const item = { time: 10.0, text: 'One Two Three' }; // 3 words
      const nextItem = { time: 16.0, text: 'Next' }; // duration 6s -> 2s per word

      // At 10.5s: First word 'One' is in matrix glitch phase (<35% of 2s), 'Two' and 'Three' are matrix scrambled
      const t1 = formatKaraokeText(item, 10.5, nextItem);
      assert.ok(t1.includes('{#ffd24d-fg}')); // Active matrix glitch color
      assert.ok(!t1.includes('Two')); // Upcoming words must be scrambled
      assert.ok(!t1.includes('Three'));

      // At 11.5s: First word 'One' is in un-matrix lock phase (white), 'Two' and 'Three' are still matrix scrambled
      const t2 = formatKaraokeText(item, 11.5, nextItem);
      assert.ok(t2.includes('{bold}{#ffffff-fg}One{/#ffffff-fg}{/bold}'));
      assert.ok(!t2.includes('Two'));

      // At 13.0s: First word 'One' is sung (green), second word 'Two' is active un-matrixing
      const t3 = formatKaraokeText(item, 13.0, nextItem);
      assert.ok(t3.includes('{bold}{#33ff33-fg}One{/#33ff33-fg}{/bold}'));
      assert.ok(!t3.includes('Three')); // 'Three' is still matrix scrambled

      // At 16.0s: All words are un-matrixed and fully sung (green)
      const t4 = formatKaraokeText(item, 16.0, nextItem);
      assert.equal(t4, '{bold}{#33ff33-fg}One Two Three{/#33ff33-fg}{/bold}');
    });

    test('handles enhanced word-level timestamps when present', () => {
      const item = {
        time: 10.0,
        text: 'One Two',
        words: [
          { time: 10.0, text: 'One ' },
          { time: 12.0, text: 'Two' }
        ]
      };
      const nextItem = { time: 14.0, text: 'Three' };

      // At 11.5s: 'One ' is un-matrixed (white), 'Two' is still matrix scrambled
      const res = formatKaraokeText(item, 11.5, nextItem);
      assert.ok(res.includes('{bold}{#ffffff-fg}One {/#ffffff-fg}{/bold}'));
      assert.ok(!res.includes('Two'));
    });

    test('returns empty string for missing or empty item', () => {
      assert.equal(formatKaraokeText(null, 10), '');
      assert.equal(formatKaraokeText({ text: '' }, 10), '');
    });
  });
});
