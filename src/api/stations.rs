use crate::state::types::{GenreFilter, MediaItem};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const RADIO_SERVERS: &[&str] = &[
    "https://de1.api.radio-browser.info",
    "https://nl1.api.radio-browser.info",
    "https://at1.api.radio-browser.info",
    "https://all.api.radio-browser.info",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStation {
    #[serde(default)]
    pub stationuuid: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub countrycode: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
    #[serde(default)]
    pub bitrate: Option<u32>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub url_resolved: Option<String>,
    #[serde(default)]
    pub clickcount: Option<u64>,
    #[serde(default)]
    pub votes: Option<u64>,
}

/// Strictly filters out banned, anti-state, or reactionary radio broadcasts
pub fn is_banned_station(name: &str, url: &str, tags: &str) -> bool {
    let lower = format!("{} {} {}", name, url, tags).to_ascii_lowercase();
    let banned_patterns = [
        "rfa", "radio free asia", "á châu tự do", "a chau tu do",
        "voa", "voice of america", "tiếng hoa kỳ", "tieng hoa ky",
        "bbc vietnamese", "bbc tiếng việt", "bbc tieng viet",
        "rfi tiếng việt", "rfi tieng viet", "radio france internationale",
        "sbtn", "saigon broadcasting", "viet tan", "việt tân",
        "chân trời mới", "chan troi moi", "đáp lời sông núi", "dap loi song nui",
        "vnch", "việt nam cộng hòa", "viet nam cong hoa",
        "chính phủ quốc gia việt nam lâm thời", "đào minh quân", "dao minh quan",
        "ba que", "cờ vàng", "co vang", "cali radio", "khang chien", "kháng chiến",
        "tin lanh dega", "dân làm báo", "dan lam bao", "vietlist", "vietlive",
        "người việt tv", "nguoi viet tv", "nửa vòng trái đất",
    ];

    for pat in banned_patterns {
        if lower.contains(pat) {
            return true;
        }
    }
    false
}

/// Deduplicate radio stations by normalized URL and name, keeping the highest bitrate version
pub fn deduplicate_stations(stations: Vec<MediaItem>) -> Vec<MediaItem> {
    let mut out: Vec<MediaItem> = Vec::new();
    let mut seen_urls: HashSet<String> = HashSet::new();
    let mut seen_names: HashSet<String> = HashSet::new();

    for s in stations {
        let norm_url = normalize_url(&s.url);
        let norm_name = normalize_station_name(&s.title);

        if seen_urls.contains(&norm_url) {
            continue;
        }

        if !norm_name.is_empty() && seen_names.contains(&norm_name) {
            // Keep the one with the higher bitrate / quality
            if let Some(existing) = out.iter_mut().find(|x| normalize_station_name(&x.title) == norm_name) {
                if s.bitrate.unwrap_or(0) > existing.bitrate.unwrap_or(0) {
                    *existing = s.clone();
                    seen_urls.insert(norm_url);
                }
            }
            continue;
        }

        seen_urls.insert(norm_url);
        if !norm_name.is_empty() {
            seen_names.insert(norm_name);
        }
        out.push(s);
    }

    out
}

fn normalize_url(url: &str) -> String {
    let trimmed = url.trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    trimmed.to_lowercase()
}

fn normalize_station_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut cleaned = String::new();
    for c in lower.chars() {
        if c.is_alphanumeric() {
            cleaned.push(c);
        }
    }
    cleaned
        .replace("128k", "")
        .replace("320k", "")
        .replace("192k", "")
        .replace("64k", "")
        .replace("kbps", "")
        .replace("mp3", "")
        .replace("aac", "")
        .replace("stream", "")
        .trim()
        .to_string()
}

