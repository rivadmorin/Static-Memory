use crate::os::{OSInterface, WindowInfo};

pub struct MacOS;

impl OSInterface for MacOS {
    #[cfg(target_os = "macos")]
    fn get_active_window(&self) -> Option<WindowInfo> {
        // Basic stub implementation for MacOS
        Some(WindowInfo {
            process_name: "MacOSApp".into(),
            title: "MacOS Window".into(),
        })
    }

    #[cfg(not(target_os = "macos"))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        None
    }
}
