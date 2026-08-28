import blessed from 'blessed';
import { MODES, GENRE_FILTERS } from '../state/store.js';
import { extractArtworkBuffer, renderHalfBlockArt } from '../audio/art.js';

const colors = {
  bgDark: '#0a0d13',
  bgPanel: '#11151f',
  bgLcd: '#06130a',
  borderDim: '#243348',
  borderFocus: '#ffb000',
  borderLcd: '#1b4d24',
  amber: '#ffb000',
  amberBright: '#ffd24d',
  amberDim: '#996900',
  greenPhosphor: '#33ff33',
  greenDim: '#114a1a',
  gold: '#f5c542',
  cyanDolby: '#00e5ff',
  redLed: '#ff3344',
  yellowLed: '#ffee33',
  cream: '#f3ead8',
  chrome: '#b8c4ce',
  muted: '#6f7e91'
};

const SPOOL_FRAMES_LEFT = ['(◐)', '(◓)', '(◑)', '(◒)'];
const SPOOL_FRAMES_RIGHT = ['(◑)', '(◒)', '(◐)', '(◓)'];
const WIDE_SPOOL_LEFT = ['(  |  )', '(  /  )', '(  -  )', '(  \\  )'];
const WIDE_SPOOL_RIGHT = ['(  \\  )', '(  |  )', '(  /  )', '(  -  )'];
const EQ_FREQ_LABELS = [' 31', ' 63', '125', '250', '500', ' 1k', ' 2k', ' 4k', ' 8k', '16k'];
const BLOCK_CHARS = [' ', ' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

function stripTags(str) {
  return String(str || '').replace(/\{[^}]+\}/g, '');
}

function padLine(str, targetWidth) {
  const len = stripTags(str).length;
  if (len >= targetWidth) return str;
  return str + ' '.repeat(targetWidth - len);
}

function sanitize(str, maxLen = 40) {
  if (!str) return '';
  const clean = String(str).replace(/[{}]/g, '').trim();
  return clean.length > maxLen ? clean.slice(0, maxLen) : clean;
}

function formatHeader(state) {
  const modeTabs = MODES.map((m, idx) => {
    let count = 0;
    if (m === 'LOCAL TRACKS') count = state.localTracks.length;
    else if (m === 'RADIO STATIONS') count = state.stations.length;
    else if (m === 'QUEUE') count = state.queue.length;
    else if (m === 'YOUTUBE MUSIC') count = state.youtubeResults.length;

    const label = `${idx + 1}. ${m} (${count})`;
    return m === state.mode
      ? `{bold}{#0a0d13-bg}{#ffb000-fg} ▶ [ ${label} ] {/#ffb000-fg}{/#0a0d13-bg}{/bold}`
      : `{#6f7e91-fg}[ ${label} ]{/#6f7e91-fg}`;
  }).join('   ');

  const stereoBadge = state.stereoMode === 'STEREO'
    ? '{#33ff33-fg}[● STEREO (S)]{/#33ff33-fg}'
    : state.stereoMode === 'MONO'
    ? '{#ffd24d-fg}[◉ MONO (S)]{/#ffd24d-fg}'
    : '{#00e5ff-fg}[✦ 3D WIDE (S)]{/#00e5ff-fg}';

  const dolbyBadge = state.dolbyMode === 'OFF'
    ? '{#566573-fg}[DOLBY: OFF (D)]{/#566573-fg}'
    : state.dolbyMode === 'DOLBY-B'
    ? '{#00e5ff-fg}[DOLBY-B (D)]{/#00e5ff-fg}'
    : state.dolbyMode === 'DOLBY-C'
    ? '{#33ff33-fg}[DOLBY-C (D)]{/#33ff33-fg}'
    : '{#ffd24d-fg}[DOLBY-S (D)]{/#ffd24d-fg}';

  const tapeBadge = state.tapeType === 'TYPE-I'
    ? '{#c79248-fg}[Type-I Fe (T)]{/#c79248-fg}'
    : state.tapeType === 'TYPE-II'
    ? '{#f5c542-fg}[Type-II CrO2 (T)]{/#f5c542-fg}'
    : '{#00e5ff-fg}[Type-IV Metal (T)]{/#00e5ff-fg}';

  const bassBadge = state.bassBoost
    ? '{bold}{#ff3344-fg}[🔊 BASS (B)]{/#ff3344-fg}{/bold}'
    : '{#566573-fg}[BASS (B)]{/#566573-fg}';

  const shufBadge = state.shuffle ? '{#00e5ff-fg}[SHUF: ON]{/#00e5ff-fg}' : '{#475466-fg}[SHUF: OFF]{/#475466-fg}';
  const repBadge = state.repeat !== 'off'
    ? `{#f5c542-fg}[REP: ${state.repeat.toUpperCase()}]{/#f5c542-fg}`
    : '{#475466-fg}[REP: OFF]{/#475466-fg}';

  return [
    ` {bold}{#ffb000-fg}▶ BOOMBOX RX-505{/#ffb000-fg}{/bold}  {#6f7e91-fg}│{/#6f7e91-fg}  ${stereoBadge}  ${dolbyBadge}  ${tapeBadge}  ${bassBadge}  {#6f7e91-fg}│{/#6f7e91-fg}  ${shufBadge}  ${repBadge}`,
    ` {bold}{#b8c4ce-fg}DECK MODE [M/TAB/1-4]:{/#b8c4ce-fg}{/bold}  ${modeTabs}`
  ].join('\n');
}

