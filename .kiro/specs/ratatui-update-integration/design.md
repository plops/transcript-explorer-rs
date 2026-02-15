# Design Document: Ratatui Update Integration

## Overview

This design integrates the auto-updater module with the ratatui TUI framework by replacing direct console I/O operations with a message-passing architecture. The solution uses Rust's mpsc channels for thread-safe communication between the update background thread and the main TUI event loop, enabling seamless update notifications, progress display, and user interactions within the TUI interface.

The design also addresses database password entry by creating a reusable password input overlay that can be used both at startup (for encrypted databases) and during runtime (if needed for future features).

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Main Thread                           │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Ratatui Event Loop                        │ │
│  │  - Keyboard input handling                             │ │
│  │  - UI rendering                                        │ │
│  │  - Update message processing                           │ │
│  └────────────────────────────────────────────────────────┘ │
│                          │                                   │
│                          │ UpdateMessage                     │
│                          ↓                                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Update Overlay Manager                    │ │
│  │  - State management                                    │ │
│  │  - Overlay rendering                                   │ │
│  │  - User input handling                                 │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                          ↕ mpsc channels
┌─────────────────────────────────────────────────────────────┐
│                     Background Thread                        │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Update Manager                            │ │
│  │  - Check for updates                                   │ │
│  │  - Download binaries                                   │ │
│  │  - Send status messages                                │ │
│  │  - Receive user responses                              │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Communication Flow

1. **Update Thread → TUI Thread**: Status updates, progress, prompts
   - Uses `Sender<UpdateMessage>` to send messages
   - Non-blocking sends to avoid stalling update operations

2. **TUI Thread → Update Thread**: User responses, confirmations
   - Uses `Sender<UserResponse>` to send responses
   - Update thread blocks on `Receiver<UserResponse>` when waiting for input

3. **TUI Event Loop Integration**:
   - Poll update message channel during each event loop iteration
   - Process messages and update overlay state
   - Render overlay on top of current view

## Components and Interfaces

### 1. UpdateMessage Enum

Messages sent from the update thread to the TUI thread.

```rust
pub enum UpdateMessage {
    /// Update check has started
    CheckStarted,
    
    /// Update check completed - no update available
    UpToDate { current_version: String },
    
    /// New version is available
    UpdateAvailable {
        current_version: String,
        new_version: String,
    },
    
    /// Requesting user confirmation to proceed with update
    ConfirmationRequired {
        new_version: String,
    },
    
    /// Download has started
    DownloadStarted {
        version: String,
        total_bytes: u64,
    },
    
    /// Download progress update
    DownloadProgress {
        downloaded_bytes: u64,
        total_bytes: u64,
        percentage: f64,
    },
    
    /// Download completed
    DownloadComplete,
    
    /// Installation has started
    InstallStarted,
    
    /// Installation completed successfully
    InstallComplete {
        new_version: String,
    },
    
    /// Update process encountered an error
    Error {
        message: String,
        recovery_instructions: Option<String>,
        is_retryable: bool,
    },
    
    /// Update was skipped
    Skipped {
        reason: String,
    },
}
```

### 2. UserResponse Enum

Messages sent from the TUI thread to the update thread.

```rust
pub enum UserResponse {
    /// User confirmed the action
    Confirmed,
    
    /// User declined the action
    Declined,
    
    /// User requested retry after error
    Retry,
    
    /// User dismissed the message
    Dismissed,
}
```

### 3. UpdateOverlayState Struct

Manages the state of the update overlay in the TUI.

```rust
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

pub enum UpdateState {
    Idle,
    Checking,
    Available,
    AwaitingConfirmation,
    Downloading,
    Installing,
    Complete,
    Error,
    Skipped,
}

pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}
```

### 4. UpdateChannels Struct

Encapsulates the communication channels.

