use crate::state::types::MediaItem;
use serde::Deserialize;
use std::net::IpAddr;
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

fn strip_stream_prefixes(input: &str) -> &str {
    input
        .trim()
        .trim_start_matches("ytdl://")
        .trim_start_matches("yt:")
        .trim_start_matches("sc:")
        .trim_start_matches("sp:")
        .trim_start_matches("dc:")
}

fn source_name(input: &str) -> Option<String> {
    for (prefix, name) in KNOWN_PREFIXES {
        if input.trim().starts_with(prefix) {
            return Some(name.to_string());
        }
    }
    let lower = strip_stream_prefixes(input).to_ascii_lowercase();
    KNOWN_SOURCES
        .iter()
        .find(|(dom, _)| lower.contains(dom))
        .map(|(_, name)| name.to_string())
}

fn is_stream_url(input: &str) -> bool {
    let trimmed = strip_stream_prefixes(input);
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
    if trimmed.contains("://") {
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

    if !is_search_query(trimmed) && !is_safe_stream_url(trimmed) {
        return ("Blocked / Dangerous URL".to_string(), Vec::new());
    }

    if is_search_query(trimmed) {
        let (source, search_cmd) = build_search_target(trimmed);
        let mut cmd = Command::new(crate::audio::player::resolve_executable("yt-dlp"));
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
                            sample_rate: None,
                            bit_depth: None,
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
        if item.url.is_empty() {
            ("Blocked / Dangerous URL".to_string(), Vec::new())
        } else {
            (format!("{} — 1 track", source), vec![item])
        }
    };

    if !is_stream || !is_collection {
        return fallback_single().await;
    }

    let clean = strip_stream_prefixes(trimmed);

    let mut cmd = Command::new(crate::audio::player::resolve_executable("yt-dlp"));
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
            sample_rate: None,
            bit_depth: None,
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

pub async fn resolve_stream_item(input: &str) -> MediaItem {
    let trimmed = input.trim();
    if !is_search_query(trimmed) && !is_safe_stream_url(trimmed) {
        return MediaItem {
            id: format!("unsafe_{}", trimmed),
            title: "Blocked / Dangerous Stream URL".to_string(),
            artist: "Security Policy".to_string(),
            album: None,
            url: String::new(),
            duration: 0.0,
            format: None,
            bitrate: None,
            is_radio: false,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: None,
            sample_rate: None,
            bit_depth: None,
        };
    }

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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();
        if let Ok(resp) = client.get(&oembed_url).send().await {
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
                        sample_rate: None,
                        bit_depth: None,
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
            sample_rate: None,
            bit_depth: None,
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
        sample_rate: None,
        bit_depth: None,
    }
}

pub fn extract_youtube_id(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.len() == 11 && !trimmed.contains('/') && !trimmed.contains('?') && !trimmed.contains('&') {
        return Some(trimmed.to_string());
    }
    if let Some(pos) = trimmed.find("v=") {
        let after = &trimmed[pos + 2..];
        let id: String = after.chars().take_while(|c| *c != '&' && *c != '#' && *c != '/').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = trimmed.find("youtu.be/") {
        let after = &trimmed[pos + 9..];
        let id: String = after.chars().take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = trimmed.find("/shorts/") {
        let after = &trimmed[pos + 8..];
        let id: String = after.chars().take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = trimmed.find("embed/") {
        let after = &trimmed[pos + 6..];
        let id: String = after.chars().take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

pub async fn fetch_youtube_radio_mix(video_id_or_url: &str) -> Vec<MediaItem> {
    let video_id = match extract_youtube_id(video_id_or_url) {
        Some(id) => id,
        None => return Vec::new(),
    };

    let mix_url = format!("https://www.youtube.com/watch?v={}&list=RD{}", video_id, video_id);
    let mut cmd = Command::new(crate::audio::player::resolve_executable("yt-dlp"));
    cmd.args(["--flat-playlist", "-J", "--no-warnings"])
        .arg("--")
        .arg(&mix_url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let collection: StreamCollection = match serde_json::from_str(&text) {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let entries = match collection.entries {
        Some(e) if !e.is_empty() => e,
        _ => return Vec::new(),
    };

    let mut tracks = Vec::new();
    for (i, entry) in entries.into_iter().enumerate() {
        let id = entry.id.unwrap_or_default();
        if id == video_id {
            continue;
        }
        let title = entry.title.unwrap_or_else(|| "YouTube Track".to_string());
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
        let artist = entry.channel.unwrap_or_else(|| "YouTube Audio".to_string());
        let media = MediaItem {
            id: format!("yt_mix_{}_{}", id, i),
            title,
            artist,
            album: Some("YouTube Music Mix".to_string()),
            url,
            duration: entry.duration.unwrap_or(0.0),
            format: Some("OPUS".to_string()),
            bitrate: Some(192),
            is_radio: false,
            is_youtube: true,
            is_favorite: false,
            file_size: None,
            track_no: Some((i + 1) as u32),
            sample_rate: None,
            bit_depth: None,
        };
        tracks.push(media);
    }
    tracks
}

/// Validates whether a remote streaming URL uses a permitted protocol
/// and prevents dangerous local file / cloud metadata access while fully
/// supporting Home Servers, NAS, and LAN streams (Navidrome, Jellyfin, Icecast, Subsonic).
pub fn is_safe_stream_url(url: &str) -> bool {
    let trimmed = url.trim();
    let lower = trimmed.to_lowercase();

    // Block dangerous protocols (local file exposure, arbitrary network protocols)
    if lower.starts_with("file://")
        || lower.starts_with("gopher://")
        || lower.starts_with("dict://")
        || lower.starts_with("smb://")
        || lower.starts_with("ftp://")
    {
        return false;
    }

    // Allowed service prefixes
    if lower.starts_with("yt:")
        || lower.starts_with("sc:")
        || lower.starts_with("sp:")
        || lower.starts_with("dc:")
        || lower.starts_with("ytdl://")
    {
        return true;
    }

    // Must start with http:// or https://
    if lower.starts_with("http://") || lower.starts_with("https://") {
        let without_proto = if lower.starts_with("https://") {
            &trimmed[8..]
        } else {
            &trimmed[7..]
        };

        // Extract host
        let host_port = without_proto
            .split('/')
            .next()
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("");
        let host = if host_port.starts_with('[') {
            if let Some(end) = host_port.find(']') {
                &host_port[1..end]
            } else {
                host_port
            }
        } else {
            host_port.split(':').next().unwrap_or(host_port)
        };

        if host.is_empty() {
            return false;
        }

        // Block AWS / GCP / Azure Cloud Metadata IP (169.254.169.254)
        if host == "169.254.169.254" {
            return false;
        }
        if let Ok(ip) = host.parse::<IpAddr>() {
            if let IpAddr::V4(ipv4) = ip {
                let o = ipv4.octets();
                if o[0] == 169 && o[1] == 254 {
                    return false;
                }
            }
        }

        // Note: Home servers & LAN IPs (192.168.x.x, 10.x.x.x, 172.16-31.x.x),
        // mDNS domains (.local), and localhost (Jellyfin/Navidrome on local port)
        // are explicitly allowed for self-hosted audio streaming.
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_stream_url() {
        // Valid Public URLs
        assert!(is_safe_stream_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
        assert!(is_safe_stream_url("https://soundcloud.com/artist/track"));
        assert!(is_safe_stream_url("http://stream.radioparadise.com/mp3-192"));
        assert!(is_safe_stream_url("yt:lofi hip hop"));
        assert!(is_safe_stream_url("sc:chillhop"));
        assert!(is_safe_stream_url("ytdl://https://youtu.be/dQw4w9WgXcQ"));

        // Valid Home Server / NAS / LAN URLs (Navidrome, Jellyfin, Icecast)
        assert!(is_safe_stream_url("http://192.168.1.100:4533/rest/stream"));
        assert!(is_safe_stream_url("http://10.0.0.5:8000/live.mp3"));
        assert!(is_safe_stream_url("http://172.16.1.20:8096/audio/stream"));
        assert!(is_safe_stream_url("http://homeserver.local:4533/stream"));
        assert!(is_safe_stream_url("http://localhost:4533/rest/stream"));
        assert!(is_safe_stream_url("http://127.0.0.1:8000/stream.flac"));

        // Dangerous schemes (file access / protocol injection)
        assert!(!is_safe_stream_url("file:///etc/passwd"));
        assert!(!is_safe_stream_url("gopher://127.0.0.1/"));
        assert!(!is_safe_stream_url("dict://127.0.0.1:11211/"));
        assert!(!is_safe_stream_url("smb://attacker/share"));

        // Cloud metadata target (blocked to protect container/VPS environments)
        assert!(!is_safe_stream_url("http://169.254.169.254/latest/meta-data/"));
    }

    #[test]
    fn test_extract_youtube_id() {
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            extract_youtube_id("https://youtu.be/dQw4w9WgXcQ?si=123"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/embed/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            extract_youtube_id("https://www.youtube.com/shorts/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[tokio::test]
    async fn test_resolve_stream_item_dangerous_url() {
        let item = resolve_stream_item("file:///etc/passwd").await;
        assert!(item.url.is_empty());
        assert_eq!(item.id, "unsafe_file:///etc/passwd");
    }
}
