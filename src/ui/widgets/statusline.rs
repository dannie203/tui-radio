use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_statusline(f: &mut Frame, area: Rect, _state: &AppState, theme: &Theme) {
    let key_style = Style::default().fg(theme.amber).add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.cream);
    let sep_style = Style::default().fg(theme.border_dim);

    let shortcuts = Line::from(vec![
        Span::styled(" [Space] ", key_style),
        Span::styled("Play/Pause", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [s] ", key_style),
        Span::styled("Stop", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [1-4] ", key_style),
        Span::styled("Modes", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [n/p] ", key_style),
        Span::styled("Next/Prev", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [l] ", key_style),
        Span::styled("Lyrics", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [w] ", key_style),
        Span::styled("Art", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [u] ", key_style),
        Span::styled("URL", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [/] ", key_style),
        Span::styled("Search", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(if _state.is_recording { " [R] ● REC " } else { " [R] " },
            if _state.is_recording { Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD) } else { key_style }),
        Span::styled("Record", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [M] ", key_style),
        Span::styled("Mixtape", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [?] ", key_style),
        Span::styled("Help", desc_style),
        Span::styled(" │", sep_style),
        Span::styled(" [q] ", key_style),
        Span::styled("Quit", desc_style),
    ]);

    let bar = Paragraph::new(shortcuts)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE).style(Style::default().bg(theme.bg_panel)));

    f.render_widget(bar, area);
}
