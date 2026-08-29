import { Worker } from 'node:worker_threads';
import { createConnection } from 'node:net';
import { spawn, execFileSync } from 'node:child_process';
import { existsSync, unlinkSync } from 'node:fs';
import { EventEmitter } from 'node:events';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { homedir } from 'node:os';
import { VisualizerBallisticsEngine } from './visualizer_engine.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const SOCKET_PATH = '/tmp/hiphop-tui-mpv.sock';
const IPC_TIMEOUT = 3000;
const CONFIG_DIR = join(homedir(), '.config', 'hiphop-tui');
const COOKIES_FILE = join(CONFIG_DIR, 'cookies.txt');

export class MpvPlayer extends EventEmitter {
  constructor({ socketPath = SOCKET_PATH, config = null } = {}) {
    super();
    this.socketPath = socketPath;
    this.process = null;
    this.socket = null;
    this.buffer = '';
    this.requestId = 0;
    this.state = 'idle';
    this.closed = false;

    // Visualizer Ballistics Engine (Dual-Rate EMA + Gravity Peak Falloff)
    this.numBands = 32;
    const peakHoldMs = config?.visualizer?.peakHoldMs ?? 1100;
    const peakDecayRate = config?.visualizer?.peakDecayRate ?? 38;
    this.ballistics = new VisualizerBallisticsEngine({
      numBands: this.numBands,
      peakHoldMs,
      peakDecayRate
    });

    // Shared Memory Buffer: 34 elements (VU Left, VU Right, 32 EQ bands) (scaled x100 as Int32)
    this.sharedBuffer = new SharedArrayBuffer((2 + this.numBands) * Int32Array.BYTES_PER_ELEMENT);
    this.sharedView = new Int32Array(this.sharedBuffer);
    this.rawBands = new Float32Array(this.numBands);

    this.worker = null;
    this.elapsedMs = 0;
    this.lastTickTime = null;
    this.spoolFrame = 0;
    this.timePos = 0;
    this.lastTimePosUpdate = 0;
    this.duration = 0;
    this.percentPos = 0;
    this.audioCodec = '';
    this.audioBitrate = 0;
    this.audioSampleRate = 0;
    this.audioSampleFormat = '';
    this.currentItem = null;
    this.currentAf = '';

    this.dspConfig = {
      stereoMode: config?.dsp?.stereoMode || 'STEREO',
      dolbyMode: config?.dsp?.dolbyMode || 'DOLBY-B',
      tapeType: config?.dsp?.tapeType || 'TYPE-II',
      bassBoost: Boolean(config?.dsp?.bassBoost)
    };
  }

  updateBallistics(options = {}) {
    if (options.peakHoldMs !== undefined) this.ballistics.peakHoldMs = options.peakHoldMs;
    if (options.peakDecayRate !== undefined) this.ballistics.peakDecayRate = options.peakDecayRate;
    if (options.attackAlpha !== undefined) this.ballistics.attackAlpha = options.attackAlpha;
    if (options.releaseAlpha !== undefined) this.ballistics.releaseAlpha = options.releaseAlpha;
  }

  setSampleRate(rate) {
    if (rate && Number.isFinite(rate) && rate > 0) {
      this.audioSampleRate = rate;
      this.worker?.postMessage({
        type: 'SET_SAMPLE_RATE',
        sampleRate: rate
      });
    }
  }

  static isInstalled() {
    try { execFileSync('mpv', ['--version'], { stdio: 'ignore' }); return true; }
    catch { return false; }
  }

  async initWorker() {
    if (this.worker) {
      try { this.worker.terminate(); } catch {}
      this.worker = null;
    }
    const workerPath = join(__dirname, 'worker', 'dsp_worker.js');
    this.worker = new Worker(workerPath, {
      workerData: { sharedBuffer: this.sharedBuffer }
    });

    this.worker.on('error', (err) => this.emit('error', err));
    if (this.audioSampleRate) {
      this.setSampleRate(this.audioSampleRate);
    }
  }

