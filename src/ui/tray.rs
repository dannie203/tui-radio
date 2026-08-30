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
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/aki".to_string());
        format!("{}/.local/share/icons/hicolor", home)
    }

    fn icon_name(&self) -> String {
        let st = self.state.lock().unwrap();
        if st.is_recording {
            "boombox-tray".into()
        } else if st.is_playing {
            "boombox-tray-playing".into()
        } else {
            "boombox-tray-paused".into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let st = self.state.lock().unwrap();
        ToolTip {
            title: "BOOMBOX RX-505".into(),
            description: if st.is_playing {
                format!("▶ Now Playing: {}\nBy: {}\nVol: {}%", st.title, st.artist, st.volume)
            } else {
                format!("⏸ Standby / Paused\nVol: {}%", st.volume)
            },
            icon_name: "audio-x-generic".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let st = self.state.lock().unwrap();
        let _ = st.action_tx.send(TrayAction::ToggleWindow);
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        let st = self.state.lock().unwrap();
        let _ = st.action_tx.send(TrayAction::TogglePlay);
    }

    fn scroll(&mut self, delta: i32, _orientation: Orientation) {
        let st = self.state.lock().unwrap();
        if delta > 0 {
            let _ = st.action_tx.send(TrayAction::VolumeUp);
        } else if delta < 0 {
            let _ = st.action_tx.send(TrayAction::VolumeDown);
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let (is_playing, title, artist, volume) = {
            let st = self.state.lock().unwrap();
            (st.is_playing, st.title.clone(), st.artist.clone(), st.volume)
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
                label: if is_playing { "Pause".into() } else { "Play".into() },
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::TogglePlay);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Next Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::NextTrack);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Previous Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::PrevTrack);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Show / Hide Window (Super+M)".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Hot-Reload App (F5)".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::Reload);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Boombox".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

#[cfg(unix)]
pub async fn spawn_tray(state: Arc<Mutex<TrayState>>) -> Option<Handle<BoomboxTray>> {
    let tray = BoomboxTray { state };
    tray.spawn().await.ok()
}

#[cfg(not(unix))]
pub async fn spawn_tray(_state: Arc<Mutex<TrayState>>) -> Option<()> {
    None
}
