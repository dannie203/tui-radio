use crate::state::store::AppState;
use crate::state::types::ActiveView;
use crate::ui::theme::Theme;
use crate::ui::widgets::{
    artwork::render_artwork,
    browser::render_browser,
    cassette::render_cassette_deck,
    header::render_header,
    lyrics::render_lyrics,
    modals::render_modal,
    monitor::render_monitor,
    statusline::render_statusline,
    visualizer::render_visualizer,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn render_ui(f: &mut Frame, state: &AppState, theme: &Theme) {
    let size = f.area();

    // 1. Master vertical split (Header, Workspace, Statusline)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Header Bar
            Constraint::Min(16),   // Main Workspace
            Constraint::Length(1), // Bottom Statusline
        ])
        .split(size);

    // Render Header & Statusline
    render_header(f, main_chunks[0], state, theme);
    render_statusline(f, main_chunks[2], state, theme);

    // 2. Horizontal workspace split (Left Browser 38%, Right Deck/Visualizer 62%)
    let workspace_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(62),
        ])
        .split(main_chunks[1]);

    // Render Left Pane (Browser)
    render_browser(f, workspace_chunks[0], state, theme);

    // Render Right Pane based on Active View
    match state.active_view {
        ActiveView::Lyrics => {
            render_lyrics(f, workspace_chunks[1], state, theme);
        }
        ActiveView::Artwork => {
            render_artwork(f, workspace_chunks[1], state, theme);
        }
        ActiveView::Deck => {
            // Split Right Pane: Cassette Bay (5), Phosphor Monitor (8), Visualizer (Fill)
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5), // Cassette Bay
                    Constraint::Length(9), // Phosphor LCD
                    Constraint::Min(8),    // Dual VU + 32-Band Visualizer
                ])
                .split(workspace_chunks[1]);

            render_cassette_deck(f, right_chunks[0], state, theme);
            render_monitor(f, right_chunks[1], state, theme);
            render_visualizer(f, right_chunks[2], state, theme);
        }
    }

    // 3. Render Modal Popups (if active)
    render_modal(f, size, state, theme);
}
