use crate::app::{self, App};
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

    // Layout: header(4) + results(min) + preview(?) + status(1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(5),
            preview_height,
            Constraint::Length(1),
        ])
        .split(area);

    // ── Header ──
    let source_preview = app::get_display_title(&app.similar_source_preview);
    let source_preview_truncated = super::list::truncate_str(
        &source_preview,
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
                source_preview_truncated,
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
    let mut items = Vec::new();
    let mut _selected_result = None;
    for (i, group) in app.grouped_similar_results.iter().enumerate() {
        if group.items.is_empty() {
            continue;
        }

        let result = &group.items[0];
        if i == app.similar_selected {
            _selected_result = Some(result.clone());
        }

        let similarity = 1.0 - result.distance;
        let sim_color = if similarity > 0.90 {
            Color::Green
        } else if similarity > 0.80 {
            Color::Yellow
        } else {
            Color::Red
        };

        let title = app::get_display_title(&result.summary);
        
        let mut line_spans = vec![
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

        line_spans.push(Span::raw(super::list::truncate_str(
            &title,
            (area.width as usize).saturating_sub(40),
        )));
        
        items.push(ListItem::new(Line::from(line_spans)));

        if group.expanded {
            for sub_item in group.items.iter().skip(1) {
                let sub_similarity = 1.0 - sub_item.distance;
                let sub_line = Line::from(vec![
                    Span::raw("        "), // padding
                    Span::styled(
                        format!("{:.3} ", sub_similarity),
                        Style::default().fg(Color::DarkGray),
                    ),
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

    let result_count = format!(" {} groups ({} total results) ", app.grouped_similar_results.len(), app.similar_results.len());
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

    // ── Preview Pane ──
    // ── Preview Pane ──
    if let Some(group) = app.grouped_similar_results.get(app.similar_selected) {
        if let Some(res) = group.items.first() {
            // Convert SimilarResult to a pseudo TranscriptListItem for preview
            let item = crate::db::TranscriptListItem {
                identifier: res.identifier,
                host: res.host.clone(),
                summary: res.summary.clone(),
                cost: res.cost,
                has_embedding: true,
                model: res.model.clone(),
                original_source_link: res.original_source_link.clone(),
                summary_input_tokens: res.summary_input_tokens,
                summary_output_tokens: res.summary_output_tokens,
                summary_timestamp_start: res.summary_timestamp_start.clone(),
                summary_timestamp_end: res.summary_timestamp_end.clone(),
            };
            super::preview::render_preview(app, frame, chunks[2], &item, Some(res.distance));
        }
    } else {
         let empty_preview = Paragraph::new("No result selected")
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        frame.render_widget(empty_preview, chunks[2]);
    }

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
            "Space",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Expand  "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Detail  "),
        Span::styled(
            "y",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Yank Link  "),
        Span::styled(
            "o",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Open Link  "),
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
    frame.render_widget(status_bar, chunks[3]);
}
