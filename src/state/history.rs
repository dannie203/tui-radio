use crate::state::types::{HistoryEntry, MediaItem};
use std::fs;
use std::path::PathBuf;

fn history_file() -> PathBuf {
    crate::state::config::get_config_dir()
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
        format: item.format.clone(),
        bitrate: item.bitrate,
        sample_rate: item.sample_rate,
        bit_depth: item.bit_depth,
    };

    history.insert(0, entry);
    if history.len() > 1000 {
        history.truncate(1000);
    }
    save_history(history);
}

use lofty::file::AudioFile;
use lofty::probe::Probe;

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

    let mut sample_rate = entry.sample_rate;
    let mut bit_depth = entry.bit_depth;
    let mut bitrate = entry.bitrate.or(Some(192));
    let mut format = entry.format.clone().or_else(|| Some(format_badge.to_string()));
    let mut file_size = None;

    // Only probe local file with lofty if metadata wasn't already recorded in history entry
    if !is_radio && !is_youtube && (sample_rate.is_none() || bit_depth.is_none()) {
        let p = std::path::Path::new(&entry.url);
        if p.exists() {
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                format = Some(ext.to_uppercase());
            }
            if let Ok(probe) = Probe::open(p) {
                if let Ok(tagged_file) = probe.read() {
                    let props = tagged_file.properties();
                    if sample_rate.is_none() {
                        sample_rate = props.sample_rate();
                    }
                    if bit_depth.is_none() {
                        bit_depth = props.bit_depth().map(|b| b as u32);
                    }
                    if entry.bitrate.is_none() {
                        bitrate = props.audio_bitrate();
                    }
                }
            }
            file_size = std::fs::metadata(p).ok().map(|m| m.len());
        }
    } else if !is_radio && !is_youtube {
        file_size = std::fs::metadata(&entry.url).ok().map(|m| m.len());
    }

    MediaItem {
        id: entry.id.clone(),
        title: entry.title.clone(),
        artist: entry.artist.clone(),
        album: entry.album.clone(),
        url: entry.url.clone(),
        duration: entry.duration,
        format,
        bitrate,
        is_radio,
        is_youtube,
        is_favorite: false,
        file_size,
        track_no: None,
        sample_rate,
        bit_depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_to_media_item_fallback() {
        let entry = HistoryEntry {
            id: "test".to_string(),
            title: "Test".to_string(),
            artist: "Artist".to_string(),
            album: None,
            url: "https://example.com/stream".to_string(),
            source: "Radio".to_string(),
            duration: 100.0,
            last_played: 0,
            play_count: 1,
            format: None,
            bitrate: None,
            sample_rate: None,
            bit_depth: None,
        };
        let item = history_to_media_item(&entry);
        assert!(item.is_radio);
        assert_eq!(item.sample_rate, None);
        assert_eq!(item.bit_depth, None);
    }
}
