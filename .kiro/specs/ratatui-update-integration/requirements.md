# Requirements Document

## Introduction

This feature integrates the auto-updater module with the ratatui TUI framework to eliminate direct console I/O operations that disrupt the TUI display. The auto-updater currently uses println!, eprintln!, print!, and std::io::stdin() for user interaction, which overwrites the TUI rendering. This integration will replace direct console I/O with a message-passing system between the update thread and the TUI thread, enabling seamless update notifications and interactions within the TUI interface.

## Glossary

- **Update_Thread**: The background thread that performs update checks and downloads
- **TUI_Thread**: The main thread that runs the ratatui event loop and renders the UI
- **Update_Message**: A message sent from the Update_Thread to the TUI_Thread containing update status, progress, or requests for user input
- **Update_Overlay**: A TUI popup or modal that displays update information over the main application interface
- **Message_Channel**: A thread-safe communication channel (e.g., mpsc) for passing Update_Messages between threads
- **Update_State**: The current state of the update process (checking, downloading, installing, complete, error)
- **Progress_Indicator**: A visual element in the TUI showing download or installation progress
- **User_Prompt**: A TUI dialog requesting user input or confirmation for update actions
- **Interactive_Mode**: The mode where the auto-updater requires user confirmation before proceeding
- **Non_Interactive_Mode**: The mode where the auto-updater proceeds automatically without user prompts

## Requirements

### Requirement 1: Message-Based Communication

**User Story:** As a developer, I want the update thread to communicate with the TUI thread through message passing, so that update operations don't directly write to the console.

#### Acceptance Criteria

1. THE Update_Thread SHALL send Update_Messages to the TUI_Thread through a Message_Channel
2. WHEN an update status changes, THE Update_Thread SHALL send a status Update_Message
3. WHEN download progress updates, THE Update_Thread SHALL send a progress Update_Message at regular intervals
4. WHEN user input is required, THE Update_Thread SHALL send a prompt Update_Message and wait for a response
5. THE TUI_Thread SHALL receive Update_Messages without blocking the main event loop
6. THE Message_Channel SHALL support bidirectional communication for user responses

### Requirement 2: Update Overlay Display

**User Story:** As a user, I want to see update notifications and progress within the TUI interface, so that the display remains consistent and readable.

#### Acceptance Criteria

1. WHEN an Update_Message is received, THE TUI_Thread SHALL display an Update_Overlay
2. THE Update_Overlay SHALL render on top of the current view without disrupting it
3. WHEN displaying update status, THE Update_Overlay SHALL show the current Update_State
4. WHEN displaying download progress, THE Update_Overlay SHALL show a Progress_Indicator with percentage and bytes transferred
5. WHEN an update completes successfully, THE Update_Overlay SHALL display a success message with the new version number
6. WHEN an update error occurs, THE Update_Overlay SHALL display the error message and recovery instructions
7. THE Update_Overlay SHALL be dismissible by the user after completion or error

### Requirement 3: TUI-Based User Prompts

**User Story:** As a user, I want to respond to update prompts through the TUI interface, so that I can control update behavior without console input.

#### Acceptance Criteria

1. WHEN Interactive_Mode is enabled and user confirmation is required, THE TUI_Thread SHALL display a User_Prompt in the Update_Overlay
2. THE User_Prompt SHALL accept keyboard input through the ratatui event loop
3. WHEN the user confirms an update, THE TUI_Thread SHALL send a confirmation response to the Update_Thread
4. WHEN the user declines an update, THE TUI_Thread SHALL send a decline response to the Update_Thread
5. THE User_Prompt SHALL display clear instructions for accepting (y/Y) or declining (n/N) the update
6. WHEN Non_Interactive_Mode is enabled, THE Update_Thread SHALL proceed without sending prompt messages

### Requirement 4: Progress Visualization

**User Story:** As a user, I want to see real-time update progress in the TUI, so that I know the update is proceeding and how long it will take.

#### Acceptance Criteria

1. WHEN a download is in progress, THE Progress_Indicator SHALL display the percentage complete
2. WHEN a download is in progress, THE Progress_Indicator SHALL display bytes downloaded and total bytes
3. THE Progress_Indicator SHALL update smoothly without flickering or disrupting the TUI
4. WHEN download speed can be calculated, THE Progress_Indicator SHALL display the current download speed
5. THE Progress_Indicator SHALL use a visual bar representation for percentage complete

### Requirement 5: Non-Blocking Update Operations

