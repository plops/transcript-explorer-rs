use crate::app::{App, DetailTab};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let detail = match &app.detail {
        Some(d) => d,
        None => return,
    };

    // Layout: header(5) + tabs(3) + content(min) + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    // ── Metadata header ──
    let meta_lines = vec![
        Line::from(vec![
            Span::styled(" ID: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                detail.identifier.to_string(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("Model: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&detail.model, Style::default().fg(Color::Cyan)),
            Span::raw("   "),
            Span::styled("Cost: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("${:.4}", detail.cost),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Host: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&detail.host, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled(" Link: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &detail.original_source_link,
                Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
            ),
            Span::raw("   "),
            Span::styled("Lang: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&detail.output_language, Style::default().fg(Color::White)),
            Span::raw("   "),
            Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}in/{}out", detail.summary_input_tokens, detail.summary_output_tokens),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let meta_block = Paragraph::new(meta_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Transcript Detail "),
    );
    frame.render_widget(meta_block, chunks[0]);

    // ── Tab strip ──
    let tab_titles: Vec<Line> = DetailTab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.detail_tab {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(t.label(), style))
        })
        .collect();

    let tab_index = DetailTab::ALL
        .iter()
        .position(|t| *t == app.detail_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .select(tab_index)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" [Tab/1-3] "),
        )
        .highlight_style(Style::default().fg(Color::Cyan));
    frame.render_widget(tabs, chunks[1]);

    // ── Content area ──
    let content_text = match app.detail_tab {
        DetailTab::Summary => &detail.summary,
        DetailTab::Transcript => &detail.transcript,
        DetailTab::Timestamps => {
            if !detail.timestamped_summary_in_youtube_format.is_empty() {
                &detail.timestamped_summary_in_youtube_format
            } else {
                &detail.timestamps
            }
        }
    };

    let content = Paragraph::new(content_text.as_str())
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" {} ", app.detail_tab.label()))
                .title_bottom(
                    Line::from(format!(" scroll: {} ", app.detail_scroll))
                        .alignment(Alignment::Right),
                ),
        );
    frame.render_widget(content, chunks[2]);

    // ── Status bar ──
    let emb_info = if detail.has_embedding {
        format!("  [emb: {}]", detail.embedding_model)
    } else {
        " [no embedding]".to_string()
    };

    let status_line = Line::from(vec![
        Span::styled(
            " ↑↓/PgUp/PgDn",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Scroll  "),
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Switch  "),
        Span::styled(
            "s",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Similar  "),
        Span::styled(
            "y",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Yank Link  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Back"),
        Span::styled(emb_info, Style::default().fg(Color::DarkGray)),
    ]);
    let status_bar = Paragraph::new(status_line);
    frame.render_widget(status_bar, chunks[3]);
}
