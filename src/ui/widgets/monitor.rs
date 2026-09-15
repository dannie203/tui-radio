use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_monitor(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let current = state.current_track.as_ref();
    let title = current.map(|t| t.title.as_str()).unwrap_or("STANDBY — NO AUDIO LOADED");
    let artist = current.map(|t| t.artist.as_str()).unwrap_or("Insert Cassette Tape or Select Station");
    let album_name = current.and_then(|t| t.album.as_deref()).unwrap_or("Single / Unknown Album");

    // Dynamic Source Detection
    let (source_badge, source_color) = if let Some(t) = current {
        if t.is_radio {
            ("📻 LIVE RADIO", theme.amber_bright)
        } else if t.url.contains("music.youtube.com") {
            ("🔴 YT MUSIC", theme.red_led)
        } else if t.url.contains("youtube.com") || t.url.contains("youtu.be") || t.url.starts_with("yt:") || t.is_youtube {
            ("📺 YOUTUBE", theme.red_led)
        } else if t.url.contains("spotify.com") || t.url.starts_with("spotify:") {
            ("🟢 SPOTIFY", theme.green_phosphor)
        } else if t.url.contains("soundcloud.com") || t.url.starts_with("sc:") {
            ("🟠 SOUNDCLOUD", theme.gold)
        } else if t.url.contains("bandcamp.com") {
            ("🟣 BANDCAMP", theme.cyan_dolby)
        } else if t.url.starts_with("http://") || t.url.starts_with("https://") {
            ("🌐 WEB STREAM", theme.cyan_dolby)
        } else {
            ("💽 LOCAL MASTER", theme.green_phosphor)
        }
    } else {
        ("📼 DECK STANDBY", theme.muted)
    };

    // Dynamic Codec, Bit Depth, Bitrate, and Sampling Rate
    let codec_name = if !state.telemetry.audio_codec.is_empty() && state.telemetry.audio_codec != "STANDBY" {
        state.telemetry.audio_codec.clone()
    } else if let Some(fmt) = current.and_then(|t| t.format.as_ref()) {
        fmt.to_uppercase()
    } else {
        "PCM".to_string()
    };

    let sample_rate_val = if let Some(sr) = current.and_then(|t| t.sample_rate) {
        sr
    } else if state.telemetry.audio_sample_rate > 0 {
        state.telemetry.audio_sample_rate
    } else {
        44100
    };

    let bit_depth_val = if let Some(bd) = current.and_then(|t| t.bit_depth) {
        bd
    } else if state.telemetry.audio_bit_depth > 0 {
        state.telemetry.audio_bit_depth
    } else {
        16
    };

    // Formatted Sampling Rate Badge: "24/96k", "24/192k", "24/44.1k", "16/44.1k", "24/48k"
    let sample_rate_badge = if sample_rate_val % 1000 == 0 {
        format!("{}/{}k", bit_depth_val, sample_rate_val / 1000)
    } else {
        format!("{}/{:.1}k", bit_depth_val, sample_rate_val as f32 / 1000.0)
    };

    let sample_rate_str = format!("{:.1}kHz", sample_rate_val as f32 / 1000.0);

    let bitrate_str = if state.telemetry.audio_bitrate > 0 {
        format!("{}k", state.telemetry.audio_bitrate)
    } else if let Some(br) = current.and_then(|t| t.bitrate) {
        format!("{}k", br)
    } else {
        String::new()
    };

    // Rich Audio Spec string (Clean, no duplicate "Lossless • Lossless")
    let audio_spec = if codec_name == "FLAC" || codec_name == "WAV" {
        if !bitrate_str.is_empty() {
            format!("{} Lossless • {}-bit/{} • {}", codec_name, bit_depth_val, sample_rate_str, bitrate_str)
        } else {
            format!("{} Lossless • {}-bit/{}", codec_name, bit_depth_val, sample_rate_str)
        }
    } else if !bitrate_str.is_empty() {
        format!("{} • {} / {} {}", codec_name, bitrate_str, sample_rate_str, state.telemetry.audio_channels)
    } else {
        format!("{} • {} {}", codec_name, sample_rate_str, state.telemetry.audio_channels)
    };

    // File path & Size info
    let file_info_str = if let Some(t) = current {
        if !t.is_radio && !t.is_youtube && !t.url.starts_with("http") {
            let path_display = if let Some(home) = dirs::home_dir() {
                if let Some(home_str) = home.to_str() {
                    if !home_str.is_empty() && t.url.starts_with(home_str) {
                        t.url.replacen(home_str, "~", 1)
                    } else {
                        t.url.clone()
                    }
                } else {
                    t.url.clone()
                }
            } else {
                t.url.clone()
            };

            let size_str = if let Some(bytes) = t.file_size {
                if bytes >= 1_048_576 {
                    format!(" • {:.1} MB", bytes as f64 / 1_048_576.0)
                } else {
                    format!(" • {:.0} KB", bytes as f64 / 1024.0)
                }
            } else {
                "".to_string()
            };

            format!("{}{}", path_display, size_str)
        } else {
            let clean_url: String = t.url.chars().take(55).collect();
            format!("Stream: {}", clean_url)
        }
    } else {
        "No media loaded".to_string()
    };

    let width = (area.width as usize).saturating_sub(4);
    let title_clip = clip_to_width(title, width.saturating_sub(12));
    let artist_clip = clip_to_width(artist, width.saturating_sub(26));
    let album_clip = clip_to_width(album_name, width.saturating_sub(30));
    let file_clip = clip_to_width(&file_info_str, width.saturating_sub(12));

    // Line 1: Source & Soundstage & Sample Rate Spec
    let live_badge_span = if state.telemetry.is_live {
        Span::styled(" [● LIVE]", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("", Style::default())
    };

    let source_line = Line::from(vec![
        Span::styled(" SOURCE  : [", Style::default().fg(theme.muted)),
        Span::styled(source_badge, Style::default().fg(source_color).add_modifier(Modifier::BOLD)),
        Span::styled("]", Style::default().fg(theme.muted)),
        live_badge_span,
        Span::styled("       SAMPLE RATE: ", Style::default().fg(theme.muted)),
        Span::styled(format!("[{}]", sample_rate_badge), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        Span::styled("   SOUNDSTAGE: ", Style::default().fg(theme.muted)),
        Span::styled(
            state.stereo_mode.label(),
            Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD),
        ),
    ]);

    // Line 2: Title & Track No
    let trk_no_span = if let Some(no) = current.and_then(|t| t.track_no) {
        Span::styled(format!(" [Trk #{:02}]", no), Style::default().fg(theme.amber_bright))
    } else {
        Span::styled("", Style::default())
    };

    let title_line = Line::from(vec![
        Span::styled(" TITLE   : ", Style::default().fg(theme.muted)),
        Span::styled(title_clip, Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        trk_no_span,
    ]);

    // Line 3: Artist & Album
    let artist_line = Line::from(vec![
        Span::styled(" ARTIST  : ", Style::default().fg(theme.muted)),
        Span::styled(artist_clip, Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        Span::styled("   ALBUM: ", Style::default().fg(theme.muted)),
        Span::styled(album_clip, Style::default().fg(theme.cyan_dolby)),
    ]);

    // Line 4: File Path & Size
    let file_line = Line::from(vec![
        Span::styled(" FILE    : ", Style::default().fg(theme.muted)),
        Span::styled(file_clip, Style::default().fg(theme.chrome)),
    ]);

    // Line 5: Progress Bar / Live Stream Scanner
    let track_dur = if state.telemetry.duration > 0.0 {
        state.telemetry.duration
    } else {
        current.map(|t| t.duration).unwrap_or(0.0)
    };

    let prog_width: usize = (width.saturating_sub(35)).clamp(18, 80);
    let (progress_bar, prog_details) = if !state.telemetry.is_live && track_dur > 0.0 {
        let pct = (state.telemetry.time_pos / track_dur).clamp(0.0, 1.0);
        let filled = (pct * prog_width as f64).round() as usize;
        let filled_str = "■".repeat(filled);
        let empty_str = "□".repeat(prog_width.saturating_sub(filled));

        let total_m = (track_dur / 60.0).floor() as u32;
        let total_s = (track_dur % 60.0).floor() as u32;
        let pct_round = (pct * 100.0).round() as u32;
        (
            format!("[{}{}]", filled_str, empty_str),
            format!("{}/ {:02}:{:02} ({}%)", state.telemetry.tape_counter, total_m, total_s, pct_round),
        )
    } else if state.telemetry.is_live || (state.is_playing && !state.is_paused) {
        let frame = state.telemetry.spool_frame;
        let max_pos = prog_width.saturating_sub(4);
        let pos = if max_pos > 0 { frame % max_pos } else { 0 };
        let mut bar = String::from("[");
        for i in 0..prog_width {
            if i >= pos && i < pos + 4 {
                bar.push('■');
            } else {
                bar.push('□');
            }
        }
        bar.push(']');
        (bar, format!("{} ● LIVE BROADCAST", state.telemetry.tape_counter))
    } else {
        (format!("[{}]", "□".repeat(prog_width)), format!("{} (STANDBY)", state.telemetry.tape_counter))
    };

    let prog_line = Line::from(vec![
        Span::styled(" PROG    : ", Style::default().fg(theme.muted)),
        Span::styled(progress_bar, Style::default().fg(theme.green_phosphor)),
        Span::styled(format!("  {}", prog_details), Style::default().fg(theme.amber_bright)),
    ]);

    // Line 6: Dynamic Audio Spec, EQ & Volume
    let vol_filled = (state.volume as usize * 10) / 100;
    let vol_bar = format!("[{}{}] {}%", "■".repeat(vol_filled), "□".repeat(10 - vol_filled), state.volume);

    let status_line = Line::from(vec![
        Span::styled(" AUDIO   : ", Style::default().fg(theme.muted)),
        Span::styled(format!("[{}]  ", audio_spec), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        Span::styled("EQ: ", Style::default().fg(theme.muted)),
        Span::styled(format!("[{}]", state.eq_preset.label()), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("  VOL: ", Style::default().fg(theme.muted)),
        Span::styled(vol_bar, Style::default().fg(theme.amber)),
    ]);

    // Line 7: System Status
    let rec_label = if state.is_recording {
        format!("[{}] ", state.recording_status)
    } else {
        String::new()
    };
    let sys_line = Line::from(vec![
        Span::styled(" SYSTEM  : ", Style::default().fg(theme.muted)),
        Span::styled(
            rec_label,
            Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD),
        ),
        Span::styled(&state.status_message, Style::default().fg(theme.amber_bright)),
    ]);

    let mut lines = vec![
        source_line,
        title_line,
        artist_line,
        file_line,
        prog_line,
        status_line,
        sys_line,
    ];

    // Dynamic CRT Oscilloscope Waveform (if monitor height has extra room)
    let inner_height = (area.height as usize).saturating_sub(2);
    if inner_height > 7 {
        let extra_lines = inner_height - 7;
        let mut scope_lines = render_crt_oscilloscope_lines(width, extra_lines, state, theme);
        lines.append(&mut scope_lines);
    }

    let monitor = Paragraph::new(lines).block(
        Block::default()
            .title(" 📟 CRT PHOSPHOR MONITOR & OSCILLOSCOPE ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_lcd))
            .style(Style::default().bg(theme.bg_lcd)),
    );

    f.render_widget(monitor, area);
}

/// Helper to map 2x4 sub-pixel coordinates to Unicode Braille bitmask
#[inline(always)]
fn braille_bit(sub_x: usize, sub_y: usize) -> u8 {
    match (sub_x, sub_y) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, 3) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, 3) => 0x80,
        _ => 0,
    }
}

/// Generates real-time 2D CRT Oscilloscope beam traces with reticle grid using high-resolution sub-pixel Braille
fn render_crt_oscilloscope_lines<'a>(
    width: usize,
    available_lines: usize,
    state: &AppState,
    theme: &Theme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    if available_lines == 0 {
        return lines;
    }

    let is_active = state.is_playing && !state.is_paused;
    let wave_l = &state.telemetry.wave_left;
    let wave_r = &state.telemetry.wave_right;

    // Fast Schmitt-trigger rising zero-crossing detection on CH1 to stabilize the oscilloscope sweep
    let mut trigger_idx = 0;

    if is_active {
        let mut peak_val = 0.0f32;
        for &s in wave_l.iter().take(256) {
            let a = s.abs();
            if a > peak_val {
                peak_val = a;
            }
        }

        if peak_val > 0.02 {
            let thresh = (peak_val * 0.12).clamp(0.008, 0.06);
            for i in 1..250 {
                if wave_l[i - 1] <= 0.0 && wave_l[i] > 0.0 && (wave_l[i] - wave_l[i - 1]) > thresh {
                    trigger_idx = i;
                    break;
                }
            }
        }
    }

    if available_lines == 1 {
        // Compact 1-line phosphor wave trace with 4x vertical Braille sub-pixels
        let wave_w = width.saturating_sub(44).clamp(16, 70);
        let w_dots = wave_w * 2;
        let mut grid = vec![0u8; wave_w];
        let mid_y = 1.5f32;
        let amp = 1.4f32;

        let max_samples = (512 - trigger_idx).saturating_sub(1);
        let sample_step = (max_samples.min(w_dots * 2).max(w_dots)) as f32 / w_dots.max(1) as f32;
        let mut prev_y: Option<usize> = None;

        for x in 0..w_dots {
            let s_idx = (trigger_idx + (x as f32 * sample_step) as usize).min(511);
            let s = wave_l[s_idx];
            let y = (mid_y - s * amp).round().clamp(0.0, 3.0) as usize;
            let col = x / 2;
            let sub_x = x % 2;

            let (y_min, y_max) = match prev_y {
                Some(p) => (p.min(y), p.max(y)),
                None => (y, y),
            };
            for sy in y_min..=y_max {
                grid[col] |= braille_bit(sub_x, sy);
            }
            prev_y = Some(y);
        }

        let mut wave_chars = String::with_capacity(wave_w);
        for &mask in &grid {
            let ch = char::from_u32(0x2800 + mask as u32).unwrap_or('─');
            wave_chars.push(ch);
        }

        lines.push(Line::from(vec![
            Span::styled(" SCOPE   : ", Style::default().fg(theme.muted)),
            Span::styled(wave_chars, Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("  [CH-1/2 REALTIME BEAM]", Style::default().fg(theme.amber_bright)),
        ]));
        return lines;
    }

    // Line 1: Header / Divider
    let header_prefix = " ── ⚡ CRT ANALOG OSCILLOSCOPE [STEREO BEAM TRACE] ";
    let fill_count = width.saturating_sub(header_prefix.chars().count());
    let fill_line = "─".repeat(fill_count);
    lines.push(Line::from(vec![
        Span::styled(header_prefix, Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        Span::styled(fill_line, Style::default().fg(theme.border_dim)),
    ]));

    // Determine number of waveform canvas rows
    let has_footer = available_lines >= 5;
    let wave_rows = available_lines.saturating_sub(1 + if has_footer { 1 } else { 0 });

    if wave_rows > 0 {
        let plot_width = width.saturating_sub(2);
        let w_dots = plot_width * 2;
        let h_dots = wave_rows * 4;
        let mid_y = (h_dots as f32 - 1.0) / 2.0;
        let amp = mid_y * 0.94;

        let mut grid_l = vec![vec![0u8; plot_width]; wave_rows];
        let mut grid_r = vec![vec![0u8; plot_width]; wave_rows];

        // Map horizontal dot space across waveform buffer
        let max_display_samples = (512 - trigger_idx).saturating_sub(1);
        let span_samples = (w_dots * 2).min(max_display_samples).max(w_dots);
        let sample_step = span_samples as f32 / w_dots.max(1) as f32;

        let mut prev_yl: Option<usize> = None;
        let mut prev_yr: Option<usize> = None;

        for x in 0..w_dots {
            let s_idx = (trigger_idx + (x as f32 * sample_step) as usize).min(511);
            let sl = wave_l[s_idx];
            let sr = wave_r[s_idx];

            let yl = (mid_y - sl * amp).round().clamp(0.0, (h_dots - 1) as f32) as usize;
            let yr = (mid_y - sr * amp).round().clamp(0.0, (h_dots - 1) as f32) as usize;

            let col = x / 2;
            let sub_x = x % 2;

            // Connect continuous vertical beam segments for Left channel
            let (l_min, l_max) = match prev_yl {
                Some(prev) => (prev.min(yl), prev.max(yl)),
                None => (yl, yl),
            };
            for y in l_min..=l_max {
                let r = y / 4;
                let sub_y = y % 4;
                grid_l[r][col] |= braille_bit(sub_x, sub_y);
            }
            prev_yl = Some(yl);

            // Connect continuous vertical beam segments for Right channel
            let (r_min, r_max) = match prev_yr {
                Some(prev) => (prev.min(yr), prev.max(yr)),
                None => (yr, yr),
            };
            for y in r_min..=r_max {
                let r = y / 4;
                let sub_y = y % 4;
                grid_r[r][col] |= braille_bit(sub_x, sub_y);
            }
            prev_yr = Some(yr);
        }

        let mid_row = (wave_rows - 1) / 2;

        // Render each row with grouped spans for maximum TUI throughput
        for r in 0..wave_rows {
            let mut row_spans = vec![Span::raw(" ")];
            let is_axis_row = r == mid_row;

            let mut current_text = String::new();
            let mut current_style: Option<Style> = None;

            for c in 0..plot_width {
                let mask_l = grid_l[r][c];
                let mask_r = grid_r[r][c];

                let (ch, style) = if mask_l > 0 && mask_r > 0 {
                    let combined = mask_l | mask_r;
                    let b_ch = char::from_u32(0x2800 + combined as u32).unwrap_or('━');
                    (b_ch.to_string(), Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD))
                } else if mask_l > 0 {
                    let b_ch = char::from_u32(0x2800 + mask_l as u32).unwrap_or('─');
                    (b_ch.to_string(), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
                } else if mask_r > 0 {
                    let b_ch = char::from_u32(0x2800 + mask_r as u32).unwrap_or('─');
                    (b_ch.to_string(), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD))
                } else if is_axis_row {
                    let graticule = if c % 10 == 0 {
                        "┼"
                    } else if c % 2 == 0 {
                        "┄"
                    } else {
                        " "
                    };
                    (graticule.to_string(), Style::default().fg(theme.border_dim))
                } else if c % 10 == 0 {
                    ("┆".to_string(), Style::default().fg(theme.border_dim))
                } else {
                    (" ".to_string(), Style::default())
                };

                if Some(style) == current_style {
                    current_text.push_str(&ch);
                } else {
                    if let Some(prev_style) = current_style {
                        if !current_text.is_empty() {
                            row_spans.push(Span::styled(current_text.clone(), prev_style));
                            current_text.clear();
                        }
                    }
                    current_style = Some(style);
                    current_text.push_str(&ch);
                }
            }

            if let Some(style) = current_style {
                if !current_text.is_empty() {
                    row_spans.push(Span::styled(current_text, style));
                }
            }

            lines.push(Line::from(row_spans));
        }
    }

    // Line Footer: CRT Scope Hardware Parameters & Fixed Telemetry Status
    if has_footer {
        let (trig_str, trig_style) = if state.is_paused {
            ("[TRIG: LOCKED]", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
        } else if state.is_playing {
            ("[TRIG: AUTO SCAN]", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD))
        } else {
            ("[TRIG: STANDBY]", Style::default().fg(theme.muted))
        };

        lines.push(Line::from(vec![
            Span::styled(" [SWEEP: 50Hz] ", Style::default().fg(theme.muted)),
            Span::styled("[CH1: L-BEAM] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("[CH2: R-BEAM] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
            Span::styled("[0.5V/DIV] ", Style::default().fg(theme.chrome)),
            Span::styled("[TIME/DIV: 1.0ms] ", Style::default().fg(theme.amber_bright)),
            Span::styled(trig_str, trig_style),
        ]));
    }

    lines
}

fn clip_to_width(s: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthChar;
    let mut out = String::new();
    let mut current_width = 0;
    for ch in s.chars() {
        let w = ch.width().unwrap_or(0);
        if current_width + w > max_width {
            break;
        }
        out.push(ch);
        current_width += w;
    }
    out
}
