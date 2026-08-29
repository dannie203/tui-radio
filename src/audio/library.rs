use crate::state::types::{AlbumGroup, MediaItem};
use lofty::prelude::*;
use lofty::probe::Probe;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_local_library(custom_dir: Option<&Path>) -> (Vec<MediaItem>, Vec<AlbumGroup>) {
    let mut tracks = Vec::new();
    let music_dir = match custom_dir {
        Some(p) if p.exists() => p.to_path_buf(),
        _ => dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Music")))
            .unwrap_or_else(|| PathBuf::from(".")),
    };

    if music_dir.exists() {
        scan_dir_recursive(&music_dir, &mut tracks, 0, 4);
    }

    if tracks.is_empty() {
        tracks.push(MediaItem {
            id: "local_demo_1".to_string(),
            title: "Midnight Tokyo Lo-Fi".to_string(),
            artist: "CyberTape 1984".to_string(),
            album: Some("Neon Memories".to_string()),
            url: "https://streams.fluxfm.de/chillhop/mp3-128/streams.fluxfm.de/".to_string(),
            duration: 194.0,
            format: Some("FLAC".to_string()),
            bitrate: Some(920),
            is_radio: false,
            is_youtube: false,
            is_favorite: true,
            file_size: Some(28_400_000),
            track_no: Some(1),
            sample_rate: Some(96000),
            bit_depth: Some(24),
        });
    }

    // Group tracks into Albums
    let mut album_map: BTreeMap<String, Vec<MediaItem>> = BTreeMap::new();
    for t in &tracks {
        let alb_name = t.album.clone().unwrap_or_else(|| "Singles & Loose Tapes".to_string());
        album_map.entry(alb_name).or_default().push(t.clone());
    }

    let mut albums = Vec::new();
    for (name, mut alb_tracks) in album_map {
        // Sort album tracks strictly by track_no ascending, then title
        alb_tracks.sort_by(|a, b| {
            match (a.track_no, b.track_no) {
                (Some(num_a), Some(num_b)) => num_a.cmp(&num_b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            }
        });

        let artist = alb_tracks[0].artist.clone();
        let dominant_format = alb_tracks[0].format.clone().unwrap_or_else(|| "FLAC".to_string());

        albums.push(AlbumGroup {
            name,
            artist,
            year: None,
            format: dominant_format,
            tracks: alb_tracks,
        });
    }

    // Also sort all tracks list by Album name, then track_no, then title
    tracks.sort_by(|a, b| {
        let alb_a = a.album.as_deref().unwrap_or("");
        let alb_b = b.album.as_deref().unwrap_or("");
        let alb_cmp = alb_a.to_lowercase().cmp(&alb_b.to_lowercase());
        if alb_cmp != std::cmp::Ordering::Equal {
            return alb_cmp;
        }
        match (a.track_no, b.track_no) {
            (Some(na), Some(nb)) => na.cmp(&nb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
        }
    });

    (tracks, albums)
}

fn scan_dir_recursive(dir: &Path, out: &mut Vec<MediaItem>, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        let mut file_entries: Vec<_> = entries.flatten().collect();
        file_entries.sort_by_key(|e| e.path());

        for entry in file_entries {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, out, depth + 1, max_depth);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "flac" | "wav" | "mp3" | "opus" | "ogg" | "m4a" | "aac") {
                    let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Track");
                    let parent_folder = path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("Music");

                    // 1. Try reading embedded metadata tags with Lofty
                    let mut tag_title = None;
                    let mut tag_artist = None;
                    let mut tag_album = None;
                    let mut tag_track_no = None;
                    let mut duration = 0.0;
                    let mut bitrate = None;
                    let mut sample_rate = None;
                    let mut bit_depth = None;

                    if let Some(tagged_file) = Probe::open(&path).ok().and_then(|p| p.read().ok()) {
                        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                            tag_title = tag.title().as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                            tag_artist = tag.artist().as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                            tag_album = tag.album().as_deref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                            tag_track_no = tag.track();
                        }
                        let props = tagged_file.properties();
                        duration = props.duration().as_secs_f64();
                        bitrate = props.audio_bitrate();
                        sample_rate = props.sample_rate();
                        bit_depth = props.bit_depth().map(|b| b as u32);
                    }

                    // 2. Fallback to smart filename parser if tags are missing
                    let (parsed_artist, parsed_title, parsed_num) = parse_song_meta(file_name, parent_folder);

                    let title = tag_title.unwrap_or(parsed_title);
                    let artist = tag_artist.unwrap_or(parsed_artist);
                    let album = tag_album.or_else(|| Some(parent_folder.to_string()));
                    let track_no = tag_track_no.or(parsed_num);
                    let format_label = ext_lower.to_uppercase();
                    let file_size = entry.metadata().ok().map(|m| m.len());

                    out.push(MediaItem {
                        id: path.to_string_lossy().to_string(),
                        title,
                        artist,
                        album,
                        url: path.to_string_lossy().to_string(),
                        duration,
                        format: Some(format_label),
                        bitrate,
                        is_radio: false,
                        is_youtube: false,
                        is_favorite: false,
                        file_size,
                        track_no,
                        sample_rate,
                        bit_depth,
                    });
                }
            }
        }
    }
}

fn parse_song_meta(filename: &str, parent_dir: &str) -> (String, String, Option<u32>) {
    let clean_name = filename.trim();

    let parts: Vec<&str> = clean_name.split(" - ").collect();
    if parts.len() >= 3 {
        let (num, artist) = strip_leading_track_num(parts[0]);
        let clean_artist = if artist.is_empty() { parent_dir.to_string() } else { artist };
        let title = parts[2..].join(" - ");
        return (clean_artist, clean_track_title(&title), num);
    } else if parts.len() == 2 {
        let (num, part0) = strip_leading_track_num(parts[0]);
        let clean_part0 = if part0.is_empty() { parent_dir.to_string() } else { part0 };
        return (clean_part0, clean_track_title(parts[1]), num);
    }

    let (num, single_title) = strip_leading_track_num(clean_name);
    (parent_dir.to_string(), clean_track_title(&single_title), num)
}

fn strip_leading_track_num(s: &str) -> (Option<u32>, String) {
    let trimmed = s.trim();
    let mut num_str = String::new();
    let mut chars = trimmed.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }

    if !num_str.is_empty() {
        let rest: String = chars.collect();
        let clean_rest = rest
            .trim_start_matches(|c: char| c == '.' || c == '-' || c == '_' || c == ')' || c == ']' || c.is_whitespace())
            .trim();
        if let Ok(num) = num_str.parse::<u32>() {
            return (Some(num), clean_rest.to_string());
        }
    }
    (None, trimmed.to_string())
}

fn clean_track_title(title: &str) -> String {
    let (_, cleaned) = strip_leading_track_num(title);
    if cleaned.is_empty() {
        title.trim().to_string()
    } else {
        cleaned
    }
}
