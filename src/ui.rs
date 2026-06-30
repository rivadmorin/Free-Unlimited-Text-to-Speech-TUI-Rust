use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Gauge},
    Frame,
};
use crate::app::{App, Focus};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ])
        .split(chunks[0]);

    // Sidebar
    let transcripts: Vec<ListItem> = if app.transcripts_list.is_empty() {
        vec![ListItem::new("No transcripts found")]
    } else {
        app.transcripts_list
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let name = path.file_name().map_or("Unknown".to_string(), |n| n.to_string_lossy().into_owned());
                let style = if i == app.selected_index && matches!(app.focus, Focus::Sidebar) {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(name).style(style)
            })
            .collect()
    };

    let sidebar_block = Block::default()
        .borders(Borders::ALL)
        .title(" Saved Transcripts ")
        .border_style(if matches!(app.focus, Focus::Sidebar) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let sidebar_list = List::new(transcripts).block(sidebar_block);
    f.render_widget(sidebar_list, main_chunks[0]);

    // Main Panel
    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" Live Dictation ")
        .border_style(if matches!(app.focus, Focus::Main) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let content_to_display = match app.focus {
        Focus::Main => app.transcript.as_str(),
        Focus::Sidebar => app.selected_content.as_str(),
    };

    let main_panel = Paragraph::new(content_to_display)
        .block(main_block)
        .wrap(Wrap { trim: true });
    f.render_widget(main_panel, main_chunks[1]);

    // VU Meter & Status
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(chunks[1]);

    let vu_title = if app.audio_active { " VU Meter " } else { " VU Meter [OFF] " };
    let vu_meter = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(vu_title))
        .gauge_style(Style::default().fg(if app.amplitude > 0.8 { Color::Red } else if app.amplitude > 0.5 { Color::Yellow } else { Color::Green }))
        .ratio(app.amplitude.min(1.0) as f64)
        .label("");
    f.render_widget(vu_meter, bottom_chunks[0]);

    let status_style = if app.is_connected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };
    let connection_status = if app.is_connected { "[CONNECTED]" } else { "[DISCONNECTED]" };
    let error_text = app.last_error.as_ref().map(|e| format!(" | ERROR: {}", e)).unwrap_or_default();
    let status_text = format!("{} | {}{}", connection_status, app.status_message, error_text);
    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(status_style)
        .wrap(Wrap { trim: true });
    f.render_widget(status_bar, bottom_chunks[1]);

    // Help bar
    let help_text = "Space: Start/Pause | S: Save | Y: Copy | Q: Quit | h/l: Switch Focus | j/k: Navigate";
    let help_bar = Paragraph::new(help_text).style(Style::default().add_modifier(Modifier::DIM));
    f.render_widget(help_bar, chunks[2]);
}
