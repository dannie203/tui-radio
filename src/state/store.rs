use crate::api::stations::get_curated_stations;
use crate::audio::library::scan_local_library;
use crate::audio::mixtape::{load_mixtapes, save_mixtapes, Mixtape};
use crate::state::config::AppConfig;
use crate::state::types::*;
use crate::ui::theme::{get_themes, Theme};

pub struct AppState {
    pub mode: AppMode,
    pub local_view_level: LocalViewLevel,
    pub active_view: ActiveView,
    pub active_modal: ModalType,
    pub input_buffer: String,
    pub local_tracks: Vec<MediaItem>,
    pub filtered_local: Vec<MediaItem>,
    pub local_albums: Vec<AlbumGroup>,
    pub filtered_albums: Vec<AlbumGroup>,
    pub selected_album_idx: Option<usize>,
    pub radio_stations: Vec<MediaItem>,
    pub filtered_radio: Vec<MediaItem>,
    pub genre_filter: GenreFilter,
    pub queue: Vec<MediaItem>,
    pub youtube_results: Vec<MediaItem>,
    pub favorites: Vec<MediaItem>,
    pub selected_index: usize,
    pub current_track: Option<MediaItem>,
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: u32,
    pub status_message: String,
    pub search_query: String,
    pub stereo_mode: StereoMode,
    pub dolby_mode: DolbyMode,
    pub tape_type: TapeType,
    pub bass_boost: bool,
    pub eq_preset: EqPreset,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub record_format: RecordFormat,
    pub is_recording: bool,
    pub recording_status: String,
    pub mixtapes: Vec<Mixtape>,
    pub selected_mixtape_idx: usize,
    pub lyrics: Vec<SyncedLyricLine>,
    pub lyrics_loading: bool,
    pub lyrics_offset: f64,
    pub current_artwork: Option<crate::api::artwork::ArtworkHalfblocks>,
    pub artwork_loading: bool,
    pub matrix_scramble: bool,
    pub visualizer_speed: VisualizerSpeed,
    pub spectrum_color_mode: SpectrumColorMode,
    pub spectrum_custom_color: Option<String>,
    pub volume_step: u32,
    pub notifications_enabled: bool,
    pub settings_selected_idx: usize,
    pub theme_index: usize,
    pub telemetry: AudioTelemetry,
}

impl AppState {
    pub fn new() -> Self {
        let cfg = AppConfig::load_or_create();
        let music_dir = cfg.resolved_music_dir();
        let (local_tracks, local_albums) = scan_local_library(Some(&music_dir));
        let radio = get_curated_stations();
        let mixtapes = load_mixtapes();

        let themes = get_themes();
        let theme_index = themes
            .iter()
            .position(|t| t.id.eq_ignore_ascii_case(&cfg.ui.theme))
            .unwrap_or(0);

        Self {
            mode: cfg.get_app_mode(),
            local_view_level: LocalViewLevel::Albums,
            active_view: ActiveView::Deck,
            active_modal: ModalType::None,
            input_buffer: String::new(),
            filtered_local: local_tracks.clone(),
            local_tracks,
            filtered_albums: local_albums.clone(),
            local_albums,
            selected_album_idx: None,
            filtered_radio: radio.clone(),
            radio_stations: radio,
            genre_filter: GenreFilter::All,
            queue: Vec::new(),
            youtube_results: Vec::new(),
            favorites: Vec::new(),
            selected_index: 0,
            current_track: None,
            is_playing: false,
            is_paused: false,
            volume: cfg.audio.default_volume.min(100),
            status_message: "READY — Select an Album or Track to Play".to_string(),
            search_query: String::new(),
            stereo_mode: cfg.get_stereo_mode(),
            dolby_mode: cfg.get_dolby_mode(),
            tape_type: cfg.get_tape_type(),
            bass_boost: cfg.audio.bass_boost,
            eq_preset: cfg.get_eq_preset(),
            repeat_mode: RepeatMode::Off,
            shuffle: false,
            record_format: cfg.get_record_format(),
            is_recording: false,
            recording_status: "IDLE".to_string(),
            mixtapes,
            selected_mixtape_idx: 0,
            lyrics: Vec::new(),
            lyrics_loading: false,
            lyrics_offset: cfg.ui.lyrics_offset,
            current_artwork: None,
            artwork_loading: false,
            matrix_scramble: cfg.ui.matrix_scramble,
            visualizer_speed: cfg.get_visualizer_speed(),
            spectrum_color_mode: cfg.get_spectrum_color_mode(),
            spectrum_custom_color: cfg.ui.spectrum_custom_color.clone(),
            volume_step: cfg.general.volume_step.max(1).min(20),
            notifications_enabled: cfg.general.notifications,
            settings_selected_idx: 0,
            theme_index,
            telemetry: AudioTelemetry::default(),
        }
    }

