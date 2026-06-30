use tuirealm::tui::layout::Rect;
use tuirealm::props::Alignment;
use tuirealm::tui::widgets::{Block, BorderType, Borders, Paragraph};
use tuirealm::{Component, Event, MockComponent, State};
use tuirealm::command::{Cmd, CmdResult};

pub mod dashboard;
pub mod status_bar;

pub struct TimelineComponent;

impl MockComponent for TimelineComponent {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new("Real-time Timeline Activity...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Timeline"),
                )
                .alignment(Alignment::Left),
            area,
        );
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        State::None
    }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for TimelineComponent {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key_event) => match key_event.code {
                tuirealm::event::Key::Char('q') => Some(crate::ui::app::Msg::AppClose),
                tuirealm::event::Key::Tab => Some(crate::ui::app::Msg::SwitchTab(crate::ui::app::Id::Dashboard)),
                _ => None,
            },
            _ => None,
        }
    }
}