function formatLyricsDisplay(state, currentItem) {
  const title = sanitize(currentItem?.title || currentItem?.name || 'UNKNOWN TRACK', 38);
  const artist = sanitize(currentItem?.artist || 'UNKNOWN ARTIST', 24);
  const lyrics = state.lyrics;
  const status = state.lyricsStatus;
  const source = lyrics?.source ? `[${lyrics.source.toUpperCase()}]` : '';
  const syncOffset = state.lyricsSyncOffset || 0;
  const syncBadge = syncOffset !== 0 ? ` {#ffd24d-fg}(Offset: ${syncOffset > 0 ? '+' : ''}${syncOffset}s){/#ffd24d-fg}` : '';

  const header = [
    ` {#f5c542-fg}┌─────────────────────────────────────────────────────────────┐{/#f5c542-fg}`,
    ` {#f5c542-fg}│{/#f5c542-fg} {#00e5ff-fg}[ 🎤 LIVE PHOSPHOR LYRICS DISPLAY ]{/#00e5ff-fg}  {#ffd24d-fg}${source}{/#ffd24d-fg}  {#6f7e91-fg}(Press 'L' to return){/#6f7e91-fg}`,
    ` {#f5c542-fg}└─────────────────────────────────────────────────────────────┘{/#f5c542-fg}`,
    ` {bold}{#33ff33-fg}TRACK :{/#33ff33-fg}{/bold} {bold}${title}{/bold}  {#6f7e91-fg}│{/#6f7e91-fg}  {bold}{#ffd24d-fg}ARTIST:{/#ffd24d-fg}{/bold} {bold}${artist}{/bold}${syncBadge}`
  ];

  if (status === 'loading') {
    return [
      ...header,
      ``,
      `   {#ffd24d-fg}⏳ Searching lyrics database for "${title}"...{/#ffd24d-fg}`,
      `   {#6f7e91-fg}Querying LRCLIB open-source synced lyrics provider...{/#6f7e91-fg}`,
      ``,
      ``,
      ``
    ].join('\n');
  }

  if (status === 'unavailable' || !lyrics || (!lyrics.synced?.length && !lyrics.plain)) {
    return [
      ...header,
      ``,
      `   {#6f7e91-fg}∅ No lyrics available for this track / broadcast.{/#6f7e91-fg}`,
      `   {#566573-fg}• Local tracks: Place a matching .lrc file in the same folder.{/#566573-fg}`,
      `   {#566573-fg}• YouTube / Radio: Auto-queried via LRCLIB if artist & title match.{/#566573-fg}`,
      ``,
      ``
    ].join('\n');
  }

  if (lyrics.synced && lyrics.synced.length > 0) {
    const activeIdx = state.activeLyricIndex >= 0 ? state.activeLyricIndex : 0;
    const centerIdx = activeIdx + (state.lyricsScrollOffset || 0);
    const windowRadius = 3;
    const lines = [];

    lines.push(...header);
    lines.push(``);

    for (let i = centerIdx - windowRadius; i <= centerIdx + windowRadius; i++) {
      if (i < 0 || i >= lyrics.synced.length) {
        lines.push('');
        continue;
      }
      const item = lyrics.synced[i];
      const m = Math.floor(item.time / 60);
      const s = String(Math.floor(item.time % 60)).padStart(2, '0');
      const timeTag = `${m}:${s}`;
      const safeText = sanitize(item.text || '♪  ♪  ♪', 48);

      if (i === activeIdx) {
        lines.push(` {bold}{#33ff33-fg} ▶ [${timeTag}]  ${safeText}{/#33ff33-fg}{/bold}`);
      } else if (Math.abs(i - activeIdx) === 1) {
        lines.push(`   {#f3ead8-fg}[${timeTag}]  ${safeText}{/#f3ead8-fg}`);
      } else {
        lines.push(`   {#6f7e91-fg}[${timeTag}]  ${safeText}{/#6f7e91-fg}`);
      }
    }

    return lines.join('\n');
  }

  // Plain lyrics fallback
  if (lyrics.plain) {
    const allLines = lyrics.plain.split(/\r?\n/).filter((l) => l.trim().length > 0);
    const offset = Math.max(0, Math.min(allLines.length - 1, state.lyricsScrollOffset || 0));
    const slice = allLines.slice(offset, offset + 7);
    const lines = [...header, ``];

    for (const l of slice) {
      lines.push(`   {#f3ead8-fg}${sanitize(l, 56)}{/#f3ead8-fg}`);
    }

    return lines.join('\n');
  }

  return header.join('\n');
}

export function formatCodecBadge(currentItem, telemetry = null) {
  if (!currentItem && !telemetry?.audioCodec) {
    return '[STANDBY]';
  }

  // Codec priority: telemetry from active mpv engine -> item metadata
  const detectedCodec = (telemetry?.audioCodec || '').trim().toUpperCase();
  const itemCodec = (currentItem?.codec || currentItem?.format || '').trim().toUpperCase();
  let rawCodec = detectedCodec || itemCodec;

  // Clean and standardize codec tags
  if (rawCodec === 'PCM_S16LE' || rawCodec === 'PCM_S24LE' || rawCodec === 'PCM_S32LE') rawCodec = 'WAV';
  else if (rawCodec === 'VORBIS') rawCodec = 'OGG';
  else if (rawCodec === 'MPEG2' || rawCodec === 'MP3FLOAT') rawCodec = 'MP3';
  else if (rawCodec === 'AAC_LATM' || rawCodec === 'MP4A') rawCodec = 'AAC';
  else if (rawCodec === 'ALAC') rawCodec = 'ALAC';
  else if (rawCodec === 'FLAC') rawCodec = 'FLAC';
  else if (rawCodec === 'OPUS') rawCodec = 'OPUS';

  if (!rawCodec || rawCodec === 'AUDIO') {
    if (currentItem?.type === 'youtube') rawCodec = 'OPUS';
    else if (currentItem?.type === 'radio') rawCodec = 'MP3';
    else if (currentItem?.type === 'local') rawCodec = currentItem.format || 'AUDIO';
    else rawCodec = 'AUDIO';
  }

  const bitrate = telemetry?.audioBitrate || currentItem?.bitrate || 0;
  const sampleRate = telemetry?.audioSampleRate || currentItem?.sampleRate || 0;
  const bitsPerSample = currentItem?.bitsPerSample || 0;

  // Hi-Res Lossless Formats (FLAC, WAV, ALAC, AIFF)
  if (['FLAC', 'WAV', 'ALAC', 'AIFF'].includes(rawCodec)) {
    if (bitsPerSample > 0 && sampleRate > 0) {
      const sr = sampleRate >= 1000 ? `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)}k` : `${sampleRate}Hz`;
      return `[${rawCodec} ${bitsPerSample}/${sr}]`;
    }
    if (sampleRate > 0) {
      const sr = sampleRate >= 1000 ? `${(sampleRate / 1000).toFixed(sampleRate % 1000 === 0 ? 0 : 1)}kHz` : `${sampleRate}Hz`;
      return `[${rawCodec} ${sr}]`;
    }
    return `[${rawCodec}]`;
  }

  // Lossy Formats (MP3, AAC, AAC+, OPUS, OGG, etc.)
  if (bitrate > 0) {
    return `[${rawCodec} ${bitrate}k]`;
  }

  return `[${rawCodec}]`;
}

