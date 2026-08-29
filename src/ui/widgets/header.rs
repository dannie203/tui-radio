use crate::state::store::AppState;
use crate::state::types::AppMode;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_header(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Title / Brand
            Constraint::Min(45),    // Mode Tabs
            Constraint::Length(35), // Soundstage & DSP Badges
        ])
        .split(area);

    // 1. Title / Brand
    let brand = Paragraph::new(Line::from(vec![
        Span::styled(" ▶ ", Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("BOOMBOX RX-505", Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border_dim)));
    f.render_widget(brand, chunks[0]);

    // 2. Mode Tabs (1-4)
    let mut tab_spans = Vec::new();
    for (i, mode) in AppMode::ALL.iter().enumerate() {
        let count = match mode {
            AppMode::LocalTracks => state.filtered_local.len(),
            AppMode::RadioStations => state.filtered_radio.len(),
            AppMode::Queue => state.queue.len(),
            AppMode::YoutubeMusic => state.youtube_results.len(),
        };

        let is_selected = state.mode == *mode;
        if i > 0 {
            tab_spans.push(Span::styled("   ", Style::default()));
        }

        if is_selected {
            tab_spans.push(Span::styled(
                format!("▶ [ {}. {} ({}) ]", i + 1, mode.label(), count),
                Style::default()
                    .fg(theme.amber)
                    .bg(theme.bg_panel)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("[ {}. {} ({}) ]", i + 1, mode.label(), count),
                Style::default().fg(theme.muted),
            ));
        }
    }

    let tabs_widget = Paragraph::new(Line::from(tab_spans))
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border_dim)));
    f.render_widget(tabs_widget, chunks[1]);

    // 3. DSP Badges
    let stereo_style = Style::default().fg(theme.green_phosphor);
    let bass_style = if state.bass_boost {
        Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };
    let eq_style = Style::default().fg(theme.amber);
    let rec_style = if state.is_recording {
        Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    };

    let badges = Line::from(vec![
        Span::styled(format!("[{}] ", state.stereo_mode.label()), stereo_style),
        Span::styled(format!("[{}] ", state.eq_preset.label()), eq_style),
        Span::styled(if state.bass_boost { "[🔊 BASS]" } else { "[FLAT]" }, bass_style),
        Span::styled(if state.is_recording { "[● REC]" } else { "" }, rec_style),
    ]);

    let dsp_widget = Paragraph::new(badges)
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border_dim)));
    f.render_widget(dsp_widget, chunks[2]);
}
