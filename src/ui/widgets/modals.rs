use crate::state::store::AppState;
use crate::state::types::ModalType;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    match state.active_modal {
        ModalType::None => {}
        ModalType::Help => render_help_modal(f, area, theme),
        ModalType::UrlInput => render_url_modal(f, area, state, theme),
        ModalType::Search => render_search_modal(f, area, state, theme),
        ModalType::Settings => render_settings_modal(f, area, state, theme),
        ModalType::Mixtape => render_mixtape_modal(f, area, state, theme),
        ModalType::History => render_history_modal(f, area, state, theme),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help_modal(f: &mut Frame, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);

    let k = |s: &'static str| Span::styled(format!(" {:<8} ", s), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD));
    let d = |s: &'static str| Span::styled(s, Style::default().fg(theme.cream));
    let h = |s: &'static str| Span::styled(format!("─── {} ───", s), Style::default().fg(theme.gold).add_modifier(Modifier::BOLD));

    let lines = vec![
        Line::from(vec![h("DECK & AUDIO PLAYBACK")]),
        Line::from(vec![k("Space"), d("Toggle Play / Pause"), k("s"), d("Stop Playback")]),
        Line::from(vec![k("n / p"), d("Next / Previous Track"), k("+ / -"), d("Volume Up / Down ±5%")]),
        Line::from(vec![k("[ / ]"), d("Seek Backward / Forward 10s")]),
        Line::from(""),
        Line::from(vec![h("MODES & NAVIGATION")]),
        Line::from(vec![k("1 - 4"), d("Direct Mode Switch (Local / Radio / Queue / Streams)")]),
        Line::from(vec![k("Tab"), d("Cycle Mode Forward"), k("j / k"), d("Move Selection Down / Up")]),
        Line::from(vec![k("Enter"), d("Play / Drill into Album"), k("Esc"), d("Drill Up / Back to Albums")]),
        Line::from(vec![k("v"), d("Toggle View (Albums ↔ All Tracks)"), k("a"), d("Add Track / Album to Queue")]),
        Line::from(vec![k("x / c"), d("Remove Item from Queue / Clear Entire Queue")]),
        Line::from(""),
        Line::from(vec![h("DSP & ENHANCEMENTS")]),
        Line::from(vec![k("b"), d("Toggle Mega Bass Boost"), k("d"), d("Cycle Dolby NR Mode")]),
        Line::from(vec![k("e"), d("Cycle 32-Band EQ Profile"), k("t"), d("Cycle Color Theme")]),
        Line::from(vec![k("r"), d("Cycle Repeat Mode"), k("z"), d("Toggle Random Shuffle")]),
        Line::from(vec![k("R"), d("Tape Record Current Track"), k("Ctrl+R"), d("Cancel All Recordings")]),
        Line::from(vec![k("M"), d("Mixtape Manager"), k("m"), d("Toggle Favorite Track")]),
        Line::from(vec![k("o"), d("Interactive Settings Dashboard")]),
        Line::from(""),
        Line::from(vec![h("VIEWS & OVERLAYS")]),
        Line::from(vec![k("l"), d("Live Synced Karaoke Lyrics"), k("w"), d("Album Cover Artwork View")]),
        Line::from(vec![k("H"), d("Playback History (Smart Deduplicated)"), k("M"), d("Mixtape Manager")]),
        Line::from(vec![k("u"), d("Universal Stream Search (YouTube, Spotify, SoundCloud)")]),
        Line::from(vec![k("/"), d("Live Filter & Search"), k("?"), d("Toggle This Help Screen")]),
        Line::from(vec![k("F5"), d("Hot-Reload App & Config"), k("q"), d("Quit Application")]),
    ];

    let block = Block::default()
        .title(" 📖 BOOMBOX-RS KEYBOARD SHORTCUTS & HELP [Esc / ? to Close] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.amber));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

fn render_url_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(68, 30, area);
    f.render_widget(Clear, popup_area);

    let input_str = state.input_buffer.trim();
    let detected_platform = if input_str.contains("spotify.com") || input_str.starts_with("spotify:") {
        Span::styled(" [SPOTIFY LINK] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
    } else if input_str.starts_with("sp:") {
        Span::styled(" [SPOTIFY SEARCH] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
    } else if input_str.contains("music.youtube.com") {
        Span::styled(" [YT MUSIC LINK] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else if input_str.contains("youtube.com") || input_str.contains("youtu.be") {
        Span::styled(" [YOUTUBE LINK] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else if input_str.starts_with("yt:") {
        Span::styled(" [YOUTUBE SEARCH] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else if input_str.contains("soundcloud.com") {
        Span::styled(" [SOUNDCLOUD LINK] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD))
    } else if input_str.starts_with("sc:") {
        Span::styled(" [SOUNDCLOUD SEARCH] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD))
    } else if input_str.starts_with("http") {
        Span::styled(" [WEB STREAM] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD))
    } else if !input_str.is_empty() {
        Span::styled(" [ONLINE SEARCH: YOUTUBE / STREAM] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [READY] ", Style::default().fg(theme.muted))
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Search songs or paste links (YouTube, SoundCloud, Spotify, Bandcamp, Apple Music):", Style::default().fg(theme.cream)),
            detected_platform,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("❯ ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
            Span::styled(&state.input_buffer, Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.green_phosphor)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Enter] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
            Span::styled("Search & Play Now   ", Style::default().fg(theme.cream)),
            Span::styled("[Ctrl+A] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("Add All to Queue   ", Style::default().fg(theme.cream)),
            Span::styled("[Esc] ", Style::default().fg(theme.muted)),
            Span::styled("Cancel", Style::default().fg(theme.muted)),
        ]),
    ];

    let block = Block::default()
        .title(" 🌐 UNIVERSAL ONLINE STREAM & SEARCH ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cyan_dolby));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, popup_area);
}

fn render_search_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(65, 25, area);
    f.render_widget(Clear, popup_area);

    let mode_hint = match state.mode {
        crate::state::types::AppMode::LocalTracks => "Filter local tracks and albums by title, artist, or album",
        crate::state::types::AppMode::RadioStations => "Filter radio stations by name, country, or genre",
        crate::state::types::AppMode::YoutubeMusic => "Search online songs across YouTube & SoundCloud",
        crate::state::types::AppMode::Queue => "Filter songs in current playback queue",
    };

    let text = vec![
        Line::from(vec![Span::styled(
            mode_hint,
            Style::default().fg(theme.cream),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(theme.amber)),
            Span::styled(&state.input_buffer, Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.green_phosphor)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "[Enter] Apply Search  •  [Esc] Clear / Close",
            Style::default().fg(theme.muted),
        )]),
    ];

    let block = Block::default()
        .title(" 🔍 SEARCH & FILTER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.amber));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, popup_area);
}

fn render_settings_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(75, 82, area);
    f.render_widget(Clear, popup_area);

    let offset_str = if state.lyrics_offset >= 0.0 {
        format!("+{:.2}s", state.lyrics_offset)
    } else {
        format!("{:.2}s", state.lyrics_offset)
    };

    let settings_items = [
        ("UI Color Theme", theme.name.to_string(), "Vintage / Cyber / Omarchy palette"),
        ("Soundstage Mode", state.stereo_mode.label().to_string(), "Stereo / Mono / 3D Wide Surround"),
        ("Dolby NR Bias", state.dolby_mode.label().to_string(), "High-frequency tape hiss reduction"),
        ("Tape Formulation", state.tape_type.label().to_string(), "Type-I Fe / Type-II CrO2 / Type-IV Metal"),
        ("Equalizer Preset", state.eq_preset.label().to_string(), "32-Band ISO Acoustic Curve"),
        ("Analog Mega Bass", if state.bass_boost { "🔴 ENABLED (+7dB Sub-Bass)".to_string() } else { "FLAT (0dB Reference)".to_string() }, "Analog low-frequency punch"),
        ("Spectrum Color Palette", state.spectrum_color_mode.label().to_string(), "32-Band ISO Chroma / LED / Cyberpunk / Matrix"),
        ("Visualizer Ballistics", state.visualizer_speed.label().to_string(), "Attack & Release EMA speed"),
        ("Tape Recording Format", state.record_format.label().to_string(), "OPUS / MP3 / FLAC / M4A"),
        ("Lyrics Sync Timing", format!("◄ {} ►", offset_str), "Fine-tune karaoke timing (±0.25s)"),
        ("Matrix Scramble Text", if state.matrix_scramble { "ENABLED (Cyberpunk)".to_string() } else { "DISABLED (Plain Text)".to_string() }, "Upcoming lyrics decryption FX"),
        ("Volume Key Step", format!("±{}%", state.volume_step), "Volume delta on +/- keypress"),
        ("Streaming Autoplay", if state.autoplay { "🟢 ENABLED (YouTube Mix)".to_string() } else { "⚪ DISABLED (Manual)".to_string() }, "Infinite stream recommendations"),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("⚙ SYSTEM HARDWARE PREFERENCES & DSP CONFIG ", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    for (i, (label, val, desc)) in settings_items.iter().enumerate() {
        let is_selected = i == state.settings_selected_idx;
        let prefix = if is_selected { " ❯ " } else { "   " };
        let prefix_style = if is_selected {
            Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };

        let label_style = if is_selected {
            Style::default().fg(theme.chrome).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.chrome)
        };

        let val_style = if is_selected {
            Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.cyan_dolby)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, prefix_style),
            Span::styled(format!("{:<24} : ", label), label_style),
            Span::styled(format!("{:<30}", val), val_style),
            Span::styled(format!(" │ {}", desc), Style::default().fg(theme.muted)),
        ]));
    }

    lines.push(Line::from(""));
    if let Some(ref update) = state.available_update {
        lines.push(Line::from(vec![
            Span::styled(" 🌟 [UPDATE AVAILABLE] ", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
            Span::styled(format!("v{} is ready! (Current: v{}) • Visit: {}", update.latest_version, update.current_version, update.release_url), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled(" ✔ [UP TO DATE] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Boombox RX-505 v{} (Official Latest)", env!("CARGO_PKG_VERSION")), Style::default().fg(theme.muted)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("─── CONTROLS ───────────────────────────────────────────────────────────", Style::default().fg(theme.gold))]));
    lines.push(Line::from(vec![
        Span::styled(" [↑ / ↓] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("Navigate   ", Style::default().fg(theme.cream)),
        Span::styled(" [Enter / Space / →] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle / Cycle   ", Style::default().fg(theme.cream)),
        Span::styled(" [[ / ]] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        Span::styled("Fine-Tune Timing   ", Style::default().fg(theme.cream)),
        Span::styled(" [Esc / o] ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Close", Style::default().fg(theme.muted)),
    ]));

    let block = Block::default()
        .title(" ⚙ SYSTEM PREFERENCES & SETTINGS DASHBOARD ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.gold));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

fn render_mixtape_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(72, 70, area);
    f.render_widget(Clear, popup_area);

    let mut lines = vec![
        Line::from(vec![Span::styled("🥏 MIXTAPE MANAGER", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("Rec. Format: ", Style::default().fg(theme.chrome)),
            Span::styled(state.record_format.label(), Style::default().fg(theme.green_phosphor)),
            Span::styled("  •  Recording: ", Style::default().fg(theme.chrome)),
            Span::styled(if state.is_recording { state.recording_status.clone() } else { "Idle".to_string() }, Style::default().fg(if state.is_recording { theme.red_led } else { theme.muted })),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("─── MIXTAPES ───────────────────────────────", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD))]),
    ];

    for (i, mt) in state.mixtapes.iter().enumerate() {
        let selected = i == state.selected_mixtape_idx;
        let prefix = if selected { "❯ ".to_string() } else { "  ".to_string() };
        let style = if selected {
            Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.cream)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{}{} [{} tracks] {}", prefix, mt.name, mt.tracks.len(), if mt.id == "mixtape:favorites" { "★" } else { "" }), style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("─── SELECTED MIXTAPE TRACKS ───────────────", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD))]));

    if let Some(mt) = state.selected_mixtape() {
        if mt.tracks.is_empty() {
            lines.push(Line::from(vec![Span::styled("  (No tracks — press [Enter] to add current track)", Style::default().fg(theme.muted))]));
        } else {
            for (i, t) in mt.tracks.iter().take(10).enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(format!("{:02}. ", i + 1), Style::default().fg(theme.amber_bright)),
                    Span::styled(format!("{} — {}", t.title, t.artist), Style::default().fg(theme.cream)),
                ]));
            }
            if mt.tracks.len() > 10 {
                lines.push(Line::from(vec![Span::styled(format!("  ... +{} more", mt.tracks.len() - 10), Style::default().fg(theme.muted))]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[↑/↓]", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled(" Select Mixtape   ", Style::default().fg(theme.cream)),
        Span::styled("[Enter]", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        Span::styled(" Add Track   ", Style::default().fg(theme.cream)),
        Span::styled("[n]", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled(" New   ", Style::default().fg(theme.cream)),
        Span::styled("[x]", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)),
        Span::styled(" Delete   ", Style::default().fg(theme.cream)),
        Span::styled("[a]", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled(" Add Current   ", Style::default().fg(theme.cream)),
        Span::styled("[R]", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)),
        Span::styled(" Record   ", Style::default().fg(theme.cream)),
        Span::styled("[Esc]", Style::default().fg(theme.muted)),
        Span::styled(" Close", Style::default().fg(theme.muted)),
    ]));

    let block = Block::default()
        .title(" 🥏 MIXTAPE & CUSTOM PLAYLIST MANAGER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.amber));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}

fn truncate_history_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}

fn render_history_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(82, 85, area);
    f.render_widget(Clear, popup_area);

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(
            format!("📜 RECENT PLAYBACK HISTORY ({} unique tracks) ", state.filtered_history.len()),
            Style::default().fg(theme.gold).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    if state.filtered_history.is_empty() {
        lines.push(Line::from(Span::styled(
            "   (No playback history yet. Start playing any track!)",
            Style::default().fg(theme.muted),
        )));
    } else {
        let max_visible = 18;
        let start_idx = if state.selected_history_idx >= max_visible {
            state.selected_history_idx - max_visible + 1
        } else {
            0
        };

        for (i, item) in state.filtered_history.iter().skip(start_idx).take(max_visible).enumerate() {
            let actual_idx = start_idx + i;
            let is_selected = actual_idx == state.selected_history_idx;
            let prefix = if is_selected { " ❯ " } else { "   " };
            let prefix_style = if is_selected {
                Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };

            let source_badge = match item.source.as_str() {
                "Radio" => Span::styled("[RADIO] ", Style::default().fg(theme.cyan_dolby)),
                "YouTube" => Span::styled("[YOUTUBE] ", Style::default().fg(theme.red_led)),
                "SoundCloud" => Span::styled("[SOUNDCLOUD] ", Style::default().fg(theme.amber)),
                "Web Stream" => Span::styled("[STREAM] ", Style::default().fg(theme.green_phosphor)),
                _ => Span::styled("[LOCAL] ", Style::default().fg(theme.gold)),
            };

            let title_span = Span::styled(
                format!("{:<32} ", truncate_history_str(&item.title, 30)),
                if is_selected {
                    Style::default().fg(theme.cream).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.cream)
                },
            );

            let artist_span = Span::styled(
                format!("{:<20} ", truncate_history_str(&item.artist, 18)),
                Style::default().fg(theme.muted),
            );

            let count_span = Span::styled(
                format!("★ {:>2} plays", item.play_count),
                Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD),
            );

            lines.push(Line::from(vec![
                Span::styled(prefix, prefix_style),
                source_badge,
                title_span,
                artist_span,
                count_span,
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("Play Track   ", Style::default().fg(theme.cream)),
        Span::styled("[a] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
        Span::styled("Add to Queue   ", Style::default().fg(theme.cream)),
        Span::styled("[m] ", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        Span::styled("Favorite   ", Style::default().fg(theme.cream)),
        Span::styled("[x] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)),
        Span::styled("Remove   ", Style::default().fg(theme.cream)),
        Span::styled("[c] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)),
        Span::styled("Clear All   ", Style::default().fg(theme.cream)),
        Span::styled("[Esc] ", Style::default().fg(theme.muted)),
        Span::styled("Close", Style::default().fg(theme.muted)),
    ]));

    let block = Block::default()
        .title(" 📜 PLAYBACK HISTORY (LOCAL PRIVACY-FIRST & SMART DEDUPLICATED) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.amber));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, popup_area);
}
