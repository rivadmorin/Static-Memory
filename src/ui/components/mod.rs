use tuirealm::tui::layout::Rect;
use tuirealm::props::Alignment;
use tuirealm::tui::widgets::{Block, BorderType, Borders, Paragraph};
use tuirealm::{Component, Event, MockComponent, State};
use tuirealm::command::{Cmd, CmdResult};

pub mod dashboard;
pub mod status_bar;
pub mod modals;

use crate::models::LogEntry;

pub struct TimelineComponent {
    pub entries: Vec<LogEntry>,
}

impl TimelineComponent {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
}

impl MockComponent for TimelineComponent {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let content: Vec<String> = self.entries.iter()
            .map(|e| format!("[{}] {}: {}", e.timestamp.format("%H:%M:%S"), e.app_name, e.buffer))
            .collect();

        frame.render_widget(
            Paragraph::new(content.join("\n"))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Timeline (Last 50 events)"),
                )
                .alignment(Alignment::Left),
            area,
        );
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        if let tuirealm::Attribute::Custom("data") = attr {
            if let tuirealm::AttrValue::Payload(tuirealm::props::PropPayload::One(tuirealm::props::PropValue::Str(s))) = value {
                if let Ok(data) = serde_json::from_str::<Vec<LogEntry>>(&s) {
                    self.entries = data;
                }
            }
        }
    }

    fn state(&self) -> State {
        State::None
    }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for TimelineComponent {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key_event) => match key_event.code {
                tuirealm::event::Key::Char('q') => Some(crate::ui::app::Msg::AppClose),
                tuirealm::event::Key::Tab => Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Dashboard)),
                tuirealm::event::Key::Char('e') if key_event.modifiers.contains(tuirealm::event::KeyModifiers::CONTROL) => Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::ExportModal)),
                _ => None,
            },
            _ => None,
        }
    }
}
