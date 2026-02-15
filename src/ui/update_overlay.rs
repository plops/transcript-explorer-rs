use crate::update::{UpdateMessage, UserResponse};
use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Gauge, Paragraph};
use std::sync::mpsc::Sender;

/// Current state of the update process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Checking,
    Available,
    AwaitingConfirmation,
    Downloading,
    Installing,
    Complete,
    UpToDate,
    Error,
    Skipped,
}

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}

impl DownloadProgress {
    /// Get the progress as a percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }
}

/// Manages the state of the update overlay in the TUI
pub struct UpdateOverlayState {
    /// Whether the overlay is currently visible
    visible: bool,

    /// Current update state
    state: UpdateState,

    /// Current version string
    current_version: Option<String>,

    /// New version string (if available)
    new_version: Option<String>,

    /// Download progress information
    download_progress: Option<DownloadProgress>,

    /// Error message (if in error state)
    error_message: Option<String>,

    /// Recovery instructions (if available)
    recovery_instructions: Option<String>,

    /// Whether the current error is retryable
    is_retryable: bool,

    /// Skip reason (if update was skipped)
    skip_reason: Option<String>,

    /// Channel for sending user responses
    response_sender: Option<Sender<UserResponse>>,
}

impl UpdateOverlayState {
    /// Create a new UpdateOverlayState with default values
    pub fn new() -> Self {
        Self {
            visible: false,
            state: UpdateState::Idle,
            current_version: None,
            new_version: None,
            download_progress: None,
            error_message: None,
            recovery_instructions: None,
            is_retryable: false,
            skip_reason: None,
            response_sender: None,
        }
    }

    /// Process an update message and update state accordingly
    pub fn process_message(&mut self, message: UpdateMessage, response_tx: Sender<UserResponse>) {
        self.visible = true;
        self.response_sender = Some(response_tx);

        match message {
            UpdateMessage::CheckStarted => {
                self.state = UpdateState::Checking;
            }
            UpdateMessage::UpToDate { current_version } => {
                self.state = UpdateState::UpToDate;
                self.current_version = Some(current_version);
                self.visible = true; // Show the up-to-date message
            }
            UpdateMessage::UpdateAvailable {
                current_version,
                new_version,
            } => {
                self.state = UpdateState::Available;
                self.current_version = Some(current_version);
                self.new_version = Some(new_version);
            }
            UpdateMessage::ConfirmationRequired { new_version } => {
                self.state = UpdateState::AwaitingConfirmation;
                self.new_version = Some(new_version);
            }
            UpdateMessage::DownloadStarted {
                version,
                total_bytes,
            } => {
                self.state = UpdateState::Downloading;
                self.new_version = Some(version);
                self.download_progress = Some(DownloadProgress {
                    downloaded_bytes: 0,
                    total_bytes,
                    percentage: 0.0,
                });
            }
            UpdateMessage::DownloadProgress {
                downloaded_bytes,
                total_bytes,
                percentage,
            } => {
                self.download_progress = Some(DownloadProgress {
                    downloaded_bytes,
                    total_bytes,
                    percentage,
                });
            }
            UpdateMessage::DownloadComplete => {
                // Keep state as Downloading, will transition to Installing
            }
            UpdateMessage::InstallStarted => {
                self.state = UpdateState::Installing;
            }
            UpdateMessage::InstallComplete { new_version } => {
                self.state = UpdateState::Complete;
                self.new_version = Some(new_version);
            }
            UpdateMessage::Error {
                message,
                recovery_instructions,
                is_retryable,
            } => {
                self.state = UpdateState::Error;
                self.error_message = Some(message);
                self.recovery_instructions = recovery_instructions;
                self.is_retryable = is_retryable;
            }
            UpdateMessage::Skipped { reason } => {
                self.state = UpdateState::Skipped;
                self.skip_reason = Some(reason);
            }
        }
    }