  async start() {
    if (!MpvPlayer.isInstalled()) throw new Error('mpv is not installed or not on PATH');
    await this.initWorker();
    this.closed = false;

    // Check if an existing MPV daemon is already active on this IPC socket
    let connectedToExisting = false;
    if (existsSync(this.socketPath)) {
      try {
        await new Promise((resolve, reject) => {
          const timer = setTimeout(() => {
            this.socket?.destroy();
            reject(new Error('timeout'));
          }, 300);
          this.socket = createConnection(this.socketPath);
          this.socket.once('connect', () => {
            clearTimeout(timer);
            connectedToExisting = true;
            this.attachSocket();
            resolve();
          });
          this.socket.once('error', (err) => {
            clearTimeout(timer);
            this.socket?.destroy();
            this.socket = null;
            reject(err);
          });
        });
      } catch {
        this.cleanupSocket();
      }
    }

    if (!connectedToExisting) {
      try {
        execFileSync('pkill', ['-f', `input-ipc-server=${this.socketPath}`], { stdio: 'ignore' });
      } catch {}
      this.cleanupSocket();

      const mpvArgs = [
        '--no-video',
        '--idle=yes',
        '--really-quiet',
        '--ao=pipewire,pulse,alsa',
        '--audio-samplerate=0',
        '--demuxer-max-bytes=16MiB',
        '--demuxer-readahead-secs=4',
        '--cache=yes',
        '--ytdl-format=bestaudio/best',
        `--input-ipc-server=${this.socketPath}`
      ];

      if (existsSync(COOKIES_FILE)) {
        mpvArgs.push(`--ytdl-raw-options=cookies=${COOKIES_FILE}`);
      }

      this.process = spawn('mpv', mpvArgs, { stdio: ['ignore', 'ignore', 'pipe'] });

      this.process.on('exit', (code) => {
        this.setState('idle');
        if (!this.closed) this.emit('error', new Error(`mpv exited (${code ?? 'unknown'})`));
      });

      this.process.stderr.on('data', (chunk) => this.emit('log', chunk.toString().trim()));

      await new Promise((resolve, reject) => {
        const deadline = setTimeout(() => reject(new Error('Timed out waiting for mpv IPC')), IPC_TIMEOUT);
        const connect = () => {
          this.socket = createConnection(this.socketPath);
          this.socket.once('connect', () => {
            clearTimeout(deadline);
            this.attachSocket();
            resolve();
          });
          this.socket.once('error', () => {
            this.socket?.destroy();
            if (this.process) setTimeout(connect, 40);
          });
        };
        connect();
      });
    }

    let obsId = 1;
    this.command('observe_property', [obsId++, 'icy-title']);
    this.command('observe_property', [obsId++, 'media-title']);
    this.command('observe_property', [obsId++, 'metadata']);
    this.command('observe_property', [obsId++, 'pause']);
    this.command('observe_property', [obsId++, 'idle-active']);
    this.command('observe_property', [obsId++, 'cache-buffering-state']);
    this.command('observe_property', [obsId++, 'time-pos']);
    this.command('observe_property', [obsId++, 'playback-time']);
    this.command('observe_property', [obsId++, 'duration']);
    this.command('observe_property', [obsId++, 'percent-pos']);
    this.command('observe_property', [obsId++, 'eof-reached']);
    this.command('observe_property', [obsId++, 'audio-codec-name']);
    this.command('observe_property', [obsId++, 'audio-bitrate']);
    this.command('observe_property', [obsId++, 'audio-params']);
    this.command('observe_property', [obsId++, 'file-format']);
    this.command('set_property', ['volume', 80]);
    this.command('set_property', ['mute', false]);
    this.applyDsp({}, true);
  }

