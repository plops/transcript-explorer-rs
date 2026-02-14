use crate::app::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Layout: header(3) + filter(3) + list(min) + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Header ──
    let header_text = format!(
        " Transcript Explorer   [{} entries]",
        app.filtered_indices.len()
    );
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(header, chunks[0]);

    // ── Filter bar ──
    let filter_style = match app.input_mode {
        InputMode::Editing => Style::default().fg(Color::Yellow),
        InputMode::Normal => Style::default().fg(Color::DarkGray),
    };
    let filter_label = if app.input_mode == InputMode::Editing {
        " 🔍 Filter (Enter to apply, Esc to cancel): "
    } else {
        " 🔍 Filter (/): "
    };
    let filter_text = format!("{}{}", filter_label, app.filter);
    let filter_bar = Paragraph::new(filter_text)
        .style(filter_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(filter_style)
                .title(" Search "),
        );
    frame.render_widget(filter_bar, chunks[1]);

    // Set cursor position when editing
    if app.input_mode == InputMode::Editing {
        let cursor_x = chunks[1].x + filter_label.len() as u16 + app.filter.len() as u16;
        let cursor_y = chunks[1].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    // ── List ──
    let items: Vec<ListItem> = app
        .list_items
        .iter()
        .map(|item| {
            let emb_indicator = if item.has_embedding { "●" } else { "○" };
            let preview = item
                .summary_preview
                .lines()
                .next()
                .unwrap_or("")
                .trim();
            let line = Line::from(vec![
                Span::styled(
                    format!("{:>5} ", item.identifier),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{} ", emb_indicator),
                    Style::default().fg(if item.has_embedding {
                        Color::Green
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::raw(truncate_str(preview, (area.width as usize).saturating_sub(30))),
                Span::styled(
                    format!("  ${:.3}", item.cost),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let page_info = format!(
        " {}-{} of {} ",
        if app.filtered_indices.is_empty() { 0 } else { app.list_offset + 1 },
        app.list_offset + app.list_items.len(),
        app.filtered_indices.len()
    );

    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Transcripts ")
                .title_bottom(Line::from(page_info).alignment(Alignment::Right)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.list_selected));
    frame.render_stateful_widget(list_widget, chunks[2], &mut list_state);

    // ── Status bar ──
    let status_line = Line::from(vec![
        Span::styled(
            " ↑↓",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            "/",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Search  "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Detail  "),
        Span::styled(
            "s",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Similar  "),
        Span::styled(
            "?",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Help  "),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit  "),
        Span::styled(
            &app.status_msg,
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let status_bar = Paragraph::new(status_line);
    frame.render_widget(status_bar, chunks[3]);
}

/// Truncate a string to `max_width` characters, adding "…" if truncated.
pub fn truncate_str(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        let mut result: String = s.chars().take(max_width.saturating_sub(1)).collect();
        result.push('…');
        result
    }
}