    /// Handle keyboard input for the overlay
    /// Returns true if the key was handled, false otherwise
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match self.state {
            UpdateState::AwaitingConfirmation => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.send_response(UserResponse::Confirmed);
                    true
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.send_response(UserResponse::Declined);
                    self.visible = false;
                    true
                }
                _ => false,
            },
            UpdateState::Complete | UpdateState::Skipped | UpdateState::UpToDate => {
                // Any key dismisses
                self.visible = false;
                self.send_response(UserResponse::Dismissed);
                true
            }
            UpdateState::Error => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') if self.is_retryable => {
                    self.send_response(UserResponse::Retry);
                    true
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.visible = false;
                    self.send_response(UserResponse::Dismissed);
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Send a response to the update thread
    fn send_response(&self, response: UserResponse) {
        if let Some(sender) = &self.response_sender {
            let _ = sender.send(response);
        }
    }

    /// Check if the overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Dismiss the overlay
    pub fn dismiss(&mut self) {
        self.visible = false;
    }

    /// Get the current state
    pub fn state(&self) -> UpdateState {
        self.state
    }

    /// Get the current version
    pub fn current_version(&self) -> Option<&str> {
        self.current_version.as_deref()
    }

    /// Get the new version
    pub fn new_version(&self) -> Option<&str> {
        self.new_version.as_deref()
    }

    /// Get the download progress
    pub fn download_progress(&self) -> Option<&DownloadProgress> {
        self.download_progress.as_ref()
    }

    /// Get the error message
    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    /// Get the recovery instructions
    pub fn recovery_instructions(&self) -> Option<&str> {
        self.recovery_instructions.as_deref()
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        self.is_retryable
    }

    /// Get the skip reason
    pub fn skip_reason(&self) -> Option<&str> {
        self.skip_reason.as_deref()
    }
}

impl Default for UpdateOverlayState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_overlay_state_creation() {
        let state = UpdateOverlayState::new();
        assert!(!state.is_visible());
        assert_eq!(state.state(), UpdateState::Idle);
        assert!(state.current_version().is_none());
        assert!(state.new_version().is_none());
    }

    #[test]
    fn test_update_overlay_state_default() {
        let state = UpdateOverlayState::default();
        assert!(!state.is_visible());
        assert_eq!(state.state(), UpdateState::Idle);
    }

    #[test]
    fn test_download_progress_percentage() {
        let progress = DownloadProgress {
            downloaded_bytes: 50,
            total_bytes: 100,
            percentage: 50.0,
        };
        assert_eq!(progress.percentage(), 50.0);

        let progress = DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: 100,
            percentage: 0.0,
        };
        assert_eq!(progress.percentage(), 0.0);

        let progress = DownloadProgress {
            downloaded_bytes: 100,
            total_bytes: 100,
            percentage: 100.0,
        };
        assert_eq!(progress.percentage(), 100.0);

        let progress = DownloadProgress {
            downloaded_bytes: 0,
            total_bytes: 0,
            percentage: 0.0,
        };
        assert_eq!(progress.percentage(), 0.0);
    }

    #[test]
    fn test_update_overlay_state_process_check_started() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(UpdateMessage::CheckStarted, tx);

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Checking);
    }

    #[test]
    fn test_update_overlay_state_process_up_to_date() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::UpToDate {
                current_version: "1.0.0".to_string(),
            },
            tx,
        );

        assert!(state.is_visible()); // Now visible
        assert_eq!(state.state(), UpdateState::UpToDate);
        assert_eq!(state.current_version(), Some("1.0.0"));
    }

    #[test]
    fn test_update_overlay_state_process_update_available() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::UpdateAvailable {
                current_version: "1.0.0".to_string(),
                new_version: "1.1.0".to_string(),
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Available);
        assert_eq!(state.current_version(), Some("1.0.0"));
        assert_eq!(state.new_version(), Some("1.1.0"));
    }

    #[test]
    fn test_update_overlay_state_process_confirmation_required() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::ConfirmationRequired {
                new_version: "1.1.0".to_string(),
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::AwaitingConfirmation);
        assert_eq!(state.new_version(), Some("1.1.0"));
    }

    #[test]
    fn test_update_overlay_state_process_download_started() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::DownloadStarted {
                version: "1.1.0".to_string(),
                total_bytes: 1024000,
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Downloading);
        assert_eq!(state.new_version(), Some("1.1.0"));
        assert!(state.download_progress().is_some());
        let progress = state.download_progress().unwrap();
        assert_eq!(progress.total_bytes, 1024000);
        assert_eq!(progress.downloaded_bytes, 0);
    }

    #[test]
    fn test_update_overlay_state_process_download_progress() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::DownloadProgress {
                downloaded_bytes: 512000,
                total_bytes: 1024000,
                percentage: 50.0,
            },
            tx,
        );

        assert!(state.is_visible());
        let progress = state.download_progress().unwrap();
        assert_eq!(progress.downloaded_bytes, 512000);
        assert_eq!(progress.total_bytes, 1024000);
        assert_eq!(progress.percentage, 50.0);
    }

    #[test]
    fn test_update_overlay_state_process_install_complete() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::InstallComplete {
                new_version: "1.1.0".to_string(),
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Complete);
        assert_eq!(state.new_version(), Some("1.1.0"));
    }

    #[test]
    fn test_update_overlay_state_process_error() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::Error {
                message: "Download failed".to_string(),
                recovery_instructions: Some("Check your connection".to_string()),
                is_retryable: true,
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Error);
        assert_eq!(state.error_message(), Some("Download failed"));
        assert_eq!(
            state.recovery_instructions(),
            Some("Check your connection")
        );
        assert!(state.is_retryable());
    }

    #[test]
    fn test_update_overlay_state_process_skipped() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(
            UpdateMessage::Skipped {
                reason: "Already up to date".to_string(),
            },
            tx,
        );

        assert!(state.is_visible());
        assert_eq!(state.state(), UpdateState::Skipped);
        assert_eq!(state.skip_reason(), Some("Already up to date"));
    }

    #[test]
    fn test_update_overlay_state_dismiss() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut state = UpdateOverlayState::new();

        state.process_message(UpdateMessage::CheckStarted, tx);
        assert!(state.is_visible());

        state.dismiss();
        assert!(!state.is_visible());
    }
}


