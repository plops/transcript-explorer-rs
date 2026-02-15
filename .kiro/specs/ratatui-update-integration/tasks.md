# Implementation Plan: Ratatui Update Integration

## Overview

This implementation plan converts the auto-updater's console I/O operations to a message-passing architecture compatible with ratatui. The work is organized into discrete steps that build incrementally, starting with core message types and channels, then refactoring the update module, adding TUI components, and finally integrating everything together.

## Tasks

- [x] 1. Create message types and channel infrastructure
  - Create `src/update/messages.rs` module
  - Define `UpdateMessage` enum with all message variants
  - Define `UserResponse` enum
  - Define `UpdateChannels`, `TuiChannels`, and `UpdateThreadChannels` structs
  - Implement `UpdateChannels::new()` and `split()` methods
  - Export types from `src/update/mod.rs`
  - _Requirements: 1.1, 1.6_

- [ ]* 1.1 Write property test for bidirectional channel communication
  - **Property 2: Bidirectional Channel Communication**
  - **Validates: Requirements 1.6**

- [x] 2. Create UpdateOverlayState and state management
  - [x] 2.1 Define UpdateOverlayState struct and UpdateState enum
    - Create `src/ui/update_overlay.rs` module
    - Define `UpdateState` enum with all states
    - Define `DownloadProgress` struct
    - Define `UpdateOverlayState` struct with all fields
    - _Requirements: 2.1, 8.1, 8.3_

  - [x] 2.2 Implement UpdateOverlayState::new() and process_message()
    - Implement constructor with default values
    - Implement `process_message()` method with pattern matching for all message types
    - Update state transitions based on message type
    - Store message data in appropriate fields
    - _Requirements: 2.1, 2.3, 2.4, 2.6, 8.2_

  - [ ]* 2.3 Write property test for state synchronization
    - **Property 5: State Synchronization**
    - **Validates: Requirements 2.3, 8.2**

  - [ ]* 2.4 Write property test for progress data completeness
    - **Property 6: Progress Data Completeness**
    - **Validates: Requirements 2.4, 4.1, 4.2**

  - [ ]* 2.5 Write property test for error information preservation
    - **Property 7: Error Information Preservation**
    - **Validates: Requirements 2.6, 6.3**

  - [x] 2.6 Implement UpdateOverlayState::handle_key()
    - Implement key handling for AwaitingConfirmation state (y/n keys)
    - Implement key handling for Complete/Skipped states (any key dismisses)
    - Implement key handling for Error state (r for retry, q/esc to dismiss)
    - Send appropriate UserResponse through response_sender
    - _Requirements: 3.2, 3.3, 3.4, 6.4, 6.5, 8.5_

  - [ ]* 2.7 Write property test for user response mapping
    - **Property 10: User Response Mapping**
    - **Validates: Requirements 3.2, 3.3, 3.4**

  - [ ]* 2.8 Write property test for dismissal after completion
    - **Property 8: Dismissal After Completion**
    - **Validates: Requirements 2.7, 6.4**

  - [ ]* 2.9 Write property test for retry option availability
    - **Property 14: Retry Option Availability**
    - **Validates: Requirements 6.5**

  - [ ]* 2.10 Write property test for prompt input capture
    - **Property 16: Prompt Input Capture**
    - **Validates: Requirements 8.5**

- [x] 3. Implement TuiModeHandler for update module
  - [x] 3.1 Create TuiModeHandler struct
    - Add `TuiModeHandler` struct in `src/update/mod.rs`
    - Store `UpdateThreadChannels` and `interactive` flag
    - Implement `new()` constructor
    - Implement helper methods `send_message()` and `wait_for_response()`
    - _Requirements: 1.1, 1.4_

  - [x] 3.2 Implement ModeHandler trait for TuiModeHandler
    - Implement `prompt_before_check()` - send CheckStarted message
    - Implement `prompt_for_update_confirmation()` - send ConfirmationRequired and wait for response
    - Implement `check_for_cancellation()` - use try_recv for non-blocking check
    - Implement `display_status()` - no-op in TUI mode
    - Implement `display_progress()` - send DownloadProgress message
    - Implement `display_success()` - send InstallComplete message
    - Implement `display_error()` - send Error message with formatted details
    - Implement `finish_progress()` - send DownloadComplete message
    - _Requirements: 1.2, 1.3, 1.4, 3.1, 3.6, 6.1_

  - [ ]* 3.3 Write property test for message channel communication
    - **Property 1: Message Channel Communication**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4**

  - [ ]* 3.4 Write property test for non-interactive mode behavior
    - **Property 11: Non-Interactive Mode Behavior**
    - **Validates: Requirements 3.6, 9.3, 9.4**

  - [ ]* 3.5 Write property test for error message transmission
    - **Property 13: Error Message Transmission**
    - **Validates: Requirements 6.1**

