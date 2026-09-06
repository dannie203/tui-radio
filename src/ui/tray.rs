#[cfg(unix)]
use ksni::{menu::*, Handle, Orientation, ToolTip, Tray, TrayMethods};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
#[cfg_attr(not(unix), allow(dead_code))]
pub enum TrayAction {
    TogglePlay,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    ToggleWindow,
    Reload,
    Quit,
}

#[derive(Debug, Clone)]
#[cfg_attr(not(unix), allow(dead_code))]
pub struct TrayState {
    pub title: String,
    pub artist: String,
    pub volume: u32,
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_recording: bool,
    pub action_tx: tokio::sync::mpsc::UnboundedSender<TrayAction>,
}

#[cfg(unix)]
pub struct BoomboxTray {
    pub state: Arc<Mutex<TrayState>>,
}

#[cfg(unix)]
impl Tray for BoomboxTray {
    fn id(&self) -> String {
        "org.omarchy.boombox".into()
    }

    fn title(&self) -> String {
        "Boombox RX-505".into()
    }

    fn icon_theme_path(&self) -> String {
        let home = dirs::home_dir()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|| std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        format!("{}/.local/share/icons/hicolor", home)
    }

    fn icon_name(&self) -> String {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if st.is_recording {
            "boombox-tray".into()
        } else if st.is_playing && !st.is_paused {
            "boombox-tray-playing".into()
        } else {
            "boombox-tray-paused".into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let description = if st.is_playing && !st.is_paused {
            format!("▶ Now Playing: {}\nBy: {}\nVol: {}%", st.title, st.artist, st.volume)
        } else if st.is_paused {
            format!("⏸ Paused: {}\nBy: {}\nVol: {}%", st.title, st.artist, st.volume)
        } else {
            format!("⏹ Standby / Stopped\nVol: {}%", st.volume)
        };
        ToolTip {
            title: "BOOMBOX RX-505".into(),
            description,
            icon_name: "audio-x-generic".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = st.action_tx.send(TrayAction::ToggleWindow);
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = st.action_tx.send(TrayAction::TogglePlay);
    }

    fn scroll(&mut self, delta: i32, _orientation: Orientation) {
        let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if delta > 0 {
            let _ = st.action_tx.send(TrayAction::VolumeUp);
        } else if delta < 0 {
            let _ = st.action_tx.send(TrayAction::VolumeDown);
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let (is_active_playing, is_paused, title, artist, volume) = {
            let st = self.state.lock().unwrap_or_else(|e| e.into_inner());
            (
                st.is_playing && !st.is_paused,
                st.is_paused,
                st.title.clone(),
                st.artist.clone(),
                st.volume,
            )
        };

        let play_pause_label = if is_active_playing {
            "⏸ Pause"
        } else if is_paused {
            "▶ Resume"
        } else {
            "▶ Play"
        };

        vec![
            StandardItem {
                label: format!("{} — {} (Vol: {}%)", title, artist, volume),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: play_pause_label.into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::TogglePlay);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Next Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::NextTrack);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Previous Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::PrevTrack);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Show / Hide Window (Super+M)".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hot-Reload App (F5)".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::Reload);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Boombox".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = st.action_tx.send(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// RAII Drop guard to cleanly unregister the StatusNotifierItem tray icon from D-Bus
#[cfg(unix)]
pub struct TrayGuard(pub Option<Handle<BoomboxTray>>);

#[cfg(unix)]
impl Drop for TrayGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            std::mem::drop(handle.shutdown());
        }
    }
}

#[cfg(not(unix))]
pub struct TrayGuard;

#[cfg(unix)]
pub async fn spawn_tray(state: Arc<Mutex<TrayState>>) -> TrayGuard {
    let tray = BoomboxTray { state };
    TrayGuard(tray.spawn().await.ok())
}

#[cfg(not(unix))]
pub async fn spawn_tray(_state: Arc<Mutex<TrayState>>) -> TrayGuard {
    TrayGuard
}
