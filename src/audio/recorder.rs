use crate::state::types::RecordFormat;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct RecordJob {
    pub url: String,
    pub title: String,
    pub artist: String,
    pub format: RecordFormat,
    pub output_path: Option<PathBuf>,
}

struct ActiveJob {
    url: String,
    job: RecordJob,
    child: tokio::process::Child,
    cancelled: bool,
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\\' | '#' => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "Track".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn recordings_dir() -> PathBuf {
    dirs::audio_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("Boombox Recordings")
}

pub fn send_notification(title: &str, message: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "BOOMBOX RX-505", "-i", "boombox", title, message])
        .spawn();
}

pub struct StreamRecorder {
    jobs: Arc<Mutex<HashMap<String, ActiveJob>>>,
    is_recording: Arc<AtomicBool>,
}

impl StreamRecorder {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            is_recording: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }

    pub fn active_count(&self) -> usize {
        self.jobs.lock().unwrap().len()
    }

    pub fn current_jobs(&self) -> Vec<RecordJob> {
        self.jobs.lock().unwrap().values().map(|a| a.job.clone()).collect()
    }

    pub fn poll(&self) -> Vec<(String, bool)> {
        let mut notifs = Vec::new();
        let mut jobs = self.jobs.lock().unwrap();
        let mut to_remove = Vec::new();
        for (url, active) in jobs.iter_mut() {
            let status = match active.child.try_wait() {
                Ok(Some(_)) => Some(true),
                Ok(None) => None,
                Err(_) => Some(true),
            };
            if let Some(code) = status {
                let ok = !active.cancelled && code;
                if active.cancelled {
                    if let Some(path) = &active.job.output_path {
                        let _ = std::fs::remove_file(path);
                    }
                }
                to_remove.push((url.clone(), active.job.clone(), ok, active.cancelled));
            }
        }
        for (url, job, ok, cancelled) in to_remove {
            jobs.remove(&url);
            if cancelled {
                continue;
            }
            let _ = &job;
            notifs.push((format!("{} - {}", job.artist, job.title), ok));
        }
        if jobs.is_empty() {
            self.is_recording.store(false, Ordering::Relaxed);
        }
        notifs
    }

    pub async fn record_track(
        &self,
        item: &crate::state::types::MediaItem,
        format: RecordFormat,
    ) -> Result<bool, String> {
        if item.url.is_empty() {
            return Err("No valid track stream URL to record".to_string());
        }

        if !item.is_radio && !item.is_youtube && !item.url.starts_with("http") {
            send_notification(
                "ℹ️ Local File Already Present",
                &format!("\"{}\" is already in your library. Original Hi-Res audio is 100% preserved.", item.title),
            );
            return Err("Track is already a local file — original audio is untouched".to_string());
        }

        let url = item.url.clone();
        let mut clean_url = item.url.trim().trim_start_matches("ytdl://").to_string();
        if clean_url.starts_with("watch?v=") {
            clean_url = format!("https://www.youtube.com/{}", clean_url);
        } else if clean_url.starts_with("music.youtube.com")
            || clean_url.starts_with("youtube.com")
            || clean_url.starts_with("youtu.be")
            || clean_url.starts_with("soundcloud.com")
            || clean_url.starts_with("bandcamp.com")
            || clean_url.starts_with("bandlab.com")
        {
            clean_url = format!("https://{}", clean_url);
        }
        let title = item.title.clone();
        let artist = item.artist.clone();

        {
            let mut jobs = self.jobs.lock().unwrap();
            let key = if jobs.contains_key(&url) {
                Some(url.clone())
            } else if jobs.contains_key(&clean_url) {
                Some(clean_url.clone())
            } else {
                None
            };
            if let Some(k) = key {
                if let Some(a) = jobs.get_mut(&k) {
                    a.cancelled = true;
                    let _ = a.child.kill().await;
                }
                jobs.remove(&k);
                send_notification("⏹️ Tape Recording Cancelled", "Recording was stopped and discarded.");
                if jobs.is_empty() {
                    self.is_recording.store(false, Ordering::Relaxed);
                }
                return Ok(false);
            }
        }

        send_notification(
            "🔴 Cassette Recording Started",
            &format!("Recording: \"{} - {}\" [{}] (Press 'R' again to cancel)", artist, title, format.label()),
        );

        let clean_artist = sanitize_filename(&artist);
        let clean_title = sanitize_filename(&title);

        let is_yt_source = !item.is_radio
            && (clean_url.contains("youtube.com")
                || clean_url.contains("youtu.be")
                || clean_url.contains("soundcloud.com")
                || clean_url.contains("bandcamp.com")
                || clean_url.contains("bandlab.com")
                || clean_url.contains("spotify:")
                || clean_url.starts_with("ytsearch:")
                || item.is_youtube);

        let dir = recordings_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return Err(format!("Could not create recordings dir: {}", e));
        }

        let mut cmd = if is_yt_source {
            let output_template = dir.join(format!("{} - {}.%(ext)s", clean_artist, clean_title));
            let mut c = Command::new("yt-dlp");
            c.args([
                "-x",
                "--audio-format",
                format.ext(),
                "--audio-quality",
                "0",
                "--no-playlist",
                "--embed-metadata",
                "-o",
                &output_template.to_string_lossy().to_string(),
                &clean_url,
            ]);
            c
        } else {
            let output_file = dir.join(format!("{} - {}.{}", clean_artist, clean_title, format.ext()));
            let mut c = Command::new("ffmpeg");
            c.arg("-y")
                .arg("-i")
                .arg(&clean_url)
                .arg("-t")
                .arg("240");
            match format {
                RecordFormat::Opus => {
                    c.args(["-c:a", "libopus", "-b:a", "160k"]);
                }
                RecordFormat::Flac => {
                    c.args(["-c:a", "flac"]);
                }
                RecordFormat::M4a => {
                    c.args(["-c:a", "aac", "-b:a", "256k"]);
                }
                RecordFormat::Mp3 => {
                    c.args(["-c:a", "libmp3lame", "-b:a", "320k"]);
                }
            }
            c.arg(&output_file.to_string_lossy().to_string());
            c
        };

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                send_notification("⚠️ Tape Recording Failed", "Could not start recorder tool (ffmpeg/yt-dlp).");
                return Err(format!("Could not spawn recorder: {}", e));
            }
        };

        let job = RecordJob {
            url: url.clone(),
            title,
            artist,
            format,
            output_path: if is_yt_source { None } else { Some(dir.join(format!("{} - {}.{}", clean_artist, clean_title, format.ext()))) },
        };

        self.jobs.lock().unwrap().insert(
            url.clone(),
            ActiveJob {
                url: url.clone(),
                job,
                child,
                cancelled: false,
            },
        );
        self.is_recording.store(true, Ordering::Relaxed);
        Ok(true)
    }

    pub async fn cancel(&self, url: Option<&str>) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(u) = url {
            if let Some(a) = jobs.get_mut(u) {
                a.cancelled = true;
                let _ = a.child.kill().await;
            }
            let removed = jobs.remove(u).is_some();
            if removed {
                send_notification("⏹️ Tape Recording Cancelled", "Recording was stopped and discarded.");
            }
            if jobs.is_empty() {
                self.is_recording.store(false, Ordering::Relaxed);
            }
            return removed;
        }

        if !jobs.is_empty() {
            for (_, a) in jobs.iter_mut() {
                a.cancelled = true;
                let _ = a.child.kill().await;
            }
            jobs.clear();
            self.is_recording.store(false, Ordering::Relaxed);
            send_notification("⏹️ Tape Recording Cancelled", "Recording was stopped and discarded.");
            return true;
        }
        false
    }
}
