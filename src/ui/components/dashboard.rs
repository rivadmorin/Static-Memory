use crate::storage::AnalyticsData;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{Alignment, Color};
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::{BarChart, Block, Borders, Paragraph};
use tuirealm::{Component, Event, MockComponent, State};

pub struct DashboardComponent {
    pub data: Option<AnalyticsData>,
}

impl Default for DashboardComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardComponent {
    pub fn new() -> Self {
        Self { data: None }
    }
}

impl MockComponent for DashboardComponent {
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }

    fn view(&mut self, frame: &mut tuirealm::Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // Top 5 Apps
        let top_apps_content = if let Some(data) = &self.data {
            data.top_apps
                .iter()
                .map(|(app, count)| format!("{:<20} | {}", app, count))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "Loading analytics...".to_string()
        };

        let top_apps_title = if let Some(data) = &self.data {
            format!(
                "Top 5 Active Apps (24h) - Total Words Today: {}",
                data.total_words
            )
        } else {
            "Top 5 Active Apps (24h)".to_string()
        };

        frame.render_widget(
            Paragraph::new(top_apps_content)
                .block(Block::default().borders(Borders::ALL).title(top_apps_title))
                .alignment(Alignment::Left),
            chunks[0],
        );

        // Hourly Activity Chart
        if let Some(data) = &self.data {
            let labels: Vec<String> = (0..24).map(|h| format!("{:02}", h)).collect();
            let chart_data: Vec<(&str, u64)> = data
                .hourly_activity
                .iter()
                .enumerate()
                .map(|(i, (_h, c))| (labels[i].as_str(), *c as u64))
                .collect();

            frame.render_widget(
                BarChart::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Hourly Activity (24h)"),
                    )
                    .data(&chart_data)
                    .bar_width(3)
                    .bar_gap(1)
                    .value_style(ratatui::style::Style::default().fg(Color::Yellow))
                    .label_style(ratatui::style::Style::default().fg(Color::White)),
                chunks[1],
            );
        }
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        if let tuirealm::Attribute::Custom("data") = attr {
            if let tuirealm::AttrValue::Payload(tuirealm::props::PropPayload::One(
                tuirealm::props::PropValue::Str(s),
            )) = value
            {
                if let Ok(data) = serde_json::from_str::<AnalyticsData>(&s) {
                    self.data = Some(data);
                }
            }
        }
    }

    fn state(&self) -> State {
        State::None
    }
}

impl Component<crate::ui::app::Msg, crate::ui::app::Event> for DashboardComponent {
    fn on(&mut self, ev: Event<crate::ui::app::Event>) -> Option<crate::ui::app::Msg> {
        match ev {
            Event::Keyboard(key_event) => match key_event.code {
                tuirealm::event::Key::Char('q') => Some(crate::ui::app::Msg::AppClose),
                tuirealm::event::Key::Tab => {
                    Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::Timeline))
                }
                tuirealm::event::Key::Char('e')
                    if key_event
                        .modifiers
                        .contains(tuirealm::event::KeyModifiers::CONTROL) =>
                {
                    Some(crate::ui::app::Msg::SwitchTab(crate::ui::Id::ExportModal))
                }
                _ => None,
            },
            _ => None,
        }
    }
}
