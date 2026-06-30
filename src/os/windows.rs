#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, SetWindowsHookExW, CallNextHookEx,
    GetMessageW, WH_KEYBOARD_LL, WM_KEYDOWN, KBDLLHOOKSTRUCT
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
#[cfg(windows)]
use windows_sys::Win32::System::ProcessStatus::GetModuleBaseNameW;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
#[cfg(windows)]
use windows_sys::Win32::Foundation::CloseHandle;

use crate::os::{OSInterface, WindowInfo};

pub struct WindowsOS;

#[cfg(windows)]
pub unsafe extern "system" fn keyboard_proc(code: i32, w_param: usize, l_param: isize) -> isize {
    if code >= 0 && w_param == WM_KEYDOWN as usize {
        let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
        // In a real implementation, we would send this to a channel
        // println!("Key pressed: {}", kbd.vkCode);
    }
    CallNextHookEx(0, code, w_param, l_param)
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
