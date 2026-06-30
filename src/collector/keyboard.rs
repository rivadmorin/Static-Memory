use tokio::sync::mpsc;
use crate::models::{InputEvent, KeyEvent, KeyType};
use chrono::Utc;
use smallvec::SmallVec;

#[cfg(target_os = "linux")]
use evdev::{Device, Key};

pub async fn run_keyboard_collector(tx: mpsc::Sender<InputEvent>) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut device) = Device::open("/dev/input/event0") {
            loop {
                if let Ok(events) = device.fetch_events() {
                    // Demonstrate SmallVec for raw event buffering as requested
                    let mut raw_buffer: SmallVec<[evdev::InputEvent; 64]> = SmallVec::new();

                    for event in events {
                        raw_buffer.push(event);

                        if raw_buffer.len() >= 64 {
                            // Process buffer if it gets full
                            raw_buffer.clear();
                        }

                        if let evdev::InputEventKind::Key(key) = event.kind() {
                            if event.value() == 1 { // Key Press
                                let key_type = match key {
                                    Key::KEY_BACKSPACE => KeyType::Backspace,
                                    Key::KEY_ENTER => KeyType::Enter,
                                    Key::KEY_SPACE => KeyType::Space,
                                    Key::KEY_TAB => KeyType::Tab,
                                    _ => KeyType::Other,
                                };
                                let _ = tx.send(InputEvent::Key(KeyEvent {
                                    key: key_type,
                                    timestamp: Utc::now(),
                                })).await;
                            }
                        }
                    }
                }
                tokio::task::yield_now().await;
            }
        }
    }
}
