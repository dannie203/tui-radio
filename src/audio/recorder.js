import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
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
  }

  async recordTrack(track) {
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

    // If already recording this exact URL
    if (this.activeJobs.has(streamUrl)) {
      return { success: false, error: 'Track is already recording in background' };
    }

    sendNotification('🔴 Cassette Recording Started', `Recording: "${artist} - ${title}"`);

    // If it's a YouTube / SoundCloud / Bandcamp stream, use yt-dlp to download highest quality audio
    if (track.source === 'youtube' || track.source === 'soundcloud' || track.source === 'bandcamp' ||
        streamUrl.includes('youtube.com') || streamUrl.includes('youtu.be') || streamUrl.includes('soundcloud.com') || streamUrl.includes('bandcamp.com')) {
      return this._recordViaYtDlp(streamUrl, cleanArtist, cleanTitle);
    }

    // Otherwise use ffmpeg to capture stream for 180s (3 mins) or direct stream copy
    return this._recordViaFfmpeg(streamUrl, cleanArtist, cleanTitle);
  }

  _recordViaYtDlp(url, artist, title) {
    const outputTemplate = join(RECORDINGS_DIR, `${artist} - ${title}.%(ext)s`);

    const child = spawn('yt-dlp', [
      '-x',
      '--audio-format', 'opus',
      '--audio-quality', '0',
      '--no-playlist',
      '--embed-metadata',
      '-o', outputTemplate,
      url
    ], { stdio: 'ignore' });

    this.activeJobs.set(url, child);

    child.on('close', (code) => {
      this.activeJobs.delete(url);
      if (code === 0) {
        sendNotification('✅ Tape Recording Complete', `Saved to ~/Music/Boombox Recordings/${artist} - ${title}.opus`);
      } else {
        sendNotification('⚠️ Tape Recording Failed', `Could not rip stream for: ${title}`);
      }
    });

    return { success: true, message: `Started recording: ${title}` };
  }

  _recordViaFfmpeg(url, artist, title) {
    const outputFile = join(RECORDINGS_DIR, `${artist} - ${title}.mp3`);

    const child = spawn('ffmpeg', [
      '-y',
      '-i', url,
      '-t', '240', // Cap stream recording to 4 minutes max
      '-c:a', 'libmp3lame',
      '-b:a', '320k',
      outputFile
    ], { stdio: 'ignore' });

    this.activeJobs.set(url, child);

    child.on('close', (code) => {
      this.activeJobs.delete(url);
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
