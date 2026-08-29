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

  describe('32-Band Equalizer Spectrum & Ballistics', () => {
    test('VisualizerBallisticsEngine handles 32 bands with peak decay', async () => {
      const { VisualizerBallisticsEngine } = await import('../src/audio/visualizer_engine.js');
      const engine = new VisualizerBallisticsEngine({ numBands: 32 });
      const rawBands = new Array(32).fill(60);

      const res = engine.update({ rawBands, rawVuLeft: 75, rawVuRight: 70 });
      assert.equal(res.bands.length, 32);
      assert.equal(res.peaks.length, 32);
      assert.ok(res.bands[0] > 0);
      assert.ok(res.peaks[0] >= res.bands[0]);
    });

    test('computeLogBands calculates 32 ISO frequency bands from magnitudes', async () => {
      const { computeLogBands, NUM_BANDS, ISO_FREQUENCIES } = await import('../src/audio/worker/dsp_worker.js');
      assert.equal(NUM_BANDS, 32);
      assert.equal(ISO_FREQUENCIES.length, 32);

      const fakeMags = new Float32Array(1024).fill(0.05);
      fakeMags[5] = 0.8; // Low frequency energy
      fakeMags[100] = 0.5; // Mid frequency energy
      fakeMags[300] = 0.3; // High frequency energy

      const bands = computeLogBands(fakeMags, 44100);
      assert.equal(bands.length, 32);
      for (let i = 0; i < 32; i++) {
        assert.ok(bands[i] >= 0 && bands[i] <= 100);
      }
    });
  });
});
