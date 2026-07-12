#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, SetWindowsHookExW, CallNextHookEx,
    GetMessageW, WH_KEYBOARD_LL, WM_KEYDOWN, KBDLLHOOKSTRUCT
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;

use crate::os::{OSInterface, WindowInfo};

pub struct WindowsOS;

#[cfg(windows)]
static KEYBOARD_TX: std::sync::OnceLock<tokio::sync::mpsc::Sender<char>> = std::sync::OnceLock::new();

/// SAFETY: This hook is managed via a dedicated background thread on Windows to capture low-level
/// keyboard events without stalling the main async engine.
#[cfg(windows)]
pub unsafe extern "system" fn keyboard_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 && w_param == WM_KEYDOWN as usize {
        let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
        if let Some(tx) = KEYBOARD_TX.get() {
            // Simplified VK code to char conversion
            let c = match kbd.vkCode {
                0x41..=0x5A => Some((kbd.vkCode as u8) as char), // A-Z
                0x30..=0x39 => Some((kbd.vkCode as u8) as char), // 0-9
                0x20 => Some(' '), // Space
                0x0D => Some('\n'), // Enter
                0x08 => Some('\u{8}'), // Backspace
                _ => None,
            };
            if let Some(ch) = c {
                let _ = tx.try_send(ch.to_ascii_lowercase());
            }
        }
    }
    CallNextHookEx(0, code, w_param, l_param)
}

#[cfg(windows)]
pub fn start_windows_hook(tx: tokio::sync::mpsc::Sender<char>) {
    std::thread::spawn(move || {
        let _ = KEYBOARD_TX.set(tx);
        unsafe {
            let _hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), 0, 0);
            let mut msg = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) != 0 {}
        }
    });
}

impl OSInterface for WindowsOS {
    #[cfg(windows)]
    fn get_active_window(&self) -> Option<WindowInfo> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 { return None; }

            // Get Title
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            let title = String::from_utf16_lossy(&title_buf[..len as usize]);

            // Get Process Name
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);

            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            let mut process_name = String::from("Unknown");

            if process_handle != 0 {
                let mut name_buf = [0u16; 1024];
                let mut name_len = 1024u32;
                let success = QueryFullProcessImageNameW(process_handle, 0, name_buf.as_mut_ptr(), &mut name_len);
                if success != 0 && name_len > 0 {
                    let full_path = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                    if let Some(file_name) = std::path::Path::new(&full_path).file_name() {
                        process_name = file_name.to_string_lossy().into_owned();
                    } else {
                        process_name = full_path;
                    }
                }
                CloseHandle(process_handle);
            }

            Some(WindowInfo {
                process_name: process_name.into(),
                title: title.into(),
            })
        }
    }

    #[cfg(not(windows))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        None
    }
}
