#![allow(dead_code)]

mod api;
mod audio;
mod state;
mod ui;

use api::artwork::fetch_artwork;
use api::lyrics::fetch_lyrics;
use api::stations::fetch_radio_browser_genre;
use audio::capture::AudioCaptureEngine;
use audio::player::MpvPlayer;
use audio::recorder::StreamRecorder;
use audio::visualizer::VisualizerEngine;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use state::store::AppState;
use state::types::*;
use std::sync::{Arc, Mutex};
use std::{io, time::Duration};
use tokio::sync::mpsc;
use ui::layout::render_ui;
use ui::tray::{spawn_tray, TrayAction, TrayState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 0. Set process name & kernel-level flock for single-instance protection (Unix)
    #[cfg(unix)]
    {
        unsafe {
            libc::prctl(libc::PR_SET_NAME, b"boombox-rs\0".as_ptr(), 0, 0, 0);
        }
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;
        if let Ok(lock_file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open("/tmp/boombox-rs.lock")
        {
            let lock_res = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
            if lock_res != 0 {
                return Ok(());
            }
        }
    }

    // 1. Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::terminal::SetTitle("BOOMBOX RX-505"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Setup Unix Signals for Instant Hot-Reload (SIGUSR1 / SIGHUP)
    #[cfg(unix)]
    let mut sigusr1 = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1())?;
    #[cfg(unix)]
    let mut sighup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())?;

    // 3. Initialize App Engine & Async Channels
    let mut state = AppState::new();
    let player = MpvPlayer::new();
    let initial_af = audio::equalizer::build_mpv_af_string(
        state.eq_preset,
        state.bass_boost,
        state.dolby_mode,
        state.stereo_mode,
        state.tape_type,
    );
    player.apply_audio_filter(&initial_af);

    let recorder = StreamRecorder::new();
    let capture = AudioCaptureEngine::new();
    let mut visualizer = VisualizerEngine::new(1024);

    let (lyrics_tx, mut lyrics_rx) = mpsc::unbounded_channel::<(String, Vec<SyncedLyricLine>)>();
    let (artwork_tx, mut artwork_rx) = mpsc::unbounded_channel::<(String, Option<api::artwork::ArtworkHalfblocks>)>();
    let (radio_tx, mut radio_rx) = mpsc::unbounded_channel::<(GenreFilter, Vec<MediaItem>)>();
    let (tray_action_tx, mut tray_action_rx) = mpsc::unbounded_channel::<TrayAction>();

    // 4. Initialize SNI StatusNotifierItem Tray Icon with AppMenu
    let tray_state = Arc::new(Mutex::new(TrayState {
        title: "Standby".into(),
        artist: "Boombox RX-505".into(),
        volume: state.volume,
        is_playing: false,
        is_recording: false,
        action_tx: tray_action_tx.clone(),
    }));
    let _tray_handle = spawn_tray(Arc::clone(&tray_state)).await;

    let mut running = true;
    let mut should_hot_reload = false;
    let mut frame_count = 0usize;

    // Helper closure to dispatch lyrics query
    let dispatch_lyrics = |title: String, artist: String, path: Option<String>, track_id: String, tx: mpsc::UnboundedSender<(String, Vec<SyncedLyricLine>)>| {
        tokio::spawn(async move {
            let lines = fetch_lyrics(&title, &artist, path.as_deref()).await;
            let _ = tx.send((track_id, lines));
        });
    };

    // Helper closure to dispatch high-res cover artwork query
    let dispatch_artwork = |title: String, artist: String, path: Option<String>, track_id: String, tx: mpsc::UnboundedSender<(String, Option<api::artwork::ArtworkHalfblocks>)>| {
        tokio::spawn(async move {
            let art = fetch_artwork(&title, &artist, path.as_deref(), 46, 20).await;
            let _ = tx.send((track_id, art));
        });
    };

    // Helper closure to dispatch live radio stations query
    let dispatch_radio_genre = |genre: GenreFilter, tx: mpsc::UnboundedSender<(GenreFilter, Vec<MediaItem>)>| {
        tokio::spawn(async move {
            let items = fetch_radio_browser_genre(genre).await;
            let _ = tx.send((genre, items));
        });
    };

    // Helper to send desktop notification (tray-like notification)
    let send_track_notification = |title: &str, artist: &str, badge: &str| {
        let title_c = title.to_string();
        let artist_c = artist.to_string();
        let badge_c = badge.to_string();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("notify-send")
                .arg("-a")
                .arg("Boombox Audio")
                .arg("-i")
                .arg("audio-x-generic")
                .arg(format!("🎵 {}", title_c))
                .arg(format!("{} • [{}]", artist_c, badge_c))
                .output()
                .await;
        });
    };

    // Initial Live Radio fetch in background
    dispatch_radio_genre(state.genre_filter, radio_tx.clone());

    // 5. Main Interactive Loop (60 FPS)
    while running {
        frame_count = frame_count.wrapping_add(1);

        // Check for SIGUSR1 / SIGHUP Hot-Reload Signal (Unix)
        #[cfg(unix)]
        tokio::select! {
            _ = sigusr1.recv() => {
                should_hot_reload = true;
                break;
            }
            _ = sighup.recv() => {
                should_hot_reload = true;
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(0)) => {}
        }

        // Handle Tray Actions (from Omarchy Tray AppMenu)
        while let Ok(action) = tray_action_rx.try_recv() {
            match action {
                TrayAction::TogglePlay => {
                    player.toggle_pause();
                }
                TrayAction::NextTrack => {
                    if let Some(item) = state.get_next_track() {
                        player.play(&item.url);
                        state.current_track = Some(item.clone());
                        state.status_message = format!("Playing: {}", item.title);
                        state.lyrics.clear();
                        state.lyrics_loading = true;
                        state.current_artwork = None;
                        state.artwork_loading = true;
                        let local_path = if !item.is_radio && !item.is_youtube && !item.url.starts_with("http") {
                            Some(item.url.clone())
                        } else {
                            None
                        };
                        dispatch_lyrics(item.title.clone(), item.artist.clone(), local_path.clone(), item.id.clone(), lyrics_tx.clone());
                        dispatch_artwork(item.title.clone(), item.artist.clone(), Some(item.url.clone()), item.id.clone(), artwork_tx.clone());
                        send_track_notification(&item.title, &item.artist, item.format.as_deref().unwrap_or("AUDIO"));
                    }
                }
                TrayAction::PrevTrack => {
                    if let Some(item) = state.get_prev_track() {
                        player.play(&item.url);
                        state.current_track = Some(item.clone());
                        state.status_message = format!("Playing: {}", item.title);
                        state.lyrics.clear();
                        state.lyrics_loading = true;
                        state.current_artwork = None;
                        state.artwork_loading = true;
                        let local_path = if !item.is_radio && !item.is_youtube && !item.url.starts_with("http") {
                            Some(item.url.clone())
                        } else {
                            None
                        };
                        dispatch_lyrics(item.title.clone(), item.artist.clone(), local_path.clone(), item.id.clone(), lyrics_tx.clone());
                        dispatch_artwork(item.title.clone(), item.artist.clone(), Some(item.url.clone()), item.id.clone(), artwork_tx.clone());
                        send_track_notification(&item.title, &item.artist, item.format.as_deref().unwrap_or("AUDIO"));
                    }
                }
                TrayAction::VolumeUp => {
                    player.set_volume(state.volume_step as i32);
                    state.volume = player.get_status().volume;
                    state.status_message = format!("Volume: {}%", state.volume);
                }
                TrayAction::VolumeDown => {
                    player.set_volume(-(state.volume_step as i32));
                    state.volume = player.get_status().volume;
                    state.status_message = format!("Volume: {}%", state.volume);
                }
                TrayAction::ToggleWindow => {
                    tokio::spawn(focus_or_raise_window());
                }
                TrayAction::Reload => {
                    should_hot_reload = true;
                    running = false;
                    break;
                }
                TrayAction::Quit => {
                    running = false;
                    break;
                }
            }
        }

        // A. Synchronize Audio Player Position & Metadata (100% Non-Blocking)
        let status = player.get_status();
        let live_audio = capture.get_live_data();

        let at_eof = status.duration > 5.0 && status.time_pos >= status.duration - 0.5;
        let is_eof = (status.eof || (at_eof && state.is_playing)) && !state.is_paused && !status.is_paused;

        state.is_playing = status.is_playing;
        state.is_paused = status.is_paused;
        state.volume = status.volume;
        state.telemetry.time_pos = status.time_pos;
        state.telemetry.duration = status.duration;
        state.telemetry.percent_pos = status.percent_pos;
        state.telemetry.spool_frame = frame_count;

        let cur_m = (status.time_pos / 60.0).floor() as u32;
        let cur_s = (status.time_pos % 60.0).floor() as u32;
        state.telemetry.tape_counter = format!("{:02}:{:02}", cur_m, cur_s);

        if let Some(ref codec) = status.metadata.codec {
            state.telemetry.audio_codec = codec.clone();
        }
        if let Some(bits) = status.metadata.bit_depth {
            state.telemetry.audio_bit_depth = bits;
        }
        if let Some(br) = status.metadata.bitrate {
            state.telemetry.audio_bitrate = br;
        }
        if let Some(sr) = status.metadata.sample_rate {
            state.telemetry.audio_sample_rate = sr;
        } else if live_audio.sample_rate > 0 {
            state.telemetry.audio_sample_rate = live_audio.sample_rate;
        }
        if let Some(ref ch) = status.metadata.channels {
            state.telemetry.audio_channels = ch.clone();
        }

        // Live ICY Radio Metadata Sync
        if let Some(ref mut track) = state.current_track {
            if track.is_radio {
                let mut changed = false;
                if let Some(ref icy_title) = status.metadata.title {
                    if !icy_title.is_empty() && icy_title != &track.title {
                        track.title = icy_title.clone();
                        changed = true;
                    }
                }
                if let Some(ref icy_artist) = status.metadata.artist {
                    if !icy_artist.is_empty() && icy_artist != &track.artist {
                        track.artist = icy_artist.clone();
                        changed = true;
                    }
                }
                if changed {
                    state.lyrics_loading = true;
                    dispatch_lyrics(track.title.clone(), track.artist.clone(), None, track.id.clone(), lyrics_tx.clone());
                    send_track_notification(&track.title, &track.artist, "RADIO LIVE");
                }
            }
        }

        // Sync Tray State on Changes
        {
            let mut ts = tray_state.lock().unwrap();
            let cur_title = state.current_track.as_ref().map(|t| t.title.as_str()).unwrap_or("Standby");
            let cur_artist = state.current_track.as_ref().map(|t| t.artist.as_str()).unwrap_or("Boombox RX-505");
            if ts.title != cur_title
                || ts.artist != cur_artist
                || ts.is_playing != state.is_playing
                || ts.is_recording != state.is_recording
                || ts.volume != state.volume
            {
                ts.title = cur_title.to_string();
                ts.artist = cur_artist.to_string();
                ts.is_playing = state.is_playing;
                ts.is_recording = state.is_recording;
                ts.volume = state.volume;
                #[cfg(unix)]
                if let Some(ref h) = _tray_handle {
                    let handle_clone = h.clone();
                    tokio::spawn(async move {
                        let _ = handle_clone.update(|_| {}).await;
                    });
                }
            }
        }

        // Poll Async Lyrics Receiver
        while let Ok((track_id, lines)) = lyrics_rx.try_recv() {
            if let Some(ref cur) = state.current_track {
                if cur.id == track_id {
                    state.lyrics = lines;
                    state.lyrics_loading = false;
                }
            }
        }

        // Poll Async Cover Artwork Receiver
        while let Ok((track_id, art_opt)) = artwork_rx.try_recv() {
            if let Some(ref cur) = state.current_track {
                if cur.id == track_id {
                    state.current_artwork = art_opt;
                    state.artwork_loading = false;
                }
            }
        }

        // Poll Async Live Radio Receiver
        while let Ok((genre, items)) = radio_rx.try_recv() {
            if state.genre_filter == genre && !items.is_empty() {
                state.radio_stations = items;
                let q = state.search_query.clone();
                state.filter(&q);
            }
        }

        // Auto-advance when track finishes (natural EOF reached)
        if is_eof {
            if let Some(next) = state.get_next_track() {
                player.play(&next.url);
                state.current_track = Some(next.clone());
                state.status_message = format!("Auto-Advance ▶ {}", next.title);
                state.lyrics.clear();
                state.lyrics_loading = true;
                state.current_artwork = None;
                state.artwork_loading = true;

                let local_path = if !next.is_radio && !next.is_youtube && !next.url.starts_with("http") {
                    Some(next.url.clone())
                } else {
                    None
                };
                dispatch_lyrics(next.title.clone(), next.artist.clone(), local_path, next.id.clone(), lyrics_tx.clone());
                dispatch_artwork(next.title.clone(), next.artist.clone(), Some(next.url.clone()), next.id.clone(), artwork_tx.clone());
                send_track_notification(&next.title, &next.artist, next.format.as_deref().unwrap_or("FLAC"));
            }
        }

        // B. Update Visualizer Spectrum with REAL-TIME PCM AUDIO OUTPUT
        let eq_gains = audio::equalizer::compute_total_gains(
            state.eq_preset,
            state.bass_boost,
            state.dolby_mode,
            state.tape_type,
        );
        let (bands, peaks, vu_l, vu_r, peak_l, peak_r) = visualizer.update_with_live_audio(
            &live_audio,
            state.is_playing,
            state.is_paused,
            state.bass_boost,
            eq_gains,
        );
        state.telemetry.eq_bands = bands;
        state.telemetry.eq_peaks = peaks;
        state.telemetry.vu_left = vu_l;
        state.telemetry.vu_right = vu_r;
        state.telemetry.peak_left = peak_l;
        state.telemetry.peak_right = peak_r;

        // B2. Poll Tape Recorder & Sync Status
        for (title, ok) in recorder.poll() {
            if ok {
                audio::recorder::send_notification(
                    "✅ Đã tải xong bài hát",
                    &format!("🎵 {}\n📁 Đã lưu vào ~/Music/Boombox Recordings/", title),
                );
                state.recording_status = format!("✅ SAVED: {}", title);
            } else {
                audio::recorder::send_notification(
                    "⚠️ Tải / Ghi âm thất bại",
                    &format!("Không thể tải stream: {}", title),
                );
                state.recording_status = format!("⚠️ FAILED: {}", title);
            }
        }
        state.is_recording = recorder.is_recording();
        if recorder.is_recording() {
            state.recording_status = format!("🔴 REC {:.0}s", frame_count as f64 / 60.0);
        } else if !recorder.current_jobs().is_empty() {
            state.recording_status = "WAITING".to_string();
        }

        // C. Render Terminal UI Frame
        let current_theme = state.current_theme();
        terminal.draw(|f| {
            render_ui(f, &state, &current_theme);
        })?;

        // D. Non-blocking Event Polling (16ms = ~60 FPS)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

                // 1. Emergency Quit (Ctrl+C)
                if is_ctrl && key.code == KeyCode::Char('c') {
                    break;
                }

                // 2. Modal Context Handling
                match state.active_modal {
                    ModalType::UrlInput => match key.code {
                        KeyCode::Esc => {
                            state.active_modal = ModalType::None;
                            state.input_buffer.clear();
                        }
                        KeyCode::Char('a') if is_ctrl => {
                            let url = state.input_buffer.trim().to_string();
                            if !url.is_empty() {
                                let (label, tracks) = api::stream::resolve_stream_queue(&url).await;
                                let count = tracks.len();
                                state.youtube_results = tracks.clone();
                                for t in tracks {
                                    if !state.queue.iter().any(|q| q.id == t.id) {
                                        state.queue.push(t);
                                    }
                                }
                                state.mode = AppMode::YoutubeMusic;
                                state.selected_index = 0;
                                state.status_message = format!("📥 Queued: {} ({} tracks)", label, count);
                            }
                            state.active_modal = ModalType::None;
                            state.input_buffer.clear();
                        }
                        KeyCode::Enter => {
                            let url = state.input_buffer.trim().to_string();
                            if !url.is_empty() {
                                let (label, tracks) = api::stream::resolve_stream_queue(&url).await;
                                if let Some(first) = tracks.first() {
                                    let first_title = first.title.clone();
                                    player.play(&first.url);
                                    state.current_track = Some(first.clone());
                                    state.lyrics.clear();
                                    state.lyrics_loading = true;
                                    state.current_artwork = None;
                                    state.artwork_loading = true;
                                    dispatch_lyrics(first.title.clone(), first.artist.clone(), None, first.id.clone(), lyrics_tx.clone());
                                    dispatch_artwork(first.title.clone(), first.artist.clone(), Some(first.url.clone()), first.id.clone(), artwork_tx.clone());
                                    send_track_notification(&first.title, &first.artist, first.format.as_deref().unwrap_or("STREAM"));

                                    state.youtube_results = tracks.clone();
                                    state.mode = AppMode::YoutubeMusic;
                                    state.selected_index = 0;

                                    for t in tracks {
                                        if !state.queue.iter().any(|q| q.id == t.id) {
                                            state.queue.push(t);
                                        }
                                    }
                                    state.status_message = format!("▶ Playing ({}) — {}", label, first_title);
                                } else {
                                    state.status_message = format!("No results found for: {}", url);
                                }
                            }
                            state.active_modal = ModalType::None;
                            state.input_buffer.clear();
                        }
                        KeyCode::Backspace => {
                            state.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            state.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    ModalType::Search => match key.code {
                        KeyCode::Esc => {
                            state.active_modal = ModalType::None;
                            state.input_buffer.clear();
                            state.filter("");
                            state.status_message = "Search filter cleared".to_string();
                        }
                        KeyCode::Enter => {
                            let q = state.input_buffer.trim().to_string();
                            if state.mode == AppMode::YoutubeMusic && !q.is_empty() {
                                let (label, tracks) = api::stream::resolve_stream_queue(&q).await;
                                state.youtube_results = tracks;
                                state.selected_index = 0;
                                state.status_message = format!("Online Search: {}", label);
                            } else {
                                state.filter(&q);
                                state.status_message = if q.is_empty() {
                                    "Search filter cleared".to_string()
                                } else {
                                    format!("Search: \"{}\" ({} results)", q, state.get_active_list_len())
                                };
                            }
                            state.active_modal = ModalType::None;
                        }
                        KeyCode::Backspace => {
                            state.input_buffer.pop();
                            let q = state.input_buffer.clone();
                            state.filter(&q);
                        }
                        KeyCode::Char(c) => {
                            state.input_buffer.push(c);
                            let q = state.input_buffer.clone();
                            state.filter(&q);
                        }
                        _ => {}
                    },
                    ModalType::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                            state.active_modal = ModalType::None;
                        }
                        _ => {}
                    },
                    ModalType::Settings => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('o') => {
                            state.active_modal = ModalType::None;
                            state.save_config();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.move_settings_selection(-1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            state.move_settings_selection(1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => {
                            state.cycle_selected_setting(1);
                            let (att, rel) = state.visualizer_speed.alphas();
                            visualizer.set_ballistics(att, rel);
                            let af = audio::equalizer::build_mpv_af_string(
                                state.eq_preset,
                                state.bass_boost,
                                state.dolby_mode,
                                state.stereo_mode,
                                state.tape_type,
                            );
                            player.apply_audio_filter(&af);
                            state.save_config();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            state.cycle_selected_setting(-1);
                            let (att, rel) = state.visualizer_speed.alphas();
                            visualizer.set_ballistics(att, rel);
                            let af = audio::equalizer::build_mpv_af_string(
                                state.eq_preset,
                                state.bass_boost,
                                state.dolby_mode,
                                state.stereo_mode,
                                state.tape_type,
                            );
                            player.apply_audio_filter(&af);
                            state.save_config();
                        }
                        KeyCode::Char('[') => {
                            state.cycle_selected_setting(-1);
                            let (att, rel) = state.visualizer_speed.alphas();
                            visualizer.set_ballistics(att, rel);
                            let af = audio::equalizer::build_mpv_af_string(
                                state.eq_preset,
                                state.bass_boost,
                                state.dolby_mode,
                                state.stereo_mode,
                                state.tape_type,
                            );
                            player.apply_audio_filter(&af);
                            state.save_config();
                        }
                        KeyCode::Char(']') => {
                            state.cycle_selected_setting(1);
                            let (att, rel) = state.visualizer_speed.alphas();
                            visualizer.set_ballistics(att, rel);
                            let af = audio::equalizer::build_mpv_af_string(
                                state.eq_preset,
                                state.bass_boost,
                                state.dolby_mode,
                                state.stereo_mode,
                                state.tape_type,
                            );
                            player.apply_audio_filter(&af);
                            state.save_config();
                        }
                        _ => {}
                    },
                    ModalType::Mixtape => match key.code {
                        KeyCode::Esc | KeyCode::Char('m') | KeyCode::Char('q') => {
                            state.active_modal = ModalType::None;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if state.selected_mixtape_idx > 0 {
                                state.selected_mixtape_idx -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if state.selected_mixtape_idx + 1 < state.mixtapes.len() {
                                state.selected_mixtape_idx += 1;
                            }
                        }
                        KeyCode::Char('n') => {
                            state.create_mixtape();
                        }
                        KeyCode::Char('x') => {
                            state.delete_mixtape();
                        }
                        KeyCode::Char('a') => {
                            state.add_current_track_to_mixtape();
                        }
                        KeyCode::Char('d') => {
                            state.remove_selected_mixtape_track();
                        }
                        KeyCode::Enter => {
                            state.add_current_track_to_mixtape();
                        }
                        _ => {}
                    },
                    ModalType::None => {
                        // 3. Normal Mode Single-Key Dispatch
                        match key.code {
                            KeyCode::Char('q') => {
                                running = false;
                            }
                            KeyCode::Esc => {
                                if !state.search_query.is_empty() {
                                    state.filter("");
                                    state.input_buffer.clear();
                                    state.status_message = "Search filter cleared".to_string();
                                } else if !state.drill_up() {
                                    // at root level
                                }
                            }
                            // Playback
                            KeyCode::Char(' ') => {
                                player.toggle_pause();
                            }
                            KeyCode::Char('s') => {
                                player.stop();
                                state.status_message = "Playback Stopped".to_string();
                            }
                            KeyCode::Char('n') => {
                                if let Some(item) = state.get_next_track() {
                                    player.play(&item.url);
                                    state.current_track = Some(item.clone());
                                    state.status_message = format!("Playing: {}", item.title);
                                    state.lyrics.clear();
                                    state.lyrics_loading = true;
                                    state.current_artwork = None;
                                    state.artwork_loading = true;

                                    let local_path = if !item.is_radio && !item.is_youtube && !item.url.starts_with("http") {
                                        Some(item.url.clone())
                                    } else {
                                        None
                                    };
                                    dispatch_lyrics(item.title.clone(), item.artist.clone(), local_path.clone(), item.id.clone(), lyrics_tx.clone());
                                    dispatch_artwork(item.title.clone(), item.artist.clone(), Some(item.url.clone()), item.id.clone(), artwork_tx.clone());
                                    send_track_notification(&item.title, &item.artist, item.format.as_deref().unwrap_or("AUDIO"));
                                }
                            }
                            KeyCode::Char('p') => {
                                if let Some(item) = state.get_prev_track() {
                                    player.play(&item.url);
                                    state.current_track = Some(item.clone());
                                    state.status_message = format!("Playing: {}", item.title);
                                    state.lyrics.clear();
                                    state.lyrics_loading = true;
                                    state.current_artwork = None;
                                    state.artwork_loading = true;

                                    let local_path = if !item.is_radio && !item.is_youtube && !item.url.starts_with("http") {
                                        Some(item.url.clone())
                                    } else {
                                        None
                                    };
                                    dispatch_lyrics(item.title.clone(), item.artist.clone(), local_path.clone(), item.id.clone(), lyrics_tx.clone());
                                    dispatch_artwork(item.title.clone(), item.artist.clone(), Some(item.url.clone()), item.id.clone(), artwork_tx.clone());
                                    send_track_notification(&item.title, &item.artist, item.format.as_deref().unwrap_or("AUDIO"));
                                }
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                player.set_volume(state.volume_step as i32);
                            }
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                player.set_volume(-(state.volume_step as i32));
                            }
                            KeyCode::Char('[') => {
                                if state.active_view == ActiveView::Lyrics {
                                    state.lyrics_offset = (state.lyrics_offset - 0.25).clamp(-60.0, 60.0);
                                    state.status_message = format!("Lyrics Sync Offset: {:+.2}s", state.lyrics_offset);
                                } else {
                                    player.seek(-10.0);
                                    state.status_message = "Seek -10s".to_string();
                                }
                            }
                            KeyCode::Char(']') => {
                                if state.active_view == ActiveView::Lyrics {
                                    state.lyrics_offset = (state.lyrics_offset + 0.25).clamp(-60.0, 60.0);
                                    state.status_message = format!("Lyrics Sync Offset: {:+.2}s", state.lyrics_offset);
                                } else {
                                    player.seek(10.0);
                                    state.status_message = "Seek +10s".to_string();
                                }
                            }
                            KeyCode::Char('{') => {
                                state.lyrics_offset = (state.lyrics_offset - 1.0).clamp(-60.0, 60.0);
                                state.status_message = format!("Lyrics Sync Offset: {:+.2}s", state.lyrics_offset);
                            }
                            KeyCode::Char('}') => {
                                state.lyrics_offset = (state.lyrics_offset + 1.0).clamp(-60.0, 60.0);
                                state.status_message = format!("Lyrics Sync Offset: {:+.2}s", state.lyrics_offset);
                            }
                            KeyCode::Char('0') if state.active_view == ActiveView::Lyrics => {
                                state.lyrics_offset = 0.0;
                                state.status_message = "Lyrics Sync Offset Reset (0.0s)".to_string();
                            }
                            KeyCode::Char('S') if state.active_view == ActiveView::Lyrics => {
                                state.matrix_scramble = !state.matrix_scramble;
                                state.status_message = if state.matrix_scramble {
                                    "Lyrics Matrix Cipher: ON".to_string()
                                } else {
                                    "Lyrics Matrix Cipher: OFF (Plain Text)".to_string()
                                };
                            }

                            // Mode Switching
                            KeyCode::Char('1') => state.set_mode(AppMode::LocalTracks),
                            KeyCode::Char('2') => state.set_mode(AppMode::RadioStations),
                            KeyCode::Char('3') => state.set_mode(AppMode::Queue),
                            KeyCode::Char('4') => state.set_mode(AppMode::YoutubeMusic),
                            KeyCode::Tab => state.cycle_mode(1),
                            KeyCode::Char('g') => {
                                state.cycle_genre(1);
                                dispatch_radio_genre(state.genre_filter, radio_tx.clone());
                            }

                            // List Navigation & Action
                            KeyCode::Up | KeyCode::Char('k') => state.move_selection(-1),
                            KeyCode::Down | KeyCode::Char('j') => state.move_selection(1),
                            KeyCode::Enter => {
                                if let Some(track) = state.drill_down() {
                                    player.play(&track.url);
                                    state.current_track = Some(track.clone());
                                    state.status_message = format!("Playing: {}", track.title);
                                    state.lyrics.clear();
                                    state.lyrics_loading = true;
                                    state.current_artwork = None;
                                    state.artwork_loading = true;

                                    let local_path = if !track.is_radio && !track.is_youtube && !track.url.starts_with("http") {
                                        Some(track.url.clone())
                                    } else {
                                        None
                                    };
                                    dispatch_lyrics(track.title.clone(), track.artist.clone(), local_path.clone(), track.id.clone(), lyrics_tx.clone());
                                    dispatch_artwork(track.title.clone(), track.artist.clone(), Some(track.url.clone()), track.id.clone(), artwork_tx.clone());
                                    send_track_notification(&track.title, &track.artist, track.format.as_deref().unwrap_or("MASTER"));
                                }
                            }
                            KeyCode::Backspace => {
                                state.drill_up();
                            }
                            KeyCode::Char('v') => {
                                state.toggle_local_view();
                            }
                            KeyCode::Char('a') => state.add_to_queue(),
                            KeyCode::Char('x') => state.remove_from_queue(),
                            KeyCode::Char('c') => state.clear_queue(),
                            KeyCode::Char('m') => state.toggle_favorite(),
                            KeyCode::Char('M') => {
                                state.active_modal = ModalType::Mixtape;
                            }

                            // Tape Recorder
                            KeyCode::Char('R') if is_ctrl => {
                                recorder.cancel(None).await;
                                state.is_recording = false;
                                state.status_message = "Tape Recorder: All recordings cancelled".to_string();
                            }
                            KeyCode::Char('R') => {
                                if let Some(item) = state.current_track.clone().or_else(|| state.get_selected_item().cloned()) {
                                    match recorder.record_track(&item, state.record_format).await {
                                        Ok(true) => {
                                            state.is_recording = true;
                                            state.recording_status = "🔴 REC".to_string();
                                            state.status_message = format!("Recording: {}", item.title);
                                        }
                                        Ok(false) => {
                                            state.is_recording = false;
                                            state.status_message = "Recording cancelled".to_string();
                                        }
                                        Err(e) => {
                                            state.status_message = e;
                                        }
                                    }
                                } else {
                                    state.status_message = "No track selected to record".to_string();
                                }
                            }

                            // DSP & Enhancements
                            KeyCode::Char('b') => {
                                state.bass_boost = !state.bass_boost;
                                state.status_message = if state.bass_boost {
                                    "MEGA BASS: +7dB Enabled".to_string()
                                } else {
                                    "MEGA BASS: Flat Disabled".to_string()
                                };
                                let af = audio::equalizer::build_mpv_af_string(
                                    state.eq_preset,
                                    state.bass_boost,
                                    state.dolby_mode,
                                    state.stereo_mode,
                                    state.tape_type,
                                );
                                player.apply_audio_filter(&af);
                                state.save_config();
                            }
                            KeyCode::Char('d') => {
                                state.dolby_mode = state.dolby_mode.cycle();
                                state.status_message = format!("Dolby Filter: {}", state.dolby_mode.label());
                                let af = audio::equalizer::build_mpv_af_string(
                                    state.eq_preset,
                                    state.bass_boost,
                                    state.dolby_mode,
                                    state.stereo_mode,
                                    state.tape_type,
                                );
                                player.apply_audio_filter(&af);
                                state.save_config();
                            }
                            KeyCode::Char('e') => {
                                state.eq_preset = state.eq_preset.cycle();
                                state.status_message = format!("EQ Profile: {}", state.eq_preset.label());
                                let af = audio::equalizer::build_mpv_af_string(
                                    state.eq_preset,
                                    state.bass_boost,
                                    state.dolby_mode,
                                    state.stereo_mode,
                                    state.tape_type,
                                );
                                player.apply_audio_filter(&af);
                                state.save_config();
                            }
                            KeyCode::Char('t') => {
                                state.cycle_theme(1);
                                state.save_config();
                            }
                            KeyCode::Char('r') => {
                                state.repeat_mode = state.repeat_mode.cycle();
                                state.status_message = format!("Repeat Mode: {}", state.repeat_mode.label());
                            }
                            KeyCode::Char('z') => {
                                state.shuffle = !state.shuffle;
                                state.status_message = if state.shuffle {
                                    "Shuffle: ON".to_string()
                                } else {
                                    "Shuffle: OFF".to_string()
                                };
                            }
                            KeyCode::Char('l') => {
                                state.active_view = match state.active_view {
                                    ActiveView::Lyrics => ActiveView::Deck,
                                    _ => ActiveView::Lyrics,
                                };
                            }
                            KeyCode::Char('w') => {
                                state.active_view = match state.active_view {
                                    ActiveView::Artwork => ActiveView::Deck,
                                    _ => {
                                        if state.current_artwork.is_none() && !state.artwork_loading {
                                            if let Some(track) = &state.current_track {
                                                state.artwork_loading = true;
                                                dispatch_artwork(track.title.clone(), track.artist.clone(), Some(track.url.clone()), track.id.clone(), artwork_tx.clone());
                                            }
                                        }
                                        ActiveView::Artwork
                                    }
                                };
                            }
                            KeyCode::Char('u') => {
                                state.active_modal = ModalType::UrlInput;
                                state.input_buffer.clear();
                            }
                            KeyCode::Char('/') => {
                                state.active_modal = ModalType::Search;
                                state.input_buffer = state.search_query.clone();
                            }
                            KeyCode::Char('o') => {
                                state.active_modal = ModalType::Settings;
                            }
                            KeyCode::Char('?') => {
                                state.active_modal = ModalType::Help;
                            }
                            KeyCode::F(5) => {
                                should_hot_reload = true;
                                running = false;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // 6. Clean Terminal Teardown
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    player.stop();

    // Cleanly unregister and shutdown the D-Bus StatusNotifierItem Tray immediately
    #[cfg(unix)]
    if let Some(ref h) = _tray_handle {
        let _ = h.shutdown().await;
    }

    // 7. If Hot-Reload requested via SIGUSR1/SIGHUP/Tray, re-execute binary in-place!
    if should_hot_reload {
        let exe = std::env::current_exe()?;
        let args: Vec<String> = std::env::args().skip(1).collect();
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            let _ = std::process::Command::new(exe).args(args).exec();
        }
        #[cfg(not(unix))]
        {
            let _ = std::process::Command::new(exe).args(args).spawn();
            std::process::exit(0);
        }
    }

    println!("📼 Boombox-rs terminated cleanly.");
    std::process::exit(0);
}

/// Dispatches window toggle to raise, focus, or minimize the Boombox window
async fn focus_or_raise_window() {
    let _ = tokio::process::Command::new("/home/aki/.local/bin/boombox-toggle")
        .output()
        .await;
}
