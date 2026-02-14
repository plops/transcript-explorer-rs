use crate::app::{App, Filter};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    
    // Header(3) + Content(min) + Status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Global Filter Configuration ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" [{} items total]", app.all_items.len()), Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(header, chunks[0]);

    // Content: Stats (Left) | Filters (Right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(chunks[1]);

    // ── Stats ──
    render_stats(app, frame, content_chunks[0]);

    // ── Filters ──
    render_filters(app, frame, content_chunks[1]);

    // Status bar
    let mut status_spans = vec![
        Span::styled(" a ", Style::default().bg(Color::Cyan).fg(Color::Black)),
        Span::raw(" Add Filter  "),
        Span::styled(" d ", Style::default().bg(Color::Red).fg(Color::Black)),
        Span::raw(" Clear All  "),
        Span::styled(" Esc ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" Back  "),
    ];

    if !app.status_msg.is_empty() {
        status_spans.push(Span::styled(format!(" | {} ", app.status_msg), Style::default().fg(Color::Yellow)));
    }

    // If entering value, show buffer and set cursor
    if let crate::app::FilterBuilderState::EnteringValue { ref buffer, .. } = app.filter_builder_state {
        status_spans.push(Span::styled(format!(" > {}", buffer), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
        
        // Calculate cursor position
        // The cursor should be at the end of the buffer.
        // We need to know the x-offset of the ">" character.
        // It's a bit tricky with nested spans, but let's approximate or just use a fixed spot if needed.
        // Actually, we can just render the buffer in a dedicated chunk if we want it clean.
    }

    let status_line = Line::from(status_spans);
    frame.render_widget(Paragraph::new(status_line), chunks[2]);

    // Set cursor if entering value
    if let crate::app::FilterBuilderState::EnteringValue { ref buffer, .. } = app.filter_builder_state {
        // Approximate x: length of all previous spans + prefix " > "
        let mut x = 0;
        // spans except the last one and the " | " one
        // Base shortcuts: " a Add Filter   d Clear All   Esc Back   "
        // " a " (3) + " Add Filter  " (13) + " d " (3) + " Clear All  " (12) + " Esc " (5) + " Back  " (7) = 43
        x += 43;
        if !app.status_msg.is_empty() {
            x += (3 + app.status_msg.len()) as u16; // " | " is 3
        }
        x += 3; // " > " is 3
        x += buffer.len() as u16;
        
        frame.set_cursor_position((chunks[2].x + x, chunks[2].y));
    }
}

fn render_stats(app: &App, frame: &mut Frame, area: Rect) {
    let mut items = Vec::new();
    
    let fields = vec!["cost", "input_tokens", "output_tokens"];
    for field in fields {
        if let Some(stats) = app.field_stats.get(field) {
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", field.to_uppercase()), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" (n={})", stats.count), Style::default().fg(Color::DarkGray)),
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("   Mean:   {:<10.4}", stats.mean), Style::default().fg(Color::White)),
                Span::styled(format!(" StdDev: {:<10.4}", stats.stddev), Style::default().fg(Color::White)),
            ])));
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("   Min:    {:<10.4}", stats.min), Style::default().fg(Color::White)),
                Span::styled(format!(" Max:    {:<10.4}", stats.max), Style::default().fg(Color::White)),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("   Median: {:<10.4}", stats.median), Style::default().fg(Color::White)),
                Span::styled(format!(" MAD:    {:<10.4}", stats.mad), Style::default().fg(Color::White)),
            ])));

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("   P5:     {:<10.4}", stats.p5), Style::default().fg(Color::White)),
                Span::styled(format!(" P95:    {:<10.4}", stats.p95), Style::default().fg(Color::White)),
            ])));
            
            items.push(ListItem::new(Line::from("")));
        }
    }

    // Unique Models snippet
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" MODELS ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    ])));
    for model in app.unique_models.iter().take(10) {
        items.push(ListItem::new(format!("   • {}", model)));
    }
    if app.unique_models.len() > 10 {
        items.push(ListItem::new(format!("   ... and {} more", app.unique_models.len() - 10)));
    }

    let stats_block = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)).title(" Metadata Statistics "));
    frame.render_widget(stats_block, area);
}

fn render_filters(app: &App, frame: &mut Frame, area: Rect) {
    let mut items = Vec::new();
    
    if let Some(ref filter) = app.global_filter {
        render_filter_recursive(filter, &mut items, 0);
    } else {
        items.push(ListItem::new(Span::styled("No active global filters. All rows shown.", Style::default().fg(Color::DarkGray))));
    }

    let filters_block = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)).title(" Active Global Filters (Applied to all views) "));
    frame.render_widget(filters_block, area);
}

fn render_filter_recursive<'a>(filter: &'a Filter, items: &mut Vec<ListItem<'a>>, indent: usize) {
    let prefix = "  ".repeat(indent);
    match filter {
        Filter::Range { field, min, max } => {
            items.push(ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(field, Style::default().fg(Color::Yellow)),
                Span::raw(" in range "),
                Span::styled(format!("[{:.3}, {:.3}]", min, max), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])));
        }
        Filter::Match { field, pattern } => {
            items.push(ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(field, Style::default().fg(Color::Magenta)),
                Span::raw(" matches "),
                Span::styled(format!("'{}'", pattern), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])));
        }
        Filter::And(fs) => {
            items.push(ListItem::new(Span::styled(format!("{}ALL OF:", prefix), Style::default().fg(Color::Cyan))));
            for f in fs {
                render_filter_recursive(f, items, indent + 1);
            }
        }
        Filter::Or(fs) => {
            items.push(ListItem::new(Span::styled(format!("{}ANY OF:", prefix), Style::default().fg(Color::Cyan))));
            for f in fs {
                render_filter_recursive(f, items, indent + 1);
            }
        }
        Filter::Not(f) => {
            items.push(ListItem::new(Span::styled(format!("{}NOT:", prefix), Style::default().fg(Color::Red))));
            render_filter_recursive(f, items, indent + 1);
        }
    }
}
