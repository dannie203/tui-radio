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
    pub automix_mode: AutomixMode,
    pub crossfade_enabled: bool,
    pub crossfade_duration: u32,
    pub crossfade_curve: CrossfadeCurve,
    pub ai_dj_smart_cues: bool,
    pub neural_engine: crate::audio::neural::NeuralEngine,
    pub neural_scanning: bool,
    pub neural_profile_count: usize,
    pub settings_selected_idx: usize,
    pub theme_index: usize,
    pub telemetry: AudioTelemetry,
    pub played_history: std::collections::VecDeque<String>,
}

impl AppState {
    pub fn new() -> Self {
        let cfg = AppConfig::load_or_create();
        let music_dir = cfg.resolved_music_dir();
        let (local_tracks, local_albums) = scan_local_library(Some(&music_dir));
        let radio = get_curated_stations();
        let mixtapes = load_mixtapes();
        let neural_engine = crate::audio::neural::NeuralEngine::new();

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
            automix_mode: cfg.get_automix_mode(),
            crossfade_enabled: cfg.get_automix_mode().is_enabled(),
            crossfade_duration: cfg.audio.crossfade_duration,
            crossfade_curve: cfg.get_crossfade_curve(),
            ai_dj_smart_cues: cfg.audio.ai_dj_smart_cues,
            neural_profile_count: neural_engine.profile_count(),
            neural_engine,
            neural_scanning: false,
            settings_selected_idx: 0,
            theme_index,
            telemetry: AudioTelemetry::default(),
            played_history: std::collections::VecDeque::new(),
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
        const SETTINGS_COUNT: usize = 14;
        let new_idx = (self.settings_selected_idx as i32 + delta).rem_euclid(SETTINGS_COUNT as i32);
        self.settings_selected_idx = new_idx as usize;
    }