```rust
pub struct UpdateChannels {
    /// Sender for update messages (used by update thread)
    pub message_tx: Sender<UpdateMessage>,
    
    /// Receiver for update messages (used by TUI thread)
    pub message_rx: Receiver<UpdateMessage>,
    
    /// Sender for user responses (used by TUI thread)
    pub response_tx: Sender<UserResponse>,
    
    /// Receiver for user responses (used by update thread)
    pub response_rx: Receiver<UserResponse>,
}

impl UpdateChannels {
    pub fn new() -> Self {
        let (message_tx, message_rx) = mpsc::channel();
        let (response_tx, response_rx) = mpsc::channel();
        
        Self {
            message_tx,
            message_rx,
            response_tx,
            response_rx,
        }
    }
    
    pub fn split(self) -> (TuiChannels, UpdateThreadChannels) {
        (
            TuiChannels {
                message_rx: self.message_rx,
                response_tx: self.response_tx,
            },
            UpdateThreadChannels {
                message_tx: self.message_tx,
                response_rx: self.response_rx,
            },
        )
    }
}

pub struct TuiChannels {
    pub message_rx: Receiver<UpdateMessage>,
    pub response_tx: Sender<UserResponse>,
}

pub struct UpdateThreadChannels {
    pub message_tx: Sender<UpdateMessage>,
    pub response_rx: Receiver<UserResponse>,
}
```

### 5. TuiModeHandler Struct

Implements the ModeHandler trait for TUI mode, replacing InteractiveMode and NonInteractiveMode console I/O.

```rust
pub struct TuiModeHandler {
    channels: UpdateThreadChannels,
    interactive: bool,
}

impl TuiModeHandler {
    pub fn new(channels: UpdateThreadChannels, interactive: bool) -> Self {
        Self { channels, interactive }
    }
    
    fn send_message(&self, message: UpdateMessage) {
        // Non-blocking send, ignore errors if TUI has shut down
        let _ = self.channels.message_tx.send(message);
    }
    
    fn wait_for_response(&self) -> UserResponse {
        // Blocking receive - waits for user input from TUI
        self.channels.response_rx.recv()
            .unwrap_or(UserResponse::Declined)
    }
}

impl ModeHandler for TuiModeHandler {
    fn prompt_before_check(&self) -> bool {
        if !self.interactive {
            return true;
        }
        
        self.send_message(UpdateMessage::CheckStarted);
        true // In TUI mode, always proceed with check
    }
    
    fn prompt_for_update_confirmation(&self, new_version: &str) -> bool {
        if !self.interactive {
            return true;
        }
        
        self.send_message(UpdateMessage::ConfirmationRequired {
            new_version: new_version.to_string(),
        });
        
        matches!(self.wait_for_response(), UserResponse::Confirmed)
    }
    
    fn check_for_cancellation(&self) -> bool {
        // Check for cancellation without blocking
        match self.channels.response_rx.try_recv() {
            Ok(UserResponse::Declined) => true,
            _ => false,
        }
    }
    
    fn display_status(&self, message: &str) {
        // Status messages are sent as part of other message types
        // This is a no-op in TUI mode
    }
    
    fn display_progress(&self, progress: &DownloadProgress) {
        self.send_message(UpdateMessage::DownloadProgress {
            downloaded_bytes: progress.downloaded_bytes,
            total_bytes: progress.total_bytes,
            percentage: progress.percentage(),
        });
    }
    
    fn display_success(&self, new_version: &str) {
        self.send_message(UpdateMessage::InstallComplete {
            new_version: new_version.to_string(),
        });
    }
    
    fn display_error(&self, error: &UpdateError) {
        let handler = ErrorHandler::new();
        self.send_message(UpdateMessage::Error {
            message: handler.get_descriptive_message(error),
            recovery_instructions: handler.get_recovery_action(error),
            is_retryable: handler.is_retryable(error),
        });
    }
    
    fn finish_progress(&self) {
        self.send_message(UpdateMessage::DownloadComplete);
    }
}
```

