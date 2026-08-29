import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import {
  normalizeMediaInput,
  detectPlatform,
  isDirectAudioStream
} from '../src/api/youtube.js';

describe('Universal Multi-Source Stream Resolver', () => {
  describe('URL Normalization & Multi-Platform Routing', () => {
    test('normalizes SoundCloud URLs and sc: prefixes', () => {
      assert.equal(
        normalizeMediaInput('soundcloud.com/artist/track'),
        'https://soundcloud.com/artist/track'
      );
      assert.equal(
        normalizeMediaInput('sc:octobersveryown/track-1'),
        'https://soundcloud.com/octobersveryown/track-1'
      );
    });

    test('normalizes Bandcamp domain formats', () => {
      assert.equal(
        normalizeMediaInput('artist.bandcamp.com/track/cool-song'),
        'https://artist.bandcamp.com/track/cool-song'
      );
      assert.equal(
        normalizeMediaInput('bc:artist/album'),
        'https://artist.bandcamp.com/album'
      );
    });

    test('normalizes Mixcloud and YouTube URLs', () => {
      assert.equal(
        normalizeMediaInput('mixcloud.com/dj/set-1'),
        'https://mixcloud.com/dj/set-1'
      );
      assert.equal(
        normalizeMediaInput('music.youtube.com/watch?v=abcdefghijk'),
        'https://www.youtube.com/watch?v=abcdefghijk'
      );
    });

    test('detects direct audio streams by file extension', () => {
      assert.equal(isDirectAudioStream('http://stream.radio.org/live.mp3'), true);
      assert.equal(isDirectAudioStream('https://cdn.audio.com/track.flac?token=123'), true);
      assert.equal(isDirectAudioStream('http://cdn.com/stream.m3u8'), true);
      assert.equal(isDirectAudioStream('https://youtube.com/watch?v=123'), false);
    });

    test('detects platform metadata and badges correctly', () => {
      const sc = detectPlatform('https://soundcloud.com/artist/track');
      assert.equal(sc.type, 'soundcloud');
      assert.equal(sc.platform, 'SoundCloud');
      assert.equal(sc.prefix, 'sc');

      const bc = detectPlatform('https://artist.bandcamp.com/album/tape');
      assert.equal(bc.type, 'bandcamp');
      assert.equal(bc.platform, 'Bandcamp');
      assert.equal(bc.prefix, 'bc');

      const mc = detectPlatform('https://mixcloud.com/dj/mixtape');
      assert.equal(mc.type, 'mixcloud');
      assert.equal(mc.platform, 'Mixcloud');

      const direct = detectPlatform('http://audio.org/live.flac');
      assert.equal(direct.type, 'stream');
      assert.equal(direct.format, 'STREAM');

      const yt = detectPlatform('https://youtube.com/watch?v=12345678901');
      assert.equal(yt.type, 'youtube');
      assert.equal(yt.format, 'OPUS');
    });
  });
});