function formatCombinedMonitor(spoolFrame, isPlaying, isPaused, currentItem, mode, state, marqueeText, artLines, showFullArtwork, telemetry = null) {
  const title = sanitize(currentItem?.title || currentItem?.name || 'SELECT A CASSETTE TAPE', 44);
  const artist = sanitize(currentItem?.artist || 'LIVE BROADCAST', 28);
  const albumOrCountry = sanitize(
    currentItem?.type === 'local'
      ? (currentItem.album || 'Single')
      : currentItem?.type === 'youtube'
      ? 'YOUTUBE'
      : (currentItem?.country || 'STREAM'),
    16
  );
  const formatBadge = formatCodecBadge(currentItem, telemetry);

  if (state.lyricsVisible) {
    return formatLyricsDisplay(state, currentItem);
  }

  if (showFullArtwork && artLines) {
    return [
      ` {#f5c542-fg}┌─────────────────────────────────────────────────────────────┐{/#f5c542-fg}`,
      ` {#f5c542-fg}│{/#f5c542-fg} {#00e5ff-fg}[ 💽 HIGH-RES ALBUM COVER ARTWORK ]{/#00e5ff-fg}  {#6f7e91-fg}(Press 'W' to return){/#6f7e91-fg}`,
      ` {#f5c542-fg}└─────────────────────────────────────────────────────────────┘{/#f5c542-fg}`,
      artLines,
      ``,
      ` {bold}{#33ff33-fg}TRACK :{/#33ff33-fg}{/bold} {bold}${title}{/bold}`,
      ` {bold}{#ffd24d-fg}ARTIST:{/#ffd24d-fg}{/bold} {bold}${artist}{/bold}  {#00e5ff-fg}[${albumOrCountry}]{/#00e5ff-fg}`
    ].join('\n');
  }

  const recLed = isPlaying && !isPaused ? '{#ff3344-fg}● REC{/#ff3344-fg}' : '{#2a3545-fg}● REC{/#2a3545-fg}';
  const playLed = isPlaying && !isPaused ? '{#33ff33-fg}▶ PLAY{/#33ff33-fg}' : '{#2a3545-fg}▶ PLAY{/#2a3545-fg}';
  const pauseLed = isPaused ? '{#ffb000-fg}❚❚ PAUSE{/#ffb000-fg}' : '{#2a3545-fg}❚❚ PAUSE{/#2a3545-fg}';
  const stopLed = !isPlaying && !isPaused ? '{#f5c542-fg}▲ STOP{/#f5c542-fg}' : '{#2a3545-fg}▲ STOP{/#2a3545-fg}';

  const playbackBadge = isPaused
    ? '{#ffb000-fg}[ PAUSED ]{/#ffb000-fg}'
    : isPlaying
    ? '{#33ff33-fg}[ PLAYING ]{/#33ff33-fg}'
    : '{#6f7e91-fg}[ STANDBY ]{/#6f7e91-fg}';

  let rawTitle = currentItem?.artist
    ? `${currentItem.artist} - ${currentItem.title || currentItem.name}`
    : currentItem?.name || (mode === 'LOCAL TRACKS' ? 'LOCAL MIXTAPE' : 'LIVE RADIO');

  const dolbyLabel = (state?.dolbyMode || 'DOLBY-B').padEnd(7).slice(0, 7);
  const tapeLabelType = (state?.tapeType || 'CrO2').padEnd(12).slice(0, 12);

  const volumeFilled = Math.round((state.volume / 100) * 12);
  const volumeBar = `[{#ffb000-fg}${'■'.repeat(volumeFilled)}{/#ffb000-fg}{#114a1a-fg}${'□'.repeat(12 - volumeFilled)}{/#114a1a-fg}] ${state.volume}%`;

  const progWidth = 24;
  const hasDuration = Boolean(telemetry?.hasDuration || (state.duration > 0) || (currentItem?.duration > 0));
  let progressBar = '';
  let progDetails = '';

  if (hasDuration) {
    const percent = Math.max(0, Math.min(100, state.percentPos || 0));
    const progFilled = Math.max(0, Math.min(progWidth, Math.round((percent / 100) * progWidth)));
    progressBar = `[{#33ff33-fg}${'■'.repeat(progFilled)}{/#33ff33-fg}{#114a1a-fg}${'□'.repeat(progWidth - progFilled)}{/#114a1a-fg}]`;
    progDetails = `${state.tapeCounter} (${percent}%)`;
  } else if (isPlaying && !isPaused) {
    // Dynamic animated phosphor scanner for live radio broadcasts and continuous streams
    const elapsed = telemetry?.elapsedMs || 0;
    const beamWidth = 6;
    const maxTravel = progWidth - beamWidth;
    const scanCycle = Math.floor(elapsed / 80) % (maxTravel * 2);
    const beamPos = scanCycle < maxTravel ? scanCycle : (maxTravel * 2 - scanCycle);

    let barStr = '';
    for (let i = 0; i < progWidth; i++) {
      if (i >= beamPos && i < beamPos + beamWidth) {
        barStr += '{#33ff33-fg}■{/#33ff33-fg}';
      } else {
        barStr += '{#114a1a-fg}□{/#114a1a-fg}';
      }
    }
    progressBar = `[${barStr}]`;
    progDetails = `${state.tapeCounter} {#33ff33-fg}● LIVE STREAM{/#33ff33-fg}`;
  } else if (isPaused) {
    progressBar = `[{#ffd24d-fg}  ❚❚  STREAM PAUSED  ❚❚  {/#ffd24d-fg}]`;
    progDetails = `${state.tapeCounter} {#ffd24d-fg}(PAUSED){/#ffd24d-fg}`;
  } else {
    progressBar = `[{#114a1a-fg}${'□'.repeat(progWidth)}{/#114a1a-fg}]`;
    progDetails = `${state.tapeCounter || '00:00'} {#6f7e91-fg}(STANDBY){/#6f7e91-fg}`;
  }

  const cleanMarquee = sanitize(marqueeText, 44);
  const cleanStatus = sanitize(state.status, 44);
  const bassTxt = state.bassBoost ? '{#ff3344-fg}MEGA BASS +7dB{/#ff3344-fg}' : '{#6f7e91-fg}FLAT{/#6f7e91-fg}';

  const leftSpool = isPlaying && !isPaused ? WIDE_SPOOL_LEFT[spoolFrame % 4] : '(  |  )';
  const rightSpool = isPlaying && !isPaused ? WIDE_SPOOL_RIGHT[spoolFrame % 4] : '(  |  )';
  const tapeLabel = sanitize(rawTitle, 16).toUpperCase().padStart(16, ' ').slice(-16);

  const lyricsBadge = state.lyricsStatus === 'found'
    ? ' {#33ff33-fg}[LYRICS: L]{/#33ff33-fg}'
    : state.lyricsStatus === 'loading'
    ? ' {#ffd24d-fg}[LYRICS: ⏳]{/#ffd24d-fg}'
    : '';
  const artBadge = artLines ? ' {#f5c542-fg}[COVER ART: W]{/#f5c542-fg}' : '';

  const deckLine1 = ` {#f5c542-fg}[A] RETRO STEREO DECK{/#f5c542-fg}        {#00e5ff-fg}${dolbyLabel}{/#00e5ff-fg}        {#ffd24d-fg}${tapeLabelType}{/#ffd24d-fg}`;
  const deckLine2 = `   {#ffb000-fg}${leftSpool}{/#ffb000-fg} ══════ [ {#f3ead8-fg}${tapeLabel}{/#f3ead8-fg} ] ══════ {#ffb000-fg}${rightSpool}{/#ffb000-fg}`;
  const deckLine3 = `       ${recLed}        ${playLed}        ${pauseLed}        ${stopLed}`;

  return [
    ` {#6f7e91-fg}┌─────────────────────────────────────────────────────────────┐{/#6f7e91-fg}`,
    ` {#6f7e91-fg}│{/#6f7e91-fg}${padLine(deckLine1, 61)}{#6f7e91-fg}│{/#6f7e91-fg}`,
    ` {#6f7e91-fg}│{/#6f7e91-fg}${padLine(deckLine2, 61)}{#6f7e91-fg}│{/#6f7e91-fg}`,
    ` {#6f7e91-fg}│{/#6f7e91-fg}${padLine(deckLine3, 61)}{#6f7e91-fg}│{/#6f7e91-fg}`,
    ` {#6f7e91-fg}└─────────────────────────────────────────────────────────────┘{/#6f7e91-fg}`,
    ``,
    ` {#33ff33-fg}COUNTER : [{#ffd24d-fg}${state.tapeCounter}{/#ffd24d-fg}]               SOUNDSTAGE: {#33ff33-fg}${state.stereoMode}{/#33ff33-fg}${lyricsBadge}${artBadge}`,
    ` {#6f7e91-fg}TITLE   :{/#6f7e91-fg} {bold}{#33ff33-fg}${title}{/#33ff33-fg}{/bold}`,
    ` {#6f7e91-fg}ARTIST  :{/#6f7e91-fg} {bold}{#f5c542-fg}${artist}{/#f5c542-fg}{/bold} {#00e5ff-fg}[${albumOrCountry}]{/#00e5ff-fg}`,
    ` {#6f7e91-fg}STREAM  :{/#6f7e91-fg} {#ffd24d-fg}${cleanMarquee}{/#ffd24d-fg}`,
    ` {#6f7e91-fg}PROG    :{/#6f7e91-fg} ${progressBar} ${progDetails}`,
    ` {#6f7e91-fg}DSP     :{/#6f7e91-fg} {#33ff33-fg}${state.stereoMode}{/#33ff33-fg} │ {#00e5ff-fg}${state.dolbyMode}{/#00e5ff-fg} │ {#ffd24d-fg}${state.tapeType}{/#ffd24d-fg} │ ${bassTxt}`,
    ` {#6f7e91-fg}STATUS  :{/#6f7e91-fg} ${playbackBadge}  {#6f7e91-fg}CODEC:{/#6f7e91-fg} {#33ff33-fg}${formatBadge}{/#33ff33-fg}  {#6f7e91-fg}VOL:{/#6f7e91-fg} ${volumeBar}`,
    ` {#6f7e91-fg}SYSTEM  :{/#6f7e91-fg} {#ffb000-fg}${cleanStatus}{/#ffb000-fg}`
  ].join('\n');
}

