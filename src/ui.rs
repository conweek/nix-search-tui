use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};

use crate::app::{App, CurrScreen, Search};

pub fn ui(frame: &mut Frame, app: &App) {
    match app.current_screen {
        CurrScreen::Searching => draw_searching(frame, app),
        CurrScreen::DisplayResults => draw_results(frame, app),
        CurrScreen::Both => draw_both(frame, app),
        CurrScreen::Detail => draw_detail(frame, app),
    }
}

// Default screen drawing just the search bar and tab
fn draw_searching(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Center the search bar vertically and horizontally
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    let tab_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(vertical[2]);

    let search = Paragraph::new(app.search_option.as_str())
        .block(Block::default().borders(Borders::ALL).title("Search"));
    frame.render_widget(search, horizontal[1]);

    // Cursor at end of input
    frame.set_cursor_position((
        horizontal[1].x + app.search_option.len() as u16 + 1,
        horizontal[1].y + 1,
    ));

    draw_tabs(frame, app, tab_horizontal[1]);
}

// Draw just results pane
fn draw_results(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Min(1),
        ])
        .split(frame.area());

    draw_tabs(frame, app, chunks[0]);
    draw_result_list(frame, app, chunks[1]);
}

// Draws search + results panes
fn draw_both(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Length(3),
            Constraint::Min(1), 
        ])
        .split(frame.area());

    let search = Paragraph::new(app.search_option.as_str())
        .block(Block::default().borders(Borders::ALL).title("Search"));
    frame.render_widget(search, chunks[0]);

    // Cursor at end of input
    frame.set_cursor_position((
        chunks[0].x + app.search_option.len() as u16 + 1,
        chunks[0].y + 1,
    ));

    draw_tabs(frame, app, chunks[1]);

    draw_result_list(frame, app, chunks[2]);
}

fn draw_detail(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Min(1),
        ])
        .split(frame.area());

    draw_tabs(frame, app, chunks[0]);

    // Parse the detail text into styled lines
    let mut lines: Vec<Line> = Vec::new();
    for (i, line) in app.detail.lines().enumerate() {
        if i == 0 {
            // Option name as bold title
            lines.push(Line::from(Span::styled(
                line.trim().to_string(),
                Style::default()
                    .fg(Color::Rgb(245, 194, 231))
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        } else if let Some(stripped) = line.trim_start().strip_prefix("Type:") {
            lines.push(Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(stripped.trim()),
            ]));
        } else if let Some(stripped) = line.trim_start().strip_prefix("Default:") {
            lines.push(Line::from(vec![
                Span::styled("Default: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(stripped.trim()),
            ]));
        } else if let Some(stripped) = line.trim_start().strip_prefix("Example:") {
            lines.push(Line::from(vec![
                Span::styled("Example: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(stripped.trim()),
            ]));
        } else if let Some(stripped) = line.trim_start().strip_prefix("Declared by:") {
            lines.push(Line::from(vec![
                Span::styled("Declared by: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(stripped.trim()),
            ]));
        } else {
            lines.push(Line::from(line.trim().to_string()));
        }
    }

    let detail = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Detail (Esc to go back)"))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(detail, chunks[1]);
}

// Draws the tabs pane
fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Package", "Configuration", "Home Manager"];
    let selected = match app.search_choice {
        Search::Package => 0,
        Search::Configuration => 1,
        Search::HomeConfiguration => 2,
    };
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Mode"))
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(245, 194, 231))
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

// Draws the results pane
fn draw_result_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let is_selected = i == app.selected_result;
            let line = if r == "Home manager not found" {
                Line::from(Span::styled(
                    r.as_str(),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ))
            } else if let Some((name, desc)) = r.split_once(" - ") {
                let name_style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(vec![
                    Span::styled(name, name_style),
                    Span::raw(" - "),
                    Span::styled(desc, Style::default().add_modifier(Modifier::ITALIC)),
                ])
            } else {
                let style = if is_selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                Line::from(Span::styled(r.as_str(), style))
            };
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Results"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    state.select(Some(app.selected_result));
    frame.render_stateful_widget(list, area, &mut state);
}
