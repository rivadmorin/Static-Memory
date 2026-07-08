use crate::storage::AnalyticsData;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{Alignment, Color};
use tuirealm::tui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::tui::widgets::{BarChart, Block, Borders, Paragraph, Wrap};
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
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[0]);

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
                "Top Apps & Window (24h) | Total Entries: {}",
                data.total_entries
            )
        } else {
            "Top Apps & Window (24h)".to_string()
        };

        let active_windows_content = if let Some(data) = &self.data {
            data.most_active_window_titles
                .iter()
                .map(|(title, count)| {
                    format!(
                        "{:<20} | {}",
                        title.chars().take(20).collect::<String>(),
                        count
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "".to_string()
        };

        let combined_top = format!(
            "Apps:\n{}\n\nWindows:\n{}",
            top_apps_content, active_windows_content
        );

        frame.render_widget(
            Paragraph::new(combined_top)
                .block(Block::default().borders(Borders::ALL).title(top_apps_title))
                .alignment(Alignment::Left),
            top_chunks[0],
        );

        // Stats Summary
        let stats_content = if let Some(data) = &self.data {
            format!(
                "Total Words (Today): {}\nTotal Characters (Today): {}\nAvg Words/Entry: {:.2}\nMost Productive Hour: {:02}:00\nBusiest Day: {}\nLongest Session (Approx): {} entries",
                data.total_words,
                data.total_characters,
                data.average_words_per_entry,
                data.most_productive_hour,
                data.busiest_day_of_week,
                data.longest_active_session
            )
        } else {
            "Loading stats...".to_string()
        };

        frame.render_widget(
            Paragraph::new(stats_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Summary Stats"),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true }),
            top_chunks[1],
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
                main_chunks[1],
            );
        }

        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_chunks[2]);

        // Daily trend
        let trend_content = if let Some(data) = &self.data {
            data.daily_activity_trend
                .iter()
                .map(|(day, count)| format!("{:<12} | {}", day, count))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            "".to_string()
        };

        frame.render_widget(
            Paragraph::new(trend_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Daily Activity Trend (7 Days)"),
                )
                .alignment(Alignment::Left),
            bottom_chunks[0],
        );

        // Recent Apps Heatmap Text
        let heatmap_content = if let Some(data) = &self.data {
            let recent_apps = data.recent_apps.join(", ");
            let top_heatmap = data
                .most_used_app_heatmap
                .iter()
                .map(|(app, hour, count)| format!("{:<10} @ {:02}:00 -> {}", app, hour, count))
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                "Recent Apps:\n{}\n\nHotspots:\n{}",
                recent_apps, top_heatmap
            )
        } else {
            "".to_string()
        };

        frame.render_widget(
            Paragraph::new(heatmap_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Recent & Hotspots (24h)"),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true }),
            bottom_chunks[1],
        );
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
                    Some(crate::ui::app::Msg::SwitchTab(crate::ui::app::Id::Timeline))
                }
                _ => None,
            },
            _ => None,
        }
    }
}
