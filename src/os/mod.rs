pub mod windows;
pub mod linux;

#[cfg(target_os = "windows")]
pub use windows as target;

#[cfg(target_os = "linux")]
pub use linux as target;

pub trait WindowTracker {
    fn get_active_window(&self) -> Option<(String, String)>;
}
