#[cfg(windows)]
use windows_sys::Win32::Foundation::*;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::*;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::*;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::*;
#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[cfg(windows)]
pub struct WindowsTracker;

#[cfg(windows)]
impl WindowsTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn get_active_window(&self) -> Option<(String, String)> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return None;
            }

            // Get Window Title
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), title_buf.len() as i32);
            let title = if len > 0 {
                String::from_utf16_lossy(&title_buf[..len as usize])
            } else {
                String::from("Unknown")
            };

            // Get Process Name
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            let process_handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION,
                0, // FALSE
                process_id,
            );

            let mut app_name = String::from("Unknown");
            if process_handle != 0 {
                let mut exe_buf = [0u16; 512];
                let mut exe_len = exe_buf.len() as u32;
                if QueryFullProcessImageNameW(process_handle, 0, exe_buf.as_mut_ptr(), &mut exe_len) != 0 {
                    let full_path = String::from_utf16_lossy(&exe_buf[..exe_len as usize]);
                    app_name = std::path::Path::new(&full_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                }
                CloseHandle(process_handle);
            }

            Some((app_name, title))
        }
    }
}
