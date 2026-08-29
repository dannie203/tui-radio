use crate::state::store::AppState;
use crate::ui::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

const BLOCK_CHARS: [&str; 9] = [" ", " ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

const EQ_LABELS_SLOT_4: [&str; 32] = [
    " 20 ", " 25 ", "31.5", " 40 ", " 50 ", " 63 ", " 80 ", "100 ", "125 ", "160 ",
    "200 ", "250 ", "315 ", "400 ", "500 ", "630 ", "800 ", "1.0k", "1.2k", "1.6k",
    "2.0k", "2.5k", "3.1k", "4.0k", "5.0k", "6.3k", "8.0k", " 10k", "12.5", " 16k", " 18k", " 20k",
];

const EQ_LABELS_SLOT_3: [&str; 32] = [
    " 20", " 25", " 31", " 40", " 50", " 63", " 80", "100", "125", "160",
    "200", "250", "315", "400", "500", "630", "800", " 1k", "1.2", "1.6",
    " 2k", "2.5", "3.1", " 4k", " 5k", "6.3", " 8k", "10k", "12k", "16k", "18k", "20k",
];

const EQ_LABELS_SLOT_2: [&str; 32] = [
    "20", "25", "31", "40", "50", "63", "80", "10", "12", "16",
    "20", "25", "31", "40", "50", "63", "80", "1k", "1.", "1.",
    "2k", "2.", "3.", "4k", "5k", "6.", "8k", "10", "12", "16", "18", "20",
];

pub fn render_visualizer(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mut lines = Vec::new();
    let box_width = area.width as usize;

    // 1. Dual VU Meters (L/R) with perfectly aligned dB Scale Header
    let vu_width = box_width.saturating_sub(26).clamp(28, 80);
    let scale_str = generate_vu_scale(vu_width);

    lines.push(Line::from(vec![
        Span::styled("  SCALE [", Style::default().fg(theme.muted)),
        Span::styled(scale_str, Style::default().fg(theme.chrome)),
        Span::styled("] dB", Style::default().fg(theme.muted)),
    ]));

    lines.push(render_vu_line("L CH", state.telemetry.vu_left, state.telemetry.peak_left, vu_width, theme));
    lines.push(render_vu_line("R CH", state.telemetry.vu_right, state.telemetry.peak_right, vu_width, theme));
    lines.push(Line::from(""));

    // 2. 32-Band Spectrum Header
    lines.push(Line::from(vec![Span::styled(
        " 32-BAND ISO CHROMA EQUALIZER SPECTRUM [20Hz — 20kHz]",
        Style::default().fg(theme.amber_bright).add_modifier(Modifier::BOLD),
    )]));

    // Determine slot and band widths based on available width
    let (band_width, slot_width) = if box_width >= 135 {
        (3, 4)
    } else if box_width >= 100 {
        (2, 3)
    } else {
        (1, 2)
    };
    let spacing = " ".repeat(slot_width - band_width);

    // Calculate vertical height for EQ (fill remaining box height)
    let eq_height = (area.height as usize).saturating_sub(8).clamp(5, 14);

    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    for r in (1..=eq_height).rev() {
        let mut row_spans = vec![Span::raw(" ")];
        for b in 0..32 {
            let val = state.telemetry.eq_bands[b];
            let peak = state.telemetry.eq_peaks[b];
            let threshold = ((r - 1) as f32 / eq_height as f32) * 100.0;
            let next_threshold = (r as f32 / eq_height as f32) * 100.0;
            let peak_r = ((peak / 100.0) * eq_height as f32).ceil() as usize;

            let color = get_eq_bar_color(
                state.spectrum_color_mode,
                b,
                r,
                eq_height,
                theme,
                state.spectrum_custom_color.as_deref(),
                now_sec,
            );

            if peak_r == r && peak > 5.0 {
                row_spans.push(Span::styled("━".repeat(band_width), Style::default().fg(theme.red_led)));
            } else if val >= next_threshold {
                row_spans.push(Span::styled("█".repeat(band_width), Style::default().fg(color)));
            } else if val > threshold {
                let sub = (((val - threshold) / (next_threshold - threshold)) * (BLOCK_CHARS.len() - 1) as f32) as usize;
                let ch = BLOCK_CHARS[sub.clamp(1, 8)];
                row_spans.push(Span::styled(ch.repeat(band_width), Style::default().fg(color)));
            } else {
                row_spans.push(Span::styled(" ".repeat(band_width), Style::default().fg(theme.border_dim)));
            }
            row_spans.push(Span::raw(&spacing));
        }
        lines.push(Line::from(row_spans));
    }

    // 3. Frequency Labels row with precise slot alignment
    let mut label_spans = vec![Span::raw(" ")];
    for b in 0..32 {
        let label = if slot_width >= 4 {
            EQ_LABELS_SLOT_4[b]
        } else if slot_width == 3 {
            EQ_LABELS_SLOT_3[b]
        } else {
            EQ_LABELS_SLOT_2[b]
        };
        label_spans.push(Span::styled(label, Style::default().fg(theme.muted)));
    }
    lines.push(Line::from(label_spans));

    let viz_block = Paragraph::new(lines).block(
        Block::default()
            .title(" 🌈 DUAL STEREO VU METERS & 32-BAND EQUALIZER ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_dim)),
    );

    f.render_widget(viz_block, area);
}

fn generate_vu_scale(width: usize) -> String {
    let mut chars = vec![' '; width];

    let marks: &[(f32, &str)] = &[
        (0.00, "-30"),
        (0.20, "-20"),
        (0.38, "-15"),
        (0.52, "-10"),
        (0.64, "-5"),
        (0.74, "-3"),
        (0.85, "0"),
        (0.92, "+2"),
        (0.98, "+3"),
    ];

    for &(pos_pct, label) in marks {
        let center_idx = (pos_pct * (width.saturating_sub(1)) as f32).round() as usize;
        let start_pos = center_idx.saturating_sub(label.len() / 2);
        if start_pos + label.len() <= width {
            for (offset, ch) in label.chars().enumerate() {
                chars[start_pos + offset] = ch;
            }
        }
    }

    chars.into_iter().collect()
}

fn render_vu_line<'a>(label: &'static str, val: f32, peak: f32, width: usize, theme: &Theme) -> Line<'a> {
    let filled = ((val / 100.0) * width as f32).round() as usize;
    let peak_pos = ((peak / 100.0) * (width.saturating_sub(1)) as f32).round() as usize;

    let mut bar_spans = vec![
        Span::styled(format!("  {} [", label), Style::default().fg(theme.gold).add_modifier(Modifier::BOLD)),
    ];

    for i in 0..width {
        let pct = (i as f32 / width as f32) * 100.0;
        let color = if pct >= 85.0 {
            theme.red_led // Red Overload / Tape Saturation (> 0 dB)
        } else if pct >= 70.0 {
            theme.yellow_led // Amber Warmth (-5dB to 0dB)
        } else {
            theme.green_phosphor // Green Safe Zone (< -5dB)
        };

        if i < filled {
            bar_spans.push(Span::styled("■", Style::default().fg(color)));
        } else if i == peak_pos && peak > 5.0 {
            bar_spans.push(Span::styled("▮", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)));
        } else {
            bar_spans.push(Span::styled("■", Style::default().fg(theme.border_dim)));
        }
    }

    let db_str = if val <= 4.0 {
        "-∞ dB".to_string()
    } else if val < 85.0 {
        format!("-{:.0}dB", (85.0 - val) * 0.35)
    } else {
        format!("+{:.1}dB", (val - 85.0) * 0.20)
    };

    bar_spans.push(Span::styled(format!("] {:>6} ", db_str), Style::default().fg(theme.chrome)));
    if peak >= 85.0 {
        bar_spans.push(Span::styled("[PEAK]", Style::default().fg(theme.red_led).add_modifier(Modifier::BOLD)));
    } else {
        bar_spans.push(Span::styled("[PEAK]", Style::default().fg(theme.border_dim)));
    }

    Line::from(bar_spans)
}