/// Render the update overlay
pub fn render(state: &UpdateOverlayState, frame: &mut Frame) {
    if !state.is_visible() {
        return;
    }

    let area = centered_rect(60, 40, frame.size());

    // Clear the area
    frame.render_widget(Clear, area);

    // Render based on state
    match state.state() {
        UpdateState::Checking => render_checking(frame, area),
        UpdateState::Available => render_available(state, frame, area),
        UpdateState::AwaitingConfirmation => render_confirmation(state, frame, area),
        UpdateState::Downloading => render_downloading(state, frame, area),
        UpdateState::Installing => render_installing(frame, area),
        UpdateState::Complete => render_complete(state, frame, area),
        UpdateState::UpToDate => render_up_to_date(state, frame, area),
        UpdateState::Error => render_error(state, frame, area),
        UpdateState::Skipped => render_skipped(state, frame, area),
        UpdateState::Idle => {}
    }
}

/// Calculate a centered rectangle for the overlay
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Render checking state
fn render_checking(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Check")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Paragraph::new("Checking for updates...")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(text, area);
}

/// Render available state
fn render_available(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Available")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let current = state.current_version().unwrap_or("unknown");
    let new = state.new_version().unwrap_or("unknown");

    let text = vec![
        Line::from(format!("Current version: {}", current)),
        Line::from(format!("New version: {}", new)),
        Line::from(""),
        Line::from("An update is available."),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render confirmation state
fn render_confirmation(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Confirm Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let new = state.new_version().unwrap_or("unknown");

    let text = vec![
        Line::from(format!("Update to version {}?", new)),
        Line::from(""),
        Line::from(vec![
            Span::styled("Y", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("es / "),
            Span::styled("N", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("o"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render downloading state
fn render_downloading(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Downloading Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(progress) = state.download_progress() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
            ])
            .split(inner);

        let label = format!(
            "Downloading: {} / {} ({:.1}%)",
            format_bytes(progress.downloaded_bytes),
            format_bytes(progress.total_bytes),
            progress.percentage
        );

        let label_widget = Paragraph::new(label).alignment(Alignment::Center);
        frame.render_widget(label_widget, chunks[0]);

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(progress.percentage / 100.0);
        frame.render_widget(gauge, chunks[1]);
    }
}

/// Render installing state
fn render_installing(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Installing Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = Paragraph::new("Installing update...")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(text, area);
}

/// Render complete state
fn render_complete(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Complete")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let new = state.new_version().unwrap_or("unknown");

    let text = vec![
        Line::from(format!("Successfully updated to version {}", new)),
        Line::from(""),
        Line::from("Please restart the application."),
        Line::from(""),
        Line::from("Press any key to dismiss"),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render up-to-date state
fn render_up_to_date(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Application Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let current = state.current_version().unwrap_or("unknown");

    let text = vec![
        Line::from(format!("Version {} is up to date", current)),
        Line::from(""),
        Line::from("No updates available."),
        Line::from(""),
        Line::from("Press any key to dismiss"),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render error state
fn render_error(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Error")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let mut text = vec![Line::from(Span::styled(
        state.error_message().unwrap_or("Unknown error"),
        Style::default().fg(Color::Red),
    ))];

    if let Some(recovery) = state.recovery_instructions() {
        text.push(Line::from(""));
        text.push(Line::from(recovery));
    }

    text.push(Line::from(""));

    if state.is_retryable() {
        text.push(Line::from(vec![
            Span::styled("R", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("etry / "),
            Span::styled("Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("uit"),
        ]));
    } else {
        text.push(Line::from("Press ESC or Q to dismiss"));
    }

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render skipped state
fn render_skipped(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Skipped")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let reason = state.skip_reason().unwrap_or("Unknown reason");

    let text = vec![
        Line::from(format!("Update skipped: {}", reason)),
        Line::from(""),
        Line::from("Press any key to dismiss"),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}