- [x] 4. Add TUI mode support to UpdateManager
  - [x] 4.1 Add new_with_tui_mode() constructor
    - Add `new_with_tui_mode()` method to `UpdateManager`
    - Accept `UpdateConfiguration` and `UpdateThreadChannels` parameters
    - Create `TuiModeHandler` instead of Interactive/NonInteractive mode
    - Initialize all other fields same as `new()`
    - _Requirements: 11.1, 11.4_

  - [ ]* 4.2 Write property test for mode detection
    - **Property 21: Mode Detection**
    - **Validates: Requirements 11.1, 11.2, 11.5**

  - [ ]* 4.3 Write unit test for configuration respect
    - **Property 17: Configuration Respect - Enabled**
    - **Validates: Requirements 9.1**

- [x] 5. Checkpoint - Ensure update module tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Implement update overlay rendering
  - [x] 6.1 Create rendering helper functions
    - Implement `centered_rect()` helper function
    - Implement `format_bytes()` helper function
    - Add imports for ratatui widgets and layout
    - _Requirements: 2.2_

  - [x] 6.2 Implement state-specific rendering functions
    - Implement `render_checking()` - display "Checking for updates..."
    - Implement `render_available()` - display current and new versions
    - Implement `render_confirmation()` - display prompt with Y/N instructions
    - Implement `render_downloading()` - display progress bar and bytes
    - Implement `render_installing()` - display "Installing update..."
    - Implement `render_complete()` - display success message with version
    - Implement `render_error()` - display error message and recovery instructions
    - Implement `render_skipped()` - display skip reason
    - _Requirements: 2.3, 2.4, 2.5, 2.6, 3.5_

  - [x] 6.3 Implement main render() function
    - Implement `render()` function that dispatches to state-specific renderers
    - Use `Clear` widget to clear overlay area
    - Call appropriate render function based on `state.state`
    - _Requirements: 2.1, 2.2_

- [x] 7. Create PasswordInputOverlay component
  - [x] 7.1 Define PasswordInputOverlay struct and methods
    - Create `src/ui/password_overlay.rs` module
    - Define `PasswordInputOverlay` struct with password buffer and error message
    - Define `PasswordInputResult` enum (Submit, Cancel)
    - Implement `new()` constructor
    - Implement `set_error()` method
    - _Requirements: 10.1_

  - [x] 7.2 Implement password input key handling
    - Implement `handle_key()` method
    - Handle Enter key - return Submit with password if non-empty
    - Handle Esc key - return Cancel
    - Handle Char key - append character to password buffer
    - Handle Backspace key - remove last character if buffer non-empty
    - _Requirements: 10.4, 10.5_

  - [ ]* 7.3 Write property test for password masking
    - **Property 18: Password Masking**
    - **Validates: Requirements 10.3**

  - [ ]* 7.4 Write property test for password input handling
    - **Property 19: Password Input Handling**
    - **Validates: Requirements 10.4**

  - [ ]* 7.5 Write property test for password backspace support
    - **Property 20: Password Backspace Support**
    - **Validates: Requirements 10.5**

  - [x] 7.6 Implement password overlay rendering
    - Implement `render()` method
    - Create centered overlay with border
    - Display prompt message
    - Display masked password (asterisks)
    - Display error message if present, otherwise show help text
    - _Requirements: 10.3, 10.8_

