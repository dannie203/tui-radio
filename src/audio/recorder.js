import { spawn } from 'node:child_process';
import { existsSync, unlinkSync } from 'node:fs';
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

const RECORDINGS_DIR = join(homedir(), 'Music', 'Boombox Recordings');

function sanitizeFilename(name) {
  return String(name || 'Track')
    .replace(/[/:*?"<>|\\#]/g, '_')
    .trim()
    .slice(0, 80);
}

function sendNotification(title, message) {
  try {
    spawn('notify-send', ['-a', 'BOOMBOX RX-505', '-i', 'boombox', title, message], {
      detached: true,
      stdio: 'ignore'
    }).unref();
  } catch {}
}

export class StreamRecorder {
  constructor() {
    this.activeJobs = new Map();
    this.onFinishCallback = null;
  }

  setOnFinish(cb) {
    this.onFinishCallback = cb;
  }

  isRecording(url = null) {
    if (url) return this.activeJobs.has(url);
    return this.activeJobs.size > 0;
  }

  cancelRecording(url = null) {
    if (url && this.activeJobs.has(url)) {
      const job = this.activeJobs.get(url);
      job.cancelled = true;
      try { job.process?.kill('SIGKILL'); } catch {}
      this.activeJobs.delete(url);
      sendNotification('⏹️ Tape Recording Cancelled', 'Recording was stopped and discarded.');
      if (this.activeJobs.size === 0) this.onFinishCallback?.(false);
      return { success: true, cancelled: true, message: 'Recording cancelled' };
    }

    if (this.activeJobs.size > 0) {
      for (const [, job] of this.activeJobs.entries()) {
        job.cancelled = true;
        try { job.process?.kill('SIGKILL'); } catch {}
      }
      this.activeJobs.clear();
      sendNotification('⏹️ Tape Recording Cancelled', 'Recording was stopped and discarded.');
      this.onFinishCallback?.(false);
      return { success: true, cancelled: true, message: 'All active recordings cancelled' };
    }

    return { success: false, message: 'No active recording to cancel' };
  }

  async recordTrack(track, format = 'opus') {
    if (!track || (!track.url && !track.streamUrl && !track.path)) {
      return { success: false, error: 'No valid track stream URL to record' };
    }

    if (!existsSync(RECORDINGS_DIR)) {
      await mkdir(RECORDINGS_DIR, { recursive: true });
    }

    const title = track.title || track.name || 'Unknown Track';
    const artist = track.artist || track.channel || track.country || 'Unknown Artist';
    const streamUrl = track.url || track.streamUrl || track.path;
    const cleanTitle = sanitizeFilename(title);
    const cleanArtist = sanitizeFilename(artist);

    // 1. Guard against local storage files: NEVER convert or duplicate local files!
    if (track.type === 'local' || (typeof streamUrl === 'string' && streamUrl.startsWith('/') && existsSync(streamUrl))) {
      sendNotification('ℹ️ Local File Already Present', `"${title}" is already in your library. Original Hi-Res audio is 100% preserved.`);
      return {
        success: false,
        isLocal: true,
        message: `Track is already a local file: ${title} (Original audio is 100% untouched)`
      };
    }

    // If already recording this exact URL, toggle / cancel it!
    if (this.activeJobs.has(streamUrl)) {
      return this.cancelRecording(streamUrl);
    }

    sendNotification('🔴 Cassette Recording Started', `Recording: "${artist} - ${title}" [${format.toUpperCase()}] (Press 'R' again to cancel)`);

    // If it's a YouTube / SoundCloud / Bandcamp stream, use yt-dlp to download audio in requested format
    if (track.source === 'youtube' || track.source === 'soundcloud' || track.source === 'bandcamp' ||
        streamUrl.includes('youtube.com') || streamUrl.includes('youtu.be') || streamUrl.includes('soundcloud.com') || streamUrl.includes('bandcamp.com')) {
      return this._recordViaYtDlp(streamUrl, cleanArtist, cleanTitle, format);
    }

    // Otherwise use ffmpeg to capture stream for 240s (4 mins)
    return this._recordViaFfmpeg(streamUrl, cleanArtist, cleanTitle, format);
  }

  _recordViaYtDlp(url, artist, title, format = 'opus') {
    const targetFmt = ['opus', 'mp3', 'flac', 'm4a'].includes(format) ? format : 'opus';
    const outputTemplate = join(RECORDINGS_DIR, `${artist} - ${title}.%(ext)s`);

    const child = spawn('yt-dlp', [
      '-x',
      '--audio-format', targetFmt,
      '--audio-quality', '0',
      '--no-playlist',
      '--embed-metadata',
      '-o', outputTemplate,
      url
    ], { stdio: 'ignore' });

    const job = { process: child, cancelled: false };
    this.activeJobs.set(url, job);

    child.on('close', (code) => {
      const wasCancelled = job.cancelled;
      this.activeJobs.delete(url);
      if (this.activeJobs.size === 0) this.onFinishCallback?.(false);

      if (wasCancelled) {
        return;
      }

      if (code === 0) {
        sendNotification('✅ Tape Recording Complete', `Saved to ~/Music/Boombox Recordings/${artist} - ${title}.${targetFmt}`);
      } else {
        sendNotification('⚠️ Tape Recording Failed', `Could not rip stream for: ${title}`);
      }
    });

    return { success: true, message: `Started recording: ${title}` };
  }

  _recordViaFfmpeg(url, artist, title, format = 'mp3') {
    const targetExt = ['mp3', 'opus', 'flac', 'm4a'].includes(format) ? format : 'mp3';
    const outputFile = join(RECORDINGS_DIR, `${artist} - ${title}.${targetExt}`);

    const args = ['-y', '-i', url, '-t', '240'];
    if (targetExt === 'opus') {
      args.push('-c:a', 'libopus', '-b:a', '160k');
    } else if (targetExt === 'flac') {
      args.push('-c:a', 'flac');
    } else if (targetExt === 'm4a') {
      args.push('-c:a', 'aac', '-b:a', '256k');
    } else {
      args.push('-c:a', 'libmp3lame', '-b:a', '320k');
    }
    args.push(outputFile);

    const child = spawn('ffmpeg', args, { stdio: 'ignore' });

    const job = { process: child, cancelled: false, outputFile };
    this.activeJobs.set(url, job);

    child.on('close', (code) => {
      const wasCancelled = job.cancelled;
      this.activeJobs.delete(url);
      if (this.activeJobs.size === 0) this.onFinishCallback?.(false);

      if (wasCancelled) {
        if (existsSync(outputFile)) {
          try { unlinkSync(outputFile); } catch {}
        }
        return;
      }

      if (code === 0) {
        sendNotification('✅ Tape Recording Complete', `Saved to ~/Music/Boombox Recordings/${artist} - ${title}.mp3`);
      } else {
        sendNotification('⚠️ Tape Recording Failed', `Could not capture radio stream: ${title}`);
      }
    });

    return { success: true, message: `Started capturing radio stream: ${title}` };
  }
}

export const streamRecorder = new StreamRecorder();
