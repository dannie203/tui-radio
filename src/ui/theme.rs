use ratatui::style::Color;
use std::fs;

#[derive(Debug, Clone)]
pub struct Theme {
    pub id: &'static str,
    pub name: &'static str,
    pub bg_dark: Color,
    pub bg_panel: Color,
    pub bg_lcd: Color,
    pub border_dim: Color,
    pub border_focus: Color,
    pub border_lcd: Color,
    pub amber: Color,
    pub amber_bright: Color,
    pub green_phosphor: Color,
    pub gold: Color,
    pub cyan_dolby: Color,
    pub red_led: Color,
    pub yellow_led: Color,
    pub cream: Color,
    pub chrome: Color,
    pub muted: Color,
}

pub fn hex(h: &str) -> Color {
    let clean = h.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}

pub fn detect_system_theme() -> Option<Theme> {
    let home = dirs::home_dir()?;
    let paths = [
        home.join(".local/state/omarchy/current/theme/colors.toml"),
        home.join(".config/omarchy/current/theme/colors.toml"),
    ];

    for path in paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return parse_omarchy_toml(&content);
            }
        }
    }
    None
}

fn parse_omarchy_toml(content: &str) -> Option<Theme> {
    let val: toml::Value = toml::from_str(content).ok()?;
    let get_color = |k: &str| -> Option<Color> {
        val.get(k).and_then(|v| v.as_str()).map(hex)
    };

    let accent = get_color("accent").or_else(|| get_color("cyan")).unwrap_or(hex("#ffb000"));
    let bg = get_color("darker_background").or_else(|| get_color("background")).unwrap_or(hex("#101315"));
    let panel = get_color("background").unwrap_or(hex("#15181b"));
    let fg = get_color("foreground").unwrap_or(hex("#cacccc"));
    let muted = get_color("muted").unwrap_or(hex("#4b4e55"));

    Some(Theme {
        id: "SYSTEM_AUTO",
        name: "🖥️ Omarchy Auto",
        bg_dark: bg,
        bg_panel: panel,
        bg_lcd: bg,
        border_dim: get_color("selection").unwrap_or(hex("#343d41")),
        border_focus: accent,
        border_lcd: get_color("selection").unwrap_or(hex("#343d41")),
        amber: accent,
        amber_bright: get_color("bright_yellow").unwrap_or(fg),
        green_phosphor: get_color("green").unwrap_or(hex("#9fa5a9")),
        gold: get_color("yellow").unwrap_or(hex("#eed49f")),
        cyan_dolby: get_color("cyan").unwrap_or(hex("#8bd5ca")),
        red_led: get_color("red").unwrap_or(hex("#ed8796")),
        yellow_led: get_color("yellow").unwrap_or(hex("#eed49f")),
        cream: fg,
        chrome: get_color("light_foreground").unwrap_or(fg),
        muted,
    })
}