function formatVuMeter(channelLabel, value, peak, width = 56) {
  const filled = Math.max(0, Math.min(width, Math.round((value / 100) * width)));
  const peakPos = Math.max(0, Math.min(width - 1, Math.round((peak / 100) * (width - 1))));

  let barStr = '';
  for (let i = 0; i < width; i++) {
    const pct = (i / width) * 100;
    if (i < filled) {
      if (pct >= 88) barStr += '{#ff3344-fg}■{/#ff3344-fg}';
      else if (pct >= 70) barStr += '{#ffee33-fg}■{/#ffee33-fg}';
      else barStr += '{#33ff33-fg}■{/#33ff33-fg}';
    } else if (i === peakPos && peak > 5) {
      if (pct >= 88) barStr += '{#ff3344-fg}▮{/#ff3344-fg}';
      else if (pct >= 70) barStr += '{#ffee33-fg}▮{/#ffee33-fg}';
      else barStr += '{#33ff33-fg}▮{/#33ff33-fg}';
    } else {
      barStr += '{#17202c-fg}■{/#17202c-fg}';
    }
  }

  const db = value <= 0
    ? '-∞'
    : value < 70
    ? `${Math.round((value - 70) / 2.5)}dB`
    : value < 88
    ? `${Math.round((value - 70) / 6)}dB`
    : `+${Math.round((value - 88) / 4)}dB`;

  const peakIndicator = peak >= 90
    ? '{#ff3344-fg}[PEAK]{/#ff3344-fg}'
    : '{#2d3748-fg}[PEAK]{/#2d3748-fg}';

  return ` {bold}{#f5c542-fg}${channelLabel}{/#f5c542-fg}{/bold} [${barStr}] {#b8c4ce-fg}${db.padStart(5)}{/#b8c4ce-fg} ${peakIndicator}`;
}

function formatEqualizer(bands = [], peaks = [], height = 6) {
  const bandWidth = 5;
  const spacing = '  ';
  const rows = [];

  for (let r = height; r >= 1; r--) {
    let rowStr = ' ';
    for (let b = 0; b < 10; b++) {
      const val = bands[b] || 0;
      const peak = peaks[b] || 0;
      const threshold = ((r - 1) / height) * 100;
      const nextThreshold = (r / height) * 100;
      const peakThreshold = (peak / 100) * height;

      let block = ' '.repeat(bandWidth);
      let color = '#33ff33';
      if (r >= height - 1) color = '#ff3344';
      else if (r >= height - 2) color = '#ffee33';

      if (val >= nextThreshold) {
        block = '█'.repeat(bandWidth);
      } else if (val > threshold) {
        const sub = Math.floor(((val - threshold) / (nextThreshold - threshold)) * (BLOCK_CHARS.length - 1));
        block = BLOCK_CHARS[Math.max(1, sub)].repeat(bandWidth);
      } else if (Math.ceil(peakThreshold) === r && peak > 5) {
        block = '━'.repeat(bandWidth);
        color = '#ff3344';
      }

      rowStr += `{${color}-fg}${block}{/${color}-fg}${spacing}`;
    }
    rows.push(rowStr);
  }

  const labelRow = ' ' + ['31Hz', '63Hz', '125Hz', '250Hz', '500Hz', ' 1kHz', ' 2kHz', ' 4kHz', ' 8kHz', '16kHz'].map((l) => `{#6f7e91-fg}${l.padStart(5)}{/#6f7e91-fg}`).join('  ');
  rows.push(labelRow);
  return rows.join('\n');
}