/// Official and verified Vietnamese National & Regional Broadcasters (VOV, VOH, Hanoi Radio, Danang, Xone)
pub fn get_official_vietnam_stations() -> Vec<MediaItem> {
    vec![
        MediaItem {
            id: "vn_vov1".to_string(),
            title: "VOV1 - Thời sự & Chính trị Quốc gia".to_string(),
            artist: "Đài Tiếng nói Việt Nam [128kbps]".to_string(),
            album: Some("Thời sự / Chính trị".to_string()),
            url: "http://51.79.160.245:8100/vov1.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: Some(1),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_vov2".to_string(),
            title: "VOV2 - Văn hóa, Xã hội & Giáo dục".to_string(),
            artist: "Đài Tiếng nói Việt Nam [128kbps]".to_string(),
            album: Some("Văn hóa / Giáo dục".to_string()),
            url: "http://51.79.160.245:8100/vov2.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: Some(2),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_vov3".to_string(),
            title: "VOV3 - Âm nhạc & Giải trí Trẻ".to_string(),
            artist: "Đài Tiếng nói Việt Nam [128kbps]".to_string(),
            album: Some("Âm nhạc / V-Pop".to_string()),
            url: "http://51.79.160.245:8100/vov3.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: Some(3),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_vov_gt_hn".to_string(),
            title: "VOV Giao Thông Hà Nội (91.0 MHz)".to_string(),
            artist: "VOV Giao Thông [128kbps]".to_string(),
            album: Some("Giao thông / Đô thị".to_string()),
            url: "http://51.79.160.245:8100/vovgthn.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: Some(4),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_vov_gt_hcm".to_string(),
            title: "VOV Giao Thông TP. Hồ Chí Minh (91.0 MHz)".to_string(),
            artist: "VOV Giao Thông [128kbps]".to_string(),
            album: Some("Giao thông / Đô thị".to_string()),
            url: "http://51.79.160.245:8100/vovgthcm.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: Some(5),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_voh_999".to_string(),
            title: "VOH FM 99.9 MHz - Đài Tiếng nói Nhân dân TP.HCM".to_string(),
            artist: "Đài Tiếng nói Nhân dân TP.HCM [128kbps]".to_string(),
            album: Some("Thời sự / Giải trí TP.HCM".to_string()),
            url: "http://stream.voh.com.vn:8000/fm999.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: Some(6),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_voh_956".to_string(),
            title: "VOH FM 95.6 MHz - Kênh Thông tin Thương mại & Giải trí".to_string(),
            artist: "Đài Tiếng nói Nhân dân TP.HCM [128kbps]".to_string(),
            album: Some("Thương mại / Giải trí".to_string()),
            url: "http://stream.voh.com.vn:8000/fm956.mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: Some(7),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "vn_xone_fm".to_string(),
            title: "XoneFM Live - Hit Music Station".to_string(),
            artist: "Xone Radio Vietnam [128kbps]".to_string(),
            album: Some("V-Pop / US-UK Hits".to_string()),
            url: "https://stream.zeno.fm/k28b08p4a8quv".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: Some(8),
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
    ]
}

pub fn get_curated_stations() -> Vec<MediaItem> {
    let mut items = get_official_vietnam_stations();

    let fallback_paths = [
        Path::new("data/fallback.json"),
        
    ];

    for path in fallback_paths {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(raw_list) = serde_json::from_str::<Vec<RawStation>>(&content) {
                    let mut file_items: Vec<MediaItem> = raw_list
                        .into_iter()
                        .filter_map(|s| {
                            let url = s.url_resolved.or(s.url)?;
                            let name = s.name.unwrap_or_else(|| "Radio Station".to_string());
                            let country = s.country.unwrap_or_else(|| "Global".to_string());
                            let tags = s.tags.unwrap_or_default();
                            if is_banned_station(&name, &url, &tags) {
                                return None;
                            }
                            let br = s.bitrate.unwrap_or(128);
                            Some(MediaItem {
                                id: format!("radio_{}", url),
                                title: name,
                                artist: format!("{} [{}kbps]", country, br),
                                album: Some(tags),
                                url,
                                duration: 0.0,
                                format: Some("MP3".to_string()),
                                bitrate: Some(br),
                                is_radio: true,
                                is_youtube: false,
                                is_favorite: false,
                                file_size: None,
                                track_no: None,
                                sample_rate: Some(44100),
                                bit_depth: Some(16),
                            })
                        })
                        .collect();
                    if !file_items.is_empty() {
                        items.append(&mut file_items);
                        return deduplicate_stations(items);
                    }
                }
            }
        }
    }

    // Default global high quality curated stations
    items.extend(vec![
        MediaItem {
            id: "radio_fluxfm_chillhop".to_string(),
            title: "Chillhop Radio Live".to_string(),
            artist: "FluxFM Germany [128kbps]".to_string(),
            album: Some("Lofi / Chill Beats".to_string()),
            url: "https://streams.fluxfm.de/chillhop/mp3-128/streams.fluxfm.de/".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "radio_defjay".to_string(),
            title: "DEFJAY - Soulful R&B & Hip-Hop".to_string(),
            artist: "Germany [128kbps]".to_string(),
            album: Some("Hip-Hop / Soul".to_string()),
            url: "https://stream.defjay.com/stream".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "radio_ukdrill".to_string(),
            title: "UK Drill & Grime Radio".to_string(),
            artist: "United Kingdom [128kbps]".to_string(),
            album: Some("UK Drill / Grime".to_string()),
            url: "https://stream.zeno.fm/6wz3q3xq8k8uv".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: false,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "radio_lofigirl".to_string(),
            title: "Lofi Girl - Relax & Study".to_string(),
            artist: "Worldwide [128kbps]".to_string(),
            album: Some("Lofi / Instrumental".to_string()),
            url: "https://play.streamafrica.net/lofi".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
        MediaItem {
            id: "radio_synthwave".to_string(),
            title: "Nightwave Plaza - Synthwave & Vaporwave".to_string(),
            artist: "Plaza Network [128kbps]".to_string(),
            album: Some("Synthwave / Retro".to_string()),
            url: "https://radio.plaza.one/mp3".to_string(),
            duration: 0.0,
            format: Some("MP3".to_string()),
            bitrate: Some(128),
            is_radio: true,
            is_youtube: false,
            is_favorite: true,
            file_size: None,
            track_no: None,
            sample_rate: Some(44100),
            bit_depth: Some(16),
        },
    ]);

    deduplicate_stations(items)
}

