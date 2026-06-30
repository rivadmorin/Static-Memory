use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::tui::layout::Rect;
use tuirealm::tui::widgets::{Block, BorderType, Borders, Paragraph};
use tuirealm::{Component, Event, MockComponent, NoUserEvent, State};
use tuirealm::command::{Cmd, CmdResult};

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

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        State::None
    }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for TimelineComponent {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        None
    }
}
