//! Cross-platform non-blocking desktop notification helper
//!
//! Avoids nested Tokio runtime block_on panics on Linux/zbus by using
//! `show_async()` on Linux and dedicated blocking tasks on Windows/macOS.

pub fn send_desktop_notification(title: &str, body: &str, icon: Option<&str>) {
    let title = title.to_string();
    let body = body.to_string();
    let icon = icon.unwrap_or("audio-x-generic").to_string();

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(async move {
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                let _ = notify_rust::Notification::new()
                    .appname("BOOMBOX RX-505")
                    .summary(&title)
                    .body(&body)
                    .icon(&icon)
                    .show_async()
                    .await;
            }
            #[cfg(not(all(unix, not(target_os = "macos"))))]
            {
                let _ = tokio::task::spawn_blocking(move || {
                    let _ = std::panic::catch_unwind(|| {
                        let _ = notify_rust::Notification::new()
                            .appname("BOOMBOX RX-505")
                            .summary(&title)
                            .body(&body)
                            .icon(&icon)
                            .show();
                    });
                })
                .await;
            }
        });
    } else {
        std::thread::spawn(move || {
            let _ = std::panic::catch_unwind(|| {
                let _ = notify_rust::Notification::new()
                    .appname("BOOMBOX RX-505")
                    .summary(&title)
                    .body(&body)
                    .icon(&icon)
                    .show();
            });
        });
    }
}
