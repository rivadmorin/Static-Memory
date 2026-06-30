use tuirealm::{
    terminal::TerminalBridge,
    Application,
    Update,
    EventListenerCfg,
};
use ratatui::layout::{Layout, Constraint, Direction};

// This is a minimal boilerplate for a tui-realm app
pub struct Model {
    pub app: Application<Id, Msg, Event>,
    pub quit: bool,
    pub terminal: TerminalBridge,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Id {
    Timeline,
    Detail,
    StatusBar,
}

#[derive(Debug, PartialEq)]
pub enum Msg {
    None,
    AppClose,
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq)]
pub enum Event {
    Tick,
    Key(crossterm::event::KeyEvent),
}

unsafe impl Send for Event {}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(Msg::AppClose) = msg {
            self.quit = true;
        }
        None
    }
}
