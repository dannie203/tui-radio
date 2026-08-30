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
        Line::from(vec![k("r"), d("Cycle Repeat Mode"), k("z"), d("Toggle Shuffle")]),
        Line::from(vec![k("X"), d("Cycle Automix & AI-DJ Modes"), k("N"), d("GPU Neural Scan")]),
        Line::from(vec![k("R"), d("Tape Record Current Track"), k("Ctrl+R"), d("Cancel All Recordings")]),
        Line::from(vec![k("M"), d("Mixtape Manager"), k("m"), d("Toggle Favorite")]),
        Line::from(vec![k("o"), d("Interactive Settings Dashboard")]),
        Line::from(""),
        Line::from(vec![h("VIEWS & OVERLAYS")]),
        Line::from(vec![k("l"), d("Live Synced Karaoke Lyrics"), k("w"), d("Album Cover Artwork View")]),
        Line::from(vec![k("u"), d("Stream URL (YouTube, Spotify, SoundCloud, etc.)")]),
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
        Span::styled(" [SPOTIFY DETECTED] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
    } else if input_str.contains("music.youtube.com") {
        Span::styled(" [YT MUSIC DETECTED] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else if input_str.contains("youtube.com") || input_str.contains("youtu.be") || input_str.starts_with("yt:") {
        Span::styled(" [YOUTUBE DETECTED] ", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
    } else if input_str.contains("soundcloud.com") || input_str.starts_with("sc:") {
        Span::styled(" [SOUNDCLOUD DETECTED] ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD))
    } else if input_str.starts_with("http") {
        Span::styled(" [WEB STREAM DETECTED] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [READY] ", Style::default().fg(theme.muted))
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Paste YouTube, Spotify, SoundCloud, Bandcamp, or Direct Stream URL:", Style::default().fg(theme.cream)),
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
            Span::styled("Play Now   ", Style::default().fg(theme.cream)),
            Span::styled("[Ctrl+A] ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("Add to Queue   ", Style::default().fg(theme.cream)),
            Span::styled("[Esc] ", Style::default().fg(theme.muted)),
            Span::styled("Cancel", Style::default().fg(theme.muted)),
        ]),
    ];

    let block = Block::default()
        .title(" 🌐 UNIVERSAL ONLINE STREAM & LINK LOADER ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.cyan_dolby));

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, popup_area);
}

fn render_search_modal(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let popup_area = centered_rect(60, 25, area);
    f.render_widget(Clear, popup_area);

    let text = vec![
        Line::from(vec![Span::styled(
            "Search local crates, global radio stations, or stream tracks:",
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
            "[Enter] Apply Search Filter  •  [Esc] Clear / Close",
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
        ("Automix & Transition", state.automix_mode.label().to_string(), "AI Neural DJ / Bass-Swap / Equal Power / S-Curve"),
        ("Transition Duration", format!("⏱️ {}s", state.crossfade_duration), "Length of transition audio overlap (3s - 16s)"),
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
