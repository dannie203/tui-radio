use crate::api::lyrics::MATRIX_CHARS;
use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_lyrics(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let title = state
        .current_track
        .as_ref()
        .map(|t| t.title.as_str())
        .unwrap_or("UNKNOWN TRACK");
    let artist = state
        .current_track
        .as_ref()
        .map(|t| t.artist.as_str())
        .unwrap_or("UNKNOWN ARTIST");

    let mut lines = Vec::new();
    let offset_info = if state.lyrics_offset != 0.0 {
        format!(" (Offset: {:+.2}s)", state.lyrics_offset)
    } else {
        String::new()
    };

    lines.push(Line::from(vec![
        Span::styled(" 🎤 LIVE SYNCED KARAOKE LYRICS ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(theme.border_dim)),
        Span::styled(format!("{} — {}{}", title, artist, offset_info), Style::default().fg(theme.cream).add_modifier(Modifier::BOLD)),
        Span::styled("  (['/'] ±0.25s • '{'/'}' ±1s • '0' reset • 'Shift+S' matrix cipher • 'l' deck)", Style::default().fg(theme.muted)),
    ]));
    lines.push(Line::from(""));

    if state.lyrics_loading {
        lines.push(Line::from(vec![Span::styled(
            "   ⏳ Querying LRCLIB open-source synced lyrics provider...",
            Style::default().fg(theme.gold),
        )]));
    } else if state.lyrics.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "   ∅ No synced lyrics found for this track / broadcast.",
            Style::default().fg(theme.muted),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "   • Local tracks: place a .lrc file next to the track or embed Vorbis/ID3 tags.",
            Style::default().fg(theme.muted),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "   • Online streams: auto-fetched via LRCLIB when artist/title are identified.",
            Style::default().fg(theme.muted),
        )]));
    } else {
        let cur_time = state.telemetry.time_pos + state.lyrics_offset;
        let active_opt = state
            .lyrics
            .iter()
            .rposition(|l| l.time <= cur_time);

        let tick = state.telemetry.spool_frame;
        let available_rows = (area.height as usize).saturating_sub(7).max(4);
        let half_rows = available_rows / 2;

        let center_idx = active_opt.unwrap_or(0);
        let start = center_idx.saturating_sub(half_rows);
        let end = (center_idx + half_rows + 4).min(state.lyrics.len());

        // Display Instrumental Intro lead-in badge if song is still in intro
        if let Some(first_lyric) = state.lyrics.first() {
            if cur_time < first_lyric.time {
                let remaining_intro = (first_lyric.time - cur_time).max(0.0);
                let intro_m = (remaining_intro / 60.0).floor() as u32;
                let intro_s = (remaining_intro % 60.0).floor() as u32;
                lines.push(Line::from(vec![
                    Span::styled("   🎵 ", Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("INSTRUMENTAL INTRO — VOCALS START IN {:02}:{:02}", intro_m, intro_s),
                        Style::default().fg(theme.amber).add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::from(""));
            }
        }

        for (i, lyric) in state.lyrics[start..end].iter().enumerate() {
            let actual_idx = start + i;
            let time_diff = lyric.time - cur_time;

            if Some(actual_idx) == active_opt {
                // ACTIVE LINE (Currently Being Sung!)
                lines.push(Line::from(vec![
                    Span::styled(" ▶ ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!(" {} ", &lyric.text),
                        Style::default()
                            .fg(theme.green_phosphor)
                            .bg(theme.bg_dark)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
            } else if active_opt.is_some() && actual_idx < active_opt.unwrap() {
                // PAST LINES (Dimmed out)
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(&lyric.text, Style::default().fg(theme.muted)),
                ]));
            } else if state.matrix_scramble {
                // UPCOMING LINES (or line 0 during intro before decrypt window)
                let words: Vec<&str> = lyric.text.split_whitespace().collect();
                let total_words = words.len();

                if total_words == 0 {
                    continue;
                }

                // If time_diff > 3.2s, line is still 100% encrypted in matrix cipher!
                // As time_diff approaches 0s (within 3.2s window), words un-matrix one by one!
                let decrypt_window = 3.2f64;
                let is_in_decrypt_window = time_diff >= 0.0 && time_diff <= decrypt_window;

                if is_in_decrypt_window {
                    let progress = ((decrypt_window - time_diff) / decrypt_window).clamp(0.0, 1.0) as f32;
                    let reveal_word_count = (progress * (total_words as f32 + 0.5)).floor() as usize;

                    let mut line_spans = vec![Span::styled("   ", Style::default())];

                    // 1. Fully un-matrixed words (clear text in bright cream)
                    if reveal_word_count > 0 {
                        let unmatrixed_slice = &words[..reveal_word_count.min(total_words)];
                        let unmatrixed_str = unmatrixed_slice.join(" ");
                        line_spans.push(Span::styled(
                            format!("{} ", unmatrixed_str),
                            Style::default().fg(theme.cream).add_modifier(Modifier::BOLD),
                        ));
                    }

                    // 2. Currently decrypting word (glitching amber matrix characters)
                    if reveal_word_count < total_words {
                        let curr_word = words[reveal_word_count];
                        let mut curr_scrambled = String::new();
                        for (char_idx, &ch) in curr_word.chars().collect::<Vec<_>>().iter().enumerate() {
                            let rnd_idx = (tick * 4 + reveal_word_count * 7 + char_idx * 11 + (ch as usize)) % MATRIX_CHARS.len();
                            curr_scrambled.push(MATRIX_CHARS[rnd_idx]);
                        }
                        line_spans.push(Span::styled(
                            format!("{} ", curr_scrambled),
                            Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD),
                        ));

                        // 3. Remaining future words (still in dim matrix cipher)
                        if reveal_word_count + 1 < total_words {
                            let mut cipher_words = Vec::new();
                            for (w_offset, word) in words[(reveal_word_count + 1)..].iter().enumerate() {
                                let w_idx = reveal_word_count + 1 + w_offset;
                                let mut word_scrambled = String::new();
                                for (c_idx, &ch) in word.chars().collect::<Vec<_>>().iter().enumerate() {
                                    let rnd_idx = (tick + w_idx * 13 + c_idx * 7 + (ch as usize)) % MATRIX_CHARS.len();
                                    word_scrambled.push(MATRIX_CHARS[rnd_idx]);
                                }
                                cipher_words.push(word_scrambled);
                            }
                            line_spans.push(Span::styled(
                                cipher_words.join(" "),
                                Style::default().fg(theme.border_dim),
                            ));
                        }
                    }
                    lines.push(Line::from(line_spans));
                } else {
                    // Fully encrypted in matrix cipher (time_diff > 3.2s)
                    let mut cipher_words = Vec::new();
                    for (w_idx, word) in words.iter().enumerate() {
                        let mut word_scrambled = String::new();
                        for (c_idx, &ch) in word.chars().collect::<Vec<_>>().iter().enumerate() {
                            let rnd_idx = (tick + w_idx * 13 + c_idx * 7 + (ch as usize)) % MATRIX_CHARS.len();
                            word_scrambled.push(MATRIX_CHARS[rnd_idx]);
                        }
                        cipher_words.push(word_scrambled);
                    }
                    lines.push(Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(cipher_words.join(" "), Style::default().fg(theme.border_dim)),
                    ]));
                }
            } else {
                // Plain readable text for upcoming lines when matrix scramble is disabled
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default()),
                    Span::styled(&lyric.text, Style::default().fg(theme.border_dim)),
                ]));
            }
        }
    }

    let lyrics_block = Paragraph::new(lines).block(
        Block::default()
            .title(" 🎤 REAL-TIME SYNCED LYRICS ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.cyan_dolby)),
    );

    f.render_widget(lyrics_block, area);
}
