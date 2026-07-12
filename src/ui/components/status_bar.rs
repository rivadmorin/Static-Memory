use tuirealm::props::{Alignment, Color};
use tuirealm::tui::layout::Rect;
use tuirealm::tui::widgets::{Block, Paragraph};
use tuirealm::{Component, Event, MockComponent, State};
use tuirealm::command::{Cmd, CmdResult};

pub struct StatusBar {
    pub is_idle: bool,
}

impl MockComponent for StatusBar {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let (text, fg) = if self.is_idle {
            ("[IDLE] System is AFK...", Color::Red)
        } else {
            ("[ACTIVE] Logging...", Color::Green)
        };

        frame.render_widget(
            Paragraph::new(text)
                .block(Block::default())
                .style(ratatui::style::Style::default().fg(fg).add_modifier(tuirealm::props::TextModifiers::BOLD))
                .alignment(Alignment::Left),
            area,
        );
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        if let tuirealm::Attribute::Custom("idle") = attr {
            if let tuirealm::AttrValue::Flag(f) = value {
                self.is_idle = f;
            }
        }
    }

    fn state(&self) -> State {
        State::None
    }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for StatusBar {
    fn on(&mut self, _ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        None
    }
}
