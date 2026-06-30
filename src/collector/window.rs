use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use crate::models::{InputEvent, WindowEvent};
use crate::os::target;
use chrono::Utc;

pub async fn run_window_collector(tx: mpsc::Sender<InputEvent>) {
    let mut last_window = None;

    let mut interval = time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;

        // Create tracker inside the loop or ensure it's not held across await if it's !Send
        // Alternatively, use spawn_blocking for tracker calls.
        let result = tokio::task::spawn_blocking(|| {
            #[cfg(target_os = "windows")]
            let tracker = target::WindowsTracker::new();

            #[cfg(target_os = "linux")]
            let tracker = target::LinuxTracker::new()?;

            tracker.get_active_window()
        }).await;

        if let Ok(Some((app, title))) = result {
            let current = (app.clone(), title.clone());
            if Some(&current) != last_window.as_ref() {
                let event = WindowEvent {
                    app_name: app.into(),
                    window_title: title.into(),
                    timestamp: Utc::now(),
                };
                let _ = tx.send(InputEvent::Window(event)).await;
                last_window = Some(current);
            }
        }
    }
}
