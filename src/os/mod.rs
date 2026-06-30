pub mod windows;
pub mod linux;

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub process_name: String,
    pub title: String,
}

pub trait OSInterface {
    fn get_active_window(&self) -> Option<WindowInfo>;
}
