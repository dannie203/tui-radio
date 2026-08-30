use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const SPOOL_FRAMES_LEFT: [&str; 4] = ["(  |  )", "(  /  )", "(  -  )", "(  \\  )"];
const SPOOL_FRAMES_RIGHT: [&str; 4] = ["(  \\  )", "(  |  )", "(  /  )", "(  -  )"];

pub fn render_cassette_deck(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let frame = state.telemetry.spool_frame;
    let is_spinning = state.is_playing && !state.is_paused;
    let is_xfading = state.telemetry.is_crossfading;
    let xfade_progress = state.telemetry.crossfade_progress.clamp(0.0, 1.0);

    let left_spool = if is_spinning {
        SPOOL_FRAMES_LEFT[frame % 4]
    } else {
        "(  |  )"
    };
    let right_spool = if is_spinning {
        SPOOL_FRAMES_RIGHT[frame % 4]
    } else {
        "(  |  )"
    };

    let title_raw = state
        .current_track
        .as_ref()
        .map(|t| t.title.as_str())
        .unwrap_or("BOOMBOX CASSETTE DECK");

    let bpm_str = state
        .telemetry
        .current_bpm
        .map(|b| format!("{:.1} BPM", b))
        .unwrap_or_else(|| "-- BPM".to_string());

    let key_str = state
        .telemetry
        .current_key
        .as_deref()
        .unwrap_or("--");

    // Dynamic Active Deck indicator
    let active_deck_label = if is_xfading {
        "🎛️ DUAL-DECK MIXING (A ⇄ B)"
    } else if state.telemetry.active_deck.contains('B') {
        "💽 DECK B [ACTIVE]"
    } else {
        "💽 DECK A [ACTIVE]"
    };

    let automix_badge = state.automix_mode.short_badge();

    let mut lines = Vec::new();

    if is_xfading {
        // ==============================================================
        // 🎛️ ACTIVE DUAL-DECK CROSSFADE ANIMATION
        // ==============================================================
        let pct = (xfade_progress * 100.0).round() as u32;

        // Visual Crossfade Gauge (16 characters)
        let total_bar_len = 16;
        let filled_b = (xfade_progress * total_bar_len as f32).round() as usize;
        let filled_a = total_bar_len - filled_b;
        let gauge_a = "█".repeat(filled_a);
        let gauge_b = "█".repeat(filled_b);

        let line1 = Line::from(vec![
            Span::styled(" [DECK A] ", Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:>8} ", gauge_a), Style::default().fg(theme.amber)),
            Span::styled(format!("◄◄ {:>2}% ►►", pct), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {:<8}", gauge_b), Style::default().fg(theme.cyan_dolby)),
            Span::styled(" [DECK B] ", Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        ]);

        let line2 = Line::from(vec![
            Span::styled(format!(" {} ", left_spool), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
            Span::styled("══════ ", Style::default().fg(theme.border_dim)),
            Span::styled(format!(" 🎹 KEY: {}  │  🎵 {}  │  ⚡ {} ", key_str, bpm_str, automix_badge), Style::default().fg(theme.cream).bg(theme.bg_dark).add_modifier(Modifier::BOLD)),
            Span::styled(" ══════ ", Style::default().fg(theme.border_dim)),
            Span::styled(format!("{} ", right_spool), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        ]);

        let line3 = Line::from(vec![
            Span::styled("⚡ AI-DJ TRANSITION: ", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<28}", title_raw.chars().take(28).collect::<String>()), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", Style::default().fg(theme.muted)),
            Span::styled(format!("MODE: {}", state.automix_mode.label()), Style::default().fg(theme.amber_bright)),
        ]);

        lines.push(line1);
        lines.push(line2);
        lines.push(line3);
    } else {
        // ==============================================================
        // 📼 NORMAL DUAL-DECK CASSETTE PLAYER DISPLAY
        // ==============================================================
        let tape_label = format!("{:<22}", title_raw.chars().take(22).collect::<String>());

        // LEDs
        let play_led = if state.is_playing && !state.is_paused {
            Span::styled("▶ PLAY", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("▶ PLAY", Style::default().fg(theme.border_dim))
        };

        let pause_led = if state.is_paused {
            Span::styled("❚❚ PAUSE", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("❚❚ PAUSE", Style::default().fg(theme.border_dim))
        };

        let stop_led = if !state.is_playing {
            Span::styled("▲ STOP", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("▲ STOP", Style::default().fg(theme.border_dim))
        };

        let rec_led = if state.is_recording {
            Span::styled("● REC", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("● REC", Style::default().fg(theme.border_dim))
        };

        let line1 = Line::from(vec![
            Span::styled(format!(" {} ", active_deck_label), Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
            Span::styled("    ", Style::default()),
            Span::styled(format!("[🎹 {} • 🎵 {}]", key_str, bpm_str), Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
            Span::styled("    ", Style::default()),
            Span::styled(format!("[{}]", state.dolby_mode.label()), Style::default().fg(theme.cyan_dolby)),
            Span::styled("  ", Style::default()),
            Span::styled(format!("[{}]", automix_badge), Style::default().fg(theme.amber_bright)),
        ]);

        let line2 = Line::from(vec![
            Span::styled(format!("  {} ", left_spool), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
            Span::styled("════════ ", Style::default().fg(theme.border_dim)),
            Span::styled(format!("[ {} ]", tape_label), Style::default().fg(theme.cream).bg(theme.bg_dark).add_modifier(Modifier::BOLD)),
            Span::styled(" ════════ ", Style::default().fg(theme.border_dim)),
            Span::styled(format!("{}  ", right_spool), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        ]);

        let line3 = Line::from(vec![
            Span::styled("      ", Style::default()),
            rec_led,
            Span::styled("          ", Style::default()),
            play_led,
            Span::styled("          ", Style::default()),
            pause_led,
            Span::styled("          ", Style::default()),
            stop_led,
        ]);

        lines.push(line1);
        lines.push(line2);
        lines.push(line3);
    }

    let border_color = if is_xfading {
        theme.green_phosphor
    } else {
        theme.border_dim
    };

    let title = if is_xfading {
        " 🎛️ DUAL-DECK CYBER-DJ MIXING BAY [CROSSFADING] "
    } else {
        " 📼 DUAL-DECK CASSETTE BAY "
    };

    let deck = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

    f.render_widget(deck, area);
}

