#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetForegroundWindow, GetMessageW, GetWindowTextW, SetWindowsHookExW,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN,
};

use crate::os::{OSInterface, WindowInfo};

pub struct WindowsOS;

#[cfg(windows)]
pub static mut GLOBAL_SENDER: Option<tokio::sync::mpsc::Sender<char>> = None;

#[cfg(windows)]
pub unsafe extern "system" fn keyboard_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 && w_param == WM_KEYDOWN as usize {
        let kbd = *(l_param as *const KBDLLHOOKSTRUCT);

        // Basic mapping for alphabet and common keys
        let c = match kbd.vkCode {
            0x41..=0x5A => Some((kbd.vkCode as u8) as char), // A-Z
            0x30..=0x39 => Some((kbd.vkCode as u8) as char), // 0-9
            0x20 => Some(' '),                               // Space
            0x0D => Some('\n'),                              // Enter
            0x08 => Some('\u{8}'),                           // Backspace
            0x09 => Some('\t'),                              // Tab
            _ => None,
        };

        if let Some(ch) = c {
            if let Some(sender) = &GLOBAL_SENDER {
                let _ = sender.try_send(ch.to_ascii_lowercase());
            }
        }
    }
    CallNextHookEx(0, code, w_param, l_param)
}

impl OSInterface for WindowsOS {
    #[cfg(windows)]
    fn get_active_window(&self) -> Option<WindowInfo> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return None;
            }

            // Get Title
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            let title = String::from_utf16_lossy(&title_buf[..len as usize]);

            // Get Process Name
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);

            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
            let mut process_name = String::from("Unknown");

            if process_handle != 0 {
                let mut name_buf = [0u16; 260];
                let name_len = GetModuleBaseNameW(process_handle, 0, name_buf.as_mut_ptr(), 260);
                if name_len > 0 {
                    process_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
                }
                CloseHandle(process_handle);
            }

            Some(WindowInfo {
                process_name,
                title,
            })
        }
    }

    #[cfg(not(windows))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        None
    }
}