pub fn get_themes() -> Vec<Theme> {
    let mut list = Vec::new();
    if let Some(sys) = detect_system_theme() {
        list.push(sys);
    }
    list.extend([
        Theme {
            id: "AMBER_GOLD",
            name: "📼 Amber Gold",
            bg_dark: hex("#0e0b06"),
            bg_panel: hex("#16120a"),
            bg_lcd: hex("#1f190e"),
            border_dim: hex("#3a2f1c"),
            border_focus: hex("#ffb000"),
            border_lcd: hex("#3a2f1c"),
            amber: hex("#ffb000"),
            amber_bright: hex("#ffd24d"),
            green_phosphor: hex("#33ff33"),
            gold: hex("#f5c542"),
            cyan_dolby: hex("#00e5ff"),
            red_led: hex("#ff3344"),
            yellow_led: hex("#ffff33"),
            cream: hex("#f3ead8"),
            chrome: hex("#b8c4ce"),
            muted: hex("#6f7e91"),
        },
        Theme {
            id: "CYBER_NEON",
            name: "⚡ Cyber Neon",
            bg_dark: hex("#080512"),
            bg_panel: hex("#0f0c20"),
            bg_lcd: hex("#130f2c"),
            border_dim: hex("#2c2250"),
            border_focus: hex("#00f0ff"),
            border_lcd: hex("#332460"),
            amber: hex("#ff007f"),
            amber_bright: hex("#ff55a3"),
            green_phosphor: hex("#00ff9f"),
            gold: hex("#ffe600"),
            cyan_dolby: hex("#00f0ff"),
            red_led: hex("#ff0055"),
            yellow_led: hex("#ffff00"),
            cream: hex("#e0f7fa"),
            chrome: hex("#a5b4fc"),
            muted: hex("#625f87"),
        },
        Theme {
            id: "GREEN_PHOSPHOR",
            name: "📟 Green Phosphor",
            bg_dark: hex("#040d06"),
            bg_panel: hex("#08180c"),
            bg_lcd: hex("#0c2010"),
            border_dim: hex("#1a3b22"),
            border_focus: hex("#00ff66"),
            border_lcd: hex("#1d4a27"),
            amber: hex("#00ff66"),
            amber_bright: hex("#66ff99"),
            green_phosphor: hex("#00ff66"),
            gold: hex("#a3e635"),
            cyan_dolby: hex("#2dd4bf"),
            red_led: hex("#f87171"),
            yellow_led: hex("#facc15"),
            cream: hex("#dcfce7"),
            chrome: hex("#86efac"),
            muted: hex("#3b6b47"),
        },
        Theme {
            id: "TOKYO_NIGHT",
            name: "🌃 Tokyo Night",
            bg_dark: hex("#16161e"),
            bg_panel: hex("#1a1b26"),
            bg_lcd: hex("#24283b"),
            border_dim: hex("#3b4261"),
            border_focus: hex("#7aa2f7"),
            border_lcd: hex("#414868"),
            amber: hex("#7aa2f7"),
            amber_bright: hex("#bb9af7"),
            green_phosphor: hex("#73daca"),
            gold: hex("#e0af68"),
            cyan_dolby: hex("#7dcfff"),
            red_led: hex("#f7768e"),
            yellow_led: hex("#e0af68"),
            cream: hex("#c0caf5"),
            chrome: hex("#9aa5ce"),
            muted: hex("#565f89"),
        },
        Theme {
            id: "CATPPUCCIN",
            name: "☕ Catppuccin Mocha",
            bg_dark: hex("#11111b"),
            bg_panel: hex("#181825"),
            bg_lcd: hex("#1e1e2e"),
            border_dim: hex("#45475a"),
            border_focus: hex("#f5c2e7"),
            border_lcd: hex("#585b70"),
            amber: hex("#fab387"),
            amber_bright: hex("#f9e2af"),
            green_phosphor: hex("#a6e3a1"),
            gold: hex("#f9e2af"),
            cyan_dolby: hex("#89dceb"),
            red_led: hex("#f38ba8"),
            yellow_led: hex("#f9e2af"),
            cream: hex("#cdd6f4"),
            chrome: hex("#bac2de"),
            muted: hex("#6c7086"),
        },
        Theme {
            id: "NORD",
            name: "❄️ Nord Frost",
            bg_dark: hex("#242933"),
            bg_panel: hex("#2e3440"),
            bg_lcd: hex("#3b4252"),
            border_dim: hex("#4c566a"),
            border_focus: hex("#88c0d0"),
            border_lcd: hex("#434c5e"),
            amber: hex("#88c0d0"),
            amber_bright: hex("#8fbcbb"),
            green_phosphor: hex("#a3be8c"),
            gold: hex("#ebcb8b"),
            cyan_dolby: hex("#81a1c1"),
            red_led: hex("#bf616a"),
            yellow_led: hex("#ebcb8b"),
            cream: hex("#eceff4"),
            chrome: hex("#d8dee9"),
            muted: hex("#616e88"),
        },
    ]);
    list
}
