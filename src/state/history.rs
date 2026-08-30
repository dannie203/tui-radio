use crate::state::types::{HistoryEntry, MediaItem};
use std::fs;
use std::path::PathBuf;

fn history_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("boombox-tui")
        .join("history.json")
}

pub fn load_history() -> Vec<HistoryEntry> {
    let path = history_file();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(list) = serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                return list;
            }
        }
    }
    Vec::new()
}

pub fn save_history(history: &[HistoryEntry]) {
    let file = history_file();
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string(history) {
        let _ = fs::write(file, json);
    }
}

pub fn record_history_entry(history: &mut Vec<HistoryEntry>, item: &MediaItem) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let source = if item.is_radio {
        "Radio".to_string()
    } else if item.is_youtube {
        "YouTube".to_string()
    } else if item.url.contains("soundcloud.com") {
        "SoundCloud".to_string()
    } else if item.url.starts_with("http") {
        "Web Stream".to_string()
    } else {
        "Local".to_string()
    };

    // Deduplicated Recency List (Smart Upsert)
    let mut play_count = 1;
    if let Some(pos) = history
        .iter()
        .position(|h| h.url == item.url || (h.title == item.title && h.artist == item.artist))
    {
        let existing = history.remove(pos);
        play_count = existing.play_count + 1;
    }

    let entry = HistoryEntry {
        id: item.id.clone(),
        title: item.title.clone(),
        artist: item.artist.clone(),
        album: item.album.clone(),
        url: item.url.clone(),
        source,
        duration: item.duration,
        last_played: now,
        play_count,
    };

    history.insert(0, entry);
    if history.len() > 1000 {
        history.truncate(1000);
    }
    save_history(history);
}

pub fn history_to_media_item(entry: &HistoryEntry) -> MediaItem {
    let is_radio = entry.source == "Radio";
    let is_youtube = entry.source == "YouTube";
    let format_badge = match entry.source.as_str() {
        "Radio" => "RADIO",
        "YouTube" => "OPUS",
        "SoundCloud" => "SC-MP3",
        "Web Stream" => "STREAM",
        _ => "AUDIO",
    };

    MediaItem {
        id: entry.id.clone(),
        title: entry.title.clone(),
        artist: entry.artist.clone(),
        album: entry.album.clone(),
        url: entry.url.clone(),
        duration: entry.duration,
        format: Some(format_badge.to_string()),
        bitrate: Some(192),
        is_radio,
        is_youtube,
        is_favorite: false,
        file_size: None,
        track_no: None,
        sample_rate: Some(48000),
        bit_depth: Some(16),
    }
}
