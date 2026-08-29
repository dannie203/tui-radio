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

    let rec_led = Span::styled("● REC", Style::default().fg(theme.border_dim));

    let line1 = Line::from(vec![
        Span::styled("[A] RETRO STEREO DECK", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        Span::styled("        ", Style::default()),
        Span::styled(format!("[{}]", state.dolby_mode.label()), Style::default().fg(theme.cyan_dolby)),
        Span::styled("        ", Style::default()),
        Span::styled(format!("[{}]", state.tape_type.label()), Style::default().fg(theme.amber_bright)),
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

    let deck = Paragraph::new(vec![line1, line2, line3])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(" 📼 CASSETTE DECK BAY ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_dim)),
        );

    f.render_widget(deck, area);
}
