#[cfg(target_os = "linux")]
use evdev::Device;
#[cfg(target_os = "linux")]
use std::error::Error;

#[cfg(target_os = "linux")]
pub fn start_evdev_collector(device_path: &str) -> Result<(), Box<dyn Error>> {
    let mut device = Device::open(device_path)?;
    println!("Listening on device: {}", device.name().unwrap_or("Unknown"));

    loop {
        for event in device.fetch_events()? {
            if event.event_type() == evdev::EventType::KEY {
                // event.code(), event.value() == 1 (down)
                // In a real implementation, send to channel
            }
        }
    }
}