    pub fn get_active_list_len(&self) -> usize {
        match self.mode {
            AppMode::LocalTracks => match self.local_view_level {
                LocalViewLevel::Albums => self.filtered_albums.len(),
                LocalViewLevel::Tracks | LocalViewLevel::AllTracks => self.filtered_local.len(),
            },
            AppMode::RadioStations => self.filtered_radio.len(),
            AppMode::Queue => self.queue.len(),
            AppMode::YoutubeMusic => self.youtube_results.len(),
        }
    }

    pub fn get_active_list(&self) -> Vec<MediaItem> {
        match self.mode {
            AppMode::LocalTracks => {
                if self.local_view_level == LocalViewLevel::Tracks && !self.filtered_local.is_empty() {
                    self.filtered_local.clone()
                } else if self.local_view_level == LocalViewLevel::AllTracks && !self.filtered_local.is_empty() {
                    self.filtered_local.clone()
                } else {
                    self.local_tracks.clone()
                }
            }
            AppMode::RadioStations => self.filtered_radio.clone(),
            AppMode::Queue => self.queue.clone(),
            AppMode::YoutubeMusic => self.youtube_results.clone(),
        }
    }

    pub fn get_selected_item(&self) -> Option<&MediaItem> {
        match self.mode {
            AppMode::LocalTracks => match self.local_view_level {
                LocalViewLevel::Albums => {
                    let album = self.filtered_albums.get(self.selected_index)?;
                    album.tracks.first()
                }
                LocalViewLevel::Tracks | LocalViewLevel::AllTracks => self.filtered_local.get(self.selected_index),
            },
            AppMode::RadioStations => self.filtered_radio.get(self.selected_index),
            AppMode::Queue => self.queue.get(self.selected_index),
            AppMode::YoutubeMusic => self.youtube_results.get(self.selected_index),
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let count = self.get_active_list_len();
        if count == 0 {
            self.selected_index = 0;
            return;
        }
        let new_idx = (self.selected_index as i32 + delta).rem_euclid(count as i32);
        self.selected_index = new_idx as usize;
    }

    pub fn move_settings_selection(&mut self, delta: i32) {
        const SETTINGS_COUNT: usize = 12;
        let new_idx = (self.settings_selected_idx as i32 + delta).rem_euclid(SETTINGS_COUNT as i32);
        self.settings_selected_idx = new_idx as usize;
    }

    pub fn cycle_selected_setting(&mut self, delta: i32) {
        match self.settings_selected_idx {
            0 => self.cycle_theme(delta),
            1 => self.stereo_mode = self.stereo_mode.cycle_dir(delta),
            2 => self.dolby_mode = self.dolby_mode.cycle_dir(delta),
            3 => self.tape_type = self.tape_type.cycle_dir(delta),
            4 => self.eq_preset = self.eq_preset.cycle_dir(delta),
            5 => self.bass_boost = !self.bass_boost,
            6 => self.spectrum_color_mode = self.spectrum_color_mode.cycle_dir(delta),
            7 => self.visualizer_speed = self.visualizer_speed.cycle_dir(delta),
            8 => self.record_format = self.record_format.cycle_dir(delta),
            9 => self.lyrics_offset = (self.lyrics_offset + (delta as f64 * 0.25)).clamp(-10.0, 10.0),
            10 => self.matrix_scramble = !self.matrix_scramble,
            11 => {
                const STEPS: [u32; 4] = [1, 2, 5, 10];
                let idx = STEPS.iter().position(|&s| s == self.volume_step).unwrap_or(2);
                let next = (idx as i32 + delta).rem_euclid(STEPS.len() as i32) as usize;
                self.volume_step = STEPS[next];
            }
            _ => {}
        }
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
        self.selected_index = 0;
        self.status_message = format!("Switched to {}", mode.title());
    }

    pub fn cycle_mode(&mut self, delta: i32) {
        let idx = (self.mode.index() as i32 + delta).rem_euclid(4) as usize;
        self.set_mode(AppMode::from_index(idx));
    }

    pub fn cycle_genre(&mut self, delta: i32) {
        self.genre_filter = self.genre_filter.cycle(delta);
        self.status_message = format!("Radio Genre: {}", self.genre_filter.label());
        let q = self.search_query.clone();
        self.filter(&q);
    }

    pub fn drill_down(&mut self) -> Option<MediaItem> {
        if self.mode == AppMode::LocalTracks && self.local_view_level == LocalViewLevel::Albums {
            if let Some(album) = self.filtered_albums.get(self.selected_index).cloned() {
                self.selected_album_idx = Some(self.selected_index);
                self.filtered_local = album.tracks.clone();
                self.local_view_level = LocalViewLevel::Tracks;
                self.selected_index = 0;
                self.status_message = format!("Opened Album: {} ({} tracks)", album.name, album.tracks.len());
                return None;
            }
        }
        self.get_selected_item().cloned()
    }

    pub fn drill_up(&mut self) -> bool {
        if self.mode == AppMode::LocalTracks && self.local_view_level == LocalViewLevel::Tracks {
            self.local_view_level = LocalViewLevel::Albums;
            self.filtered_local = self.local_tracks.clone();
            self.selected_index = self.selected_album_idx.unwrap_or(0);
            self.status_message = "Returned to Albums list".to_string();
            return true;
        }
        false
    }

    pub fn toggle_local_view(&mut self) {
        if self.mode == AppMode::LocalTracks {
            if self.local_view_level == LocalViewLevel::Albums {
                self.local_view_level = LocalViewLevel::AllTracks;
                self.filtered_local = self.local_tracks.clone();
                self.selected_index = 0;
                self.status_message = "View: All Tracks (Flat List)".to_string();
            } else {
                self.local_view_level = LocalViewLevel::Albums;
                self.filtered_albums = self.local_albums.clone();
                self.selected_index = 0;
                self.status_message = "View: Albums & Crates".to_string();
            }
        }
    }

    pub fn add_to_queue(&mut self) {
        if self.mode == AppMode::LocalTracks && self.local_view_level == LocalViewLevel::Albums {
            if let Some(album) = self.filtered_albums.get(self.selected_index) {
                let count = album.tracks.len();
                self.queue.extend(album.tracks.clone());
                self.status_message = format!("Queued Album '{}' ({} tracks)", album.name, count);
                return;
            }
        }

        if let Some(item) = self.get_selected_item().cloned() {
            self.queue.push(item.clone());
            self.status_message = format!("Added '{}' to Queue ({})", item.title, self.queue.len());
        }
    }

    pub fn add_item_to_queue(&mut self, item: MediaItem) {
        let title = item.title.clone();
        self.queue.push(item);
        self.status_message = format!("Added '{}' to Queue ({})", title, self.queue.len());
    }

    pub fn remove_from_queue(&mut self) {
        if self.mode == AppMode::Queue && !self.queue.is_empty() && self.selected_index < self.queue.len() {
            let removed = self.queue.remove(self.selected_index);
            if self.selected_index >= self.queue.len() && !self.queue.is_empty() {
                self.selected_index = self.queue.len() - 1;
            }
            self.status_message = format!("Removed '{}' from Queue", removed.title);
        }
    }

    pub fn clear_queue(&mut self) {
        self.queue.clear();
        self.selected_index = 0;
        self.status_message = "Playback Queue Cleared".to_string();
    }

    pub fn toggle_favorite(&mut self) {
        let target = self.current_track.as_ref().or_else(|| self.get_selected_item());
        if let Some(item) = target.cloned() {
            if let Some(pos) = self.favorites.iter().position(|f| f.id == item.id) {
                self.favorites.remove(pos);
                self.status_message = format!("Removed '{}' from Favorites", item.title);
            } else {
                let mut fav = item.clone();
                fav.is_favorite = true;
                self.favorites.push(fav);
                self.status_message = format!("Saved '{}' to Favorites ★", item.title);
            }
        }
    }

    pub fn get_next_track(&mut self) -> Option<MediaItem> {
        if self.repeat_mode == RepeatMode::One && self.current_track.is_some() {
            return self.current_track.clone();
        }

        let pool = if !self.queue.is_empty() {
            self.queue.clone()
        } else {
            self.get_active_list()
        };

        if pool.is_empty() {
            return None;
        }

        if self.shuffle {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let candidates: Vec<(usize, MediaItem)> = pool
                .into_iter()
                .enumerate()
                .filter(|(_, t)| Some(&t.id) != cur_id)
                .collect();
            if candidates.is_empty() {
                return self.current_track.clone();
            }
            let pick = rng.gen_range(0..candidates.len());
            let (idx, item) = &candidates[pick];
            self.selected_index = *idx;
            return Some(item.clone());
        }

        if !self.queue.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = self.queue.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(0);
            if cur_idx + 1 < self.queue.len() {
                self.selected_index = cur_idx + 1;
                return self.queue.get(cur_idx + 1).cloned();
            } else if self.repeat_mode == RepeatMode::All {
                self.selected_index = 0;
                return self.queue.first().cloned();
            }
            return None;
        }

        let list = self.get_active_list();
        if !list.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = list.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(self.selected_index);
            if cur_idx + 1 < list.len() {
                self.selected_index = cur_idx + 1;
                return list.get(cur_idx + 1).cloned();
            } else if self.repeat_mode == RepeatMode::All {
                self.selected_index = 0;
                return list.first().cloned();
            }
        }
        None
    }

    pub fn get_prev_track(&mut self) -> Option<MediaItem> {
        if !self.queue.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = self.queue.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(0);
            if cur_idx > 0 {
                self.selected_index = cur_idx - 1;
                return self.queue.get(cur_idx - 1).cloned();
            }
            return self.queue.first().cloned();
        }

        let list = self.get_active_list();
        if !list.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = list.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(0);
            if cur_idx > 0 {
                self.selected_index = cur_idx - 1;
                return list.get(cur_idx - 1).cloned();
            }
            return list.first().cloned();
        }
        None
    }

    pub fn filter(&mut self, query: &str) {
        self.search_query = query.to_string();
        let q = query.trim().to_lowercase();

        // Filter local
        if q.is_empty() {
            self.filtered_local = self.local_tracks.clone();
            self.filtered_albums = self.local_albums.clone();
        } else {
            self.filtered_local = self
                .local_tracks
                .iter()
                .filter(|t| t.title.to_lowercase().contains(&q) || t.artist.to_lowercase().contains(&q))
                .cloned()
                .collect();
            self.filtered_albums = self
                .local_albums
                .iter()
                .filter(|a| a.name.to_lowercase().contains(&q) || a.artist.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }

        // Filter radio by genre + query
        let mut stations: Vec<MediaItem> = match self.genre_filter {
            GenreFilter::All => self.radio_stations.clone(),
            GenreFilter::Favorites => {
                let fav_ids: std::collections::HashSet<_> = self.favorites.iter().map(|f| &f.id).collect();
                self.radio_stations.iter().filter(|s| fav_ids.contains(&s.id)).cloned().collect()
            }
            GenreFilter::LoFi => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("lofi") || txt.contains("lo-fi") || txt.contains("chill") || txt.contains("beat")
            }).cloned().collect(),
            GenreFilter::Synthwave => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("synth") || txt.contains("retro") || txt.contains("wave") || txt.contains("cyber")
            }).cloned().collect(),
            GenreFilter::Jazz => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("jazz") || txt.contains("blues") || txt.contains("smooth") || txt.contains("bossa")
            }).cloned().collect(),
            GenreFilter::HipHop => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("hip") || txt.contains("hop") || txt.contains("rap") || txt.contains("boom")
            }).cloned().collect(),
            GenreFilter::Rock => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("rock") || txt.contains("metal") || txt.contains("punk") || txt.contains("indie")
            }).cloned().collect(),
            GenreFilter::Electronic => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("electro") || txt.contains("edm") || txt.contains("house") || txt.contains("dance")
            }).cloned().collect(),
            GenreFilter::Classical => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("classic") || txt.contains("orchestra") || txt.contains("piano") || txt.contains("ambient")
            }).cloned().collect(),
            GenreFilter::Pop => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("pop") || txt.contains("hit") || txt.contains("top40")
            }).cloned().collect(),
            GenreFilter::Vietnam => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("vietnam") || txt.contains("viet nam") || txt.contains("vov") || txt.contains("voh")
            }).cloned().collect(),
            GenreFilter::Japan => self.radio_stations.iter().filter(|s| {
                let txt = format!("{} {}", s.title, s.artist).to_lowercase();
                txt.contains("japan") || txt.contains("anime") || txt.contains("jpop") || txt.contains("tokyo")
            }).cloned().collect(),
            GenreFilter::GlobalTop => self.radio_stations.iter().filter(|s| {
                s.bitrate.unwrap_or(0) >= 192 || s.is_favorite
            }).cloned().collect(),
        };

        if !q.is_empty() {
            stations.retain(|s| {
                s.title.to_lowercase().contains(&q)
                    || s.artist.to_lowercase().contains(&q)
                    || s.album.as_deref().unwrap_or("").to_lowercase().contains(&q)
            });
        }

        self.filtered_radio = stations;
        self.selected_index = 0;
    }

    pub fn cycle_theme(&mut self, delta: i32) {
        let themes = get_themes();
        let count = themes.len() as i32;
        self.theme_index = (self.theme_index as i32 + delta).rem_euclid(count) as usize;
        self.status_message = format!("Theme: {}", themes[self.theme_index].name);
    }

    pub fn current_theme(&self) -> Theme {
        let themes = get_themes();
        themes[self.theme_index % themes.len()].clone()
    }

    pub fn cycle_record_format(&mut self) {
        self.record_format = self.record_format.cycle();
        self.status_message = format!("Rec Format: {}", self.record_format.label());
    }

    pub fn selected_mixtape(&self) -> Option<&Mixtape> {
        self.mixtapes.get(self.selected_mixtape_idx)
    }

    pub fn create_mixtape(&mut self) {
        let name = format!("Mixtape #{}", self.mixtapes.len() + 1);
        let id = format!("mixtape:{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let mt = Mixtape {
            id,
            name: name.clone(),
            created_at: "".to_string(),
            tracks: Vec::new(),
        };
        self.mixtapes.push(mt);
        self.selected_mixtape_idx = self.mixtapes.len() - 1;
        save_mixtapes(&self.mixtapes);
        self.status_message = format!("Created Mixtape: {}", name);
    }

    pub fn delete_mixtape(&mut self) {
        if self.mixtapes.len() > 1 && self.selected_mixtape_idx < self.mixtapes.len() {
            let removed = self.mixtapes.remove(self.selected_mixtape_idx);
            if self.selected_mixtape_idx >= self.mixtapes.len() {
                self.selected_mixtape_idx = self.mixtapes.len().saturating_sub(1);
            }
            save_mixtapes(&self.mixtapes);
            self.status_message = format!("Deleted Mixtape: {}", removed.name);
        }
    }

    pub fn add_current_track_to_mixtape(&mut self) {
        if let Some(track) = self.current_track.clone().or_else(|| self.get_selected_item().cloned()) {
            if let Some(mt) = self.mixtapes.get_mut(self.selected_mixtape_idx) {
                let name = mt.name.clone();
                if !mt.tracks.iter().any(|t| t.id == track.id) {
                    mt.tracks.push(track.clone());
                    save_mixtapes(&self.mixtapes);
                    self.status_message = format!("Added '{}' to Mixtape '{}'", track.title, name);
                } else {
                    self.status_message = format!("'{}' already in Mixtape '{}'", track.title, name);
                }
            }
        }
    }

    pub fn remove_selected_mixtape_track(&mut self) {
        if let Some(mt) = self.mixtapes.get_mut(self.selected_mixtape_idx) {
            if !mt.tracks.is_empty() {
                let removed = mt.tracks.pop();
                save_mixtapes(&self.mixtapes);
                if let Some(t) = removed {
                    self.status_message = format!("Removed '{}' from Mixtape", t.title);
                }
            }
        }
    }

    pub fn save_config(&self) {
        let theme_id = self.current_theme().id;
        let cfg = AppConfig {
            general: crate::state::config::GeneralConfig {
                music_dir: Some("~/Music".to_string()),
                default_mode: Some(match self.mode {
                    AppMode::RadioStations => "radio".to_string(),
                    AppMode::Queue => "queue".to_string(),
                    AppMode::YoutubeMusic => "youtube".to_string(),
                    AppMode::LocalTracks => "local".to_string(),
                }),
                volume_step: self.volume_step,
                notifications: self.notifications_enabled,
                auto_save_session: true,
            },
            audio: crate::state::config::AudioConfig {
                default_volume: self.volume,
                stereo_mode: match self.stereo_mode {
                    StereoMode::Mono => "mono".to_string(),
                    StereoMode::Wide3D => "wide3d".to_string(),
                    StereoMode::Stereo => "stereo".to_string(),
                },
                dolby_mode: match self.dolby_mode {
                    DolbyMode::Off => "off".to_string(),
                    DolbyMode::DolbyB => "dolby_b".to_string(),
                    DolbyMode::DolbyC => "dolby_c".to_string(),
                    DolbyMode::DolbyS => "dolby_s".to_string(),
                },
                tape_type: match self.tape_type {
                    TapeType::TypeI => "type_i".to_string(),
                    TapeType::TypeII => "type_ii".to_string(),
                    TapeType::TypeIV => "type_iv".to_string(),
                },
                eq_preset: match self.eq_preset {
                    EqPreset::Flat => "flat".to_string(),
                    EqPreset::MegaBass => "megabass".to_string(),
                    EqPreset::VocalClear => "vocal".to_string(),
                    EqPreset::RockPunch => "rock".to_string(),
                    EqPreset::LofiWarmth => "lofi".to_string(),
                    EqPreset::CyberSynth => "cyberpunk".to_string(),
                    EqPreset::ClubEdm => "club".to_string(),
                },
                bass_boost: self.bass_boost,
                record_format: match self.record_format {
                    RecordFormat::Opus => "opus".to_string(),
                    RecordFormat::Mp3 => "mp3".to_string(),
                    RecordFormat::Flac => "flac".to_string(),
                    RecordFormat::M4a => "m4a".to_string(),
                },
            },
            ui: crate::state::config::UiConfig {
                theme: theme_id.to_string(),
                visualizer_speed: match self.visualizer_speed {
                    VisualizerSpeed::UltraSnappy => "snappy".to_string(),
                    VisualizerSpeed::Standard => "standard".to_string(),
                    VisualizerSpeed::SmoothLiquid => "liquid".to_string(),
                },
                spectrum_color_mode: match self.spectrum_color_mode {
                    SpectrumColorMode::RgbCycle => "rgb_cycle".to_string(),
                    SpectrumColorMode::ChromaRainbow => "static_iso".to_string(),
                    SpectrumColorMode::VerticalGradient => "gradient".to_string(),
                    SpectrumColorMode::CyberpunkNeon => "cyberpunk".to_string(),
                    SpectrumColorMode::FireAndIce => "fire_ice".to_string(),
                    SpectrumColorMode::MatrixPhosphor => "matrix".to_string(),
                    SpectrumColorMode::AmberVintage => "amber".to_string(),
                    SpectrumColorMode::ThemeAccent => "theme".to_string(),
                },
                spectrum_custom_color: self.spectrum_custom_color.clone(),
                matrix_scramble: self.matrix_scramble,
                lyrics_offset: self.lyrics_offset,
            },
        };
        let _ = cfg.save();
    }
}
