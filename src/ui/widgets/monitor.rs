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

    let sample_rate_val = if state.telemetry.audio_sample_rate > 0 {
        state.telemetry.audio_sample_rate
    } else if let Some(sr) = current.and_then(|t| t.sample_rate) {
        sr
    } else {
        44100
    };

    let bit_depth_val = if state.telemetry.audio_bit_depth > 0 {
        state.telemetry.audio_bit_depth
    } else if let Some(bd) = current.and_then(|t| t.bit_depth) {
        bd
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
                if t.url.starts_with(home.to_str().unwrap_or("")) {
                    t.url.replacen(home.to_str().unwrap_or(""), "~", 1)
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
    let title_clip: String = title.chars().take(width.saturating_sub(12)).collect();
    let artist_clip: String = artist.chars().take(width.saturating_sub(26)).collect();
    let album_clip: String = album_name.chars().take(width.saturating_sub(30)).collect();
    let file_clip: String = file_info_str.chars().take(width.saturating_sub(12)).collect();

    // Line 1: Source & Soundstage & Sample Rate Spec
    let source_line = Line::from(vec![
        Span::styled(" SOURCE  : [", Style::default().fg(theme.muted)),
        Span::styled(source_badge, Style::default().fg(source_color).add_modifier(Modifier::BOLD)),
        Span::styled("]       SAMPLE RATE: ", Style::default().fg(theme.muted)),
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
    let (progress_bar, prog_details) = if track_dur > 0.0 {
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
    } else if state.is_playing && !state.is_paused {
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
        (bar, format!("{} ● LIVE", state.telemetry.tape_counter))
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

/// Generates real-time 2D CRT Oscilloscope beam traces with reticle grid
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
    let t = state.telemetry.spool_frame as f32 * 0.26;
    let vu_l = state.telemetry.vu_left;
    let vu_r = state.telemetry.vu_right;

    let b_low = (state.telemetry.eq_bands[0] + state.telemetry.eq_bands[1] + state.telemetry.eq_bands[2]) / 300.0;
    let b_mid = (state.telemetry.eq_bands[8] + state.telemetry.eq_bands[10] + state.telemetry.eq_bands[12]) / 300.0;
    let b_high = (state.telemetry.eq_bands[20] + state.telemetry.eq_bands[24]) / 200.0;

    let amp_l = if is_active { (vu_l / 100.0).clamp(0.08, 1.0) } else { 0.04 };
    let amp_r = if is_active { (vu_r / 100.0).clamp(0.08, 1.0) } else { 0.04 };

    if available_lines == 1 {
        // Compact 1-line phosphor wave trace
        let wave_w = width.saturating_sub(42).clamp(16, 70);
        let mut wave_chars = String::with_capacity(wave_w);
        for x in 0..wave_w {
            let val = if is_active {
                ((x as f32 * 0.35 + t).sin() * 0.7 + (x as f32 * 0.7 - t * 1.5).cos() * 0.3) * amp_l
            } else {
                0.0
            };
            let ch = if val > 0.4 { "∿" } else if val < -0.4 { "∼" } else { "─" };
            wave_chars.push_str(ch);
        }

        lines.push(Line::from(vec![
            Span::styled(" SCOPE   : ", Style::default().fg(theme.muted)),
            Span::styled(wave_chars, Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("  [CH-1/2 ANALOG BEAM • 50Hz]", Style::default().fg(theme.amber_bright)),
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
        let mid_r = (wave_rows as f32 - 1.0) / 2.0;
        let plot_width = width.saturating_sub(2);

        // Precompute waveform target rows for all columns
        let mut target_l = Vec::with_capacity(plot_width);
        let mut target_r = Vec::with_capacity(plot_width);

        for x in 0..plot_width {
            let x_f = x as f32;
            let val_l = if is_active {
                let w1 = (x_f * 0.11 + t).sin() * (0.60 + b_low * 0.40);
                let w2 = (x_f * 0.25 - t * 1.3).sin() * (0.30 + b_mid * 0.35);
                let w3 = (x_f * 0.55 + t * 2.1).cos() * (0.15 + b_high * 0.25);
                ((w1 + w2 + w3) * amp_l).clamp(-1.0, 1.0)
            } else {
                ((x_f * 0.08 + t * 0.1).sin() * 0.05).clamp(-1.0, 1.0)
            };

            let val_r = if is_active {
                let w1 = (x_f * 0.13 - t * 0.95 + 1.2).sin() * (0.60 + b_low * 0.40);
                let w2 = (x_f * 0.22 + t * 1.45).cos() * (0.30 + b_mid * 0.35);
                let w3 = (x_f * 0.48 - t * 1.80).sin() * (0.15 + b_high * 0.25);
                ((w1 + w2 + w3) * amp_r).clamp(-1.0, 1.0)
            } else {
                ((x_f * 0.08 - t * 0.1).cos() * 0.05).clamp(-1.0, 1.0)
            };

            let row_l = (mid_r - val_l * mid_r * 0.94).round().clamp(0.0, (wave_rows - 1) as f32) as usize;
            let row_r = (mid_r - val_r * mid_r * 0.94).round().clamp(0.0, (wave_rows - 1) as f32) as usize;
            target_l.push(row_l);
            target_r.push(row_r);
        }

        // Render each row with grouped spans for high efficiency
        for r in 0..wave_rows {
            let mut row_spans = vec![Span::raw(" ")];
            let is_axis_row = r == mid_r.round() as usize;

            let mut current_text = String::new();
            let mut current_style: Option<Style> = None;

            for x in 0..plot_width {
                let hit_l = target_l[x] == r;
                let hit_r = target_r[x] == r;

                let (ch, style) = if hit_l && hit_r {
                    ("━", Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD))
                } else if hit_l {
                    ("─", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
                } else if hit_r {
                    ("─", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD))
                } else if is_axis_row {
                    if x % 10 == 0 {
                        ("┼", Style::default().fg(theme.border_dim))
                    } else if x % 2 == 0 {
                        ("┄", Style::default().fg(theme.border_dim))
                    } else {
                        (" ", Style::default())
                    }
                } else if x % 10 == 0 {
                    ("┆", Style::default().fg(theme.border_dim))
                } else {
                    (" ", Style::default())
                };

                if Some(style) == current_style {
                    current_text.push_str(ch);
                } else {
                    if let Some(prev_style) = current_style {
                        if !current_text.is_empty() {
                            row_spans.push(Span::styled(current_text.clone(), prev_style));
                            current_text.clear();
                        }
                    }
                    current_style = Some(style);
                    current_text.push_str(ch);
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

    // Line Footer: CRT Scope Hardware Parameters & Telemetry
    if has_footer {
        lines.push(Line::from(vec![
            Span::styled(" [SWEEP: 50Hz] ", Style::default().fg(theme.muted)),
            Span::styled("[CH1: L-BEAM] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("[CH2: R-BEAM] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
            Span::styled("[0.5V/DIV] ", Style::default().fg(theme.chrome)),
            Span::styled("[TIME/DIV: 1.0ms] ", Style::default().fg(theme.amber_bright)),
            Span::styled("[TRIG: AUTO SYNC]", Style::default().fg(theme.gold)),
        ]));
    }

    lines
}
