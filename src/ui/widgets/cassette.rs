use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const SPOOL_FRAMES_LEFT: [&str; 4] = ["( | )", "( / )", "( - )", "( \\ )"];
const SPOOL_FRAMES_RIGHT: [&str; 4] = ["( \\ )", "( | )", "( / )", "( - )"];

pub fn render_cassette_deck(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let frame = state.telemetry.spool_frame;
    let is_playing = state.is_playing && !state.is_paused;

    let spool_l = if is_playing {
        SPOOL_FRAMES_LEFT[frame % 4]
    } else {
        "( | )"
    };

    let spool_r = if is_playing {
        SPOOL_FRAMES_RIGHT[frame % 4]
    } else {
        "( | )"
    };

    let tape_title = state
        .current_track
        .as_ref()
        .map(|t| t.title.as_str())
        .unwrap_or("STANDBY / NO TAPE LOADED");

    let width = area.width as usize;
    let max_tape_width = width.saturating_sub(58).clamp(16, 48);
    let label_chars: Vec<char> = tape_title.chars().collect();
    let label_display = if label_chars.len() > max_tape_width {
        let clip: String = label_chars[..max_tape_width.saturating_sub(3)].iter().collect();
        format!("{:<width$}...", clip, width = max_tape_width.saturating_sub(3))
    } else {
        format!("{:<width$}", tape_title, width = max_tape_width)
    };

    // Transport status LEDs
    let (play_led, pause_led, stop_led) = ("▶ PLAY", "⏸ PAUSE", "■ STOP");

    let play_style = if is_playing {
        Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    let pause_style = if state.is_playing && state.is_paused {
        Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    let stop_style = if !state.is_playing {
        Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    let rec_style = if state.is_recording {
        Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    // Line 1: Tape Deck Mechanism & Hardware Labels
    let line1 = Line::from(vec![
        Span::styled(format!(" {} ", spool_l), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("══ ", Style::default().fg(theme.border_dim)),
        Span::styled(format!("[ {} ]", label_display), Style::default().fg(theme.cream).bg(theme.bg_dark).add_modifier(Modifier::BOLD)),
        Span::styled(" ══", Style::default().fg(theme.border_dim)),
        Span::styled(format!(" {} ", spool_r), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  [{}]", state.dolby_mode.short_label()), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" [{}]", state.tape_type.short_label()), Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" [{}]", state.stereo_mode.label()), Style::default().fg(theme.cyan_dolby)),
    ]);

    // Line 2: LED Indicators & Audio Settings
    let line2 = Line::from(vec![
        Span::styled(format!(" [{}]", play_led), play_style),
        Span::styled(format!(" [{}]", pause_led), pause_style),
        Span::styled(format!(" [{}]", stop_led), stop_style),
        Span::styled(" [⏺ REC]", rec_style),
        Span::styled(format!("   EQ: {}", state.eq_preset.label()), Style::default().fg(theme.amber_bright)),
        Span::styled(format!(" • BASS: {}", if state.bass_boost { "ON" } else { "OFF" }), Style::default().fg(if state.bass_boost { theme.green_phosphor } else { theme.muted })),
        Span::styled(format!(" • SHUFFLE: {}", if state.shuffle { "ON" } else { "OFF" }), Style::default().fg(if state.shuffle { theme.green_phosphor } else { theme.muted })),
        Span::styled(format!(" • REPEAT: {}", state.repeat_mode.label()), Style::default().fg(theme.green_phosphor)),
    ]);

    let deck = Paragraph::new(vec![line1, line2]).block(
        Block::default()
            .title(" 📼 BOOMBOX RX-505 CASSETTE DECK ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_dim)),
    );

    f.render_widget(deck, area);
}


