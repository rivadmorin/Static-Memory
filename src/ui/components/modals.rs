use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::Alignment;
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::{Block, Borders, Clear, Paragraph};
use tuirealm::{Component, Event, MockComponent, State};

pub struct ExportModal;

impl MockComponent for ExportModal {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let area = centered_rect(60, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("Export Data (Ctrl+E)\n\nFormat: CSV / TXT\n\n[Press Enter to Export, Esc to Cancel]")
                .block(Block::default().borders(Borders::ALL).title("Data Export"))
                .alignment(Alignment::Center),
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

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for ExportModal {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key) => match key.code {
                tuirealm::event::Key::Esc => {
                    Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline))
                }
                tuirealm::event::Key::Enter => {
                    Some(crate::ui::app::Msg::ExportExecuted("csv".into()))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
