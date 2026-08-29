use crate::state::store::AppState;
use crate::state::types::{AppMode, LocalViewLevel};
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render_browser(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let total_width = area.width as usize;
    let usable_text_width = total_width.saturating_sub(23).max(20);
    let title_width = (usable_text_width * 58) / 100;
    let artist_width = usable_text_width.saturating_sub(title_width);

    let (title, list_items) = match state.mode {
        AppMode::LocalTracks => match state.local_view_level {
            LocalViewLevel::Albums => {
                let t = format!(" 💽 LOCAL ALBUMS ({}) [v: All Tracks] ", state.filtered_albums.len());
                let items: Vec<ListItem> = if state.filtered_albums.is_empty() {
                    vec![ListItem::new(Span::styled("  (No albums found)", Style::default().fg(theme.muted)))]
                } else {
                    state.filtered_albums.iter().enumerate().map(|(i, album)| {
                        let icon_span = Span::styled("💽 ", Style::default().fg(theme.gold).add_modifier(Modifier::BOLD));
                        let idx_span = Span::styled(format!("{:02}. ", i + 1), Style::default().fg(theme.amber_bright));

                        let name_text = truncate_to_width(&album.name, title_width);
                        let name_span = Span::styled(
                            format!("{:<width$} ", name_text, width = title_width),
                            Style::default().fg(theme.cream).add_modifier(Modifier::BOLD),
                        );

                        let artist_text = truncate_to_width(&album.artist, artist_width);
                        let artist_span = Span::styled(
                            format!("{:<width$} ", artist_text, width = artist_width),
                            Style::default().fg(theme.cyan_dolby),
                        );

                        let badge_span = Span::styled(
                            format!("[{}t / {}]", album.tracks.len(), album.format),
                            Style::default().fg(theme.green_phosphor),
                        );

                        ListItem::new(Line::from(vec![
                            icon_span,
                            idx_span,
                            name_span,
                            artist_span,
                            badge_span,
                        ]))
                    }).collect()
                };
                (t, items)
            }
            LocalViewLevel::Tracks => {
                let album_name = state.selected_album_idx
                    .and_then(|idx| state.filtered_albums.get(idx))
                    .map(|a| a.name.as_str())
                    .unwrap_or("Album");
                let t = format!(" 💿 ALBUM: {} ({}) [Esc: Back] ", album_name, state.filtered_local.len());
                let items = render_tracks_items(&state.filtered_local, state, theme, title_width, artist_width, true);
                (t, items)
            }
            LocalViewLevel::AllTracks => {
                let t = format!(" 🎵 ALL LOCAL TRACKS ({}) [v: Albums] ", state.filtered_local.len());
                let items = render_tracks_items(&state.filtered_local, state, theme, title_width, artist_width, false);
                (t, items)
            }
        },
        AppMode::RadioStations => {
            let t = format!(" 📻 RADIO STATIONS ({}) [g: {}] ", state.filtered_radio.len(), state.genre_filter.label());
            let items = render_tracks_items(&state.filtered_radio, state, theme, title_width, artist_width, false);
            (t, items)
        }
        AppMode::Queue => {
            let t = format!(" 📋 PLAYLIST QUEUE ({}) [c: Clear, x: Delete] ", state.queue.len());
            let items = render_tracks_items(&state.queue, state, theme, title_width, artist_width, false);
            (t, items)
        }
        AppMode::YoutubeMusic => {
            let t = format!(" 📺 YOUTUBE & STREAMS ({}) [u: Paste Link] ", state.youtube_results.len());
            let items = render_tracks_items(&state.youtube_results, state, theme, title_width, artist_width, false);
            (t, items)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_focus));

    let list_widget = List::new(list_items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.bg_dark)
                .bg(theme.amber)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("❯ ");

    let mut list_state = ListState::default();
    if state.get_active_list_len() > 0 {
        list_state.select(Some(state.selected_index));
    }

    f.render_stateful_widget(list_widget, area, &mut list_state);
}

fn render_tracks_items(
    tracks: &[crate::state::types::MediaItem],
    state: &AppState,
    theme: &Theme,
    title_width: usize,
    artist_width: usize,
    prefer_track_no: bool,
) -> Vec<ListItem<'static>> {
    if tracks.is_empty() {
        return vec![ListItem::new(Span::styled(
            "  (No tracks found)",
            Style::default().fg(theme.muted),
        ))];
    }

    tracks
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_current = state.current_track.as_ref().map_or(false, |c| c.id == item.id);
            let is_fav = item.is_favorite || state.favorites.iter().any(|f| f.id == item.id);

            let play_icon = if is_current {
                Span::styled("▶ ", Style::default().fg(theme.green_phosphor).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("  ", Style::default())
            };

            let fav_icon = if is_fav {
                Span::styled("★ ", Style::default().fg(theme.gold))
            } else {
                Span::styled("• ", Style::default().fg(theme.border_dim))
            };

            let idx_str = if prefer_track_no {
                if let Some(no) = item.track_no {
                    format!("{:02}. ", no)
                } else {
                    format!("{:02}. ", i + 1)
                }
            } else {
                format!("{:02}. ", i + 1)
            };

            let idx_span = Span::styled(
                idx_str,
                Style::default().fg(theme.amber_bright),
            );

            let title_text = truncate_to_width(&item.title, title_width);
            let title_span = Span::styled(
                format!("{:<width$} ", title_text, width = title_width),
                Style::default().fg(theme.cream).add_modifier(Modifier::BOLD),
            );

            let artist_text = truncate_to_width(&item.artist, artist_width);
            let artist_span = Span::styled(
                format!("{:<width$} ", artist_text, width = artist_width),
                Style::default().fg(theme.cyan_dolby),
            );

            let badge_span = if let Some(ref fmt) = item.format {
                Span::styled(format!("[{}]", fmt), Style::default().fg(theme.muted))
            } else if item.duration > 0.0 {
                let m = (item.duration / 60.0).floor() as u32;
                let s = (item.duration % 60.0).floor() as u32;
                Span::styled(format!("{:02}:{:02}", m, s), Style::default().fg(theme.green_phosphor))
            } else {
                Span::styled("[LIVE]", Style::default().fg(theme.green_phosphor))
            };

            ListItem::new(Line::from(vec![
                play_icon,
                fav_icon,
                idx_span,
                title_span,
                artist_span,
                badge_span,
            ]))
        })
        .collect()
}

fn truncate_to_width(s: &str, max_len: usize) -> String {
    let mut out = String::new();
    let mut count = 0;
    for ch in s.chars() {
        if count >= max_len {
            break;
        }
        out.push(ch);
        count += 1;
    }
    out
}
