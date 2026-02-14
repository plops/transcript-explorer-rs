use crate::app::App;
use crate::db::TranscriptListItem;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use chrono::{DateTime, NaiveDateTime};

pub fn render_preview(_app: &App, frame: &mut Frame, area: Rect, item: &TranscriptListItem, similarity: Option<f64>) {
    let sim_info = if let Some(dist) = similarity {
        let similarity_val = 1.0 - dist;
        let sim_color = if similarity_val > 0.90 {
            Color::Green
        } else if similarity_val > 0.80 {
            Color::Yellow
        } else {
            Color::Red
        };
        vec![
            Span::styled(" Similarity: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.1}%", similarity_val * 100.0), Style::default().fg(sim_color).add_modifier(Modifier::BOLD)),
        ]
    } else {
        vec![]
    };

    // Duration calculation
    let duration_str = if !item.summary_timestamp_start.is_empty() && !item.summary_timestamp_end.is_empty() {
        // Try RFC3339 first
        if let (Ok(start), Ok(end)) = (
            DateTime::parse_from_rfc3339(&item.summary_timestamp_start),
            DateTime::parse_from_rfc3339(&item.summary_timestamp_end)
        ) {
            let duration = end.signed_duration_since(start);
            format!("{}s", duration.num_seconds())
        } else {
             // Try the format seen in the database: 2024-09-26T15:14:34.277795
             let format = "%Y-%m-%dT%H:%M:%S%.f";
             if let (Ok(start), Ok(end)) = (
                 NaiveDateTime::parse_from_str(&item.summary_timestamp_start, format),
                 NaiveDateTime::parse_from_str(&item.summary_timestamp_end, format)
             ) {
                 let duration = end.signed_duration_since(start);
                 format!("{}s", duration.num_seconds())
             } else {
                 // Try another common format: 2026-02-14 18:37:03
                 let format2 = "%Y-%m-%d %H:%M:%S";
                 if let (Ok(start), Ok(end)) = (
                     NaiveDateTime::parse_from_str(&item.summary_timestamp_start.split('.').next().unwrap_or(&item.summary_timestamp_start), format2),
                     NaiveDateTime::parse_from_str(&item.summary_timestamp_end.split('.').next().unwrap_or(&item.summary_timestamp_end), format2)
                 ) {
                     let duration = end.signed_duration_since(start);
                     format!("{}s", duration.num_seconds())
                 } else {
                     "N/A".to_string()
                 }
             }
        }
    } else {
        "N/A".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Selected Result Preview ");
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Length(3), // Metadata rows
            ratatui::layout::Constraint::Min(1),    // Summary
        ])
        .split(inner_area);

    let mut info_line = vec![
        Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
        Span::styled(item.identifier.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled("Host: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&item.host, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("Model: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&item.model, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("Cost: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("${:.3}", item.cost), Style::default().fg(Color::Yellow)),
    ];
    info_line.extend(sim_info);

    let token_line = vec![
        Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("In:{} Out:{}", item.summary_input_tokens, item.summary_output_tokens), Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled("Finished: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&item.summary_timestamp_end, Style::default().fg(Color::White)),
        Span::raw(" (duration: "),
        Span::styled(duration_str, Style::default().fg(Color::Cyan)),
        Span::raw(")"),
    ];

    let link_line = vec![
        Span::styled("Link: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&item.original_source_link, Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)),
    ];

    let metadata_text = vec![
        Line::from(info_line),
        Line::from(token_line),
        Line::from(link_line),
    ];

    frame.render_widget(Paragraph::new(metadata_text), chunks[0]);

    // ── Summary Rendering with tui-markdown ──
    let summary_text = tui_markdown::from_str(&item.summary);
    
    // Prepend "Summary:" title
    let mut final_text = vec![
        Line::from(vec![Span::styled("Summary:", Style::default().fg(Color::DarkGray))]),
    ];
    final_text.extend(summary_text.lines);
    
    let summary_para = Paragraph::new(final_text)
        .wrap(Wrap { trim: false });
    
    frame.render_widget(summary_para, chunks[1]);
}
