export class VisualizerBallisticsEngine {
  constructor({
    numBands = 32,
    attackAlpha = 0.72,
    releaseAlpha = 0.16,
    peakHoldMs = 1100, // Longer peak hold (1.1s)
    peakDecayRate = 38 // Smooth gentle peak descent
  } = {}) {
    this.numBands = numBands;
    this.attackAlpha = attackAlpha;
    this.releaseAlpha = releaseAlpha;
    this.peakHoldMs = peakHoldMs;
    this.peakDecayRate = peakDecayRate;

    // Smoothed values (EMA)
    this.smoothedBands = new Float32Array(numBands);
    this.smoothedVuLeft = 0;
    this.smoothedVuRight = 0;

    // Peak tracking
    this.peaks = new Float32Array(numBands);
    this.peakTimers = new Float64Array(numBands);
    this.peakLeft = 0;
    this.peakRight = 0;
    this.peakTimerLeft = 0;
    this.peakTimerRight = 0;

    this.lastTimestamp = performance.now();
    this.outBands = new Array(numBands).fill(0);
    this.outPeaks = new Array(numBands).fill(0);
  }

  reset() {
    this.smoothedBands.fill(0);
    this.smoothedVuLeft = 0;
    this.smoothedVuRight = 0;
    this.peaks.fill(0);
    this.peakTimers.fill(0);
    this.outBands.fill(0);
    this.outPeaks.fill(0);
    this.peakLeft = 0;
    this.peakRight = 0;
    this.peakTimerLeft = 0;
    this.peakTimerRight = 0;
    this.lastTimestamp = performance.now();
  }

  update(rawTelemetry) {
    const now = performance.now();
    const dt = Math.max(0.001, Math.min(0.1, (now - this.lastTimestamp) / 1000));
    this.lastTimestamp = now;

    const { rawBands = [], rawVuLeft = 0, rawVuRight = 0 } = rawTelemetry;

    // 1. Dual-Rate EMA Smoothing for Equalizer Spectrum
    for (let b = 0; b < this.numBands; b++) {
      const currentRaw = rawBands[b] || 0;
      const prevSmooth = this.smoothedBands[b];
      const alpha = currentRaw >= prevSmooth ? this.attackAlpha : this.releaseAlpha;
      this.smoothedBands[b] = alpha * currentRaw + (1 - alpha) * prevSmooth;

      // Peak Hold & Gravitational Decay
      if (this.smoothedBands[b] >= this.peaks[b]) {
        this.peaks[b] = this.smoothedBands[b];
        this.peakTimers[b] = now;
      } else if (now - this.peakTimers[b] > this.peakHoldMs) {
        this.peaks[b] = Math.max(0, this.peaks[b] - this.peakDecayRate * dt);
      }

      this.outBands[b] = Math.round(this.smoothedBands[b]);
      this.outPeaks[b] = Math.round(this.peaks[b]);
    }

    // 2. Dual-Rate EMA Smoothing for Stereo VU Meters (Left)
    const alphaL = rawVuLeft >= this.smoothedVuLeft ? this.attackAlpha : this.releaseAlpha;
    this.smoothedVuLeft = alphaL * rawVuLeft + (1 - alphaL) * this.smoothedVuLeft;

    if (this.smoothedVuLeft >= this.peakLeft) {
      this.peakLeft = this.smoothedVuLeft;
      this.peakTimerLeft = now;
    } else if (now - this.peakTimerLeft > this.peakHoldMs) {
      this.peakLeft = Math.max(0, this.peakLeft - this.peakDecayRate * dt);
    }

    // Dual-Rate EMA Smoothing for Stereo VU Meters (Right)
    const alphaR = rawVuRight >= this.smoothedVuRight ? this.attackAlpha : this.releaseAlpha;
    this.smoothedVuRight = alphaR * rawVuRight + (1 - alphaR) * this.smoothedVuRight;

    if (this.smoothedVuRight >= this.peakRight) {
      this.peakRight = this.smoothedVuRight;
      this.peakTimerRight = now;
    } else if (now - this.peakTimerRight > this.peakHoldMs) {
      this.peakRight = Math.max(0, this.peakRight - this.peakDecayRate * dt);
    }

    return {
      bands: this.outBands,
      peaks: this.outPeaks,
      vuLeft: Math.round(this.smoothedVuLeft),
      vuRight: Math.round(this.smoothedVuRight),
      peakLeft: Math.round(this.peakLeft),
      peakRight: Math.round(this.peakRight)
    };
  }
}
