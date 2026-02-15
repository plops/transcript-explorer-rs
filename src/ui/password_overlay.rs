use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Result of password input
#[derive(Debug, Clone)]
pub enum PasswordInputResult {
    /// User submitted the password
    Submit(String),
    /// User cancelled the input
    Cancel,
}

/// Password input overlay for encrypted databases
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
    /// Create a new password input overlay
    pub fn new(prompt: String) -> Self {
        Self {
            password: String::new(),
            error_message: None,
            prompt,
            active: true,
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<PasswordInputResult> {
        match key.code {
            KeyCode::Enter => {
                if !self.password.is_empty() {
                    Some(PasswordInputResult::Submit(std::mem::take(
                        &mut self.password,
                    )))
                } else {
                    None
                }
            }
            KeyCode::Esc => Some(PasswordInputResult::Cancel),
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

    /// Set an error message
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.password.clear();
    }

    /// Render the password input overlay
    pub fn render(&self, frame: &mut Frame) {
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
        let prompt = Paragraph::new(self.prompt.as_str()).alignment(Alignment::Center);
        frame.render_widget(prompt, chunks[0]);

        // Password field (masked)
        let masked = "*".repeat(self.password.len());
        let password_field = Paragraph::new(masked)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        frame.render_widget(password_field, chunks[2]);

        // Error message if present
        if let Some(error) = &self.error_message {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            frame.render_widget(error_widget, chunks[3]);
        } else {
            let help = Paragraph::new("Enter: Submit | Esc: Cancel")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(help, chunks[3]);
        }
    }

    /// Check if the overlay is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Deactivate the overlay
    pub fn deactivate(&mut self) {
        self.active = false;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_input_overlay_creation() {
        let overlay = PasswordInputOverlay::new("Enter password:".to_string());
        assert!(overlay.is_active());
        assert_eq!(overlay.prompt, "Enter password:");
        assert!(overlay.error_message.is_none());
    }

    #[test]
    fn test_password_input_overlay_char_input() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());

        let key = crossterm::event::KeyEvent::new(KeyCode::Char('a'), Default::default());
        let result = overlay.handle_key(key);

        assert!(result.is_none());
        assert_eq!(overlay.password, "a");
    }

    #[test]
    fn test_password_input_overlay_backspace() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());

        overlay.password = "abc".to_string();

        let key = crossterm::event::KeyEvent::new(KeyCode::Backspace, Default::default());
        let result = overlay.handle_key(key);

        assert!(result.is_none());
        assert_eq!(overlay.password, "ab");
    }

    #[test]
    fn test_password_input_overlay_enter_with_password() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());
        overlay.password = "mypassword".to_string();

        let key = crossterm::event::KeyEvent::new(KeyCode::Enter, Default::default());
        let result = overlay.handle_key(key);

        assert!(result.is_some());
        match result.unwrap() {
            PasswordInputResult::Submit(pwd) => assert_eq!(pwd, "mypassword"),
            _ => panic!("Expected Submit result"),
        }
    }

    #[test]
    fn test_password_input_overlay_enter_without_password() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());

        let key = crossterm::event::KeyEvent::new(KeyCode::Enter, Default::default());
        let result = overlay.handle_key(key);

        assert!(result.is_none());
    }

    #[test]
    fn test_password_input_overlay_escape() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());
        overlay.password = "mypassword".to_string();

        let key = crossterm::event::KeyEvent::new(KeyCode::Esc, Default::default());
        let result = overlay.handle_key(key);

        assert!(result.is_some());
        match result.unwrap() {
            PasswordInputResult::Cancel => {}
            _ => panic!("Expected Cancel result"),
        }
    }

    #[test]
    fn test_password_input_overlay_set_error() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());
        overlay.password = "wrongpassword".to_string();

        overlay.set_error("Incorrect password".to_string());

        assert_eq!(overlay.error_message, Some("Incorrect password".to_string()));
        assert_eq!(overlay.password, "");
    }

    #[test]
    fn test_password_input_overlay_deactivate() {
        let mut overlay = PasswordInputOverlay::new("Enter password:".to_string());
        assert!(overlay.is_active());

        overlay.deactivate();
        assert!(!overlay.is_active());
    }
}
