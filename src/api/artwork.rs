use image::DynamicImage;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use ratatui::style::Color;
use std::path::Path;
use std::time::Duration;

pub type ArtworkHalfblocks = Vec<Vec<(Color, Color)>>;

pub async fn fetch_artwork(
    title: &str,
    artist: &str,
    source_url_or_path: Option<&str>,
    width: u32,
    height_chars: u32,
) -> Option<ArtworkHalfblocks> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
        .unwrap_or_default();

    if let Some(src) = source_url_or_path {
        // 1. Check if source is a YouTube / YouTube Music video or URL
        if let Some(yt_id) = extract_youtube_id(src) {
            if let Some(blocks) = fetch_youtube_thumbnail(&client, &yt_id, width, height_chars).await {
                return Some(blocks);
            }
        }

        // 2. Check if source is an online streaming URL (SoundCloud, Spotify, Bandcamp, etc.)
        if src.starts_with("http://") || src.starts_with("https://") {
            // Try extracting thumbnail URL via yt-dlp
            if let Ok(output) = tokio::process::Command::new("yt-dlp")
                .args(["--no-warnings", "--print", "thumbnail", src])
                .output()
                .await
            {
                if output.status.success() {
                    let thumb_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !thumb_url.is_empty() && thumb_url.starts_with("http") {
                        if let Ok(resp) = client.get(&thumb_url).header("User-Agent", "boombox-rs/3.2.0").send().await {
                            if resp.status().is_success() {
                                if let Ok(bytes) = resp.bytes().await {
                                    if let Ok(img) = image::load_from_memory(&bytes) {
                                        return Some(image_to_halfblocks(&img, width, height_chars));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // 3. Local Audio File (FLAC, MP3, M4A, OGG, WAV, etc.)
            let path = Path::new(src);
            if path.exists() {
                // Check embedded cover in file metadata
                if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
                    if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                        if let Some(picture) = tag.pictures().first() {
                            if let Ok(img) = image::load_from_memory(picture.data()) {
                                return Some(image_to_halfblocks(&img, width, height_chars));
                            }
                        }
                    }
                }

                // Check cover image files in directory
                if let Some(parent) = path.parent() {
                    for name in [
                        "cover.jpg", "cover.png", "cover.jpeg", "cover.webp",
                        "folder.jpg", "folder.png", "folder.jpeg",
                        "artwork.jpg", "artwork.png", "front.jpg", "front.png",
                        "Swag.jpg",
                    ] {
                        let img_path = parent.join(name);
                        if img_path.exists() {
                            if let Ok(img) = image::open(&img_path) {
                                return Some(image_to_halfblocks(&img, width, height_chars));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Query Apple iTunes Search API (Hi-Res 600x600 artwork)
    let clean_title = title.split('(').next().unwrap_or(title).split('[').next().unwrap_or(title).trim();
    let clean_artist = artist.split('&').next().unwrap_or(artist).split(',').next().unwrap_or(artist).trim();

    let search_queries = [
        format!("{}+{}", urlencoding::encode(clean_artist), urlencoding::encode(clean_title)),
        urlencoding::encode(clean_title).to_string(),
    ];

    for q in search_queries {
        let itunes_url = format!("https://itunes.apple.com/search?term={}&entity=song&limit=1", q);
        if let Ok(resp) = client.get(&itunes_url).header("User-Agent", "boombox-rs/3.2.0").send().await {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(results) = data["results"].as_array() {
                        if let Some(first) = results.first() {
                            if let Some(art_url) = first["artworkUrl100"].as_str() {
                                let hi_res_url = art_url.replace("100x100bb.jpg", "600x600bb.jpg");
                                if let Ok(img_resp) = client.get(&hi_res_url).send().await {
                                    if let Ok(bytes) = img_resp.bytes().await {
                                        if let Ok(img) = image::load_from_memory(&bytes) {
                                            return Some(image_to_halfblocks(&img, width, height_chars));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn extract_youtube_id(url: &str) -> Option<String> {
    if let Some(pos) = url.find("v=") {
        let after = &url[pos + 2..];
        let id: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = url.find("youtu.be/") {
        let after = &url[pos + 9..];
        let id: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    if let Some(pos) = url.find("embed/") {
        let after = &url[pos + 6..];
        let id: String = after.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() == 11 {
            return Some(id);
        }
    }
    None
}

async fn fetch_youtube_thumbnail(
    client: &reqwest::Client,
    yt_id: &str,
    width: u32,
    height_chars: u32,
) -> Option<ArtworkHalfblocks> {
    let urls = [
        format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", yt_id),
        format!("https://img.youtube.com/vi/{}/hqdefault.jpg", yt_id),
        format!("https://img.youtube.com/vi/{}/mqdefault.jpg", yt_id),
    ];

    for u in urls {
        if let Ok(resp) = client.get(&u).header("User-Agent", "boombox-rs/3.2.0").send().await {
            if resp.status().is_success() {
                if let Ok(bytes) = resp.bytes().await {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        return Some(image_to_halfblocks(&img, width, height_chars));
                    }
                }
            }
        }
    }
    None
}

pub fn image_to_halfblocks(
    img: &DynamicImage,
    target_width: u32,
    target_height_chars: u32,
) -> ArtworkHalfblocks {
    let target_height_px = target_height_chars * 2;
    let resized = img.resize_exact(target_width, target_height_px, image::imageops::FilterType::Triangle);
    let rgb_img = resized.to_rgb8();

    let mut rows = Vec::with_capacity(target_height_chars as usize);
    for y in 0..target_height_chars {
        let mut row = Vec::with_capacity(target_width as usize);
        let top_y = y * 2;
        let bot_y = y * 2 + 1;

        for x in 0..target_width {
            let top_p = rgb_img.get_pixel(x, top_y);
            let bot_p = if bot_y < target_height_px {
                rgb_img.get_pixel(x, bot_y)
            } else {
                top_p
            };

            let top_color = Color::Rgb(top_p[0], top_p[1], top_p[2]);
            let bot_color = Color::Rgb(bot_p[0], bot_p[1], bot_p[2]);
            row.push((top_color, bot_color));
        }
        rows.push(row);
    }
    rows
}
