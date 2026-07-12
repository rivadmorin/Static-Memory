use tuirealm::props::Alignment;
use tuirealm::tui::layout::{Rect, Layout, Constraint, Direction};
use tuirealm::tui::widgets::{Block, Borders, Paragraph, Clear};
use tuirealm::{Component, Event, MockComponent, State};
use tuirealm::command::{Cmd, CmdResult};

pub struct ExportModal;

impl MockComponent for ExportModal {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult { CmdResult::None }
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let area = centered_rect(60, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("Export Data (Ctrl+E)\n\nPress 'c' for CSV\nPress 't' for TXT\nPress 'j' for JSON\n\n[Esc to Cancel]")
                .block(Block::default().borders(Borders::ALL).title("Data Export"))
                .alignment(Alignment::Center),
            area,
        );
    }
    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> { None }
    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}
    fn state(&self) -> State { State::None }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for ExportModal {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key) => match key.code {
                tuirealm::event::Key::Esc => Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline)),
                tuirealm::event::Key::Char('c') => Some(crate::ui::app::Msg::ExportExecuted("csv".into())),
                tuirealm::event::Key::Char('t') => Some(crate::ui::app::Msg::ExportExecuted("txt".into())),
                tuirealm::event::Key::Char('j') => Some(crate::ui::app::Msg::ExportExecuted("json".into())),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct SearchModal {
    pub input: String,
}

impl MockComponent for SearchModal {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult { CmdResult::None }
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let area = centered_rect(60, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(format!("Search Term:\n\n{}\n\n[Press Enter to Search, Esc to Cancel]", self.input))
                .block(Block::default().borders(Borders::ALL).title("Search History (Ctrl+S)"))
                .alignment(Alignment::Center),
            area,
        );
    }
    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> { None }
    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}
    fn state(&self) -> State { State::None }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for SearchModal {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key) => match key.code {
                tuirealm::event::Key::Esc => Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline)),
                tuirealm::event::Key::Enter => {
                    let term = self.input.clone();
                    self.input.clear();
                    Some(crate::ui::app::Msg::SearchExecuted(term))
                },
                tuirealm::event::Key::Backspace => {
                    self.input.pop();
                    None
                }
                tuirealm::event::Key::Char(c) => {
                    self.input.push(c);
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}

pub struct SyncModal;

impl MockComponent for SyncModal {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult { CmdResult::None }
    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let area = centered_rect(60, 20, area);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new("Sync Backup to Remote (Ctrl+B)\n\nAre you sure you want to trigger a remote sync?\n\n[Press Enter to Sync, Esc to Cancel]")
                .block(Block::default().borders(Borders::ALL).title("Sync Backup"))
                .alignment(Alignment::Center),
            area,
        );
    }
    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> { None }
    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}
    fn state(&self) -> State { State::None }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for SyncModal {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key) => match key.code {
                tuirealm::event::Key::Esc => Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline)),
                tuirealm::event::Key::Enter => Some(crate::ui::app::Msg::SyncBackupExecuted),
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