    pub fn cycle_selected_setting(&mut self, delta: i32) {
        match self.settings_selected_idx {
            0 => self.cycle_theme(),
            1 => self.stereo_mode = self.stereo_mode.cycle(),
            2 => self.dolby_mode = self.dolby_mode.cycle(),
            3 => self.tape_type = self.tape_type.cycle(),
            4 => self.eq_preset = self.eq_preset.cycle(),
            5 => self.bass_boost = !self.bass_boost,
            6 => self.spectrum_color_mode = self.spectrum_color_mode.cycle(),
            7 => self.visualizer_speed = self.visualizer_speed.cycle(),
            8 => self.record_format = self.record_format.cycle(),
            9 => self.lyrics_offset = (self.lyrics_offset + (delta as f64 * 0.25)).clamp(-10.0, 10.0),
            10 => self.matrix_scramble = !self.matrix_scramble,
            11 => {
                self.volume_step = match self.volume_step {
                    1 => 2,
                    2 => 5,
                    5 => 10,
                    _ => 1,
                };
            }
            12 => {
                self.automix_mode = self.automix_mode.cycle();
                self.crossfade_enabled = self.automix_mode.is_enabled();
            }
            13 => {
                self.crossfade_duration = match self.crossfade_duration {
                    3 => 5,
                    5 => 6,
                    6 => 8,
                    8 => 10,
                    10 => 12,
                    12 => 16,
                    _ => 3,
                };
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

        if self.shuffle {
            return self.pick_dj_next();
        }

        if !self.queue.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = self.queue.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(0);
            if cur_idx + 1 < self.queue.len() {
                self.selected_index = cur_idx + 1;
                let track = self.queue.get(cur_idx + 1).cloned();
                if let Some(ref t) = track {
                    self.record_history(&t.id);
                }
                return track;
            } else if self.repeat_mode == RepeatMode::All {
                self.selected_index = 0;
                let track = self.queue.first().cloned();
                if let Some(ref t) = track {
                    self.record_history(&t.id);
                }
                return track;
            }
            return None;
        }

        let list = self.get_active_list();
        if !list.is_empty() {
            let cur_id = self.current_track.as_ref().map(|t| &t.id);
            let cur_idx = list.iter().position(|t| Some(&t.id) == cur_id).unwrap_or(self.selected_index);
            if cur_idx + 1 < list.len() {
                self.selected_index = cur_idx + 1;
                let track = list.get(cur_idx + 1).cloned();
                if let Some(ref t) = track {
                    self.record_history(&t.id);
                }
                return track;
            } else if self.repeat_mode == RepeatMode::All {
                self.selected_index = 0;
                let track = list.first().cloned();
                if let Some(ref t) = track {
                    self.record_history(&t.id);
                }
                return track;
            }
        }
        None
    }

    pub fn record_history(&mut self, track_id: &str) {
        if self.played_history.iter().any(|id| id == track_id) {
            return;
        }
        self.played_history.push_back(track_id.to_string());
        if self.played_history.len() > 60 {
            self.played_history.pop_front();
        }
    }

    /// AI-DJ Smart Harmonic Shuffle:
    /// Evaluates unplayed candidate tracks for harmonic key compatibility, BPM matching, and energy.
    /// Samples randomly among the Top-K best musical matches so songs are NEVER looped back and forth.
    pub fn pick_dj_next(&mut self) -> Option<MediaItem> {
        let pool: Vec<MediaItem> = if !self.queue.is_empty() {
            self.queue.clone()
        } else if self.mode == AppMode::LocalTracks {
            if self.local_view_level == LocalViewLevel::Tracks && !self.filtered_local.is_empty() {
                self.filtered_local.clone()
            } else {
                self.local_tracks.clone()
            }
        } else {
            self.get_active_list()
        };
        if pool.is_empty() {
            return None;
        }

        if pool.len() == 1 {
            return pool.first().cloned();
        }

        let cur_id = self.current_track.as_ref().map(|t| t.id.clone());
        if let Some(ref id) = cur_id {
            self.record_history(id);
        }

        let cur_prof = self.current_track.as_ref().and_then(|t| self.neural_engine.get_profile(&t.url));
        let cur_key = cur_prof.as_ref().map(|p| p.camelot_key.clone()).unwrap_or_default();
        let cur_bpm = cur_prof.as_ref().map(|p| p.bpm).unwrap_or(0.0);

        // Filter out currently playing track and recently played tracks
        let mut candidates: Vec<(usize, MediaItem)> = pool
            .iter()
            .enumerate()
            .filter(|(_, t)| Some(&t.id) != cur_id.as_ref() && !self.played_history.contains(&t.id))
            .map(|(idx, t)| (idx, t.clone()))
            .collect();

        // If all tracks in pool have been played, reset history and use all other tracks
        if candidates.is_empty() {
            self.played_history.clear();
            if let Some(ref id) = cur_id {
                self.played_history.push_back(id.clone());
            }
            candidates = pool
                .iter()
                .enumerate()
                .filter(|(_, t)| Some(&t.id) != cur_id.as_ref())
                .map(|(idx, t)| (idx, t.clone()))
                .collect();
        }

        if candidates.is_empty() {
            return pool.first().cloned();
        }

        // Score candidates with Neural Compatibility
        let mut scored: Vec<(f64, usize, MediaItem)> = candidates
            .into_iter()
            .map(|(idx, track)| {
                let p = self.neural_engine.get_profile(&track.url);
                let key = p.as_ref().map(|x| x.camelot_key.as_str()).unwrap_or("");
                let bpm = p.as_ref().map(|x| x.bpm).unwrap_or(0.0);

                let mut score: f64 = 1.0;

                // 1. Harmonic Key Compatibility (+4.0 for harmonic match)
                if !cur_key.is_empty() && !key.is_empty() {
                    if crate::audio::neural::NeuralEngine::is_harmonic_match(&cur_key, key) {
                        score += 4.0;
                    } else {
                        score -= 1.0;
                    }
                }

                // 2. BPM Proximity (+3.0 for close tempo within +-10%)
                if cur_bpm > 30.0 && bpm > 30.0 {
                    let ratio = (bpm / cur_bpm).max(cur_bpm / bpm);
                    if ratio <= 1.08 {
                        score += 3.0;
                    } else if ratio <= 1.15 {
                        score += 1.5;
                    }
                }

                if p.is_some() {
                    score += 0.5;
                }

                (score, idx, track)
            })
            .collect();

        // Sort descending by score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Sample with random selection among Top-K (up to 8 candidates)
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let top_k = scored.len().min(8);
        let pick_idx = rng.gen_range(0..top_k);
        let chosen = &scored[pick_idx];

        self.record_history(&chosen.2.id);
        self.selected_index = chosen.1;
        Some(chosen.2.clone())
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

    pub fn cycle_theme(&mut self) {
        let themes = get_themes();
        self.theme_index = (self.theme_index + 1) % themes.len();
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
}