### 6. PasswordInputOverlay Struct

Reusable password input component for encrypted databases.

```rust
pub struct PasswordInputOverlay {
    /// Current password input buffer
    password: String,
    
    /// Error message to display (if any)
    error_message: Option<String>,
    
    /// Prompt message
    prompt: String,
    
    /// Whether the overlay is active
    active: bool,
}

impl PasswordInputOverlay {
    pub fn new(prompt: String) -> Self {
        Self {
            password: String::new(),
            error_message: None,
            prompt,
            active: true,
        }
    }
    
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<PasswordInputResult> {
        match key.code {
            KeyCode::Enter => {
                if !self.password.is_empty() {
                    Some(PasswordInputResult::Submit(
                        std::mem::take(&mut self.password)
                    ))
                } else {
                    None
                }
            }
            KeyCode::Esc => {
                Some(PasswordInputResult::Cancel)
            }
            KeyCode::Char(c) => {
                self.password.push(c);
                None
            }
            KeyCode::Backspace => {
                self.password.pop();
                None
            }
            _ => None,
        }
    }
    
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.password.clear();
    }
    
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render centered overlay with password input field
        // Show asterisks for password characters
        // Display error message if present
    }
}

pub enum PasswordInputResult {
    Submit(String),
    Cancel,
}
```

## Data Models

### UpdateOverlayState Methods

```rust
impl UpdateOverlayState {
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
    
    pub fn process_message(&mut self, message: UpdateMessage, response_tx: Sender<UserResponse>) {
        self.visible = true;
        self.response_sender = Some(response_tx);
        
        match message {
            UpdateMessage::CheckStarted => {
                self.state = UpdateState::Checking;
            }
            UpdateMessage::UpToDate { current_version } => {
                self.state = UpdateState::Idle;
                self.current_version = Some(current_version);
                self.visible = false; // Auto-hide if up to date
            }
            UpdateMessage::UpdateAvailable { current_version, new_version } => {
                self.state = UpdateState::Available;
                self.current_version = Some(current_version);
                self.new_version = Some(new_version);
            }
            UpdateMessage::ConfirmationRequired { new_version } => {
                self.state = UpdateState::AwaitingConfirmation;
                self.new_version = Some(new_version);
            }
            UpdateMessage::DownloadStarted { version, total_bytes } => {
                self.state = UpdateState::Downloading;
                self.new_version = Some(version);
                self.download_progress = Some(DownloadProgress {
                    downloaded_bytes: 0,
                    total_bytes,
                    percentage: 0.0,
                });
            }
            UpdateMessage::DownloadProgress { downloaded_bytes, total_bytes, percentage } => {
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
            UpdateMessage::Error { message, recovery_instructions, is_retryable } => {
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
    
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.state {
            UpdateState::AwaitingConfirmation => {
                match key.code {
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
                }
            }
            UpdateState::Complete | UpdateState::Skipped => {
                // Any key dismisses
                self.visible = false;
                self.send_response(UserResponse::Dismissed);
                true
            }
            UpdateState::Error => {
                match key.code {
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
                }
            }
            _ => false,
        }
    }
    
    fn send_response(&self, response: UserResponse) {
        if let Some(sender) = &self.response_sender {
            let _ = sender.send(response);
        }
    }
    
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    pub fn dismiss(&mut self) {
        self.visible = false;
    }
}
```

### App Integration

The App struct needs to be extended with update overlay state:

