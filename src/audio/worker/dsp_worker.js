import { parentPort, workerData, isMainThread } from 'node:worker_threads';
import { spawn } from 'node:child_process';

const FFT_SIZE = 2048;
export const NUM_BANDS = 32;
export const ISO_FREQUENCIES = [
  20, 25, 31.5, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630,
  800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 18000, 20000
];

// SharedArrayBuffer View for zero-copy memory transfer to main thread
// Layout: [0: vuLeft, 1: vuRight, 2..33: 32 EQ bands] (scaled x100 as Int32)
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

// Preallocated reusable buffers to guarantee zero GC churn during FFT processing
const REUSABLE_REAL = new Float32Array(FFT_SIZE);
const REUSABLE_IMAG = new Float32Array(FFT_SIZE);
const REUSABLE_MAGS = new Float32Array(FFT_SIZE / 2);
const REUSABLE_BANDS = new Float32Array(NUM_BANDS);

// 1/3-octave band limits factor (2^(1/6) approx 1.122462)
const FACTOR_LOWER = Math.pow(2, -1 / 6);
const FACTOR_UPPER = Math.pow(2, 1 / 6);

// Precalculated 32-band Bin Range Table Cache (Dynamic Sample Rate)
let cachedSampleRate = 0;
let cachedBandLUT = null;

export function getBandLUT(sampleRate = 44100, fftSize = FFT_SIZE) {
  if (cachedSampleRate === sampleRate && cachedBandLUT) {
    return cachedBandLUT;
  }

  const binHz = sampleRate / fftSize;
  const nyquistHz = sampleRate / 2;
  const maxBin = (fftSize / 2) - 1;
  const lut = [];

  for (let b = 0; b < NUM_BANDS; b++) {
    const centerFreq = ISO_FREQUENCIES[b];
    const lowFreq = centerFreq * FACTOR_LOWER;
    const highFreq = centerFreq * FACTOR_UPPER;

    // Check if band is above Nyquist ceiling for current sample rate
    if (lowFreq >= nyquistHz) {
      lut.push({
        startBin: maxBin,
        endBin: maxBin,
        startFrac: maxBin,
        endFrac: maxBin,
        isAboveNyquist: true,
        tiltBoost: 1.0
      });
      continue;
    }

    const startBinExact = lowFreq / binHz;
    const endBinExact = Math.min(maxBin, highFreq / binHz);

    const startBin = Math.max(1, Math.floor(startBinExact));
    const endBin = Math.min(maxBin, Math.max(startBin, Math.ceil(endBinExact)));

    // ISO 226 / Pink-noise acoustic slope compensation (+4.5dB per octave)
    // Progressively boosts high frequencies from 0dB at 20Hz up to +34dB at 20kHz
    const tiltDb = Math.pow(b / (NUM_BANDS - 1), 0.85) * 34.0;

    lut.push({
      startBin,
      endBin,
      startFrac: startBinExact,
      endFrac: endBinExact,
      isAboveNyquist: false,
      tiltDb
    });
  }

  cachedSampleRate = sampleRate;
  cachedBandLUT = lut;
  return lut;
}

// Calculate log-scaled ISO 32 frequency bands with dynamic sample rate & fractional interpolation
export function computeLogBands(magnitudes, sampleRate = 44100, outBands = REUSABLE_BANDS) {
  const lut = getBandLUT(sampleRate, FFT_SIZE);
  const maxBin = (FFT_SIZE / 2) - 1;

  for (let b = 0; b < NUM_BANDS; b++) {
    const band = lut[b];

    if (band.isAboveNyquist) {
      outBands[b] = 0;
      continue;
    }

    let sum = 0;
    let maxMag = 0;
    let count = 0;

    if (band.startBin === band.endBin) {
      // Sub-bin fractional interpolation for narrow bass bands at high sample rates
      const binIdx = Math.min(maxBin, band.startBin);
      const nextBin = Math.min(maxBin, binIdx + 1);
      const frac = band.startFrac - Math.floor(band.startFrac);
      const interpMag = magnitudes[binIdx] * (1 - frac) + magnitudes[nextBin] * frac;
      sum = interpMag * interpMag;
      maxMag = interpMag;
      count = 1;
    } else {
      for (let bin = band.startBin; bin <= band.endBin; bin++) {
        const mag = magnitudes[bin];
        sum += mag * mag;
        if (mag > maxMag) maxMag = mag;
        count++;
      }
    }

    const rms = count > 0 ? Math.sqrt(sum / count) : 0;
    // Blend RMS with peak bin energy (65% RMS + 35% Peak) to preserve fast percussion/hi-hat transients in wide upper bands
    const effectiveMag = Math.max(1e-5, rms * 0.65 + maxMag * 0.35);
    const db = 20 * Math.log10(effectiveMag);

    // Apply acoustic tilt in dB before dynamic range scaling
    const equalizedDb = db + band.tiltDb;
    const rawVal = Math.max(0, Math.min(100, (equalizedDb + 66) * 1.55));
    outBands[b] = rawVal;
  }
  return outBands;
}