  setState(state) {
    if (this.state === state) return;
    this.state = state;
    if (state === 'idle') {
      this.lastTickTime = null;
      this.elapsedMs = 0;
      this.timePos = 0;
      this.duration = 0;
      this.percentPos = 0;
      this.audioCodec = '';
      this.audioBitrate = 0;
      this.ballistics.reset();
    }
    this.worker?.postMessage({
      type: 'SET_STATE',
      isPlaying: state === 'playing' || state === 'buffering'
    });
    this.emit('state', state);
  }

  attachSocket() {
    this.socket.setEncoding('utf8');
    this.socket.on('data', (chunk) => {
      this.buffer += chunk;
      let newline;
      while ((newline = this.buffer.indexOf('\n')) >= 0) {
        const line = this.buffer.slice(0, newline);
        this.buffer = this.buffer.slice(newline + 1);
        try { this.handleMessage(JSON.parse(line)); } catch { /* Ignore incomplete JSON */ }
      }
      if (this.buffer.length > 65536) this.buffer = '';
    });
    this.socket.on('error', (error) => this.emit('error', error));
  }

  handleMessage(message) {
    if (message.event === 'start-file') {
      this.timePos = 0;
      this.duration = this.currentItem?.duration || 0;
      this.percentPos = 0;
      this.audioCodec = this.currentItem?.codec || this.currentItem?.format || '';
      this.audioBitrate = this.currentItem?.bitrate || 0;
      this.audioSampleRate = this.currentItem?.sampleRate || 0;
      this.audioSampleFormat = '';
      this.setState('buffering');
    }
    if (message.event === 'file-loaded' || message.event === 'playback-restart') {
      this.setState('playing');
    }
    if (message.event === 'end-file') {
      this.setState('idle');
      if (message.reason === 'eof' || message.reason === 'error') {
        this.emit('ended');
      }
    }
    if (message.event !== 'property-change') return;
    if (message.name === 'icy-title') {
      this.setState('playing');
      this.emit('metadata', { 'icy-title': message.data });
    }
    if (message.name === 'media-title') this.emit('metadata', { title: message.data });
    if (message.name === 'metadata') this.emit('metadata', message.data || {});
    if (message.name === 'pause') this.setState(message.data ? 'paused' : 'playing');
    if (message.name === 'idle-active') {
      if (message.data === true) this.setState('idle');
      else if (message.data === false && this.state === 'idle') this.setState('playing');
    }
    if (message.name === 'cache-buffering-state') {
      if (message.data === true) this.setState('buffering');
      else if (message.data === false && this.state === 'buffering') this.setState('playing');
    }
    if ((message.name === 'time-pos' || message.name === 'playback-time') && typeof message.data === 'number') {
      this.timePos = message.data;
      this.lastTimePosUpdate = performance.now();
      if (this.state === 'buffering') this.setState('playing');
      this.emit('time-pos', this.timePos);
    }
    if (message.name === 'duration' && typeof message.data === 'number') {
      this.duration = message.data;
      this.emit('duration', this.duration);
    }
    if (message.name === 'percent-pos' && typeof message.data === 'number') {
      this.percentPos = message.data;
    }
    if (message.name === 'audio-codec-name' && typeof message.data === 'string' && message.data.trim()) {
      this.audioCodec = message.data.trim();
      this.emit('audio-codec', this.audioCodec);
    }
    if (message.name === 'audio-bitrate' && typeof message.data === 'number' && message.data > 0) {
      this.audioBitrate = Math.round(message.data / 1000);
      this.emit('audio-bitrate', this.audioBitrate);
    }
    if (message.name === 'audio-params' && message.data && typeof message.data === 'object') {
      if (message.data.samplerate) {
        this.setSampleRate(message.data.samplerate);
      }
      if (message.data.format) {
        this.audioSampleFormat = message.data.format;
      }
    }
    if (message.name === 'eof-reached' && message.data === true) {
      this.emit('ended');
    }
  }

