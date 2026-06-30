#[cfg(target_os = "linux")]
use evdev::Device;
#[cfg(target_os = "linux")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use crate::os::linux::detect_keyboard_device;

#[cfg(target_os = "linux")]
pub async fn start_evdev_collector(
    device_path: Option<String>,
    engine: std::sync::Arc<tokio::sync::RwLock<crate::engine::Engine<crate::os::linux::LinuxOS>>>
) {
    loop {
        let path = device_path.clone().or_else(detect_keyboard_device);

        if let Some(path) = path {
            match Device::open(&path) {
                Ok(device) => {
                    if let Ok(mut event_stream) = device.into_event_stream() {
                        println!("Listening on device: {}", event_stream.device().name().unwrap_or("Unknown"));

                        loop {
                            match event_stream.next_event().await {
                                Ok(event) => {
                                    if event.event_type() == evdev::EventType::KEY && event.value() == 1 {
                                        if let Some(key) = event_code_to_char(event.code()) {
                                            let mut engine_lock = engine.write().await;
                                            engine_lock.handle_key(key).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Device error: {}. Attempting re-scan...", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open device {}: {}", path, e);
                }
            }
        } else {
            eprintln!("No keyboard device detected. Retrying in 5 seconds...");
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[cfg(target_os = "linux")]
fn event_code_to_char(code: u16) -> Option<char> {
    match code {
        2..=11 => Some((b'0' + ((code + 9) % 10) as u8) as char), // 1-9, 0
        16 => Some('q'), 17 => Some('w'), 18 => Some('e'), 19 => Some('r'), 20 => Some('t'),
        21 => Some('y'), 22 => Some('u'), 23 => Some('i'), 24 => Some('o'), 25 => Some('p'),
        30 => Some('a'), 31 => Some('s'), 32 => Some('d'), 33 => Some('f'), 34 => Some('g'),
        35 => Some('h'), 36 => Some('j'), 37 => Some('k'), 38 => Some('l'),
        44 => Some('z'), 45 => Some('x'), 46 => Some('c'), 47 => Some('v'), 48 => Some('b'),
        49 => Some('n'), 50 => Some('m'),
        57 => Some(' '), // Space
        28 => Some('\n'), // Enter
        14 => Some('\u{8}'), // Backspace
        15 => Some('\t'), // Tab
        _ => None,
    }
}
