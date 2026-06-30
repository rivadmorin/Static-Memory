use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use crate::models::{ActivityRecord, SystemMetrics};
use std::collections::VecDeque;

pub struct TuiState {
    pub current_tab: usize,
    pub timeline: VecDeque<ActivityRecord>,
    pub metrics: Option<SystemMetrics>,
    pub running: bool,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            current_tab: 0,
            timeline: VecDeque::with_capacity(100),
            metrics: None,
            running: true,
        }
    }
}

pub fn draw_ui(f: &mut Frame, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Status Bar
        ])
        .split(f.size());

    // 1. Tabs
    let titles = vec!["Timeline", "Search", "Management"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Static-Memory"))
        .select(state.current_tab)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // 2. Main Content based on Tab
    match state.current_tab {
        0 => render_timeline(f, chunks[1], state),
        1 => render_search(f, chunks[1], state),
        2 => render_management(f, chunks[1], state),
        _ => {}
    }

    // 3. Status Bar
    render_status_bar(f, chunks[2], state);
}

fn render_timeline(f: &mut Frame, area: Rect, state: &TuiState) {
    let items: Vec<ListItem> = state.timeline.iter().map(|rec| {
        let time = rec.timestamp.format("%H:%M:%S").to_string();
        let content = format!("[{}] {} - {}: {}", time, rec.app_name, rec.window_title, rec.buffer);
        ListItem::new(content)
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Activity Timeline"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(list, area);
}

fn render_search(f: &mut Frame, area: Rect, _state: &TuiState) {
    let p = Paragraph::new("Search Feature Coming Soon...")
        .block(Block::default().borders(Borders::ALL).title("Search"));
    f.render_widget(p, area);
}

fn render_management(f: &mut Frame, area: Rect, _state: &TuiState) {
    let p = Paragraph::new("Press 'C' to Clear Logs | 'E' to Export (JSON)")
        .block(Block::default().borders(Borders::ALL).title("Management"));
    f.render_widget(p, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, state: &TuiState) {
    let metrics_text = if let Some(m) = &state.metrics {
        format!(" CPU: {:.1}% | RAM: {} MB | Status: Active", m.cpu_usage, m.memory_usage_mb)
    } else {
        " Initializing metrics...".to_string()
    };

    let status_bar = Paragraph::new(metrics_text)
        .block(Block::default().borders(Borders::ALL).title("System Metrics"));
    f.render_widget(status_bar, area);
}
