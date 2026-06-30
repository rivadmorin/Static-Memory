use crate::os::{OSInterface, WindowInfo};
#[cfg(target_os = "linux")]
use x11_dl::xlib;
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use std::ffi::CStr;

pub struct LinuxOS;

impl OSInterface for LinuxOS {
    #[cfg(target_os = "linux")]
    fn get_active_window(&self) -> Option<WindowInfo> {
        // Basic X11 implementation for window title
        // In a real scenario, we'd use XGetWindowProperty for _NET_ACTIVE_WINDOW
        // and _NET_WM_NAME or WM_CLASS for process name.

        unsafe {
            let xlib = xlib::Xlib::open().ok()?;
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() { return None; }

            let mut root_return = 0;
            let mut child_return = 0;
            let mut root_x_return = 0;
            let mut root_y_return = 0;
            let mut win_x_return = 0;
            let mut win_y_return = 0;
            let mut mask_return = 0;

            let root = (xlib.XDefaultRootWindow)(display);
            let active_window_atom = (xlib.XInternAtom)(display, "_NET_ACTIVE_WINDOW\0".as_ptr() as *const i8, xlib::False);

            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut nitems = 0;
            let mut bytes_after = 0;
            let mut prop = ptr::null_mut();

            (xlib.XGetWindowProperty)(
                display, root, active_window_atom, 0, 1, xlib::False,
                xlib::AnyPropertyType as u64, &mut actual_type, &mut actual_format,
                &mut nitems, &mut bytes_after, &mut prop
            );

            if !prop.is_null() {
                let window_id = *(prop as *const xlib::Window);
                (xlib.XFree)(prop as *mut _);

                // Now get title of window_id
                let name_atom = (xlib.XInternAtom)(display, "_NET_WM_NAME\0".as_ptr() as *const i8, xlib::False);
                let mut title_prop = ptr::null_mut();
                (xlib.XGetWindowProperty)(
                    display, window_id, name_atom, 0, 1024, xlib::False,
                    xlib::AnyPropertyType as u64, &mut actual_type, &mut actual_format,
                    &mut nitems, &mut bytes_after, &mut title_prop
                );

                if !title_prop.is_null() {
                    let title = CStr::from_ptr(title_prop as *const i8).to_string_lossy().into_owned();
                    (xlib.XFree)(title_prop as *mut _);
                    (xlib.XCloseDisplay)(display);
                    return Some(WindowInfo {
                        process_name: "LinuxApp".to_string(), // Full proc name lookup would be here
                        title,
                    });
                }
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
    use std::fs;
    use evdev::Device;

    let mut internal_kbd = None;
    let paths = fs::read_dir("/dev/input/").ok()?;

    for path in paths.flatten() {
        let path_str = path.path();
        if let Ok(mut device) = Device::open(&path_str) {
            // Check if it's a keyboard by looking at supported keys
            // KeyMask for essential keys (e.g., KEY_A, KEY_Z)
            if device.supported_keys().map_or(false, |keys| keys.contains(evdev::KeyCode::KEY_A)) {
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
