#!/bin/bash
# 1. Models
sed -i 's/pub idle_threshold_seconds: u64,/pub idle_threshold_seconds: u64,\n    pub flush_threshold_chars: usize,/g' src/models/mod.rs
sed -i 's/idle_threshold_seconds: 180,/idle_threshold_seconds: 180,\n                flush_threshold_chars: 512,/g' src/models/mod.rs

# 2. Buffer
sed -i '/pub fn is_empty(&self) -> bool {/i \
    pub fn len(&self) -> usize {\
        self.buffer.len()\
    }\
' src/engine/buffer.rs

# 3. Engine
sed -i '/match key {/i \
        if self.buffer.len() >= self.config.engine.flush_threshold_chars {\
            self.flush().await;\
        }\
' src/engine/mod.rs

# 4. Linux OS
cat << 'INNER_EOF' > src/os/linux.rs
use crate::os::{OSInterface, WindowInfo};
#[cfg(target_os = "linux")]
use std::ffi::CStr;
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use x11_dl::xlib;
#[cfg(target_os = "linux")]
use std::fs;

pub struct LinuxOS;

impl OSInterface for LinuxOS {
    #[cfg(target_os = "linux")]
    fn get_active_window(&self) -> Option<WindowInfo> {
        unsafe {
            let xlib = match xlib::Xlib::open() {
                Ok(x) => x,
                Err(_) => return None,
            };
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                return None;
            }

            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut nitems = 0;
            let mut bytes_after = 0;
            let mut prop = ptr::null_mut();

            let root = (xlib.XDefaultRootWindow)(display);
            let active_window_atom = (xlib.XInternAtom)(
                display,
                c"_NET_ACTIVE_WINDOW".as_ptr() as *const i8,
                xlib::False,
            );

            (xlib.XGetWindowProperty)(
                display,
                root,
                active_window_atom,
                0,
                1,
                xlib::False,
                xlib::AnyPropertyType as u64,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop,
            );

            if !prop.is_null() {
                let window_id = *(prop as *const xlib::Window);
                (xlib.XFree)(prop as *mut _);

                // 1. Get Title
                let name_atom = (xlib.XInternAtom)(
                    display,
                    c"_NET_WM_NAME".as_ptr() as *const i8,
                    xlib::False,
                );
                let mut title_prop = ptr::null_mut();
                (xlib.XGetWindowProperty)(
                    display,
                    window_id,
                    name_atom,
                    0,
                    1024,
                    xlib::False,
                    xlib::AnyPropertyType as u64,
                    &mut actual_type,
                    &mut actual_format,
                    &mut nitems,
                    &mut bytes_after,
                    &mut title_prop,
                );

                let mut title = "Linux Window".to_string();
                if !title_prop.is_null() {
                    title = CStr::from_ptr(title_prop as *const i8)
                        .to_string_lossy()
                        .into_owned();
                    (xlib.XFree)(title_prop as *mut _);
                }

                // 2. Get PID and Process Name
                let pid_atom = (xlib.XInternAtom)(
                    display,
                    c"_NET_WM_PID".as_ptr() as *const i8,
                    xlib::False,
                );
                let mut pid_prop = ptr::null_mut();
                (xlib.XGetWindowProperty)(
                    display,
                    window_id,
                    pid_atom,
                    0,
                    1,
                    xlib::False,
                    xlib::AnyPropertyType as u64,
                    &mut actual_type,
                    &mut actual_format,
                    &mut nitems,
                    &mut bytes_after,
                    &mut pid_prop,
                );

                let mut process_name = "LinuxApp".to_string();
                if !pid_prop.is_null() && nitems > 0 {
                    let pid = *(pid_prop as *const u32);
                    (xlib.XFree)(pid_prop as *mut _);

                    if let Ok(comm) = fs::read_to_string(format!("/proc/{}/comm", pid)) {
                        process_name = comm.trim().to_string();
                    }
                }

                (xlib.XCloseDisplay)(display);
                return Some(WindowInfo {
                    process_name,
                    title,
                });
            }

            (xlib.XCloseDisplay)(display);

            Some(WindowInfo {
                process_name: "LinuxApp".to_string(),
                title: "Linux Window".to_string(),
            })
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        None
    }
}

#[cfg(target_os = "linux")]
pub fn detect_keyboard_device() -> Option<String> {
    use evdev::Device;
    use std::fs;

    let mut internal_kbd = None;
    let paths = fs::read_dir("/dev/input/").ok()?;

    for path in paths.flatten() {
        let path_str = path.path();
        if let Ok(device) = Device::open(&path_str) {
            // Check if it's a keyboard by looking at supported keys
            // KeyMask for essential keys (e.g., KEY_A, KEY_Z)
            if device
                .supported_keys()
                .is_some_and(|keys| keys.contains(evdev::KeyCode::KEY_A))
            {
                let name = device.name().unwrap_or("Unknown").to_lowercase();
                let is_usb = name.contains("usb") || name.contains("external");

                if is_usb {
                    return Some(path_str.to_string_lossy().into_owned());
                } else if internal_kbd.is_none() {
                    internal_kbd = Some(path_str.to_string_lossy().into_owned());
                }
            }
        }
    }
    internal_kbd
}
INNER_EOF

# 5. DB Queries
sed -i "s/length(buffer) - length(replace(buffer, ' ', '')) + 1/CASE WHEN trim(buffer) = '' THEN 0 ELSE length(trim(buffer)) - length(replace(trim(buffer), ' ', '')) + 1 END/g" src/storage/db.rs

# 6. Clippy fixes (no manual_flatten to avoid block issues for now)
sed -i 's/std::io::Error::new(std::io::ErrorKind::Other, e.to_string())/std::io::Error::other(e.to_string())/g' src/storage/db.rs
sed -i 's/UpdateAnalytics(crate::storage::AnalyticsData)/UpdateAnalytics(Box<crate::storage::AnalyticsData>)/g' src/ui/app.rs
sed -i 's/serde_json::to_string(&data).unwrap()/serde_json::to_string(data.as_ref()).unwrap()/g' src/ui/app.rs