export function createLayout(store, actions, player) {
  const screen = blessed.screen({
    smartCSR: true,
    title: 'NEON//WAVE CYBERPUNK AUDIO TERMINAL',
    fullUnicode: true
  });
  const leftPaneWidth = '37%';
  const rightPaneLeft = '38%';
  const rightPaneWidth = '62%';

  let marqueeOffset = 0;
  let lastMetadata = '';
  let currentArtLines = null;
  let currentArtKey = null;
  let showFullArtwork = false;

  async function updateArtwork(item) {
    const targetPath = item?.path || item?.url;
    if (!targetPath || currentArtKey === targetPath) return;
    currentArtKey = targetPath;

    if (item.type === 'local' || (typeof targetPath === 'string' && targetPath.startsWith('/'))) {
      const art = await extractArtworkBuffer(targetPath);
      if (art && currentArtKey === targetPath) {
        currentArtLines = renderHalfBlockArt(art.data, art.format, 28, 12);
        screen.render();
      }
    } else {
      currentArtLines = null;
    }
  }

  // Top Radio Receiver / Master Header Bar (Spans 100% width)
  const headerBox = blessed.box({
    parent: screen,
    top: 0,
    left: 0,
    width: '67%',
    height: 3,
    wrap: false,
    tags: true,
    style: { fg: colors.cream, bg: colors.bgDark }
  });

  // Top Search / Frequency Tuning Dial
  const searchBox = blessed.textbox({
    parent: screen,
    top: 0,
    left: '67%',
    width: '33%',
    height: 3,
    label: ' 🔍 SEEK / SEARCH [/] ',
    inputOnFocus: false,
    keys: true,
    mouse: true,
    wrap: false,
    border: { type: 'line' },
    style: {
      fg: colors.amber,
      bg: colors.bgPanel,
      border: { fg: colors.borderDim },
      focus: { border: { fg: colors.borderFocus } }
    }
  });

  // Left Sub-header / Filter Tabs Bar
  const subBar = blessed.box({
    parent: screen,
    top: 3,
    left: 0,
    width: leftPaneWidth,
    height: 1,
    wrap: false,
    tags: true,
    style: { fg: colors.cream, bg: colors.bgDark }
  });

  // Left Main Explorer List (Tracks / Stations / Queue) -> Expands all the way down to bottom!
  const explorerList = blessed.list({
    parent: screen,
    top: 4,
    left: 0,
    width: leftPaneWidth,
    bottom: 0,
    label: ' 📼 CASSETTE CRATES // MUSIC LIBRARY ',
    keys: false,
    vi: false,
    mouse: true,
    tags: true,
    border: { type: 'line' },
    scrollbar: { ch: '█', style: { fg: colors.amber, bg: colors.bgPanel } },
    style: {
      fg: colors.cream,
      bg: colors.bgPanel,
      border: { fg: colors.borderDim },
      selected: { fg: colors.bgDark, bg: colors.amber, bold: true },
      item: { hover: { bg: '#1a2230' } },
      focus: { border: { fg: colors.borderFocus } }
    }
  });

  // Right Top: Cassette deck and live signal monitor.
  const monitorConsole = blessed.box({
    parent: screen,
    top: 3,
    left: rightPaneLeft,
    width: rightPaneWidth,
    height: '50%',
    label: ' 📟 CASSETTE DECK & PHOSPHOR LCD MONITOR ',
    tags: true,
    wrap: false,
    border: { type: 'line' },
    padding: { left: 1, right: 1 },
    style: {
      fg: colors.greenPhosphor,
      bg: colors.bgLcd,
      border: { fg: colors.borderLcd }
    }
  });

  // Right Middle: Dual stereo VU needle meters and 10-band equalizer spectrum.
  const vuEqualizerBox = blessed.box({
    parent: screen,
    top: '53%',
    left: rightPaneLeft,
    width: rightPaneWidth,
    height: '34%',
    label: ' 🎛 DUAL STEREO VU METERS & 10-BAND GRAPHIC EQUALIZER ',
    tags: true,
    wrap: false,
    border: { type: 'line' },
    padding: { left: 1, right: 1 },
    style: {
      fg: colors.cream,
      bg: colors.bgPanel,
      border: { fg: colors.borderDim }
    }
  });

  // Right Bottom: Hardware keypad shortcuts.
  const controlsBox = blessed.box({
    parent: screen,
    top: '87%',
    left: rightPaneLeft,
    width: rightPaneWidth,
    bottom: 0,
    label: ' 🎚 DECK HARDWARE CONTROLS & SHORTCUTS ',
    tags: true,
    wrap: false,
    border: { type: 'line' },
    padding: { left: 1, right: 1 },
    style: {
      fg: colors.muted,
      bg: colors.bgPanel,
      border: { fg: colors.borderDim }
    },
    content: [
      ` {bold}{#ffb000-fg}[ ↵ / → ]{/#ffb000-fg}{/bold} Dive In (Artist/Album/Play) {bold}{#ffb000-fg}[ S ]{/#ffb000-fg}{/bold} Stereo/Mono/Wide     {bold}{#ffb000-fg}[ TAB / M ]{/#ffb000-fg}{/bold} Mode Select        {bold}{#ffb000-fg}[ + / - ]{/#ffb000-fg}{/bold} Volume ±5%`,
      ` {bold}{#ffb000-fg}[ ⎋ / ← ]{/#ffb000-fg}{/bold} Back Up (Parent Crate)     {bold}{#ffb000-fg}[ D ]{/#ffb000-fg}{/bold} Dolby NR (Off/B/C/S)  {bold}{#ffb000-fg}[ 1 - 4 ]{/#ffb000-fg}{/bold} Crates Categories  {bold}{#ffb000-fg}[ ␣ ]{/#ffb000-fg}{/bold} Pause / Resume`,
      ` {bold}{#ffb000-fg}[  N/P  ]{/#ffb000-fg}{/bold} Next / Previous Track      {bold}{#ffb000-fg}[ T ]{/#ffb000-fg}{/bold} Tape Bias (I/II/IV)  {bold}{#ffb000-fg}[ W / L ]{/#ffb000-fg}{/bold} Cover Art / Lyrics  {bold}{#ffb000-fg}[   /   ]{/#ffb000-fg}{/bold} Search / Filter`,
      ` {bold}{#ffb000-fg}[ Y / U ]{/#ffb000-fg}{/bold} Load YouTube / URL Stream  {bold}{#ffb000-fg}[ B ]{/#ffb000-fg}{/bold} Mega Bass Boost EQ   {bold}{#ffb000-fg}[   A   ]{/#ffb000-fg}{/bold} Queue Crate/Track     {bold}{#ffb000-fg}[   Q   ]{/#ffb000-fg}{/bold} Eject / Quit`
    ].join('\n')
  });

  // Stream / YouTube URL & Search Prompt Modal
  const urlModal = blessed.box({
    parent: screen,
    top: 'center',
    left: 'center',
    width: '64%',
    height: 9,
    hidden: true,
    border: { type: 'line' },
    label: ' 📺 STREAM / YOUTUBE & YT MUSIC URL LOADER [Y] ',
    tags: true,
    style: {
      fg: colors.cream,
      bg: colors.bgDark,
      border: { fg: colors.amber }
    }
  });

  const urlPromptText = blessed.box({
    parent: urlModal,
    top: 0,
    left: 1,
    right: 1,
    height: 2,
    tags: true,
    content: ' Paste YouTube or YouTube Music song/playlist/album URL, or search query:\n {#6f7e91-fg}e.g. "https://music.youtube.com/playlist?list=..." or "music.youtube.com/watch?v=..."{/#6f7e91-fg}'
  });

  const urlInput = blessed.textbox({
    parent: urlModal,
    top: 3,
    left: 1,
    right: 1,
    height: 3,
    inputOnFocus: false,
    keys: true,
    mouse: true,
    border: { type: 'line' },
    style: {
      fg: colors.amberBright,
      bg: colors.bgPanel,
      border: { fg: colors.borderDim },
      focus: { border: { fg: colors.amber } }
    }
  });

  const focusables = [explorerList, searchBox];
  for (const widget of focusables) {
    widget.on('focus', () => {
      widget.style.border.fg = colors.borderFocus;
      screen.render();
    });
    widget.on('blur', () => {
      widget.style.border.fg = colors.borderDim;
      screen.render();
    });
  }

  const focusPane = (index) => focusables[index].focus();
  const cycleFocus = (delta) => {
    const currentIndex = Math.max(0, focusables.indexOf(screen.focused));
    focusPane((currentIndex + delta + focusables.length) % focusables.length);
  };

  function getMarqueeText(metadata, maxLen = 38) {
    if (!metadata || metadata === 'Nothing playing' || metadata === 'Waiting for metadata...') {
      return metadata || 'READY';
    }
    if (metadata !== lastMetadata) {
      lastMetadata = metadata;
      marqueeOffset = 0;
    }
    if (metadata.length <= maxLen) return metadata;
    const padded = `${metadata}   ★★★   `;
    const offset = marqueeOffset % padded.length;
    return (padded + padded).slice(offset, offset + maxLen);
  }

  function render(state) {
    // 1. Render Header
    headerBox.setContent(formatHeader(state));

    const activeItem = state.current || store.selected()?.raw || store.selected();
    updateArtwork(activeItem);

    // 2. Render Sub-Bar based on active Mode
    if (state.mode === 'LOCAL TRACKS') {
      const nav = state.nav || { level: 'ARTISTS' };
      let breadcrumb = '{#ffb000-fg}CRATES{/#ffb000-fg}';
      if (nav.selectedArtist) breadcrumb += ` › {#00e5ff-fg}${sanitize(nav.selectedArtist, 14)}{/#00e5ff-fg}`;
      if (nav.selectedAlbumKey) {
        const alb = state.library?.albums?.[nav.selectedAlbumKey];
        breadcrumb += ` › {#ffd24d-fg}${sanitize(alb?.title || 'Album', 14)}{/#ffd24d-fg}`;
      } else if (nav.selectedPlaylist) {
        breadcrumb += ` › {#f5c542-fg}★ ${sanitize(nav.selectedPlaylist, 14)}{/#f5c542-fg}`;
      }

      const viewTabs = [
        { key: 'ARTISTS', label: '1:ARTISTS' },
        { key: 'ALBUMS', label: '2:ALBUMS' },
        { key: 'PLAYLISTS', label: '3:PLAYLISTS' },
        { key: 'ALL TRACKS', label: '4:TRACKS' }
      ].map((tab) => {
        const isSel = nav.level === tab.key && !nav.selectedArtist && !nav.selectedAlbumKey && !nav.selectedPlaylist;
        return isSel
          ? `{bold}{#ffb000-fg}▶[${tab.label}]{/#ffb000-fg}{/bold}`
          : `{#6f7e91-fg}[${tab.label}]{/#6f7e91-fg}`;
      }).join(' ');

      subBar.setContent(` ${viewTabs}  {#6f7e91-fg}│{/#6f7e91-fg}  ${breadcrumb}`);

      // Set explorer list label
      if (nav.level === 'ARTISTS') {
        explorerList.setLabel(` 🎤 CRATES // ARTISTS (${state.localViewItems?.length || 0}) `);
      } else if (nav.level === 'ALBUMS') {
        explorerList.setLabel(` 💽 ALBUMS // ${sanitize(nav.selectedArtist || 'ALL', 18)} (${state.localViewItems?.length || 0}) `);
      } else if (nav.level === 'PLAYLISTS') {
        explorerList.setLabel(` 📋 PLAYLISTS & MIXTAPES (${state.localViewItems?.length || 0}) `);
      } else if (nav.level === 'TRACKS') {
        if (nav.selectedAlbumKey) {
          const alb = state.library?.albums?.[nav.selectedAlbumKey];
          explorerList.setLabel(` 💿 ${sanitize(alb?.title || 'ALBUM', 16)} [${alb?.format || 'FLAC'}] (${state.localViewItems?.length || 0}) `);
        } else if (nav.selectedPlaylist) {
          explorerList.setLabel(` ★ ${sanitize(nav.selectedPlaylist, 16)} (${state.localViewItems?.length || 0}) `);
        } else {
          explorerList.setLabel(` 🎵 AUDIO TRACKS (${state.localViewItems?.length || 0}) `);
        }
      } else {
        explorerList.setLabel(` 🎵 ALL AUDIO TRACKS (${state.localViewItems?.length || 0}) `);
      }

      searchBox.setLabel(' 🔍 CRATES [/] ');
    } else if (state.mode === 'RADIO STATIONS') {
      const genreTabsContent = ' ' + GENRE_FILTERS.map((g) => {
        return g === state.genreFilter
          ? `{bold}{#ffb000-fg}▶ [ ${g} ]{/#ffb000-fg}{/bold}`
          : `{#6f7e91-fg}[ ${g} ]{/#6f7e91-fg}`;
      }).join('  ');
      subBar.setContent(genreTabsContent);
      explorerList.setLabel(` 📻 RADIO STATIONS // ${state.genreFilter} (${state.filteredStations.length}) `);
      searchBox.setLabel(' 🔍 STATIONS [/] ');
    } else if (state.mode === 'YOUTUBE MUSIC') {
      const q = state.youtubeQuery ? `"${sanitize(state.youtubeQuery, 20)}"` : 'None';
      subBar.setContent(` {#ffb000-fg}YOUTUBE MUSIC:{/#ffb000-fg} {#ffd24d-fg}${q}{/#ffd24d-fg} {#6f7e91-fg}(${state.youtubeResults.length} tracks)  [Press '/' to search, '↵' to play, 'A' to queue]{/#6f7e91-fg}`);
      explorerList.setLabel(` 📺 YOUTUBE MUSIC // ${state.youtubeQuery ? sanitize(state.youtubeQuery, 14) : 'SEARCH'} (${state.youtubeResults.length}) `);
      searchBox.setLabel(' 🔍 YT MUSIC [/] ');
    } else if (state.mode === 'QUEUE') {
      subBar.setContent(` {#f5c542-fg}PLAYBACK QUEUE:{/#f5c542-fg} {#b8c4ce-fg}${state.queue.length} tracks{/#b8c4ce-fg}  {#6f7e91-fg}(C: clear, X: remove){/#6f7e91-fg}`);
      explorerList.setLabel(` 📋 PLAYLIST QUEUE (${state.queue.length}) `);
      searchBox.setLabel(' 🔍 QUEUE [/] ');
    }

    // 3. Render List Items
    const activeList = store.getActiveList();
    let itemsToRender = [];

    if (activeList.length === 0) {
      if (state.mode === 'YOUTUBE MUSIC') {
        itemsToRender = [
          state.youtubeLoading
            ? ` {#ffd24d-fg}⏳ Searching YouTube Music for "${sanitize(state.youtubeQuery, 22)}"...{/#ffd24d-fg}`
            : ` {#6f7e91-fg}🔍 Press '/' to search songs, artists or albums on YouTube Music{/#6f7e91-fg}`
        ];
      } else if (state.mode === 'QUEUE') {
        itemsToRender = [` {#6f7e91-fg}(Playback queue is empty - press 'A' to add tracks){/#6f7e91-fg}`];
      } else if (state.mode === 'RADIO STATIONS') {
        itemsToRender = [` {#6f7e91-fg}(No radio stations found for "${sanitize(state.query, 16)}"){/#6f7e91-fg}`];
      } else {
        itemsToRender = [` {#6f7e91-fg}(No tracks found in crate){/#6f7e91-fg}`];
      }
    } else {
      itemsToRender = activeList.map((item, index) => {
        const isFav = state.favorites.some((fav) => fav.id === item.id);
        const isCurrent = state.current?.id === item.id;
        const favIcon = isFav ? '{#f5c542-fg}★{/#f5c542-fg}' : '{#3b4859-fg}•{/#3b4859-fg}';
        const playIcon = isCurrent ? '{#33ff33-fg}▶{/#33ff33-fg}' : ' ';

        if (state.mode === 'LOCAL TRACKS') {
          if (item.type === 'artist') {
            const name = sanitize(item.name, 16);
            return ` {#ffd24d-fg}▶{/#ffd24d-fg} {bold}${name}{/bold} {#6f7e91-fg}(${item.albumCount} alb, ${item.trackCount} trk){/#6f7e91-fg}`;
          }

          if (item.type === 'album') {
            const title = sanitize(item.title, 16);
            const yr = item.year ? `(${item.year}) ` : '';
            const fmt = item.format ? `[${item.format}]` : '[FLAC]';
            return ` {#00e5ff-fg}●{/#00e5ff-fg} {bold}${title}{/bold} {#ffd24d-fg}${yr}{/#ffd24d-fg}{#6f7e91-fg}${fmt} (${item.trackCount} trk){/#6f7e91-fg}`;
          }

          if (item.type === 'playlist') {
            const name = sanitize(item.name, 18);
            return ` {#f5c542-fg}★{/#f5c542-fg} {bold}${name}{/bold} {#6f7e91-fg}(${item.trackCount} trk){/#6f7e91-fg}`;
          }

          // Track item
          const num = String(item.trackNo || index + 1).padStart(2, '0');
          const title = sanitize(item.title || item.name, 16);
          const artist = sanitize(item.artist || 'Unknown', 10);
          const format = item.format ? `[${item.format}]` : '[FLAC]';
          let dur = '';
          if (item.duration) {
            const m = Math.floor(item.duration / 60);
            const s = String(Math.floor(item.duration % 60)).padStart(2, '0');
            dur = `${m}:${s}`;
          }
          return `${favIcon} ${playIcon} {#ffd24d-fg}${num}.{/#ffd24d-fg} {bold}${title}{/bold} {#00e5ff-fg}${artist}{/#00e5ff-fg} {#6f7e91-fg}${format}{/#6f7e91-fg} {#33ff33-fg}${dur}{/#33ff33-fg}`;
        }

        if (state.mode === 'RADIO STATIONS') {
          const name = sanitize(item.name, 22);
          const country = sanitize(item.country || 'WW', 6);
          const bitrate = `${item.bitrate || 128}k`;
          return `${favIcon} ${playIcon} {bold}${name}{/bold} {#00e5ff-fg}[${country}]{/#00e5ff-fg} {#ffd24d-fg}${bitrate}{/#ffd24d-fg}`;
        }

        if (state.mode === 'YOUTUBE MUSIC') {
          const num = String(index + 1).padStart(2, '0');
          const title = sanitize(item.title || item.name, 18);
          const artist = sanitize(item.artist || 'YouTube', 10);
          const badge = item.isTopic ? '{#00e5ff-fg}[TOPIC]{/#00e5ff-fg}' : '{#ff3344-fg}[YT]{/#ff3344-fg}';
          let dur = '';
          if (item.duration) {
            const m = Math.floor(item.duration / 60);
            const s = String(Math.floor(item.duration % 60)).padStart(2, '0');
            dur = `${m}:${s}`;
          }
          return `${favIcon} ${playIcon} {#ffd24d-fg}${num}.{/#ffd24d-fg} {bold}${title}{/bold} {#00e5ff-fg}${artist}{/#00e5ff-fg} ${badge} {#33ff33-fg}${dur}{/#33ff33-fg}`;
        }

        // QUEUE mode
        const num = String(index + 1).padStart(2, '0');
        const title = sanitize(item.title || item.name, 24);
        const tag = item.type === 'local'
          ? '{#00e5ff-fg}[LOCAL]{/#00e5ff-fg}'
          : item.type === 'youtube'
          ? '{#ff3344-fg}[YOUTUBE]{/#ff3344-fg}'
          : '{#f5c542-fg}[RADIO]{/#f5c542-fg}';
        return `${playIcon} {#ffd24d-fg}${num}.{/#ffd24d-fg} {bold}${title}{/bold} ${tag}`;
      });
    }

    explorerList.setItems(itemsToRender);
    explorerList.select(Math.max(0, state.selectedIndex));

    if (!searchBox.focused) searchBox.setValue(state.query);
    screen.render();
  }

  function search() {
    searchBox.focus();
    searchBox.readInput();
    screen.render();
  }

  function openUrlModal() {
    urlModal.show();
    urlInput.setValue('');
    urlInput.focus();
    urlInput.readInput();
    screen.render();
  }

  urlInput.on('submit', (value) => {
    urlModal.hide();
    explorerList.focus();
    if (value && value.trim()) {
      actions.loadYouTubeUrl(value.trim());
    }
    render(store.state);
  });

  urlInput.on('cancel', () => {
    urlModal.hide();
    explorerList.focus();
    render(store.state);
  });

  function isWebStreamUrl(val) {
    if (!val) return false;
    const s = val.trim();
    return s.startsWith('http://') ||
           s.startsWith('https://') ||
           s.startsWith('yt:') ||
           /^(music\.)?youtube\.com\//i.test(s) ||
           /^youtu\.be\//i.test(s) ||
           /^www\.youtube\.com\//i.test(s);
  }

  searchBox.on('submit', (value) => {
    const val = (value || '').trim();
    if (isWebStreamUrl(val)) {
      actions.loadYouTubeUrl(val);
    } else if (store.state.mode === 'YOUTUBE MUSIC') {
      if (val) actions.searchYouTube(val);
    } else {
      store.filter(val);
    }
    explorerList.focus();
    render(store.state);
  });

  searchBox.on('cancel', () => {
    explorerList.focus();
    render(store.state);
  });

  // Centralized, Context-Aware Keyboard Dispatcher (Collision-Free)
  screen.on('keypress', (ch, key = {}) => {
    const keyName = key.name || '';
    const isShift = Boolean(key.shift);
    const isCtrl = Boolean(key.ctrl);
    const fullKey = key.full || keyName;

    // 1. Emergency Quit (Ctrl+C)
    if (fullKey === 'C-c') {
      actions.quit();
      return;
    }

    // 2. TEXT_INPUT Context: When search or URL modal is active, isolate typing
    if (urlModal.visible || urlInput.focused || searchBox.focused) {
      if (keyName === 'escape') {
        if (urlModal.visible) {
          urlModal.hide();
          explorerList.focus();
        }
        if (searchBox.focused) {
          explorerList.focus();
        }
        render(store.state);
      }
      return; // Do not process any single-key shortcuts while typing!
    }

    // Global Quit when not typing
    if (keyName === 'q' && !isCtrl && !isShift) {
      actions.quit();
      return;
    }

    // 3. LYRICS_VIEW Context
    if (store.state.lyricsVisible) {
      if (ch === 'L' || ch === 'l' || keyName === 'escape' || keyName === 'backspace') {
        actions.toggleLyrics(false);
        render(store.state);
        return;
      }
      if (keyName === 'j' || keyName === 'down') {
        actions.scrollLyrics(1);
        render(store.state);
        return;
      }
      if (keyName === 'k' || keyName === 'up') {
        actions.scrollLyrics(-1);
        render(store.state);
        return;
      }
      if (keyName === 'pageup') {
        actions.scrollLyrics(-5);
        render(store.state);
        return;
      }
      if (keyName === 'pagedown') {
        actions.scrollLyrics(5);
        render(store.state);
        return;
      }
      if (ch === '<' || ch === ',' || ch === '[') {
        actions.adjustLyricsSyncOffset(-0.5);
        render(store.state);
        return;
      }
      if (ch === '>' || ch === '.' || ch === ']') {
        actions.adjustLyricsSyncOffset(0.5);
        render(store.state);
        return;
      }
      if (keyName === 'space') {
        actions.togglePause();
        return;
      }
      if (keyName === 'n') {
        actions.next();
        return;
      }
      if (keyName === 'p') {
        actions.prev();
        return;
      }
      if (ch === '+' || ch === '=') {
        actions.volume(5);
        return;
      }
      if (ch === '-' || ch === '_') {
        actions.volume(-5);
        return;
      }
      return;
    }

    // 4. ARTWORK_VIEW Context
    if (showFullArtwork) {
      if (ch === 'W' || ch === 'w' || keyName === 'escape' || keyName === 'backspace') {
        showFullArtwork = false;
        store.update({ status: 'Deck View: [ CASSETTE DECK ]' });
        render(store.state);
        return;
      }
      if (keyName === 'space') {
        actions.togglePause();
        return;
      }
      if (keyName === 'n') {
        actions.next();
        return;
      }
      if (keyName === 'p') {
        actions.prev();
        return;
      }
      if (ch === '+' || ch === '=') {
        actions.volume(5);
        return;
      }
      if (ch === '-' || ch === '_') {
        actions.volume(-5);
        return;
      }
      return;
    }

    // 5. EXPLORER Context (Main Crates & Deck Navigation)

    // Navigation (k/up, j/down, pageup, pagedown)
    if (keyName === 'k' || keyName === 'up') {
      actions.move(-1);
      render(store.state);
      return;
    }
    if (keyName === 'j' || keyName === 'down') {
      actions.move(1);
      render(store.state);
      return;
    }
    if (keyName === 'pageup') {
      actions.move(-8);
      render(store.state);
      return;
    }
    if (keyName === 'pagedown') {
      actions.move(8);
      render(store.state);
      return;
    }

    // Drill In (Enter, Right Arrow, Lowercase 'l')
    if (keyName === 'return' || keyName === 'enter' || keyName === 'right' || ch === 'l') {
      if (store.state.mode === 'LOCAL TRACKS') {
        actions.drillDown();
      } else {
        actions.play();
      }
      render(store.state);
      return;
    }

    // Drill Out / Back Up (Escape, Backspace, Left Arrow, Lowercase 'h')
    if (keyName === 'escape' || keyName === 'backspace' || keyName === 'left' || ch === 'h') {
      if (store.state.mode === 'LOCAL TRACKS') {
        const handled = actions.drillUp();
        if (handled) {
          render(store.state);
          return;
        }
      }
      return;
    }

    // Toggle Lyrics (Shift+L / Uppercase 'L')
    if (ch === 'L') {
      showFullArtwork = false;
      actions.toggleLyrics();
      render(store.state);
      return;
    }

    // Toggle Full Artwork ('W' / 'w')
    if (ch === 'W' || ch === 'w') {
      showFullArtwork = !showFullArtwork;
      if (showFullArtwork) store.state.lyricsVisible = false;
      store.update({ status: showFullArtwork ? 'Mode: [ FULL ALBUM ARTWORK VIEW ]' : 'Mode: [ CASSETTE DECK VIEW ]' });
      render(store.state);
      return;
    }

    // Search & URL Loader
    if (ch === '/') {
      search();
      return;
    }
    if (ch === 'y' || ch === 'Y' || ch === 'u' || ch === 'U') {
      openUrlModal();
      return;
    }

    // Mode Selection (Tab / m / Shift+Tab / M / Numbers 1-4)
    if (keyName === 'tab' || ch === 'm') {
      if (isShift || ch === 'M') actions.cycleMode(-1);
      else actions.cycleMode(1);
      render(store.state);
      return;
    }
    if (ch === 'M') {
      actions.cycleMode(-1);
      render(store.state);
      return;
    }
    if (ch === '1') {
      if (store.state.mode === 'LOCAL TRACKS') store.setNavLevel('ARTISTS');
      else store.setMode('LOCAL TRACKS');
      render(store.state);
      return;
    }
    if (ch === '2') {
      if (store.state.mode === 'LOCAL TRACKS') store.setNavLevel('ALBUMS');
      else store.setMode('RADIO STATIONS');
      render(store.state);
      return;
    }
    if (ch === '3') {
      if (store.state.mode === 'LOCAL TRACKS') store.setNavLevel('PLAYLISTS');
      else store.setMode('QUEUE');
      render(store.state);
      return;
    }
    if (ch === '4') {
      if (store.state.mode === 'LOCAL TRACKS') store.setNavLevel('ALL TRACKS');
      else store.setMode('YOUTUBE MUSIC');
      render(store.state);
      return;
    }
    if (ch === 'v' || ch === 'V') {
      if (store.state.mode === 'LOCAL TRACKS') {
        actions.cycleNavLevel(1);
        render(store.state);
      }
      return;
    }

    // Genre Filtering ('g' / 'G')
    if (ch === 'g') {
      store.cycleGenre(1);
      render(store.state);
      return;
    }
    if (ch === 'G') {
      store.cycleGenre(-1);
      render(store.state);
      return;
    }

    // Playback Controls
    if (keyName === 'space') {
      actions.togglePause();
      return;
    }
    if (keyName === 'n') {
      actions.next();
      return;
    }
    if (keyName === 'p' || ch === 'P') {
      actions.prev();
      return;
    }

    // Seeking (Shift+Left / Shift+Right / H)
    if (fullKey === 'S-left' || ch === 'H') {
      actions.seek(-10);
      return;
    }
    if (fullKey === 'S-right') {
      actions.seek(10);
      return;
    }

    // Volume (+ / = / - / _)
    if (ch === '+' || ch === '=') {
      actions.volume(5);
      return;
    }
    if (ch === '-' || ch === '_') {
      actions.volume(-5);
      return;
    }

    // DSP Hardware Toggles (s/S, d/D, t/T, b/B)
    if (ch === 's' || ch === 'S') {
      actions.cycleStereoMode(1);
      render(store.state);
      return;
    }
    if (ch === 'd' || ch === 'D') {
      actions.cycleDolbyMode(1);
      render(store.state);
      return;
    }
    if (ch === 't' || ch === 'T') {
      actions.cycleTapeType(1);
      render(store.state);
      return;
    }
    if (ch === 'b' || ch === 'B') {
      actions.toggleBassBoost();
      render(store.state);
      return;
    }

    // Queue & Track Management (z, r, a/A, c/C, x/X/Delete, f, o)
    if (ch === 'z') {
      actions.toggleShuffle();
      return;
    }
    if (ch === 'r') {
      actions.toggleRepeat();
      return;
    }
    if (ch === 'a' || ch === 'A') {
      actions.addToQueue();
      return;
    }
    if (ch === 'c' || ch === 'C') {
      actions.clearQueue();
      return;
    }
    if (ch === 'x' || ch === 'X' || keyName === 'delete') {
      actions.removeFromQueue();
      return;
    }
    if (ch === 'f' || ch === 'F') {
      actions.favorite();
      return;
    }
    if (ch === 'o' || ch === 'O') {
      actions.scanMusicDir();
      return;
    }
  });

  store.subscribe(render);
  explorerList.focus();
  render(store.state);

  // Audio Telemetry & Visualizer Master Frame Clock (Strict 30 FPS = ~33.3ms)
  const FRAME_INTERVAL = 1000 / 30;
  let lastFrameTime = performance.now();
  let marqueeTick = 0;

  const animTimer = setInterval(() => {
    const now = performance.now();
    const elapsed = now - lastFrameTime;
    if (elapsed < FRAME_INTERVAL - 2) return;
    lastFrameTime = now;

    // Update marquee text scroll offset every ~100ms (every 3 frames)
    if (++marqueeTick >= 3) {
      marqueeOffset++;
      marqueeTick = 0;
    }

    const telemetry = player?.getTelemetry?.() || {
      vuLeft: 0,
      vuRight: 0,
      peakLeft: 0,
      peakRight: 0,
      eqBands: Array(10).fill(0),
      eqPeaks: Array(10).fill(0),
      spoolFrame: 0,
      timePos: 0,
      duration: 0,
      percentPos: 0,
      tapeCounter: '00:00',
      elapsedMs: 0,
      hasDuration: false,
      audioCodec: '',
      audioBitrate: 0,
      audioSampleRate: 0
    };

    store.state.tapeCounter = telemetry.tapeCounter;
    store.state.timePos = telemetry.timePos;
    store.state.duration = telemetry.duration;
    store.state.percentPos = telemetry.percentPos;
    store.updateActiveLyric(telemetry.timePos);

    // Render Unified Monitor Console (Cassette Bay + Phosphor LCD Screen + Album Art)
    const marquee = getMarqueeText(store.state.metadata, 46);
    monitorConsole.setContent(
      formatCombinedMonitor(
        telemetry.spoolFrame,
        store.state.playing,
        store.state.paused,
        store.state.current,
        store.state.mode,
        store.state,
        marquee,
        currentArtLines,
        showFullArtwork,
        telemetry
      )
    );

    // Render Wide VU Meters & 10-band Graphic Equalizer with Smooth Ballistics
    const vuL = formatVuMeter('L CH', telemetry.vuLeft, telemetry.peakLeft, 56);
    const vuR = formatVuMeter('R CH', telemetry.vuRight, telemetry.peakRight, 56);
    const eq = formatEqualizer(telemetry.eqBands, telemetry.eqPeaks, 6);

    const scaleLine = ` {#6f7e91-fg}SCALE  -30    -20    -15    -10     -7     -5     -3     -1      0     +1     +2     +3 dB{/#6f7e91-fg}`;

    vuEqualizerBox.setContent([
      scaleLine,
      vuL,
      vuR,
      ``,
      ` {bold}{#f5c542-fg}10-BAND GRAPHIC EQUALIZER SPECTRUM{/#f5c542-fg}{/bold}`,
      eq
    ].join('\n'));

    screen.render();
  }, 16); // 60Hz tick, locked & clamped to 30 FPS redraw

  screen.on('destroy', () => {
    clearInterval(animTimer);
  });

  return { screen, render, animTimer };
}