/// Query live Radio-Browser API with server failover, genre tags, deduplication, and anti-state blacklist filtering
pub async fn fetch_radio_browser_genre(genre: GenreFilter) -> Vec<MediaItem> {
    if genre == GenreFilter::Vietnam {
        let mut vn_stations = get_official_vietnam_stations();
        // Also query API for other legal Vietnamese stations
        if let Ok(mut extra) = query_radio_browser("/json/stations/bycountrycodeexact/VN?limit=60&order=clickcount&reverse=true").await {
            extra.retain(|s| !vn_stations.iter().any(|v| v.id == s.id || v.title == s.title));
            vn_stations.append(&mut extra);
        }
        return deduplicate_stations(vn_stations);
    }

    let query_path = match genre {
        GenreFilter::All => "/json/stations/topclick/60",
        GenreFilter::Favorites => "/json/stations/topvote/60",
        GenreFilter::LoFi => "/json/stations/bytag/lofi?limit=50&order=clickcount&reverse=true",
        GenreFilter::Synthwave => "/json/stations/bytag/synthwave?limit=50&order=clickcount&reverse=true",
        GenreFilter::Jazz => "/json/stations/bytag/jazz?limit=50&order=clickcount&reverse=true",
        GenreFilter::HipHop => "/json/stations/bytag/hiphop?limit=50&order=clickcount&reverse=true",
        GenreFilter::Rock => "/json/stations/bytag/rock?limit=50&order=clickcount&reverse=true",
        GenreFilter::Electronic => "/json/stations/bytag/electronic?limit=50&order=clickcount&reverse=true",
        GenreFilter::Classical => "/json/stations/bytag/classical?limit=50&order=clickcount&reverse=true",
        GenreFilter::Pop => "/json/stations/bytag/pop?limit=50&order=clickcount&reverse=true",
        GenreFilter::Vietnam => "/json/stations/bycountrycodeexact/VN?limit=60&order=clickcount&reverse=true",
        GenreFilter::Japan => "/json/stations/bycountrycodeexact/JP?limit=50&order=clickcount&reverse=true",
        GenreFilter::GlobalTop => "/json/stations/topclick/60",
    };

    if let Ok(items) = query_radio_browser(query_path).await {
        if !items.is_empty() {
            return deduplicate_stations(items);
        }
    }

    get_curated_stations()
}

async fn query_radio_browser(endpoint: &str) -> Result<Vec<MediaItem>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
        .unwrap_or_default();

    for server in RADIO_SERVERS {
        let url = format!("{}{}", server, endpoint);
        if let Ok(resp) = client.get(&url).header("User-Agent", "boombox-rs/3.2.0").send().await {
            if resp.status().is_success() {
                if let Ok(raw_list) = resp.json::<Vec<RawStation>>().await {
                    let mut items: Vec<MediaItem> = raw_list
                        .into_iter()
                        .filter_map(|s| {
                            let url = s.url_resolved.or(s.url)?;
                            if !url.starts_with("http://") && !url.starts_with("https://") {
                                return None;
                            }
                            let name = s.name.unwrap_or_else(|| "Radio Station".to_string()).trim().to_string();
                            let country = s.country.unwrap_or_else(|| "Global".to_string());
                            let tags = s.tags.unwrap_or_default();

                            // STRICT ANTI-STATE / REACTIONARY BLACKLIST
                            if is_banned_station(&name, &url, &tags) {
                                return None;
                            }

                            let br = s.bitrate.unwrap_or(128);
                            Some(MediaItem {
                                id: format!("rb_{}", url),
                                title: name,
                                artist: format!("{} [{}kbps]", country, br),
                                album: Some(tags),
                                url,
                                duration: 0.0,
                                format: Some("MP3".to_string()),
                                bitrate: Some(br),
                                is_radio: true,
                                is_youtube: false,
                                is_favorite: false,
                                file_size: None,
                                track_no: None,
                                sample_rate: Some(44100),
                                bit_depth: Some(16),
                            })
                        })
                        .collect();

                    // Sort by bitrate / quality descending
                    items.sort_by(|a, b| b.bitrate.unwrap_or(0).cmp(&a.bitrate.unwrap_or(0)));

                    if !items.is_empty() {
                        return Ok(deduplicate_stations(items));
                    }
                }
            }
        }
    }

    Ok(Vec::new())
}
