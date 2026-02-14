use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Layout: header(4) + results(min) + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Header ──
    let source_preview = super::list::truncate_str(
        &app.similar_source_preview,
        (area.width as usize).saturating_sub(25),
    );
    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                " Similar to ID ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                app.similar_source_id.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                source_preview,
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];
    let header = Paragraph::new(header_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Vector Similarity Search "),
    );
    frame.render_widget(header, chunks[0]);

    // ── Results list ──
    let items: Vec<ListItem> = app
        .similar_results
        .iter()
        .enumerate()
        .map(|(i, result)| {
            let preview = result
                .summary_preview
                .lines()
                .next()
                .unwrap_or("")
                .trim();
            let similarity = 1.0 - result.distance; // Convert distance to similarity
            let sim_color = if similarity > 0.90 {
                Color::Green
            } else if similarity > 0.80 {
                Color::Yellow
            } else {
                Color::Red
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>2}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:.3} ", similarity),
                    Style::default().fg(sim_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:>5} ", result.identifier),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(super::list::truncate_str(
                    preview,
                    (area.width as usize).saturating_sub(25),
                )),
            ]);
            ListItem::new(line)
        })
        .collect();

    let result_count = format!(" {} results ", app.similar_results.len());
    let list_widget = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Results (sorted by similarity) ")
                .title_bottom(Line::from(result_count).alignment(Alignment::Right)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.similar_selected));
    frame.render_stateful_widget(list_widget, chunks[1], &mut list_state);

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
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Detail  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Back  "),
        Span::styled(&app.status_msg, Style::default().fg(Color::DarkGray)),
    ]);
    let status_bar = Paragraph::new(status_line);
    frame.render_widget(status_bar, chunks[2]);
}
