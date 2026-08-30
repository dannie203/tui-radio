use crate::state::types::MediaItem;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct SpotifyOEmbed {
    title: Option<String>,
    author_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    webpage_url: Option<String>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct StreamCollection {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    entries: Option<Vec<StreamEntry>>,
}

const KNOWN_SOURCES: &[(&str, &str)] = &[
    ("youtube.com", "YouTube"),
    ("youtu.be", "YouTube"),
    ("music.youtube.com", "YouTube Music"),
    ("soundcloud.com", "SoundCloud"),
    ("bandlab.com", "BandLab"),
    ("qobuz.com", "Qobuz"),
    ("spotify.com", "Spotify"),
    ("deezer.com", "Deezer"),
    ("tidal.com", "Tidal"),
    ("apple.com", "Apple Music"),
    ("bandcamp.com", "Bandcamp"),
    ("mixcloud.com", "Mixcloud"),
    ("vimeo.com", "Vimeo"),
    ("twitch.tv", "Twitch"),
];

const KNOWN_PREFIXES: &[(&str, &str)] = &[
    ("yt:", "YouTube"),
    ("sc:", "SoundCloud"),
    ("sp:", "Spotify"),
    ("dc:", "Deezer"),
];

fn source_name(input: &str) -> Option<String> {
    let trimmed = input
        .trim()
        .trim_start_matches("ytdl://")
        .trim_start_matches("yt:")
        .trim_start_matches("sc:")
        .trim_start_matches("sp:")
        .trim_start_matches("dc:");
    for (prefix, name) in KNOWN_PREFIXES {
        if input.trim().starts_with(prefix) {
            return Some(name.to_string());
        }
    }
    let lower = trimmed.to_ascii_lowercase();
    KNOWN_SOURCES
        .iter()
        .find(|(dom, _)| lower.contains(dom))
        .map(|(_, name)| name.to_string())
}

fn is_stream_url(input: &str) -> bool {
    let trimmed = input
        .trim()
        .trim_start_matches("ytdl://")
        .trim_start_matches("yt:")
        .trim_start_matches("sc:")
        .trim_start_matches("sp:")
        .trim_start_matches("dc:");
    trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || input.trim().starts_with("yt:")
        || input.trim().starts_with("sc:")
        || input.trim().starts_with("sp:")
        || input.trim().starts_with("dc:")
        || trimmed.starts_with("ytsearch:")
}

fn is_collection_url(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    if lower.contains("playlist?list=")
        || lower.contains("list=")
        || lower.contains("/playlist/")
        || lower.contains("/channel/")
        || lower.contains("/@")
        || lower.contains("/c/")
        || lower.contains("/videos")
        || lower.contains("/mix")
    {
        return true;
    }
    if lower.contains("soundcloud.com/") {
        return lower.contains("/sets/")
            || lower.contains("/discover/")
            || (!lower.contains("/listen/") && lower.matches('/').count() >= 3);
    }
    if lower.contains("bandlab.com") {
        return lower.contains("/albums")
            || lower.contains("/projects")
            || lower.contains("/artists");
    }
    if lower.contains("qobuz.com") {
        return lower.contains("/album/")
            || lower.contains("/artist/")
            || lower.contains("/label/")
            || lower.contains("/playlist/");
    }
    false
}

fn is_search_query(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.starts_with("ytdl://") {
        return false;
    }
    true
}

fn build_search_target(input: &str) -> (String, String) {
    let trimmed = input.trim();
    if trimmed.starts_with("sc:") || trimmed.starts_with("soundcloud:") {
        let q = trimmed.trim_start_matches("sc:").trim_start_matches("soundcloud:").trim();
        ("SoundCloud".to_string(), format!("scsearch15:{}", q))
    } else if trimmed.starts_with("sp:") || trimmed.starts_with("spotify:") {
        let q = trimmed.trim_start_matches("sp:").trim_start_matches("spotify:").trim();
        ("Spotify".to_string(), format!("ytsearch15:{} audio", q))
    } else if trimmed.starts_with("yt:") || trimmed.starts_with("youtube:") {
        let q = trimmed.trim_start_matches("yt:").trim_start_matches("youtube:").trim();
        ("YouTube".to_string(), format!("ytsearch15:{}", q))
    } else if trimmed.starts_with("scsearch") {
        ("SoundCloud".to_string(), trimmed.to_string())
    } else if trimmed.starts_with("ytsearch") {
        ("YouTube".to_string(), trimmed.to_string())
    } else {
        ("YouTube".to_string(), format!("ytsearch15:{}", trimmed))
    }
}

pub async fn resolve_stream_queue(input: &str) -> (String, Vec<MediaItem>) {
    let trimmed = input.trim().trim_start_matches("ytdl://");

    if is_search_query(trimmed) {
        let (source, search_cmd) = build_search_target(trimmed);
        let mut cmd = Command::new("yt-dlp");
        cmd.args(["--flat-playlist", "-J", "--no-warnings"])
            .arg("--")
            .arg(&search_cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        if let Ok(output) = cmd.output().await {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(collection) = serde_json::from_str::<StreamCollection>(&text) {
                if let Some(entries) = collection.entries {
                    let mut tracks = Vec::new();
                    for (i, entry) in entries.into_iter().enumerate() {
                        let id = entry.id.unwrap_or_default();
                        let title = entry.title.unwrap_or_else(|| format!("{} Track", source));
                        let url = entry
                            .webpage_url
                            .filter(|u| !u.is_empty())
                            .or_else(|| entry.url.filter(|u| !u.is_empty()))
                            .unwrap_or_else(|| {
                                if id.is_empty() {
                                    String::new()
                                } else {
                                    format!("https://www.youtube.com/watch?v={}", id)
                                }
                            });
                        if url.is_empty() {
                            continue;
                        }
                        let artist = entry.channel.unwrap_or_else(|| source.clone());
                        let format_badge = if source == "SoundCloud" { "SC-MP3" } else { "OPUS" };
                        let media = MediaItem {
                            id: format!("str_{}_{}_{}", source.to_lowercase(), id, i),
                            title,
                            artist,
                            album: Some(format!("{} Search: {}", source, trimmed)),
                            url,
                            duration: entry.duration.unwrap_or(0.0),
                            format: Some(format_badge.to_string()),
                            bitrate: Some(192),
                            is_radio: false,
                            is_youtube: source != "SoundCloud",
                            is_favorite: false,
                            file_size: None,
                            track_no: Some((i + 1) as u32),
                            sample_rate: Some(48000),
                            bit_depth: Some(16),
                        };
                        tracks.push(media);
                    }
                    if !tracks.is_empty() {
                        let label = format!("{} Search: {} ({} tracks)", source, trimmed, tracks.len());
                        return (label, tracks);
                    }
                }
            }
        }
    }

    let source = source_name(trimmed).unwrap_or_else(|| "Web Stream".to_string());
    let is_stream = is_stream_url(trimmed);
    let is_collection = is_collection_url(trimmed);

    let fallback_single = || async {
        let item = resolve_stream_item(trimmed).await;
        (format!("{} — 1 track", source), vec![item])
    };

    if !is_stream || !is_collection {
        return fallback_single().await;
    }

    let clean = trimmed
        .trim_start_matches("yt:")
        .trim_start_matches("sc:")
        .trim_start_matches("sp:")
        .trim_start_matches("dc:");

    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--flat-playlist", "-J", "--no-warnings"])
        .arg("--")
        .arg(clean)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(_) => return fallback_single().await,
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let collection: StreamCollection = match serde_json::from_str(&text) {
        Ok(p) => p,
        Err(_) => return fallback_single().await,
    };

    let entries = match collection.entries {
        Some(e) if !e.is_empty() => e,
        _ => return fallback_single().await,
    };

    let album_title = collection
        .title
        .clone()
        .unwrap_or_else(|| format!("{} Queue", source.clone()));

    let mut tracks = Vec::new();
    for (i, entry) in entries.into_iter().enumerate() {
        let id = entry.id.unwrap_or_default();
        let title = entry.title.unwrap_or_else(|| format!("{} Track", source.clone()));
        let url = entry
            .webpage_url
            .filter(|u| !u.is_empty())
            .or_else(|| entry.url.filter(|u| !u.is_empty()))
            .unwrap_or_else(|| {
                if id.is_empty() {
                    String::new()
                } else {
                    format!("https://www.youtube.com/watch?v={}", id)
                }
            });
        if url.is_empty() {
            continue;
        }
        let artist = entry.channel.unwrap_or_else(|| source.clone());
        let media = MediaItem {
            id: format!("strq_{}_{}", source.to_lowercase(), id),
            title: title.clone(),
            artist: artist.clone(),
            album: Some(album_title.clone()),
            url: url.clone(),
            duration: entry.duration.unwrap_or(0.0),
            format: Some("OPUS".to_string()),
            bitrate: Some(192),
            is_radio: false,
            is_youtube: source_name(&url).as_deref() == Some("YouTube")
                || source_name(&url).as_deref() == Some("YouTube Music"),
            is_favorite: false,
            file_size: None,
            track_no: Some((i + 1) as u32),
            sample_rate: Some(48000),
            bit_depth: Some(16),
        };
        tracks.push(media);
    }

    let label = if tracks.len() > 1 {
        format!("{} — {} tracks", album_title, tracks.len())
    } else {
        format!("{} — 1 track", album_title)
    };
    (label, tracks)
}

pub async fn enqueue_stream_url(
    input: &str,
    queue: &mut Vec<MediaItem>,
) -> String {
    let (label, tracks) = resolve_stream_queue(input).await;
    for t in tracks {
        if !queue.iter().any(|q| q.id == t.id) {
            queue.push(t);
        }
    }
    label
}

pub async fn resolve_stream_item(input: &str) -> MediaItem {
    let trimmed = input.trim();
    let source = source_name(trimmed);
    let is_supported = source.is_some();

    if trimmed.contains("open.spotify.com") || trimmed.starts_with("spotify:") {
        let clean_url = if trimmed.starts_with("spotify:track:") {
            let id = trimmed.trim_start_matches("spotify:track:");
            format!("https://open.spotify.com/track/{}", id)
        } else {
            trimmed.to_string()
        };

        let oembed_url = format!("https://open.spotify.com/oembed?url={}", urlencoding::encode(&clean_url));
        if let Ok(resp) = reqwest::get(&oembed_url).await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<SpotifyOEmbed>().await {
                    let raw_title = data.title.unwrap_or_else(|| "Spotify Track".to_string());
                    let artist = data.author_name.unwrap_or_else(|| "Spotify".to_string());
                    let search_query = format!("ytsearch:{} {}", artist, raw_title);

                    return MediaItem {
                        id: format!("spotify_{}", clean_url),
                        title: raw_title,
                        artist,
                        album: Some("Spotify Music".to_string()),
                        url: search_query,
                        duration: 0.0,
                        format: Some("SPOTIFY".to_string()),
                        bitrate: Some(320),
                        is_radio: false,
                        is_youtube: true,
                        is_favorite: false,
                        file_size: None,
                        track_no: None,
                        sample_rate: Some(44100),
                        bit_depth: Some(16),
                    };
                }
            }
        }

        return MediaItem {
            id: format!("spotify_{}", trimmed),
            title: "Spotify Stream".to_string(),
            artist: "Spotify Audio".to_string(),
            album: Some("Spotify".to_string()),
            url: trimmed.to_string(),
            duration: 0.0,
            format: Some("SPOTIFY".to_string()),
            bitrate: Some(320),
            is_radio: false,
            is_youtube: true,
            is_favorite: false,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        };
    }

    let label = source.clone().unwrap_or_else(|| "Web Stream".to_string());
    let (title, artist, format_badge) = match source.as_deref() {
        Some("YouTube Music") => ("YouTube Music Track".to_string(), "YouTube Music".to_string(), "YT-MUSIC".to_string()),
        Some("YouTube") => ("YouTube Video Stream".to_string(), "YouTube Audio".to_string(), "OPUS".to_string()),
        Some("SoundCloud") => ("SoundCloud Track".to_string(), "SoundCloud Audio".to_string(), "SC-MP3".to_string()),
        Some("BandLab") => ("BandLab Track".to_string(), "BandLab Audio".to_string(), "BANDLAB".to_string()),
        Some("Qobuz") => ("Qobuz Track".to_string(), "Qobuz Audio".to_string(), "QOBUZ".to_string()),
        Some("Deezer") => ("Deezer Track".to_string(), "Deezer Audio".to_string(), "DEEZER".to_string()),
        Some("Tidal") => ("Tidal Track".to_string(), "Tidal Audio".to_string(), "TIDAL".to_string()),
        Some("Bandcamp") => ("Bandcamp Track".to_string(), "Bandcamp Audio".to_string(), "BC-MP3".to_string()),
        Some("Mixcloud") => ("Mixcloud Track".to_string(), "Mixcloud Audio".to_string(), "MIXCLOUD".to_string()),
        Some("Apple Music") => ("Apple Music Track".to_string(), "Apple Music Audio".to_string(), "AM".to_string()),
        Some(_) => (format!("{} Track", label), format!("{} Audio", label), "STREAM".to_string()),
        None => ("Direct Web Stream".to_string(), "Live Internet Broadcast".to_string(), "STREAM".to_string()),
    };

    let stream_url = if is_supported {
        if trimmed.starts_with("http") || trimmed.starts_with("yt:") || trimmed.starts_with("sc:") || trimmed.starts_with("sp:") || trimmed.starts_with("dc:") || trimmed.starts_with("ytsearch:") {
            trimmed.to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        trimmed.to_string()
    };

    MediaItem {
        id: format!("stream_{}", trimmed),
        title,
        artist,
        album: Some("Web Stream".to_string()),
        url: stream_url,
        duration: 0.0,
        format: Some(format_badge),
        bitrate: Some(192),
        is_radio: !is_supported,
        is_youtube: matches!(source.as_deref(), Some("YouTube") | Some("YouTube Music") | Some("Spotify")),
        is_favorite: false,
        file_size: None,
        track_no: None,
        sample_rate: Some(48000),
        bit_depth: Some(16),
    }
}