```rust
// In src/app.rs
pub struct App {
    // ... existing fields ...
    
    /// Update overlay state
    pub update_overlay: UpdateOverlayState,
    
    /// Channel for receiving update messages
    update_message_rx: Option<Receiver<UpdateMessage>>,
    
    /// Channel for sending user responses
    update_response_tx: Option<Sender<UserResponse>>,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            // ... existing initialization ...
            update_overlay: UpdateOverlayState::new(),
            update_message_rx: None,
            update_response_tx: None,
        }
    }
    
    pub fn set_update_channels(&mut self, message_rx: Receiver<UpdateMessage>, response_tx: Sender<UserResponse>) {
        self.update_message_rx = Some(message_rx);
        self.update_response_tx = Some(response_tx);
    }
    
    pub fn poll_update_messages(&mut self) {
        if let Some(rx) = &self.update_message_rx {
            while let Ok(message) = rx.try_recv() {
                if let Some(tx) = &self.update_response_tx {
                    self.update_overlay.process_message(message, tx.clone());
                }
            }
        }
    }
}
```

## 
Rendering

### Update Overlay Rendering

The update overlay will be rendered in `src/ui/mod.rs` after the main view but before the help overlay:

```rust
// In src/ui/mod.rs
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
```

### Update Overlay Layout

```rust
// In src/ui/update_overlay.rs
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

pub fn render(state: &UpdateOverlayState, frame: &mut Frame) {
    let area = centered_rect(60, 40, frame.size());
    
    // Clear the area
    frame.render_widget(Clear, area);
    
    // Render based on state
    match state.state {
        UpdateState::Checking => render_checking(frame, area),
        UpdateState::Available => render_available(state, frame, area),
        UpdateState::AwaitingConfirmation => render_confirmation(state, frame, area),
        UpdateState::Downloading => render_downloading(state, frame, area),
        UpdateState::Installing => render_installing(state, frame, area),
        UpdateState::Complete => render_complete(state, frame, area),
        UpdateState::Error => render_error(state, frame, area),
        UpdateState::Skipped => render_skipped(state, frame, area),
        UpdateState::Idle => {}
    }
}

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

fn render_checking(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Check")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let text = Paragraph::new("Checking for updates...")
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(text, area);
}

fn render_available(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Available")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let current = state.current_version.as_deref().unwrap_or("unknown");
    let new = state.new_version.as_deref().unwrap_or("unknown");
    
    let text = vec![
        Line::from(format!("Current version: {}", current)),
        Line::from(format!("New version: {}", new)),
        Line::from(""),
        Line::from("An update is available."),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

fn render_confirmation(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Confirm Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let new = state.new_version.as_deref().unwrap_or("unknown");
    
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
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

fn render_downloading(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Downloading Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    if let Some(progress) = &state.download_progress {
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
        
        let label_widget = Paragraph::new(label)
            .alignment(Alignment::Center);
        frame.render_widget(label_widget, chunks[0]);
        
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(progress.percentage / 100.0);
        frame.render_widget(gauge, chunks[1]);
    }
}

fn render_installing(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Installing Update")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    let text = Paragraph::new("Installing update...")
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(text, area);
}

fn render_complete(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Complete")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    
    let new = state.new_version.as_deref().unwrap_or("unknown");
    
    let text = vec![
        Line::from(format!("Successfully updated to version {}", new)),
        Line::from(""),
        Line::from("Please restart the application."),
        Line::from(""),
        Line::from("Press any key to dismiss"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

fn render_error(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Error")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    
    let mut text = vec![
        Line::from(Span::styled(
            state.error_message.as_deref().unwrap_or("Unknown error"),
            Style::default().fg(Color::Red),
        )),
    ];
    
    if let Some(recovery) = &state.recovery_instructions {
        text.push(Line::from(""));
        text.push(Line::from(recovery.as_str()));
    }
    
    text.push(Line::from(""));
    
    if state.is_retryable {
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
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

fn render_skipped(state: &UpdateOverlayState, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Update Skipped")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let reason = state.skip_reason.as_deref().unwrap_or("Unknown reason");
    
    let text = vec![
        Line::from(format!("Update skipped: {}", reason)),
        Line::from(""),
        Line::from("Press any key to dismiss"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    frame.render_widget(paragraph, area);
}

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
```

### Password Input Overlay Rendering

