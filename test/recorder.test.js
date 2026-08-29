import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { StreamRecorder } from '../src/audio/recorder.js';

describe('Tape Recorder & Stream Ripper Engine', () => {
  test('handles invalid or empty track gracefully without throwing', async () => {
    const recorder = new StreamRecorder();
    const res = await recorder.recordTrack(null);
    assert.equal(res.success, false);
    assert.ok(res.error);
  });

  test('validates stream URL before attempting recording', async () => {
    const recorder = new StreamRecorder();
    const res = await recorder.recordTrack({
      title: 'Synthwave Night',
      artist: 'Kavinsky'
      // Missing url
    });
    assert.equal(res.success, false);
  });
});
