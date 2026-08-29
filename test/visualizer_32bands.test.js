import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { VisualizerBallisticsEngine } from '../src/audio/visualizer_engine.js';
import { computeLogBands, computeFFT, getBandLUT, NUM_BANDS, ISO_FREQUENCIES } from '../src/audio/worker/dsp_worker.js';
import { formatCodecBadge, createLayout } from '../src/ui/layout.js';
import { Store } from '../src/state/store.js';

describe('32-Band Visualizer & EQ Spectrum Engine', () => {
  describe('ISO Frequencies & Band Mapping', () => {
    test('contains exactly 32 ISO standard center frequencies', () => {
      assert.equal(NUM_BANDS, 32);
      assert.equal(ISO_FREQUENCIES.length, 32);
      assert.deepEqual(ISO_FREQUENCIES, [
        20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630,
        800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 18000, 20000
      ]);
    });

    test('accurately accumulates energy into corresponding frequency bands at 44.1kHz', () => {
      const sampleRate = 44100;
      const fftSize = 2048;
      const binHz = sampleRate / fftSize; // ~21.533 Hz
      const magnitudes = new Float32Array(fftSize / 2).fill(0.001);

      // Inject strong signal at 1 kHz (bin ~46)
      const bin1k = Math.round(1000 / binHz);
      magnitudes[bin1k] = 1.0;

      const bands = computeLogBands(magnitudes, sampleRate);
      assert.equal(bands.length, 32);

      // Band index 17 is 1000 Hz (1.0k)
      const band1kIndex = 17;
      assert.ok(bands[band1kIndex] > 60, `Expected 1kHz band to be active (>60), got ${bands[band1kIndex]}`);
      
      // Far bands should be significantly lower
      assert.ok(bands[0] < bands[band1kIndex], 'Sub-bass band should be quieter than 1kHz signal');
      assert.ok(bands[31] < bands[band1kIndex], '20kHz band should be quieter than 1kHz signal');
    });

    test('computes FFT without throwing NaN or Infinity', () => {
      const size = 64;
      const real = new Float32Array(size);
      const imag = new Float32Array(size);
      
      // Sine wave input
      for (let i = 0; i < size; i++) {
        real[i] = Math.sin((2 * Math.PI * i) / 16);
      }

      computeFFT(real, imag);

      for (let i = 0; i < size; i++) {
        assert.ok(!Number.isNaN(real[i]) && Number.isFinite(real[i]));
        assert.ok(!Number.isNaN(imag[i]) && Number.isFinite(imag[i]));
      }
    });
  });

  describe('Dynamic Sample Rate Adaptation (Hi-Res & Low-Bitrate)', () => {
    test('accurately identifies 1kHz signal at 96kHz Hi-Res FLAC sample rate', () => {
      const sampleRate = 96000;
      const fftSize = 2048;
      const binHz = sampleRate / fftSize; // ~46.875 Hz
      const magnitudes = new Float32Array(fftSize / 2).fill(0.001);

      // Inject 1kHz tone at 96kHz bin (~21)
      const bin1k = Math.round(1000 / binHz);
      magnitudes[bin1k] = 1.0;

      const bands = computeLogBands(magnitudes, sampleRate);
      const band1kIndex = 17; // 1000 Hz
      assert.ok(bands[band1kIndex] > 60, `Expected 1kHz band active at 96kHz, got ${bands[band1kIndex]}`);
      assert.ok(bands[0] < bands[band1kIndex]);
    });

    test('handles 192kHz Ultra Hi-Res with fractional sub-bass interpolation without NaN', () => {
      const sampleRate = 192000;
      const fftSize = 2048;
      const magnitudes = new Float32Array(fftSize / 2).fill(0.001);
      magnitudes[1] = 0.8; // Low sub-bass energy in bin 1 (~93.75 Hz)

      const bands = computeLogBands(magnitudes, sampleRate);
      assert.equal(bands.length, 32);
      for (let i = 0; i < 32; i++) {
        assert.ok(!Number.isNaN(bands[i]), `Band ${i} was NaN`);
        assert.ok(bands[i] >= 0 && bands[i] <= 100);
      }
      // Sub-bass bands interpolated from bin 1 should have energy
      assert.ok(bands[3] > 0); // 40Hz
    });

    test('zeroes out bands above Nyquist for low-sample-rate audio (22.05kHz)', () => {
      const sampleRate = 22050; // Nyquist is 11,025 Hz
      const fftSize = 2048;
      const magnitudes = new Float32Array(fftSize / 2).fill(0.5);

      const bands = computeLogBands(magnitudes, sampleRate);
      assert.equal(bands.length, 32);

      // Bands >= 12.5kHz (indices 28, 29, 30, 31: 12.5k, 16k, 18k, 20k) are above 11.025kHz
      assert.equal(bands[29], 0, '16kHz band should be zeroed above 11.025kHz Nyquist');
      assert.equal(bands[30], 0, '18kHz band should be zeroed above 11.025kHz Nyquist');
      assert.equal(bands[31], 0, '20kHz band should be zeroed above 11.025kHz Nyquist');
    });

    test('caches LUT across identical sample rates for zero-allocation performance', () => {
      const lut1 = getBandLUT(44100);
      const lut2 = getBandLUT(44100);
      assert.equal(lut1, lut2, 'Should return cached reference when sample rate does not change');

      const lut96k = getBandLUT(96000);
      assert.notEqual(lut1, lut96k, 'Should create new LUT when sample rate changes');
    });
  });

  describe('Ballistics Engine Dynamics (Attack, Release, Peak Falloff)', () => {
    test('applies asymmetric EMA attack and release smoothing', () => {
      const engine = new VisualizerBallisticsEngine({
        numBands: 32,
        attackAlpha: 0.8,
        releaseAlpha: 0.2,
        peakHoldMs: 500,
        peakDecayRate: 50
      });

      // 1. Initial jump from 0 to 100 (attack phase)
      const step1 = engine.update({ rawBands: new Array(32).fill(100) });
      assert.equal(step1.bands[0], 80); // 0.8 * 100 = 80
      assert.equal(step1.peaks[0], 80);

      // 2. Continuous high signal stabilizes at 100
      let stepHigh;
      for (let i = 0; i < 10; i++) {
        stepHigh = engine.update({ rawBands: new Array(32).fill(100) });
      }
      assert.equal(stepHigh.bands[0], 100);
      assert.equal(stepHigh.peaks[0], 100);

      // 3. Drop to 0 (release phase - smooth falloff)
      const stepDrop = engine.update({ rawBands: new Array(32).fill(0) });
      assert.ok(stepDrop.bands[0] < 100 && stepDrop.bands[0] >= 75, `Expected smoothed falloff, got ${stepDrop.bands[0]}`);
      assert.equal(stepDrop.peaks[0], 100, 'Peak should hold at 100 during peakHoldMs');
    });

    test('resets ballistics state cleanly', () => {
      const engine = new VisualizerBallisticsEngine({ numBands: 32 });
      engine.update({ rawBands: new Array(32).fill(80), rawVuLeft: 90, rawVuRight: 85 });
      engine.reset();

      const result = engine.update({ rawBands: new Array(32).fill(0) });
      assert.equal(result.bands.every(v => v === 0), true);
      assert.equal(result.peaks.every(v => v === 0), true);
      assert.equal(result.vuLeft, 0);
      assert.equal(result.vuRight, 0);
    });
  });

  describe('UI Layout & Spool Animation Robustness', () => {
    test('renders cassette spool animations without ReferenceError when playing', async () => {
      const store = new Store();
      await store.initSettings();
      store.update({
        playing: true,
        paused: false,
        current: { title: 'Test Song', artist: 'Artist', type: 'local' }
      });

      const mockPlayer = {
        getTelemetry: () => ({
          vuLeft: 45,
          vuRight: 50,
          peakLeft: 55,
          peakRight: 60,
          eqBands: new Array(32).fill(40),
          eqPeaks: new Array(32).fill(50),
          spoolFrame: 2,
          timePos: 30,
          duration: 180,
          percentPos: 16,
          tapeCounter: '0:30 / 3:00',
          elapsedMs: 30000,
          hasDuration: true,
          audioCodec: 'FLAC',
          audioBitrate: 850,
          audioSampleRate: 96000
        }),
        on: () => {}
      };

      const layout = createLayout(store, {}, mockPlayer);
      assert.ok(layout.screen);
      layout.render(store.state);
      clearInterval(layout.animTimer);
      layout.screen.destroy();
      layout.screen.program?.input?.pause();
    });
  });
});