```rust
// In src/ui/password_overlay.rs
pub fn render(state: &PasswordInputOverlay, frame: &mut Frame) {
    let area = centered_rect(50, 30, frame.size());
    
    frame.render_widget(Clear, area);
    
    let block = Block::default()
        .title("Password Required")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    
    let inner = block.inner(area);
    frame.render_widget(block, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);
    
    // Prompt
    let prompt = Paragraph::new(state.prompt.as_str())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(prompt, chunks[0]);
    
    // Password field (masked)
    let masked = "*".repeat(state.password.len());
    let password_field = Paragraph::new(masked)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    frame.render_widget(password_field, chunks[2]);
    
    // Error message if present
    if let Some(error) = &state.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(error_widget, chunks[3]);
    } else {
        let help = Paragraph::new("Enter: Submit | Esc: Cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[3]);
    }
}
```

## Integration Points

### Main Event Loop Integration

The main event loop in `src/main.rs` needs to be updated to:

1. Create update channels before starting the TUI
2. Pass TUI channels to the App
3. Pass update thread channels to UpdateManager
4. Poll for update messages in the event loop
5. Route update overlay key events appropriately

```rust
// In src/main.rs run_app function
async fn run_app(
    terminal: &mut Terminal<impl Backend>,
    mut app: App,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create update channels
    let channels = UpdateChannels::new();
    let (tui_channels, update_channels) = channels.split();
    
    // Set up app with TUI channels
    app.set_update_channels(tui_channels.message_rx, tui_channels.response_tx);
    
    // Spawn update thread with update channels
    if let Ok(config) = UpdateConfiguration::load() {
        if config.enabled {
            let update_manager = UpdateManager::new_with_tui_mode(config, update_channels)?;
            let _update_handle = update_manager.spawn_background_thread();
        }
    }
    
    loop {
        // Poll for update messages
        app.poll_update_messages();
        
        terminal.draw(|f| crate::ui::render(&app, f))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // If update overlay is visible and captures the key, don't process further
                if app.update_overlay.is_visible() {
                    if app.update_overlay.handle_key(key) {
                        continue;
                    }
                }
                
                // Otherwise handle normally
                handle_key(&mut app, key).await?;
                
                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}
```

### UpdateManager Integration

The UpdateManager needs to support both console mode and TUI mode:

```rust
// In src/update/mod.rs
impl UpdateManager {
    pub fn new(config: UpdateConfiguration) -> Result<Self, UpdateError> {
        // Existing implementation for console mode
        // ...
    }
    
    pub fn new_with_tui_mode(
        config: UpdateConfiguration,
        channels: UpdateThreadChannels,
    ) -> Result<Self, UpdateError> {
        let platform = PlatformDetector::detect()?;
        let error_handler = ErrorHandler::new();
        let lock_manager = LockFileManager::new()?;
        let bad_version_tracker = BadVersionTracker::load()?;
        
        // Create TUI mode handler instead of Interactive/NonInteractive
        let mode_handler = Box::new(TuiModeHandler::new(
            channels,
            config.interactive_mode,
        ));
        
        Ok(Self {
            config,
            platform,
            error_handler,
            lock_manager,
            bad_version_tracker,
            mode_handler,
        })
    }
}
```

### Database Password Entry Integration

The database password entry needs to be integrated into the TUI startup flow:

