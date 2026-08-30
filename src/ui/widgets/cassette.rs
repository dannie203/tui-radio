use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const SPOOL_FRAMES_LEFT: [&str; 4] = ["( | )", "(/ )", "(- )", "(\\ )"];
const SPOOL_FRAMES_RIGHT: [&str; 4] = ["(\\ )", "( | )", "(/ )", "(- )"];

pub fn render_cassette_deck(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let frame = state.telemetry.spool_frame;
    let is_playing = state.is_playing && !state.is_paused;
    let is_xfading = state.telemetry.is_crossfading;
    let xfade_progress = state.telemetry.crossfade_progress.clamp(0.0, 1.0);
    let active_deck_name = state.telemetry.active_deck.to_uppercase();
    let is_deck_b_master = active_deck_name.contains('B');

    // Spool animations
    let spool_a = if is_playing && (!is_deck_b_master || is_xfading) {
        SPOOL_FRAMES_LEFT[frame % 4]
    } else {
        "( | )"
    };

    let spool_b = if is_playing && (is_deck_b_master || is_xfading) {
        SPOOL_FRAMES_RIGHT[frame % 4]
    } else {
        "( | )"
    };

    // Track for Deck A (Master or outgoing)
    let current_track = state.current_track.as_ref();
    let next_track = state.peek_next_track();

    let (track_a, track_b) = if is_deck_b_master && !is_xfading {
        (next_track.as_ref(), current_track)
    } else {
        (current_track, next_track.as_ref())
    };

    let title_a = track_a
        .map(|t| t.title.as_str())
        .unwrap_or("EMPTY / NO TAPE");
    let title_b = track_b
        .map(|t| t.title.as_str())
        .unwrap_or("EMPTY / NO TAPE");

    // Neural Profiles for Deck A & Deck B
    let prof_a = track_a.and_then(|t| state.neural_engine.get_profile(&t.url));
    let prof_b = track_b.and_then(|t| state.neural_engine.get_profile(&t.url));

    let key_a = prof_a.as_ref().map(|p| p.camelot_key.as_str()).unwrap_or("--");
    let bpm_a = prof_a.as_ref().map(|p| format!("{:.1}", p.bpm)).unwrap_or_else(|| "--".to_string());

    let key_b = prof_b.as_ref().map(|p| p.camelot_key.as_str()).unwrap_or("--");
    let bpm_b = prof_b.as_ref().map(|p| format!("{:.1}", p.bpm)).unwrap_or_else(|| "--".to_string());

    // Harmonic check between Deck A and Deck B
    let is_harmonic = if track_a.is_some() && track_b.is_some() && key_a != "--" && key_b != "--" {
        crate::audio::neural::NeuralEngine::is_harmonic_match(key_a, key_b)
    } else {
        false
    };

    let width = area.width as usize;
    let deck_width = (width.saturating_sub(26) / 2).clamp(24, 48);
    let title_width = deck_width.saturating_sub(14).clamp(10, 26);

    let tape_label_a: String = format!("{:<width$}", title_a.chars().take(title_width).collect::<String>(), width = title_width);
    let tape_label_b: String = format!("{:<width$}", title_b.chars().take(title_width).collect::<String>(), width = title_width);

    // Crossfader Slider Rendering (15 chars)
    let crossfader_bar = if state.current_track.is_none() {
        "[A ─────●───── B]".to_string()
    } else if is_xfading {
        let pct = (xfade_progress * 100.0).round() as u32;
        let pos = (xfade_progress * 10.0).round().clamp(0.0, 10.0) as usize;
        let mut bar = String::from("[A ");
        for i in 0..=10 {
            if i == pos {
                bar.push('●');
            } else if i < pos {
                bar.push('─');
            } else {
                bar.push('─');
            }
        }
        bar.push_str(&format!(" B] {:>2}%", pct));
        bar
    } else if is_deck_b_master {
        "[A ──────────● B]".to_string()
    } else {
        "[A ●────────── B]".to_string()
    };

    // Header Status for Deck A and Deck B
    let (deck_a_badge, deck_a_color) = if state.current_track.is_none() {
        ("💽 DECK A [STANDBY]", theme.muted)
    } else if is_xfading && !is_deck_b_master {
        ("💽 DECK A [FADING OUT]", theme.amber_bright)
    } else if is_deck_b_master {
        ("💽 DECK A [CUE / STANDBY]", theme.muted)
    } else {
        ("💽 DECK A [MASTER ▶]", theme.green_phosphor)
    };

    let (deck_b_badge, deck_b_color) = if state.current_track.is_none() || track_b.is_none() {
        ("💽 DECK B [STANDBY]", theme.muted)
    } else if is_xfading && !is_deck_b_master {
        ("💽 DECK B [FADING IN]", theme.cyan_dolby)
    } else if is_deck_b_master {
        ("💽 DECK B [MASTER ▶]", theme.cyan_dolby)
    } else {
        ("💽 DECK B [CUE / NEXT]", theme.muted)
    };

    // Calculate Dynamic Volumes during crossfade
    let (vol_a_str, vol_b_str) = if state.current_track.is_none() {
        ("Vol:  0%".to_string(), "Vol:  0%".to_string())
    } else if is_xfading {
        let pct_a = ((1.0 - xfade_progress) * 100.0).round() as u32;
        let pct_b = (xfade_progress * 100.0).round() as u32;
        (format!("Vol: {:>2}%", pct_a), format!("Vol: {:>2}%", pct_b))
    } else if is_deck_b_master {
        ("Vol:  0%".to_string(), format!("Vol: {:>2}%", state.volume))
    } else {
        (format!("Vol: {:>2}%", state.volume), "Vol:  0%".to_string())
    };

    // Line 1: Deck Titles & Center Strategy
    let line1 = Line::from(vec![
        Span::styled(format!(" {:<deck_width$}", deck_a_badge, deck_width = deck_width), Style::default().fg(deck_a_color).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" 🎛️ {:^18} ", state.automix_mode.short_badge()), Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {:<deck_width$}", deck_b_badge, deck_width = deck_width), Style::default().fg(deck_b_color).add_modifier(Modifier::BOLD)),
    ]);

    // Line 2: Cassette Tapes with Spinning Spools & Center Crossfader
    let line2 = Line::from(vec![
        Span::styled(format!(" {} ", spool_a), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled("══ ", Style::default().fg(theme.border_dim)),
        Span::styled(format!("[ {} ]", tape_label_a), Style::default().fg(theme.cream).bg(theme.bg_dark).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ══ {} ", spool_a), Style::default().fg(theme.amber).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {:^20} ", crossfader_bar), Style::default().fg(if is_xfading { theme.green_phosphor } else { theme.amber }).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {} ", spool_b), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
        Span::styled("══ ", Style::default().fg(theme.border_dim)),
        Span::styled(format!("[ {} ]", tape_label_b), Style::default().fg(theme.cream).bg(theme.bg_dark).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" ══ {} ", spool_b), Style::default().fg(theme.cyan_dolby).add_modifier(Modifier::BOLD)),
    ]);

    // Line 3: Deck Info (Key, BPM, Volume) & Harmonic Match Indicator
    let harmonic_str = if track_a.is_none() || track_b.is_none() {
        "STANDBY"
    } else if is_harmonic {
        "HARMONIC MATCH ✓"
    } else if is_xfading {
        "TEMPO SYNC"
    } else {
        "DJ AUTO-MIX"
    };

    let line3 = Line::from(vec![
        Span::styled(format!(" [🎹 {} • 🎵 {} BPM • {}]", key_a, bpm_a, vol_a_str), Style::default().fg(theme.amber_bright)),
        Span::styled(format!("   ⚡ {:^16}   ", harmonic_str), Style::default().fg(if is_harmonic { theme.green_phosphor } else { theme.muted }).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" [🎹 {} • 🎵 {} BPM • {}]", key_b, bpm_b, vol_b_str), Style::default().fg(theme.cyan_dolby)),
    ]);

    let border_color = if is_xfading {
        theme.green_phosphor
    } else {
        theme.border_dim
    };

    let bay_title = if is_xfading {
        " 🎛️ DUAL-DECK CYBER-DJ MIXING CONSOLE [LIVE CROSSFADE] "
    } else {
        " 🎛️ DUAL-DECK CYBER-DJ MIXING CONSOLE "
    };

    let deck = Paragraph::new(vec![line1, line2, line3])
        .block(
            Block::default()
                .title(bay_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

    f.render_widget(deck, area);
}