- [x] 8. Integrate update overlay into App
  - [x] 8.1 Add update overlay fields to App struct
    - Add `update_overlay: UpdateOverlayState` field
    - Add `update_message_rx: Option<Receiver<UpdateMessage>>` field
    - Add `update_response_tx: Option<Sender<UserResponse>>` field
    - Initialize in `App::new()`
    - _Requirements: 8.1_

  - [x] 8.2 Add channel setup and message polling methods
    - Implement `set_update_channels()` method
    - Implement `poll_update_messages()` method using try_recv
    - Call `update_overlay.process_message()` for each received message
    - _Requirements: 1.5, 5.1_

  - [ ]* 8.3 Write property test for non-blocking message reception
    - **Property 3: Non-Blocking Message Reception**
    - **Validates: Requirements 1.5, 5.1, 5.5**

  - [ ]* 8.4 Write property test for overlay visibility on message receipt
    - **Property 4: Overlay Visibility on Message Receipt**
    - **Validates: Requirements 2.1**

  - [ ]* 8.5 Write property test for overlay persistence across views
    - **Property 12: Overlay Persistence Across Views**
    - **Validates: Requirements 5.4**

- [x] 9. Update main UI rendering to include overlay
  - Modify `src/ui/mod.rs` render function
  - Add conditional rendering of update overlay after main view
  - Call `update_overlay::render()` if overlay is visible
  - Ensure overlay renders before help overlay
  - Export `update_overlay` module
  - _Requirements: 2.1, 2.2, 5.4_

- [x] 10. Integrate update channels into main event loop
  - [x] 10.1 Create and split update channels in run_app()
    - Create `UpdateChannels` before starting event loop
    - Call `split()` to get TUI and update thread channels
    - Pass TUI channels to `app.set_update_channels()`
    - _Requirements: 1.1, 1.6_

  - [x] 10.2 Spawn update thread with TUI mode
    - Load `UpdateConfiguration`
    - Check if updates are enabled
    - Create `UpdateManager` with `new_with_tui_mode()` if enabled
    - Spawn background thread
    - Store join handle
    - _Requirements: 9.1, 11.1_

  - [x] 10.3 Add update message polling to event loop
    - Call `app.poll_update_messages()` at start of each loop iteration
    - Ensure polling happens before rendering
    - _Requirements: 1.5, 5.1, 5.5_

  - [x] 10.4 Add update overlay key handling to event loop
    - Check if `app.update_overlay.is_visible()` before normal key handling
    - If visible, call `app.update_overlay.handle_key(key)`
    - If handle_key returns true, skip normal key handling
    - Otherwise, proceed with normal key handling
    - _Requirements: 3.2, 8.5_

- [x] 11. Implement password entry for encrypted databases
  - [x] 11.1 Add password entry before TUI initialization
    - Check if database is encrypted using `codec::is_encrypted()`
    - If encrypted, initialize terminal early
    - Create `PasswordInputOverlay` with appropriate prompt
    - Enter loop to render overlay and handle input
    - _Requirements: 10.1_

  - [x] 11.2 Handle password submission and decryption
    - On Enter key, attempt decryption with entered password
    - If successful, break loop and continue with TUI
    - If failed, call `set_error()` and continue loop
    - On Esc key, restore terminal and exit
    - _Requirements: 10.6, 10.7_

  - [ ]* 11.3 Write unit tests for password entry flow
    - Test that encrypted database triggers password overlay
    - Test that correct password proceeds to TUI
    - Test that incorrect password shows error and retries
    - Test that Esc cancels and exits

- [x] 12. Checkpoint - Integration testing
  - Ensure all tests pass, ask the user if questions arise.

- [x] 13. Clean up console I/O from update module (optional verification)
  - Verify no println!, eprintln!, print!, or eprint! remain in TUI code paths
  - Verify no std::io::stdin() calls remain in TUI code paths
  - Console I/O should only exist in non-TUI mode (UpdateManager::new())
  - _Requirements: 7.1, 7.2, 7.3, 7.4_

## Notes

- Tasks marked with `*` are optional property-based tests and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The implementation maintains backward compatibility with non-TUI mode (CLI commands)