**User Story:** As a user, I want to continue using the application while updates are checked or downloaded, so that my workflow is not interrupted.

#### Acceptance Criteria

1. WHILE an update check is in progress, THE TUI_Thread SHALL remain responsive to user input
2. WHILE a download is in progress, THE TUI_Thread SHALL continue rendering the main application interface
3. THE Update_Thread SHALL perform all network operations asynchronously
4. WHEN the user navigates to different views, THE Update_Overlay SHALL remain visible and accessible
5. THE TUI_Thread SHALL process Update_Messages during its normal event loop without blocking

### Requirement 6: Error Handling and Display

**User Story:** As a user, I want to see clear error messages when updates fail, so that I understand what went wrong and what actions I can take.

#### Acceptance Criteria

1. WHEN an update error occurs, THE Update_Thread SHALL send an error Update_Message with the error details
2. THE Update_Overlay SHALL display the error message in a visually distinct style
3. IF recovery instructions are available, THEN THE Update_Overlay SHALL display them
4. THE Update_Overlay SHALL allow the user to dismiss the error message
5. WHEN an error is dismissible and retryable, THE Update_Overlay SHALL offer a retry option

### Requirement 7: Refactoring Console I/O

**User Story:** As a developer, I want all direct console I/O removed from the update module, so that it can integrate cleanly with the TUI.

#### Acceptance Criteria

1. THE Update_Thread SHALL NOT use println! for status messages
2. THE Update_Thread SHALL NOT use eprintln! for error messages
3. THE Update_Thread SHALL NOT use print! for progress updates
4. THE Update_Thread SHALL NOT use std::io::stdin() for user input
5. THE UserFeedback struct SHALL be refactored to send messages instead of printing
6. THE InteractiveMode struct SHALL be refactored to use message-based prompts
7. THE NonInteractiveMode struct SHALL be refactored to send log messages instead of printing

### Requirement 8: Update State Management

**User Story:** As a developer, I want the TUI to track update state, so that it can display appropriate UI elements and handle user interactions correctly.

#### Acceptance Criteria

1. THE TUI_Thread SHALL maintain the current Update_State
2. WHEN an Update_Message is received, THE TUI_Thread SHALL update the Update_State accordingly
3. THE Update_State SHALL include: idle, checking, available, downloading, installing, complete, error, and user_prompt states
4. THE Update_Overlay SHALL render differently based on the current Update_State
5. WHEN the Update_State is user_prompt, THE TUI_Thread SHALL capture keyboard input for the prompt

### Requirement 9: Configuration Compatibility

**User Story:** As a user, I want existing update configuration to work with the TUI integration, so that my settings are preserved.

#### Acceptance Criteria

1. THE Update_Thread SHALL respect the enabled configuration setting
2. THE Update_Thread SHALL respect the check_interval_hours configuration setting
3. THE Update_Thread SHALL respect the interactive_mode configuration setting
4. WHEN interactive_mode is false, THE Update_Thread SHALL NOT send User_Prompt messages
5. THE UpdateConfiguration SHALL remain unchanged by the TUI integration

### Requirement 10: Password Entry for Encrypted Databases

**User Story:** As a user, I want to enter database decryption passwords through the TUI interface, so that I can open encrypted databases without the TUI being disrupted.

#### Acceptance Criteria

1. WHEN an encrypted database is detected at startup, THE Application SHALL display a password input overlay before initializing the TUI
2. THE password input overlay SHALL use ratatui rendering instead of console I/O
3. THE password input field SHALL mask entered characters with asterisks for security
4. THE password input field SHALL accept keyboard input through crossterm event handling
5. THE password input field SHALL support backspace for editing
6. WHEN the user submits the password, THE Application SHALL attempt to decrypt the database
7. WHEN the password is incorrect, THE Application SHALL display an error message and request the password again
8. THE password input overlay SHALL provide clear instructions for entering the password and submitting

### Requirement 11: Graceful Degradation

**User Story:** As a developer, I want the update system to work in both TUI and non-TUI modes, so that command-line usage remains functional.

#### Acceptance Criteria

1. WHEN the application runs in TUI mode, THE Update_Thread SHALL use message-based communication
2. WHEN the application runs in non-TUI mode (e.g., CLI commands), THE Update_Thread SHALL use console I/O
3. THE Update_Thread SHALL detect which mode is active at initialization
4. THE UpdateManager SHALL accept an optional Message_Channel during construction
5. IF no Message_Channel is provided, THEN THE Update_Thread SHALL fall back to console I/O
