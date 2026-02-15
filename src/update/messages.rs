use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Messages sent from the update thread to the TUI thread
#[derive(Debug, Clone)]
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
    ConfirmationRequired { new_version: String },

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
    InstallComplete { new_version: String },

    /// Update process encountered an error
    Error {
        message: String,
        recovery_instructions: Option<String>,
        is_retryable: bool,
    },

    /// Update was skipped
    Skipped { reason: String },
}

/// Messages sent from the TUI thread to the update thread
#[derive(Debug, Clone)]
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

/// Encapsulates the communication channels for update operations
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
    /// Create new bidirectional channels for update communication
    pub fn new() -> Self {
        let (message_tx, message_rx) = std::sync::mpsc::channel();
        let (response_tx, response_rx) = std::sync::mpsc::channel();

        Self {
            message_tx,
            message_rx,
            response_tx,
            response_rx,
        }
    }

    /// Split channels into TUI and update thread components
    pub fn split(self) -> (TuiChannels, UpdateThreadChannels) {
        (
            TuiChannels {
                message_rx: self.message_rx,
                response_tx: self.response_tx,
            },
            UpdateThreadChannels {
                message_tx: self.message_tx,
                response_rx: Arc::new(Mutex::new(self.response_rx)),
            },
        )
    }
}

impl Default for UpdateChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Channels used by the TUI thread
pub struct TuiChannels {
    pub message_rx: Receiver<UpdateMessage>,
    pub response_tx: Sender<UserResponse>,
}

/// Channels used by the update thread
#[derive(Debug, Clone)]
pub struct UpdateThreadChannels {
    pub message_tx: Sender<UpdateMessage>,
    pub response_rx: Arc<Mutex<Receiver<UserResponse>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_channels_creation() {
        let channels = UpdateChannels::new();
        // Verify channels are created successfully
        assert!(channels.message_tx.send(UpdateMessage::CheckStarted).is_ok());
        assert!(channels.message_rx.recv().is_ok());
    }

    #[test]
    fn test_update_channels_split() {
        let channels = UpdateChannels::new();
        let (tui_channels, update_channels) = channels.split();

        // Send from update thread
        assert!(update_channels
            .message_tx
            .send(UpdateMessage::CheckStarted)
            .is_ok());

        // Receive on TUI thread
        assert!(matches!(
            tui_channels.message_rx.recv(),
            Ok(UpdateMessage::CheckStarted)
        ));

        // Send response from TUI thread
        assert!(tui_channels
            .response_tx
            .send(UserResponse::Confirmed)
            .is_ok());

        // Receive response on update thread
        assert!(matches!(
            update_channels.response_rx.recv(),
            Ok(UserResponse::Confirmed)
        ));
    }

    #[test]
    fn test_update_message_variants() {
        let msg = UpdateMessage::CheckStarted;
        assert!(matches!(msg, UpdateMessage::CheckStarted));

        let msg = UpdateMessage::UpToDate {
            current_version: "1.0.0".to_string(),
        };
        assert!(matches!(msg, UpdateMessage::UpToDate { .. }));

        let msg = UpdateMessage::UpdateAvailable {
            current_version: "1.0.0".to_string(),
            new_version: "1.1.0".to_string(),
        };
        assert!(matches!(msg, UpdateMessage::UpdateAvailable { .. }));

        let msg = UpdateMessage::Error {
            message: "Test error".to_string(),
            recovery_instructions: None,
            is_retryable: false,
        };
        assert!(matches!(msg, UpdateMessage::Error { .. }));
    }

    #[test]
    fn test_user_response_variants() {
        let resp = UserResponse::Confirmed;
        assert!(matches!(resp, UserResponse::Confirmed));

        let resp = UserResponse::Declined;
        assert!(matches!(resp, UserResponse::Declined));

        let resp = UserResponse::Retry;
        assert!(matches!(resp, UserResponse::Retry));

        let resp = UserResponse::Dismissed;
        assert!(matches!(resp, UserResponse::Dismissed));
    }
}
