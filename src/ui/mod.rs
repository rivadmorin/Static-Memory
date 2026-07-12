pub mod app;
pub mod components;

use tuirealm::{Application, EventListenerCfg};
use tuirealm::terminal::TerminalBridge;
use crate::ui::app::Model;
use crate::ui::components::{TimelineComponent, dashboard::DashboardComponent, status_bar::StatusBar};
use std::time::Duration;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Id {
    Timeline,
    Dashboard,
    StatusBar,
    ExportModal,
    SearchModal,
    SyncModal,
}

pub fn setup_app() -> Model {
    let mut app = Application::init(
        EventListenerCfg::default()
            .tick_interval(Duration::from_millis(100))
    );

    app.mount(Id::Timeline, Box::new(TimelineComponent::new()), vec![]).expect("Failed to mount Timeline");
    app.mount(Id::Dashboard, Box::new(DashboardComponent::new()), vec![]).expect("Failed to mount Dashboard");
    app.mount(Id::StatusBar, Box::new(StatusBar { is_idle: false }), vec![]).expect("Failed to mount StatusBar");
    app.mount(Id::ExportModal, Box::new(crate::ui::components::modals::ExportModal), vec![]).expect("Failed to mount ExportModal");
    app.mount(Id::SearchModal, Box::new(crate::ui::components::modals::SearchModal::default()), vec![]).expect("Failed to mount SearchModal");
    app.mount(Id::SyncModal, Box::new(crate::ui::components::modals::SyncModal), vec![]).expect("Failed to mount SyncModal");

    Model {
        app,
        quit: false,
        terminal: TerminalBridge::new().expect("Failed to init terminal"),
        active_tab: Id::Timeline,
    }
}
