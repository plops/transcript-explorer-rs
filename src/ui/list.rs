use crate::app::{self, App, InputMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    let preview_height = if area.height > 60 {
        Constraint::Percentage(50)
    } else {
        Constraint::Length(10)
    };

    // Layout: header(3) + filter(3) + list(min) + preview(?) + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            preview_height,
            Constraint::Length(1),
        ])
        .split(area);

    // ── Header ──
    let entries_count = if app.filter.is_empty() {
        app.all_items.len()
    } else {
        app.filtered_indices.len()
    };
    
    let header_text = format!(
        " Transcript Explorer   [{} entries in {} groups]",
        entries_count,
        app.grouped_items.len()
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
        " 🔍 Filter (Esc to finish): "
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
    let mut items = Vec::new();
    for group in &app.list_items {
        if group.items.is_empty() {
            continue;
        }

        let first = &group.items[0];
        let emb_indicator = if first.has_embedding { "●" } else { "○" };
        let title = app::get_display_title(&first.summary);
        
        let mut line_spans = vec![
            Span::styled(
                format!("{:>5} ", first.identifier),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("{} ", emb_indicator),
                Style::default().fg(if first.has_embedding {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
        ];

        if !group.expanded && group.items.len() > 1 {
            line_spans.push(Span::styled(
                format!("[+{}] ", group.items.len() - 1),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ));
        } else if group.expanded {
            line_spans.push(Span::styled(
                "[-] ",
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ));
        }

        line_spans.push(Span::raw(truncate_str(&title, (area.width as usize).saturating_sub(40))));
        
        line_spans.push(Span::styled(
            format!("  ${:.3}", first.cost),
            Style::default().fg(Color::Yellow),
        ));

        items.push(ListItem::new(Line::from(line_spans)));

        // If expanded, show other items or a marker
        if group.expanded {
            for sub_item in group.items.iter().skip(1) {
                let sub_line = Line::from(vec![
                    Span::raw("      "), // padding
                    Span::styled(
                        format!("⤷ {:>5} ", sub_item.identifier),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" (duplicate summary)"),
                ]);
                items.push(ListItem::new(sub_line));
            }
        }
    }

    let page_info = format!(
        " Group {}-{} of {} ",
        if app.grouped_items.is_empty() { 0 } else { app.list_offset + 1 },
        app.list_offset + app.list_items.len(),
        app.grouped_items.len()
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

    // ── Preview Pane ──
    if let Some(group) = app.list_items.get(app.list_selected) {
        if let Some(item) = group.items.first() {
            super::preview::render_preview(app, frame, chunks[3], item, None);
        }
    } else {
        let empty_preview = Paragraph::new("No result selected")
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(empty_preview, chunks[3]);
    }

    // ── Status bar ──
    let status_line = Line::from(vec![
        Span::styled(
            " ↑↓/PgUpDn",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Nav "),
        Span::styled(
            "Space",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Expand "),
        Span::styled(
            "f", // Changed to match implementation plan
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Filters "),
        Span::styled(
            "/",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("Search "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Detail "),
         Span::styled(
            "s",
             Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Similar "),
        Span::styled(
            "?",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Help "),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Exit  "),
        Span::styled(
            &app.status_msg,
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    let status_bar = Paragraph::new(status_line);
    frame.render_widget(status_bar, chunks[4]);
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
