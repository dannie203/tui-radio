import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { formatCodecBadge } from '../src/ui/layout.js';
import { fetchStations } from '../src/api/stations.js';

describe('Audio Codec & Progress Telemetry', () => {
  describe('formatCodecBadge', () => {
    test('formats radio station with MP3 codec and bitrate', () => {
      const item = { type: 'radio', codec: 'MP3', bitrate: 128 };
      assert.equal(formatCodecBadge(item), '[MP3 128k]');
    });

    test('formats radio station with AAC+ codec and bitrate', () => {
      const item = { type: 'radio', codec: 'AAC+', bitrate: 96 };
      assert.equal(formatCodecBadge(item), '[AAC+ 96k]');
    });

    test('formats radio station with dynamic telemetry override from mpv', () => {
      const item = { type: 'radio', codec: 'MP3', bitrate: 128 };
      const telemetry = { audioCodec: 'aac', audioBitrate: 192 };
      assert.equal(formatCodecBadge(item, telemetry), '[AAC 192k]');
    });

    test('formats local lossless Hi-Res FLAC with bit depth and sample rate', () => {
      const item = { type: 'local', format: 'FLAC', bitsPerSample: 24, sampleRate: 96000 };
      assert.equal(formatCodecBadge(item), '[FLAC 24/96k]');
    });

    test('formats local 16-bit 44.1kHz FLAC', () => {
      const item = { type: 'local', format: 'FLAC', bitsPerSample: 16, sampleRate: 44100 };
      assert.equal(formatCodecBadge(item), '[FLAC 16/44.1k]');
    });

    test('formats local lossy MP3 with bitrate', () => {
      const item = { type: 'local', format: 'MP3', bitrate: 320 };
      assert.equal(formatCodecBadge(item), '[MP3 320k]');
    });

    test('formats YouTube stream as OPUS with bitrate', () => {
      const item = { type: 'youtube', format: 'OPUS', codec: 'OPUS', bitrate: 160 };
      assert.equal(formatCodecBadge(item), '[OPUS 160k]');
    });

    test('formats standby when nothing is playing', () => {
      assert.equal(formatCodecBadge(null, null), '[STANDBY]');
    });
  });

  describe('Station Codec Normalization', () => {
    test('normalizes fallback stations with valid codec and bitrate', async () => {
      const mockFetch = async () => { throw new Error('Offline'); };
      const { stations } = await fetchStations({ fetchImpl: mockFetch });

      assert.ok(stations.length > 0);
      for (const s of stations) {
        assert.ok(s.codec, `Station ${s.name} should have a codec`);
        assert.equal(typeof s.codec, 'string');
        assert.ok(s.bitrate >= 0);
      }
    });
  });
});