// Audio telemetry generator running at high resolution (50Hz = 20ms)
let phase = 0;
let isPlaying = false;
let currentSampleRate = 44100;
let dspMultipliers = { bass: 1.0, metal: 1.0, stereoMode: 'STEREO' };
let audioCaptureProc = null;
let pcmBuffer = Buffer.alloc(0);
let lastLiveAudioTime = 0;
let lastCaptureAttempt = 0;
const CAPTURE_RETRY_INTERVAL = 3000; // 3 seconds backoff between spawn retries
const MAX_PCM_BUFFER = FFT_SIZE * 4 * 2; // 16384 bytes (2 full FFT blocks)

function stopAudioCapture(resetAttempt = false) {
  if (audioCaptureProc) {
    try {
      audioCaptureProc.stdout?.destroy();
      audioCaptureProc.kill('SIGTERM');
    } catch {}
    audioCaptureProc = null;
  }
  pcmBuffer = Buffer.alloc(0);
  if (resetAttempt) {
    lastCaptureAttempt = 0;
  }
}

function startAudioCapture() {
  if (audioCaptureProc) return;
  const now = Date.now();
  if (now - lastCaptureAttempt < CAPTURE_RETRY_INTERVAL) return;
  lastCaptureAttempt = now;

  try {
    audioCaptureProc = spawn('pw-record', [
      '--rate', String(currentSampleRate || 44100),
      '--channels', '2',
      '--format', 's16',
      '--raw', '-'
    ], { stdio: ['ignore', 'pipe', 'ignore'] });

    audioCaptureProc.stdout.on('data', (chunk) => {
      if (!isPlaying || !sharedView) return;
      try {
        lastLiveAudioTime = Date.now();
        pcmBuffer = Buffer.concat([pcmBuffer, chunk]);

        // Guard against unbounded accumulation
        if (pcmBuffer.length > MAX_PCM_BUFFER * 2) {
          pcmBuffer = Buffer.from(pcmBuffer.subarray(pcmBuffer.length - MAX_PCM_BUFFER));
        }

        const bytesPerBlock = FFT_SIZE * 4; // 2048 samples * 2 channels * 2 bytes = 8192 bytes
        while (pcmBuffer.length >= bytesPerBlock) {
          const block = pcmBuffer.subarray(0, bytesPerBlock);
          pcmBuffer = Buffer.from(pcmBuffer.subarray(bytesPerBlock));

          let sumL = 0;
          let sumR = 0;

          for (let i = 0; i < FFT_SIZE; i++) {
            const left = block.readInt16LE(i * 4) / 32768;
            const right = block.readInt16LE(i * 4 + 2) / 32768;

            sumL += left * left;
            sumR += right * right;

            REUSABLE_REAL[i] = ((left + right) * 0.5) * HANN_WINDOW[i];
            REUSABLE_IMAG[i] = 0;
          }

          // 1. Calculate Real RMS for Left & Right Channels
          const rmsL = Math.sqrt(sumL / FFT_SIZE);
          const rmsR = Math.sqrt(sumR / FFT_SIZE);

          const dbL = 20 * Math.log10(Math.max(1e-5, rmsL));
          const dbR = 20 * Math.log10(Math.max(1e-5, rmsR));

          let vuLeft = Math.max(0, Math.min(100, (dbL + 50) * 2.0));
          let vuRight = Math.max(0, Math.min(100, (dbR + 50) * 2.0));

          const normStereo = String(dspMultipliers.stereoMode || '').toUpperCase().trim();
          if (normStereo === 'MONO') {
            const mono = (vuLeft + vuRight) * 0.5;
            vuLeft = mono;
            vuRight = mono;
          } else if (normStereo === 'WIDE' || normStereo === '3D WIDE' || normStereo === 'STEREO-3D' || normStereo === '3D') {
            const mid = (vuLeft + vuRight) * 0.5;
            const diffL = vuLeft - mid;
            const diffR = vuRight - mid;
            vuLeft = Math.max(0, Math.min(100, mid + diffL * 1.35 + 2));
            vuRight = Math.max(0, Math.min(100, mid + diffR * 1.35 + 2));
          }

          // 2. Real Radix-2 FFT Frequency Decomposition
          computeFFT(REUSABLE_REAL, REUSABLE_IMAG);

          // Hann coherent gain correction: 2.0 / (FFT_SIZE * 0.5) = 4.0 / FFT_SIZE
          const normFactor = 2.0 / (FFT_SIZE * 0.5);
          for (let i = 0; i < FFT_SIZE / 2; i++) {
            REUSABLE_MAGS[i] = Math.sqrt(REUSABLE_REAL[i] * REUSABLE_REAL[i] + REUSABLE_IMAG[i] * REUSABLE_IMAG[i]) * normFactor;
          }

          // 3. Compute 32 ISO Logarithmic Frequency Bands (Dynamic Sample Rate)
          computeLogBands(REUSABLE_MAGS, currentSampleRate || 44100, REUSABLE_BANDS);

          // 4. Apply Vintage DSP Multipliers
          const bMultiplier = dspMultipliers.bass;
          const mMultiplier = dspMultipliers.metal;

          for (let b = 0; b < NUM_BANDS; b++) {
            let factor = 1.0;
            if (b < 8 && bMultiplier > 1.0) factor = bMultiplier; // Bass: 20Hz - 100Hz
            if (b >= 24 && mMultiplier > 1.0) factor = mMultiplier; // Treble: 8kHz - 20kHz
            const bandVal = Math.min(100, REUSABLE_BANDS[b] * factor);
            Atomics.store(sharedView, 2 + b, Math.round(bandVal * 100));
          }

          Atomics.store(sharedView, 0, Math.round(vuLeft * 100));
          Atomics.store(sharedView, 1, Math.round(vuRight * 100));
        }
      } catch (err) {
        // Guard against any buffer decode issues without crashing worker
      }
    });

    audioCaptureProc.on('error', () => {
      stopAudioCapture(false);
    });

    audioCaptureProc.on('exit', () => {
      stopAudioCapture(false);
    });
  } catch {
    audioCaptureProc = null;
  }
}

