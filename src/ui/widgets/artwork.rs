use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render_artwork(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
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
    let album = state
        .current_track
        .as_ref()
        .and_then(|t| t.album.as_deref())
        .unwrap_or("Single / Live Broadcast");

    let art_ascii = [
        "  ╭───────────────────────────────╮  ",
        "  │  ┌─────────────────────────┐  │  ",
        "  │  │   ███████████████████   │  │  ",
        "  │  │   ██  BOOMBOX RX  ███   │  │  ",
        "  │  │   ██   HI-RES HI-FI ██  │  │  ",
        "  │  │   ██   (◐)   (◑)    ██  │  │  ",
        "  │  │   ██ ══════════════ ██  │  │  ",
        "  │  │   ███████████████████   │  │  ",
        "  │  └─────────────────────────┘  │  ",
        "  ╰───────────────────────────────╯  ",
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled(" 💽 HIGH-RESOLUTION CASSETTE & ALBUM ARTWORK ", Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD)),
        Span::styled(" │ (Press 'w' to return)", Style::default().fg(theme.muted)),
    ]));
    lines.push(Line::from(""));

    if let Some(art) = &state.current_artwork {
        for row in art {
            let mut row_spans = vec![Span::raw("  ")];
            for &(top, bot) in row {
                row_spans.push(Span::styled("▀", Style::default().fg(top).bg(bot)));
            }
            lines.push(Line::from(row_spans));
        }
    } else if state.artwork_loading {
        lines.push(Line::from(vec![
            Span::styled("  ⏳ Extracting / Querying HD Cover Artwork...", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));
        for l in art_ascii {
            lines.push(Line::from(vec![
                Span::styled(l, Style::default().fg(theme.border_dim)),
            ]));
        }
    } else {
        for l in art_ascii {
            lines.push(Line::from(vec![
                Span::styled(l, Style::default().fg(theme.amber)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" TRACK  : ", Style::default().fg(theme.muted)),
        Span::styled(title, Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" ARTIST : ", Style::default().fg(theme.muted)),
        Span::styled(artist, Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" ALBUM  : ", Style::default().fg(theme.muted)),
        Span::styled(album, Style::default().fg(theme.cyan_dolby)),
    ]));

    let art_block = Paragraph::new(lines).block(
        Block::default()
            .title(" 💽 ALBUM ARTWORK ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.amber_bright)),
    );

    f.render_widget(art_block, area);
}
