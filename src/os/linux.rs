use crate::os::{OSInterface, WindowInfo};
#[cfg(all(target_os = "linux", feature = "x11"))]
use std::ffi::CStr;
#[cfg(all(target_os = "linux", feature = "x11"))]
use std::ptr;
#[cfg(all(target_os = "linux", feature = "x11"))]
use x11_dl::xlib;

pub struct LinuxOS;

impl OSInterface for LinuxOS {
    #[cfg(all(target_os = "linux", feature = "x11"))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        // Basic X11 implementation for window title
        // In a real scenario, we'd use XGetWindowProperty for _NET_ACTIVE_WINDOW
        // and _NET_WM_NAME or WM_CLASS for process name.

        unsafe {
            let xlib = xlib::Xlib::open().ok()?;
            let display = (xlib.XOpenDisplay)(ptr::null());
            if display.is_null() { return None; }

            let _root_return = 0;
            let _child_return = 0;
            let _root_x_return = 0;
            let _root_y_return = 0;
            let _win_x_return = 0;
            let _win_y_return = 0;
            let _mask_return = 0;

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
                        process_name: "LinuxApp".into(), // Full proc name lookup would be here
                        title: title.into(),
                    });
                }
            }

            (xlib.XCloseDisplay)(display);

            Some(WindowInfo {
                process_name: "LinuxApp".into(),
                title: "Linux Window".into(),
            })
        }
    }

    #[cfg(not(all(target_os = "linux", feature = "x11")))]
    fn get_active_window(&self) -> Option<WindowInfo> {
        None
    }
}

#[cfg(all(target_os = "linux", feature = "evdev_support"))]
pub fn detect_keyboard_devices() -> Vec<String> {
    use evdev::Device;
    use std::fs;

    let mut keyboards = Vec::new();
    let paths = match fs::read_dir("/dev/input/") {
        Ok(paths) => paths,
        Err(_) => return keyboards,
    };

    for path in paths.flatten() {
        let path_str = path.path();
        if let Ok(device) = Device::open(&path_str) {
            // Check if it's a keyboard by looking at supported keys
            // KeyMask for essential keys (e.g., KEY_A, KEY_Z)
            if device
                .supported_keys()
                .is_some_and(|keys| keys.contains(evdev::KeyCode::KEY_A))
            {
                keyboards.push(path_str.to_string_lossy().into_owned());
            }
        }
    }
    keyboards
}