  command(command, args = []) {
    if (!this.socket || this.socket.destroyed) return;
    this.socket.write(`${JSON.stringify({ command: [command, ...args], request_id: ++this.requestId })}\n`);
  }

  applyDsp(options = {}, force = false) {
    if (options.stereoMode) this.dspConfig.stereoMode = options.stereoMode;
    if (options.dolbyMode) this.dspConfig.dolbyMode = options.dolbyMode;
    if (options.tapeType) this.dspConfig.tapeType = options.tapeType;
    if (options.bassBoost !== undefined) this.dspConfig.bassBoost = options.bassBoost;

    const filters = [];

    // 1. Stereo soundstage DSP
    const normStereo = String(this.dspConfig.stereoMode || '').toUpperCase().trim();
    if (normStereo === 'MONO') {
      filters.push('lavfi=[pan=stereo|c0=0.5*c0+0.5*c1|c1=0.5*c0+0.5*c1]');
    } else if (normStereo === 'WIDE' || normStereo === '3D WIDE' || normStereo === 'STEREO-3D' || normStereo === '3D') {
      filters.push('lavfi=[stereotools=mlev=1.0:slev=1.45:base=0.35,equalizer=f=12000:width_type=h:width=4000:g=1.5]');
    }

    // 2. Mega Bass Boost
    if (this.dspConfig.bassBoost) {
      filters.push('equalizer=f=60:width_type=o:w=1.5:g=7');
      filters.push('equalizer=f=125:width_type=o:w=1.5:g=4');
    }

    // 3. Dolby Noise Reduction Profiles
    if (this.dspConfig.dolbyMode === 'DOLBY-B') {
      filters.push('equalizer=f=12000:width_type=h:width=2000:g=-3');
    } else if (this.dspConfig.dolbyMode === 'DOLBY-C') {
      filters.push('equalizer=f=10000:width_type=h:width=3000:g=-5');
    }

    // 4. Tape Bias Formulations
    if (this.dspConfig.tapeType === 'TYPE-I') {
      filters.push('equalizer=f=100:width_type=o:w=1:g=2');
    } else if (this.dspConfig.tapeType === 'TYPE-II') {
      filters.push('equalizer=f=8000:width_type=o:w=1:g=2');
    } else if (this.dspConfig.tapeType === 'TYPE-IV') {
      filters.push('equalizer=f=60:width_type=o:w=1:g=3');
      filters.push('equalizer=f=12000:width_type=o:w=1:g=3');
    }

    const afString = filters.join(',');
    if (force || afString !== this.currentAf) {
      this.currentAf = afString;
      this.command('set_property', ['af', afString]);
    }

    this.worker?.postMessage({
      type: 'SET_DSP',
      ...this.dspConfig
    });
  }

  play(url, item = null) {
    if (!url) return;
    this.setState('buffering');
    this.currentItem = item;
    this.elapsedMs = 0;
    this.lastTickTime = null;
    this.timePos = 0;
    this.duration = item?.duration || 0;
    this.percentPos = 0;
    this.audioCodec = item?.codec || item?.format || '';
    this.audioBitrate = item?.bitrate || 0;
    this.audioSampleRate = item?.sampleRate || 0;
    if (this.audioSampleRate) {
      this.setSampleRate(this.audioSampleRate);
    }
    this.audioSampleFormat = '';
    this.command('loadfile', [url, 'replace']);
    this.command('set_property', ['pause', false]);
    this.command('set_property', ['mute', false]);
    this.applyDsp();
  }

  togglePause() {
    if (this.state === 'paused') this.setState('playing');
    else if (this.state === 'playing') this.setState('paused');
    this.command('cycle', ['pause']);
  }

  seek(seconds, type = 'relative') {
    this.command('seek', [seconds, type]);
  }

  seekPercent(percent) {
    this.command('seek', [percent, 'absolute-percent']);
  }

  setVolume(volume) {
    this.command('set_property', ['volume', volume]);
  }

