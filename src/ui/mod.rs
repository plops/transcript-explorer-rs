mod list;
mod detail;
pub mod similar;
pub mod preview;
pub mod filters;
mod help;
pub mod update_overlay;
pub mod password_overlay;

use crate::app::App;
use ratatui::Frame;

/// Top-level render dispatch.
pub fn render(app: &App, frame: &mut Frame) {
    match app.view {
        crate::app::View::List => list::render(app, frame),
        crate::app::View::Detail => detail::render(app, frame),
        crate::app::View::Similar => similar::render(app, frame),
        crate::app::View::Filters => filters::render(app, frame),
    }

    // Render update overlay if visible
    if app.update_overlay.is_visible() {
        update_overlay::render(&app.update_overlay, frame);
    }

    // Render help overlay on top if active
    if app.show_help {
        help::render(frame);
    }
}
