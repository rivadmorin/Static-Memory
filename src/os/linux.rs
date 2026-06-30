#[cfg(target_os = "linux")]
use x11_dl::xlib;
#[cfg(target_os = "linux")]
use std::ptr;
#[cfg(target_os = "linux")]
use std::ffi::CStr;

#[cfg(target_os = "linux")]
pub struct LinuxTracker {
    xlib: xlib::Xlib,
    display: *mut xlib::Display,
}

#[cfg(target_os = "linux")]
impl LinuxTracker {
    pub fn new() -> Option<Self> {
        let xlib = xlib::Xlib::open().ok()?;
        let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
        if display.is_null() {
            return None;
        }
        Some(Self { xlib, display })
    }

    pub fn get_active_window(&self) -> Option<(String, String)> {
        unsafe {
            let mut focus_return: xlib::Window = 0;
            let mut revert_to_return: i32 = 0;
            (self.xlib.XGetInputFocus)(self.display, &mut focus_return, &mut revert_to_return);

            if focus_return == 0 {
                return None;
            }

            // Get Window Title
            let mut name_ptr: *mut i8 = ptr::null_mut();
            (self.xlib.XFetchName)(self.display, focus_return, &mut name_ptr);
            let title = if !name_ptr.is_null() {
                let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                (self.xlib.XFree)(name_ptr as *mut _);
                name
            } else {
                "Unknown".to_string()
            };

            // Get App Name (WM_CLASS)
            let app_name = self.get_wm_class(focus_return).unwrap_or_else(|| "Unknown".to_string());

            Some((app_name, title))
        }
    }

    fn get_wm_class(&self, window: xlib::Window) -> Option<String> {
        unsafe {
            let mut class_hint: xlib::XClassHint = std::mem::zeroed();
            if (self.xlib.XGetClassHint)(self.display, window, &mut class_hint) != 0 {
                let name = if !class_hint.res_name.is_null() {
                    Some(CStr::from_ptr(class_hint.res_name).to_string_lossy().into_owned())
                } else {
                    None
                };
                if !class_hint.res_name.is_null() { (self.xlib.XFree)(class_hint.res_name as *mut _); }
                if !class_hint.res_class.is_null() { (self.xlib.XFree)(class_hint.res_class as *mut _); }
                name
            } else {
                None
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for LinuxTracker {
    fn drop(&mut self) {
        unsafe {
            (self.xlib.XCloseDisplay)(self.display);
        }
    }
}

// Ensure the code can at least compile on other platforms even if not used
#[cfg(not(target_os = "linux"))]
pub struct LinuxTracker;
#[cfg(not(target_os = "linux"))]
impl LinuxTracker {
    pub fn new() -> Option<Self> { None }
    pub fn get_active_window(&self) -> Option<(String, String)> { None }
}
