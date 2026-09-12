use crate::state::types::MediaItem;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mixtape {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub tracks: Vec<MediaItem>,
}

fn config_dir() -> PathBuf {
    crate::state::config::get_config_dir()
}

fn legacy_mixtapes_file() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("boombox-tui")
        .join("mixtapes.json")
}

fn mixtapes_file() -> PathBuf {
    config_dir().join("mixtapes.json")
}

pub fn load_mixtapes() -> Vec<Mixtape> {
    let path = mixtapes_file();
    let legacy = legacy_mixtapes_file();
    let target = if path.exists() {
        path
    } else if legacy.exists() {
        legacy
    } else {
        path
    };

    if target.exists() {
        if let Ok(content) = fs::read_to_string(&target) {
            if let Ok(list) = serde_json::from_str::<Vec<Mixtape>>(&content) {
                if !list.is_empty() {
                    return list;
                }
            }
        }
    }
    vec![Mixtape {
        id: "mixtape:favorites".to_string(),
        name: "★ Favorites Mixtape".to_string(),
        created_at: now_iso(),
        tracks: Vec::new(),
    }]
}

pub fn save_mixtapes(mixtapes: &[Mixtape]) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    if let Ok(json) = serde_json::to_string_pretty(mixtapes) {
        let _ = fs::write(mixtapes_file(), json);
    }
}

fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}