if (!isMainThread) {
  // 50Hz falloff & fallback loop
  setInterval(() => {
    if (!isPlaying) {
      stopAudioCapture(false);
      if (sharedView) {
        Atomics.store(sharedView, 0, 0);
        Atomics.store(sharedView, 1, 0);
        for (let b = 0; b < NUM_BANDS; b++) {
          Atomics.store(sharedView, 2 + b, 0);
        }
      }
      return;
    }

    startAudioCapture();

    // If live audio capture is silent or unavailable, provide subtle standby animation
    if (Date.now() - lastLiveAudioTime > 500 && isPlaying && sharedView) {
      phase += 0.28;
      const bMultiplier = dspMultipliers.bass;
      const mMultiplier = dspMultipliers.metal;

      const fallbackBands = new Float32Array(NUM_BANDS);
      for (let b = 0; b < NUM_BANDS; b++) {
        const wave1 = Math.abs(Math.sin(phase * (1.1 + b * 0.06) + b * 0.32));
        const wave2 = Math.abs(Math.cos(phase * 1.8 + b * 0.45));
        let baseVal = 15 + wave1 * 50 + wave2 * 22 + Math.random() * 8;
        if (b < 8 && bMultiplier > 1.0) baseVal *= bMultiplier;
        if (b >= 24 && mMultiplier > 1.0) baseVal *= mMultiplier;
        fallbackBands[b] = Math.max(5, Math.min(100, baseVal));
      }

      let vuLeft = Math.min(100, fallbackBands[4] * 0.35 + fallbackBands[10] * 0.25 + fallbackBands[18] * 0.25 + fallbackBands[26] * 0.15);
      let vuRight = Math.min(100, fallbackBands[4] * 0.32 + fallbackBands[11] * 0.26 + fallbackBands[19] * 0.24 + fallbackBands[27] * 0.18);

      const normFallbackStereo = String(dspMultipliers.stereoMode || '').toUpperCase().trim();
      if (normFallbackStereo === 'MONO') {
        const mono = (vuLeft + vuRight) * 0.5;
        vuLeft = mono;
        vuRight = mono;
      } else if (normFallbackStereo === 'WIDE' || normFallbackStereo === '3D WIDE' || normFallbackStereo === 'STEREO-3D' || normFallbackStereo === '3D') {
        const mid = (vuLeft + vuRight) * 0.5;
        const diffL = vuLeft - mid;
        const diffR = vuRight - mid;
        vuLeft = Math.max(0, Math.min(100, mid + diffL * 1.35 + 2));
        vuRight = Math.max(0, Math.min(100, mid + diffR * 1.35 + 2));
      }

      Atomics.store(sharedView, 0, Math.round(vuLeft * 100));
      Atomics.store(sharedView, 1, Math.round(vuRight * 100));
      for (let b = 0; b < NUM_BANDS; b++) {
        Atomics.store(sharedView, 2 + b, Math.round(fallbackBands[b] * 100));
      }
    }
  }, 20);

  parentPort?.on('message', (msg) => {
    if (msg.type === 'SET_STATE') {
      isPlaying = msg.isPlaying;
      if (!isPlaying) {
        stopAudioCapture(false);
      } else {
        lastCaptureAttempt = 0;
        startAudioCapture();
      }
    } else if (msg.type === 'SET_DSP') {
      dspMultipliers.bass = msg.bassBoost ? 1.25 : 1.0;
      dspMultipliers.metal = msg.tapeType === 'TYPE-IV' ? 1.2 : 1.0;
      dspMultipliers.stereoMode = msg.stereoMode || 'STEREO';
    } else if (msg.type === 'SET_SAMPLE_RATE') {
      if (msg.sampleRate && Number.isFinite(msg.sampleRate) && msg.sampleRate > 0) {
        const newRate = Math.round(msg.sampleRate);
        if (currentSampleRate !== newRate) {
          currentSampleRate = newRate;
          if (audioCaptureProc) {
            stopAudioCapture(true);
            startAudioCapture();
          }
        }
      }
    }
  });
}
