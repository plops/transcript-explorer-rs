mod list;
mod detail;
mod similar;
mod help;

use crate::app::App;
use ratatui::Frame;

/// Top-level render dispatch.
pub fn render(app: &App, frame: &mut Frame) {
    match app.view {
        crate::app::View::List => list::render(app, frame),
        crate::app::View::Detail => detail::render(app, frame),
        crate::app::View::Similar => similar::render(app, frame),
    }

    // Render help overlay on top if active
    if app.show_help {
        help::render(frame);
    }
}
