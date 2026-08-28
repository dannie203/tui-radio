import { parentPort, workerData } from 'node:worker_threads';

const FFT_SIZE = 2048;
const NUM_BANDS = 10;
const ISO_FREQUENCIES = [31.5, 63, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

// SharedArrayBuffer View for zero-copy memory transfer to main thread
// Layout: [0: vuLeft, 1: vuRight, 2..11: 10 EQ bands] (scaled x100 as Int32)
const sharedBuffer = workerData?.sharedBuffer;
const sharedView = sharedBuffer ? new Int32Array(sharedBuffer) : null;

// Precomputed Hann Window to eliminate spectral leakage
const HANN_WINDOW = new Float32Array(FFT_SIZE);
for (let i = 0; i < FFT_SIZE; i++) {
  HANN_WINDOW[i] = 0.5 * (1 - Math.cos((2 * Math.PI * i) / (FFT_SIZE - 1)));
}

// In-place Radix-2 Cooley-Tukey Fast Fourier Transform
export function computeFFT(real, imag) {
  const n = real.length;
  let j = 0;
  for (let i = 0; i < n - 1; i++) {
    if (i < j) {
      let tr = real[i]; real[i] = real[j]; real[j] = tr;
      let ti = imag[i]; imag[i] = imag[j]; imag[j] = ti;
    }
    let k = n >> 1;
    while (k <= j) {
      j -= k;
      k >>= 1;
    }
    j += k;
  }

  for (let len = 2; len <= n; len <<= 1) {
    const half = len >> 1;
    const angle = (-2 * Math.PI) / len;
    const wStepR = Math.cos(angle);
    const wStepI = Math.sin(angle);

    for (let i = 0; i < n; i += len) {
      let wR = 1.0;
      let wI = 0.0;
      for (let k = 0; k < half; k++) {
        const pos = i + k;
        const match = pos + half;
        const tr = wR * real[match] - wI * imag[match];
        const ti = wR * imag[match] + wI * real[match];

        real[match] = real[pos] - tr;
        imag[match] = imag[pos] - ti;
        real[pos] += tr;
        imag[pos] += ti;

        const nextWR = wR * wStepR - wI * wStepI;
        wI = wR * wStepI + wI * wStepR;
        wR = nextWR;
      }
    }
  }
}

// Calculate log-scaled ISO frequency bands
export function computeLogBands(magnitudes, sampleRate = 44100) {
  const binHz = sampleRate / FFT_SIZE;
  const bands = new Float32Array(NUM_BANDS);

  for (let b = 0; b < NUM_BANDS; b++) {
    const centerFreq = ISO_FREQUENCIES[b];
    const lowerFreq = centerFreq * Math.SQRT1_2;
    const upperFreq = centerFreq * Math.SQRT2;

    const startBin = Math.max(1, Math.floor(lowerFreq / binHz));
    const endBin = Math.min(magnitudes.length - 1, Math.ceil(upperFreq / binHz));

    let sum = 0;
    let count = 0;
    for (let bin = startBin; bin <= endBin; bin++) {
      sum += magnitudes[bin] * magnitudes[bin];
      count++;
    }
    const rms = count > 0 ? Math.sqrt(sum / count) : 0;
    const db = 20 * Math.log10(Math.max(1e-5, rms));
    bands[b] = Math.max(0, Math.min(100, (db + 60) * 1.66));
  }
  return bands;
}

// Audio telemetry generator running at high resolution (50Hz = 20ms)
let phase = 0;
let isPlaying = false;
let dspMultipliers = { bass: 1.0, metal: 1.0, stereoMode: 'STEREO' };

setInterval(() => {
  if (!isPlaying) {
    if (sharedView) {
      Atomics.store(sharedView, 0, 0);
      Atomics.store(sharedView, 1, 0);
      for (let b = 0; b < NUM_BANDS; b++) {
        Atomics.store(sharedView, 2 + b, 0);
      }
    }
    return;
  }

  phase += 0.28;

  const bMultiplier = dspMultipliers.bass;
  const mMultiplier = dspMultipliers.metal;

  const rawBands = [
    Math.min(100, (18 + Math.abs(Math.sin(phase * 1.3)) * 74 + Math.random() * 8) * bMultiplier),
    Math.min(100, (22 + Math.abs(Math.sin(phase * 1.1 + 0.4)) * 76 + Math.random() * 6) * bMultiplier),
    Math.min(100, (16 + Math.abs(Math.sin(phase * 0.9 + 0.8)) * 78 + Math.random() * 6) * bMultiplier),
    Math.min(100, (14 + Math.abs(Math.sin(phase * 1.5 + 1.2)) * 68 + Math.random() * 8)),
    Math.min(100, (12 + Math.abs(Math.sin(phase * 1.7 + 1.6)) * 62 + Math.random() * 6)),
    Math.min(100, (15 + Math.abs(Math.sin(phase * 1.4 + 2.0)) * 70 + Math.random() * 7)),
    Math.min(100, (14 + Math.abs(Math.sin(phase * 1.8 + 2.4)) * 66 + Math.random() * 8)),
    Math.min(100, (12 + Math.abs(Math.sin(phase * 2.1 + 2.8)) * 58 + Math.random() * 8)),
    Math.min(100, (10 + Math.abs(Math.sin(phase * 2.3 + 3.2)) * 52 + Math.random() * 8) * mMultiplier),
    Math.min(100, (8 + Math.abs(Math.sin(phase * 2.6 + 3.6)) * 46 + Math.random() * 6) * mMultiplier)
  ];

  let vuLeft = Math.min(100, rawBands[1] * 0.45 + rawBands[2] * 0.3 + rawBands[5] * 0.25 + (Math.random() * 6 - 3));
  let vuRight = Math.min(100, rawBands[1] * 0.42 + rawBands[3] * 0.32 + rawBands[6] * 0.26 + (Math.random() * 6 - 3));

  if (dspMultipliers.stereoMode === 'MONO') {
    const mono = (vuLeft + vuRight) * 0.5;
    vuLeft = mono;
    vuRight = mono;
  } else if (dspMultipliers.stereoMode === 'WIDE') {
    vuLeft = Math.min(100, vuLeft * 1.12);
    vuRight = Math.min(100, vuRight * 1.10);
  }

  if (sharedView) {
    Atomics.store(sharedView, 0, Math.round(vuLeft * 100));
    Atomics.store(sharedView, 1, Math.round(vuRight * 100));
    for (let b = 0; b < NUM_BANDS; b++) {
      Atomics.store(sharedView, 2 + b, Math.round(rawBands[b] * 100));
    }
  }
}, 20);

parentPort?.on('message', (msg) => {
  if (msg.type === 'SET_STATE') {
    isPlaying = msg.isPlaying;
  } else if (msg.type === 'SET_DSP') {
    dspMultipliers.bass = msg.bassBoost ? 1.25 : 1.0;
    dspMultipliers.metal = msg.tapeType === 'TYPE-IV' ? 1.2 : 1.0;
    dspMultipliers.stereoMode = msg.stereoMode || 'STEREO';
  }
});
