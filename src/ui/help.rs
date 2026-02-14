use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn render(frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Global", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ?         ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("    q         ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("    Esc       ", Style::default().fg(Color::Yellow)),
            Span::raw("Back / cancel"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  List View", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ↑/k ↓/j   ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate up/down"),
        ]),
        Line::from(vec![
            Span::styled("    Enter     ", Style::default().fg(Color::Yellow)),
            Span::raw("Open transcript detail"),
        ]),
        Line::from(vec![
            Span::styled("    /         ", Style::default().fg(Color::Yellow)),
            Span::raw("Start filtering (type to search)"),
        ]),
        Line::from(vec![
            Span::styled("    s         ", Style::default().fg(Color::Yellow)),
            Span::raw("Find similar transcripts (vector search)"),
        ]),
        Line::from(vec![
            Span::styled("    g/G       ", Style::default().fg(Color::Yellow)),
            Span::raw("Jump to first/last page"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Detail View", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ↑/↓       ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll content"),
        ]),
        Line::from(vec![
            Span::styled("    PgUp/PgDn ", Style::default().fg(Color::Yellow)),
            Span::raw("Scroll page up/down"),
        ]),
        Line::from(vec![
            Span::styled("    Tab/1-3   ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch between Summary/Transcript/Timestamps"),
        ]),
        Line::from(vec![
            Span::styled("    s         ", Style::default().fg(Color::Yellow)),
            Span::raw("Find similar transcripts"),
        ]),
        Line::from(vec![
            Span::styled("    y         ", Style::default().fg(Color::Yellow)),
            Span::raw("Copy source link to clipboard"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Similar View", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("    ↑/↓       ", Style::default().fg(Color::Yellow)),
            Span::raw("Navigate results"),
        ]),
        Line::from(vec![
            Span::styled("    Enter     ", Style::default().fg(Color::Yellow)),
            Span::raw("Open selected result"),
        ]),
        Line::from(""),
    ];

    let help = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help — Keybindings ")
                .title_bottom(Line::from(" Press ? or Esc to close ").style(Style::default().fg(Color::DarkGray))),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(help, area);
}

/// Create a centered rectangle using percentage of parent area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}
