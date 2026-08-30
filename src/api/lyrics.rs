use crate::state::types::SyncedLyricLine;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub const MATRIX_CHARS: &[char] = &[
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', '0', '1', '2', '8', '9',
    '#', '$', '%', '&', '*', '@', '§', '¶', 'Δ', 'Ω', 'Ξ', 'Ψ', 'λ', 'θ', 'π',
    '0', '1', '4', '7',
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsResponse {
    pub id: Option<u64>,
    #[serde(rename = "trackName")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
}

pub async fn fetch_lyrics(title: &str, artist: &str, file_path: Option<&str>) -> Vec<SyncedLyricLine> {
    // 1. Check for local .lrc file first (zero latency)
    if let Some(path_str) = file_path {
        let path = Path::new(path_str);
        if path.exists() {
            let lrc_path1 = path.with_extension("lrc");
            let lrc_path2 = Path::new(&format!("{}.lrc", path_str)).to_path_buf();

            for lp in [lrc_path1, lrc_path2] {
                if lp.exists() {
                    if let Ok(content) = fs::read_to_string(lp) {
                        let lines = parse_lrc(&content);
                        if !lines.is_empty() {
                            return lines;
                        }
                    }
                }
            }
        }
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    let clean_title = title.split('(').next().unwrap_or(title).split('[').next().unwrap_or(title).trim();
    let clean_artist = artist.split('&').next().unwrap_or(artist).split(',').next().unwrap_or(artist).trim();

    let mut fallback_plain: Option<String> = None;

    // 2. Query LRCLIB Exact Match API
    let url = format!(
        "https://lrclib.net/api/get?track_name={}&artist_name={}",
        urlencoding::encode(clean_title),
        urlencoding::encode(clean_artist)
    );

    if let Ok(resp) = client.get(&url).header("User-Agent", "boombox-rs/3.2.0").send().await {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<LyricsResponse>().await {
                if let Some(lrc) = data.synced_lyrics {
                    let lines = parse_lrc(&lrc);
                    if !lines.is_empty() {
                        cache_lrc_if_local(file_path, &lrc);
                        return lines;
                    }
                }
                if data.plain_lyrics.is_some() {
                    fallback_plain = data.plain_lyrics;
                }
            }
        }
    }

    // 3. Fallback to LRCLIB Search API (Artist + Title and Title Only)
    let search_queries = [
        format!("{} {}", clean_artist, clean_title),
        clean_title.to_string(),
    ];

    for search_q in &search_queries {
        let search_url = format!(
            "https://lrclib.net/api/search?q={}",
            urlencoding::encode(search_q)
        );

        if let Ok(resp) = client.get(&search_url).header("User-Agent", "boombox-rs/3.2.0").send().await {
            if resp.status().is_success() {
                if let Ok(list) = resp.json::<Vec<LyricsResponse>>().await {
                    // First pass: look specifically for synced_lyrics across all search results!
                    for item in &list {
                        if let Some(lrc) = &item.synced_lyrics {
                            let lines = parse_lrc(lrc);
                            if !lines.is_empty() {
                                cache_lrc_if_local(file_path, lrc);
                                return lines;
                            }
                        }
                    }

                    // Secondary fallback: plain lyrics
                    if fallback_plain.is_none() {
                        for item in list {
                            if let Some(plain) = item.plain_lyrics {
                                fallback_plain = Some(plain);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. If no synced lyrics exist anywhere, fallback to plain lyrics
    if let Some(plain) = fallback_plain {
        return parse_plain_lyrics(&plain);
    }

    Vec::new()
}

fn cache_lrc_if_local(file_path: Option<&str>, lrc: &str) {
    if let Some(path_str) = file_path {
        let p = Path::new(path_str);
        if p.exists() {
            let lrc_path = p.with_extension("lrc");
            let _ = fs::write(lrc_path, lrc);
        }
    }
}

pub fn parse_lrc(content: &str) -> Vec<SyncedLyricLine> {
    let mut lines = Vec::new();
    let time_re = Regex::new(r"\[(\d{1,3}):(\d{2})(?:[.:](\d{1,3}))?\](.*)").unwrap();
    let offset_re = Regex::new(r"(?i)\[offset:\s*([+-]?\d+)\]").unwrap();

    let mut global_offset_secs: f64 = 0.0;

    for line in content.lines() {
        if let Some(cap) = offset_re.captures(line) {
            if let Ok(ms) = cap[1].parse::<f64>() {
                global_offset_secs = ms / 1000.0;
            }
            continue;
        }

        if let Some(cap) = time_re.captures(line) {
            let mins: f64 = cap[1].parse().unwrap_or(0.0);
            let secs: f64 = cap[2].parse().unwrap_or(0.0);
            let ms: f64 = cap
                .get(3)
                .map_or(0.0, |m| {
                    let s = m.as_str();
                    format!("0.{}", s).parse::<f64>().unwrap_or(0.0)
                });
            let time = (mins * 60.0 + secs + ms) + global_offset_secs;
            let text = cap[4].trim().to_string();
            if !text.is_empty() {
                lines.push(SyncedLyricLine { time, text });
            }
        }
    }
    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    lines
}

fn parse_plain_lyrics(plain: &str) -> Vec<SyncedLyricLine> {
    let mut lines = Vec::new();
    for (i, line) in plain.lines().enumerate() {
        let text = line.trim().to_string();
        if !text.is_empty() {
            lines.push(SyncedLyricLine {
                time: (i as f64) * 4.0,
                text,
            });
        }
    }
    lines
}
