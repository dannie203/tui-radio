import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { StreamRecorder } from '../src/audio/recorder.js';
import { Store } from '../src/state/store.js';

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

  test('supports cancelling active recording jobs on demand', () => {
    const recorder = new StreamRecorder();
    assert.equal(recorder.isRecording(), false);

    const cancelRes = recorder.cancelRecording();
    assert.equal(cancelRes.success, false);

    // Mock an active job
    let killed = false;
    recorder.activeJobs.set('https://stream.test/audio.mp3', {
      process: { kill: () => { killed = true; } },
      cancelled: false
    });
    assert.equal(recorder.isRecording(), true);

    const cancelActive = recorder.cancelRecording('https://stream.test/audio.mp3');
    assert.equal(cancelActive.success, true);
    assert.equal(cancelActive.cancelled, true);
    assert.equal(killed, true);
    assert.equal(recorder.isRecording(), false);
  });

  test('store toggles recording state when recordCurrentTrack is called repeatedly', async () => {
    const store = new Store();
    store.state.current = {
      title: 'Resonance',
      artist: 'HOME',
      url: 'https://youtube.com/watch?v=mock'
    };

    // First call initiates recording
    const start = await store.recordCurrentTrack();
    assert.equal(store.state.recording, true);

    // Second call toggles & cancels recording immediately
    const cancel = await store.recordCurrentTrack();
    assert.equal(cancel.cancelled, true);
    assert.equal(store.state.recording, false);
    assert.ok(store.state.status.includes('Cancelled'));
  });

  test('refuses to convert or overwrite local library audio files', async () => {
    const store = new Store();
    store.state.current = {
      title: 'Hi-Res Symphonic Master',
      artist: 'Hans Zimmer',
      type: 'local',
      path: '/home/aki/Music/master.flac'
    };

    const res = await store.recordCurrentTrack();
    assert.equal(res.isLocal, true);
    assert.equal(store.state.recording, false);
    assert.ok(store.state.status.includes('untouched') || store.state.status.includes('preserved'));
  });
});