  stop() {
    this.command('stop');
    this.timePos = 0;
    this.duration = 0;
    this.percentPos = 0;
    this.audioCodec = '';
    this.audioBitrate = 0;
    this.currentItem = null;
    this.setState('idle');
  }

  getTelemetry() {
    const isPlaying = this.state === 'playing' || this.state === 'buffering';
    const now = Date.now();

    if (isPlaying) {
      if (this.lastTickTime) {
        this.elapsedMs += (now - this.lastTickTime);
      }
      this.lastTickTime = now;
      this.spoolFrame = Math.floor(this.elapsedMs / 160) % 4;
    } else {
      this.lastTickTime = null;
    }

    const effectiveDuration = this.duration > 0 ? this.duration : (this.currentItem?.duration || 0);
    const hasDuration = effectiveDuration > 0;

    let currentTimePos = this.timePos;
    if (isPlaying && this.state !== 'paused' && this.lastTimePosUpdate > 0) {
      const deltaSec = (performance.now() - this.lastTimePosUpdate) / 1000;
      if (deltaSec > 0 && deltaSec < 2.0) {
        currentTimePos = this.timePos + deltaSec;
      }
    }

    const currentSec = hasDuration ? currentTimePos : Math.floor(this.elapsedMs / 1000);
    const totalSec = hasDuration ? effectiveDuration : 0;

    const formatSec = (s) => {
      const m = Math.floor(s / 60);
      const rem = Math.floor(s % 60);
      return `${String(m).padStart(2, '0')}:${String(rem).padStart(2, '0')}`;
    };

    const tapeCounter = hasDuration
      ? `${formatSec(currentSec)} / ${formatSec(totalSec)}`
      : formatSec(currentSec);

    const percent = hasDuration
      ? Math.max(0, Math.min(100, Math.round((currentSec / totalSec) * 100)))
      : 0;

    // Zero-copy read from Shared Memory populated by Worker Thread
    const rawVuLeft = isPlaying ? Atomics.load(this.sharedView, 0) / 100 : 0;
    const rawVuRight = isPlaying ? Atomics.load(this.sharedView, 1) / 100 : 0;
    for (let i = 0; i < this.numBands; i++) {
      this.rawBands[i] = isPlaying ? Atomics.load(this.sharedView, 2 + i) / 100 : 0;
    }

    // Apply Asymmetric Dual-Rate EMA & Gravitational Ballistics Decay
    const smoothed = this.ballistics.update({ rawBands: this.rawBands, rawVuLeft, rawVuRight });

    return {
      vuLeft: smoothed.vuLeft,
      vuRight: smoothed.vuRight,
      peakLeft: smoothed.peakLeft,
      peakRight: smoothed.peakRight,
      eqBands: smoothed.bands,
      eqPeaks: smoothed.peaks,
      spoolFrame: this.spoolFrame,
      timePos: currentTimePos,
      duration: effectiveDuration,
      percentPos: percent,
      tapeCounter,
      elapsedMs: this.elapsedMs,
      hasDuration,
      audioCodec: this.audioCodec || this.currentItem?.codec || this.currentItem?.format || '',
      audioBitrate: this.audioBitrate || this.currentItem?.bitrate || 0,
      audioSampleRate: this.audioSampleRate || this.currentItem?.sampleRate || 0,
      audioSampleFormat: this.audioSampleFormat || ''
    };
  }

  cleanupSocket() {
    if (existsSync(this.socketPath)) unlinkSync(this.socketPath);
  }

  async close() {
    this.closed = true;
    this.setState('idle');
    this.worker?.terminate();
    this.worker = null;
    if (!this.process && !this.socket) {
      this.cleanupSocket();
      return;
    }
    this.stop();
    this.socket?.destroy();
    if (this.process && !this.process.killed) this.process.kill('SIGTERM');
    this.process = null;
    this.socket = null;
    this.cleanupSocket();
  }
}