```rust
// In src/main.rs
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... existing CLI parsing ...
    
    // Check if database is encrypted
    let is_encrypted = codec::is_encrypted(&db_path)?;
    
    let target_db_path = if is_encrypted {
        // Initialize terminal for password entry
        let mut terminal = setup_terminal()?;
        
        let password = loop {
            let mut password_overlay = PasswordInputOverlay::new(
                format!("Enter password for {}", db_path.display())
            );
            
            // Render password overlay
            terminal.draw(|f| {
                password_overlay.render(f, f.size());
            })?;
            
            // Handle input
            if let Event::Key(key) = event::read()? {
                if let Some(result) = password_overlay.handle_key(key) {
                    match result {
                        PasswordInputResult::Submit(pwd) => {
                            // Try to decrypt
                            let temp = tempfile::NamedTempFile::new()?;
                            match codec::decrypt_stream(&db_path, temp.path(), Secret::new(pwd.clone())) {
                                Ok(_) => {
                                    restore_terminal(&mut terminal)?;
                                    break pwd;
                                }
                                Err(e) => {
                                    password_overlay.set_error(format!("Decryption failed: {}", e));
                                }
                            }
                        }
                        PasswordInputResult::Cancel => {
                            restore_terminal(&mut terminal)?;
                            return Ok(());
                        }
                    }
                }
            }
        };
        
        // Decrypt to temp file
        let temp = tempfile::NamedTempFile::new()?;
        codec::decrypt_stream(&db_path, temp.path(), Secret::new(password))?;
        _temp_file = temp;
        _temp_file.as_ref().unwrap().path().to_path_buf()
    } else {
        db_path
    };
    
    // Continue with normal TUI initialization
    // ...
}
```

## 
Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Message Channel Communication

For any update operation (status change, progress update, or user prompt), the Update_Thread should send a corresponding Update_Message through the Message_Channel to the TUI_Thread.

**Validates: Requirements 1.1, 1.2, 1.3, 1.4**

### Property 2: Bidirectional Channel Communication

For any User_Prompt message sent by the Update_Thread, the TUI_Thread should be able to send a UserResponse back through the response channel, and the Update_Thread should receive it.

**Validates: Requirements 1.6**

### Property 3: Non-Blocking Message Reception

For any Update_Message in the channel, the TUI_Thread should be able to receive it using try_recv without blocking the event loop.

**Validates: Requirements 1.5, 5.1, 5.5**

### Property 4: Overlay Visibility on Message Receipt

For any Update_Message received (except UpToDate), the Update_Overlay should become visible.

**Validates: Requirements 2.1**

### Property 5: State Synchronization

For any Update_Message received, the UpdateOverlayState should transition to the appropriate Update_State that corresponds to the message type.

**Validates: Requirements 2.3, 8.2**

### Property 6: Progress Data Completeness

For any DownloadProgress message, the UpdateOverlayState should contain both the downloaded bytes, total bytes, and percentage values.

**Validates: Requirements 2.4, 4.1, 4.2**

### Property 7: Error Information Preservation

For any Error message, the UpdateOverlayState should store the error message, recovery instructions (if present), and retryable flag.

**Validates: Requirements 2.6, 6.3**

### Property 8: Dismissal After Completion

For any UpdateOverlayState in Complete, Error, or Skipped state, calling handle_key with any appropriate key should set visible to false.

**Validates: Requirements 2.7, 6.4**

### Property 9: Interactive Mode Prompt Display

For any ConfirmationRequired message received when interactive mode is enabled, the UpdateOverlayState should transition to AwaitingConfirmation state.

**Validates: Requirements 3.1**

### Property 10: User Response Mapping

For any key press ('y', 'Y', 'n', 'N') in AwaitingConfirmation state, handle_key should send the corresponding UserResponse (Confirmed or Declined).

**Validates: Requirements 3.2, 3.3, 3.4**

### Property 11: Non-Interactive Mode Behavior

For any TuiModeHandler with interactive set to false, prompt_for_update_confirmation should return true without sending a ConfirmationRequired message.

**Validates: Requirements 3.6, 9.3, 9.4**

### Property 12: Overlay Persistence Across Views

For any UpdateOverlayState with visible set to true, changing the App view should not affect the visible flag.

**Validates: Requirements 5.4**

### Property 13: Error Message Transmission

For any UpdateError encountered in the update thread, an Error message should be sent with the error's descriptive message.

**Validates: Requirements 6.1**

### Property 14: Retry Option Availability

For any UpdateOverlayState in Error state with is_retryable set to true, handle_key with 'r' or 'R' should send a Retry response.

