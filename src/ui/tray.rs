use ksni::{menu::*, Handle, ToolTip, Tray, TrayMethods};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum TrayAction {
    TogglePlay,
    NextTrack,
    PrevTrack,
    ToggleWindow,
    Reload,
    Quit,
}

#[derive(Debug, Clone)]
pub struct TrayState {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub is_recording: bool,
    pub action_tx: tokio::sync::mpsc::UnboundedSender<TrayAction>,
}

pub struct BoomboxTray {
    pub state: Arc<Mutex<TrayState>>,
}

impl Tray for BoomboxTray {
    fn id(&self) -> String {
        "org.omarchy.boombox".into()
    }

    fn title(&self) -> String {
        let st = self.state.lock().unwrap();
        if st.is_playing {
            format!("Boombox: {} - {}", st.title, st.artist)
        } else {
            "Boombox Hi-Fi Studio".into()
        }
    }

    fn icon_name(&self) -> String {
        let st = self.state.lock().unwrap();
        if st.is_recording {
            "media-record".into()
        } else if st.is_playing {
            "media-playback-start".into()
        } else {
            "audio-x-generic".into()
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let st = self.state.lock().unwrap();
        ToolTip {
            title: "Boombox Hi-Fi Studio".into(),
            description: if st.is_playing {
                format!("▶ Now Playing: {}\nBy: {}", st.title, st.artist)
            } else {
                "⏹ Standby / Paused".into()
            },
            icon_name: "audio-x-generic".into(),
            icon_pixmap: Vec::new(),
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let st = self.state.lock().unwrap();
        let _ = st.action_tx.send(TrayAction::ToggleWindow);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let (is_playing, title, artist) = {
            let st = self.state.lock().unwrap();
            (st.is_playing, st.title.clone(), st.artist.clone())
        };

        vec![
            StandardItem {
                label: format!("🎵 {} — {}", title, artist),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: if is_playing { "⏸ Pause".into() } else { "▶ Play".into() },
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::TogglePlay);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "⏭ Next Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::NextTrack);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "⏮ Previous Track".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::PrevTrack);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "🎛️ Show / Focus Boombox (Super+Shift+Alt+M)".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::ToggleWindow);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "⚡ Hot-Reload App".into(),
                activate: Box::new(|this: &mut Self| {
                    let st = this.state.lock().unwrap();
                    let _ = st.action_tx.send(TrayAction::Reload);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "🛑 Quit Boombox".into(),
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

pub async fn spawn_tray(state: Arc<Mutex<TrayState>>) -> Option<Handle<BoomboxTray>> {
    let tray = BoomboxTray { state };
    tray.spawn().await.ok()
}