fn get_eq_bar_color(
    mode: crate::state::types::SpectrumColorMode,
    b: usize,
    r: usize,
    eq_height: usize,
    theme: &Theme,
    custom_color: Option<&str>,
    now_sec: f64,
) -> ratatui::style::Color {
    use crate::state::types::SpectrumColorMode;
    match mode {
        SpectrumColorMode::RgbCycle => {
            // Dynamic 60 FPS Fluid RGB Wave Cycle
            let cycle_speed = 0.40; // Full rainbow wave cycle every ~2.5s
            let phase = (now_sec * cycle_speed) % 1.0;
            let hue = ((phase + (b as f64 / 32.0)) % 1.0) * 360.0;
            hsv_to_rgb(hue, 0.95, 1.0)
        }
        SpectrumColorMode::ChromaRainbow => {
            // Smooth Static 32-Band ISO Frequency Rainbow
            match b {
                0..=2 => ratatui::style::Color::Rgb(255, 45, 85),    // Deep Crimson (20-31.5 Hz)
                3..=5 => ratatui::style::Color::Rgb(255, 95, 30),    // Vibrant Orange (40-63 Hz)
                6..=8 => ratatui::style::Color::Rgb(255, 165, 0),    // Golden Amber (80-125 Hz)
                9..=11 => ratatui::style::Color::Rgb(255, 215, 0),   // Warm Yellow (160-250 Hz)
                12..=14 => ratatui::style::Color::Rgb(175, 240, 45), // Lime Green (315-500 Hz)
                15..=17 => ratatui::style::Color::Rgb(50, 225, 95),  // Emerald Green (630-1.2k Hz)
                18..=20 => ratatui::style::Color::Rgb(30, 220, 185), // Aquamarine (1.6k-2.5k Hz)
                21..=23 => ratatui::style::Color::Rgb(0, 210, 255),  // Electric Cyan (3.1k-5k Hz)
                24..=26 => ratatui::style::Color::Rgb(65, 145, 255), // Sky Blue (6.3k-10k Hz)
                27..=29 => ratatui::style::Color::Rgb(165, 90, 255), // Neon Purple (12.5k-16k Hz)
                _ => ratatui::style::Color::Rgb(255, 80, 220),       // Neon Magenta (18k-20k Hz)
            }
        }
        SpectrumColorMode::VerticalGradient => {
            if r >= eq_height {
                theme.red_led
            } else if r >= eq_height.saturating_sub(2) {
                theme.yellow_led
            } else if r >= eq_height.saturating_sub(4) {
                theme.amber
            } else {
                theme.cyan_dolby
            }
        }
        SpectrumColorMode::CyberpunkNeon => {
            // Animated Neon Wave: Hot Pink / Magenta -> Electric Cyan
            let wave = ((now_sec * 1.5 + (b as f64 * 0.22)).sin() * 0.5 + 0.5) as f32;
            let r_col = (255.0 * (1.0 - wave * 0.85)) as u8;
            let g_col = (30.0 + 210.0 * wave) as u8;
            let b_col = (220.0 + 35.0 * wave) as u8;
            ratatui::style::Color::Rgb(r_col, g_col, b_col)
        }
        SpectrumColorMode::MatrixPhosphor => {
            if r >= eq_height.saturating_sub(1) {
                theme.yellow_led
            } else {
                let intensity = ((r as f32 / eq_height as f32) * 155.0 + 100.0) as u8;
                ratatui::style::Color::Rgb(20, intensity, 40)
            }
        }
        SpectrumColorMode::AmberVintage => {
            if r >= eq_height {
                theme.red_led
            } else if r >= eq_height.saturating_sub(2) {
                theme.amber_bright
            } else {
                theme.amber
            }
        }
        SpectrumColorMode::FireAndIce => {
            // Dynamic Flame & Ocean Pulse Wave
            let wave = ((now_sec * 1.3 + (b as f64 * 0.25)).sin() * 0.5 + 0.5) as f32;
            if wave < 0.5 {
                let sub_t = wave * 2.0;
                let r_col = (20.0 + 200.0 * sub_t) as u8;
                let g_col = (100.0 + 120.0 * sub_t) as u8;
                let b_col = (255.0 * (1.0 - sub_t * 0.8)) as u8;
                ratatui::style::Color::Rgb(r_col, g_col, b_col)
            } else {
                let sub_t = (wave - 0.5) * 2.0;
                let r_col = 255;
                let g_col = (220.0 * (1.0 - sub_t * 0.7)) as u8;
                let b_col = (50.0 * (1.0 - sub_t)) as u8;
                ratatui::style::Color::Rgb(r_col, g_col, b_col)
            }
        }
        SpectrumColorMode::ThemeAccent => {
            if let Some(custom) = custom_color {
                if let Some(c) = parse_hex_color(custom) {
                    return c;
                }
            }
            theme.cyan_dolby
        }
    }
}

fn hsv_to_rgb(h_deg: f64, s: f64, v: f64) -> ratatui::style::Color {
    let h = (h_deg % 360.0).max(0.0) / 60.0;
    let i = h.floor() as i32;
    let f = h - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    ratatui::style::Color::Rgb(
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

fn parse_hex_color(h: &str) -> Option<ratatui::style::Color> {
    let clean = h.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(ratatui::style::Color::Rgb(r, g, b))
    } else {
        None
    }
}