**Validates: Requirements 6.5**

### Property 15: State Validity

For any UpdateOverlayState, the state field should always contain a valid UpdateState enum value.

**Validates: Requirements 8.1**

### Property 16: Prompt Input Capture

For any UpdateOverlayState in AwaitingConfirmation state, handle_key should process 'y', 'Y', 'n', 'N', and Esc keys and return true.

**Validates: Requirements 8.5**

### Property 17: Configuration Respect - Enabled

For any UpdateManager with config.enabled set to false, check_and_update should return immediately without performing network operations.

**Validates: Requirements 9.1**

### Property 18: Password Masking

For any PasswordInputOverlay with a password string of length n, the rendered output should contain exactly n asterisk characters.

**Validates: Requirements 10.3**

### Property 19: Password Input Handling

For any PasswordInputOverlay, handle_key with KeyCode::Char(c) should append c to the password buffer.

**Validates: Requirements 10.4**

### Property 20: Password Backspace Support

For any PasswordInputOverlay with a non-empty password buffer, handle_key with KeyCode::Backspace should remove the last character.

**Validates: Requirements 10.5**

### Property 21: Mode Detection

For any UpdateManager constructed with new_with_tui_mode, the mode_handler should be a TuiModeHandler that sends messages; for any UpdateManager constructed with new, the mode_handler should use console I/O.

**Validates: Requirements 11.1, 11.2, 11.5**

## Error Handling

### Update Thread Errors

The update thread handles errors by:

1. Catching all UpdateError instances
2. Using ErrorHandler to categorize and format errors
3. Sending Error messages to the TUI thread with:
   - Descriptive error message
   - Recovery instructions (if available)
   - Retryable flag

### Channel Communication Errors

Channel send errors are handled gracefully:

- Update thread: Ignores send errors (TUI may have shut down)
- TUI thread: Uses try_recv to avoid blocking on empty channels
- Response waiting: Uses recv with timeout to avoid indefinite blocking

### Password Decryption Errors

Password decryption errors are handled by:

1. Catching decryption failures
2. Displaying error message in the password overlay
3. Clearing the password buffer
4. Allowing the user to retry or cancel

### State Transition Errors

Invalid state transitions are prevented by:

- Using enum types for states and messages
- Pattern matching on all message types
- Defaulting to safe states on unexpected messages

## Testing Strategy

This feature will use both unit tests and property-based tests to ensure correctness.

### Unit Testing

Unit tests will focus on:

- Specific examples of message processing
- State transitions for known scenarios
- Key handling for specific inputs
- Error cases and edge conditions
- Integration between components

Example unit tests:
- Test that ConfirmationRequired message transitions to AwaitingConfirmation state
- Test that 'y' key in AwaitingConfirmation state sends Confirmed response
- Test that password overlay masks characters correctly
- Test that error messages are displayed with recovery instructions

### Property-Based Testing

Property-based tests will verify universal properties across all inputs using a property-based testing library (e.g., quickcheck or proptest for Rust).

Each property test will:
- Run a minimum of 100 iterations
- Generate random inputs appropriate to the property
- Verify the property holds for all generated inputs
- Reference the design document property in a comment tag

Example property tests:
- **Property 5**: For any randomly generated UpdateMessage, verify state transitions are correct
- **Property 10**: For any key in ['y', 'Y', 'n', 'N'], verify correct UserResponse is sent
- **Property 18**: For any random string, verify password masking produces correct number of asterisks

### Test Configuration

All property-based tests will:
- Use the proptest crate for Rust
- Run minimum 100 iterations per test
- Include a comment tag: `// Feature: ratatui-update-integration, Property N: [property text]`
- Be placed in the same module as the code they test

### Testing Balance

- Unit tests handle specific examples and edge cases
- Property tests handle comprehensive input coverage
- Together they provide confidence in correctness without excessive test count
