use ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{terminal::TerminalBridge, Application, Update};

// This is a minimal boilerplate for a tui-realm app
pub struct Model {
    pub app: Application<Id, Msg, Event>,
    pub quit: bool,
    pub terminal: TerminalBridge,
    pub active_tab: Id,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Id {
    Timeline,
    Detail,
    Dashboard,
    StatusBar,
}

#[derive(Debug, PartialEq)]
pub enum Msg {
    None,
    AppClose,
    SwitchTab(Id),
    UpdateAnalytics(Box<crate::storage::AnalyticsData>),
    SetIdle(bool),
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq)]
pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
}

unsafe impl Send for Event {}

impl Model {
    pub fn view(&mut self) {
        let _ = self.terminal.raw_mut().draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                .split(f.size());

            match self.active_tab {
                Id::Timeline => self.app.view(&Id::Timeline, f, chunks[0]),
                Id::Dashboard => self.app.view(&Id::Dashboard, f, chunks[0]),
                _ => {}
            }

            self.app.view(&Id::StatusBar, f, chunks[1]);
        });
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        match msg {
            Some(Msg::AppClose) => {
                self.quit = true;
                None
            }
            Some(Msg::SwitchTab(id)) => {
                self.active_tab = id;
                None
            }
            Some(Msg::UpdateAnalytics(data)) => {
                let _ = self.app.attr(
                    &Id::Dashboard,
                    tuirealm::Attribute::Custom("data"),
                    tuirealm::AttrValue::Payload(tuirealm::props::PropPayload::One(
                        tuirealm::props::PropValue::Str(
                            serde_json::to_string(data.as_ref()).unwrap(),
                        ),
                    )),
                );
                None
            }
            Some(Msg::SetIdle(idle)) => {
                let _ = self.app.attr(
                    &Id::StatusBar,
                    tuirealm::Attribute::Custom("idle"),
                    tuirealm::AttrValue::Flag(idle),
                );
                None
            }
            _ => None,
        }
    }
}
