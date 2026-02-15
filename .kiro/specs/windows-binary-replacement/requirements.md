# Requirements Document: Windows Binary Replacement Fix

## Introduction

The transcript-explorer auto-update feature currently fails on Windows due to file locking constraints. Windows prevents overwriting a currently executing binary file, causing `AccessDenied` errors during the update process. This specification defines requirements for implementing the standard Windows binary replacement pattern: rename the running executable to a temporary backup, move the new executable to the original location, and clean up the backup file.

## Glossary

- **BinaryReplacer**: The component responsible for replacing the running executable with a new version
- **Current Executable**: The running binary file that is currently executing (e.g., `app.exe`)
- **New Executable**: The downloaded and verified binary ready to replace the current executable
- **Backup File**: A temporary file created by renaming the current executable (e.g., `app.exe.old`)
- **File Locking**: Windows OS constraint preventing write/overwrite operations on executing binaries
- **Atomic Operation**: An operation that completes entirely or not at all, without intermediate states
- **Health Check**: Verification that the new executable is valid and executable

## Requirements

### Requirement 1: Windows Binary Replacement via Rename-Move Pattern

**User Story:** As a Windows user, I want the auto-update feature to successfully replace the running binary, so that I can receive updates without manual intervention.

#### Acceptance Criteria

1. WHEN the BinaryReplacer attempts to replace the current executable on Windows THEN the system SHALL rename the current executable to a backup file with `.old` extension
2. WHEN the current executable has been renamed to a backup file THEN the system SHALL move the new executable to the original location
3. WHEN the new executable has been moved to the original location THEN the system SHALL verify the replacement was successful by checking file existence
4. IF the new executable move fails THEN the system SHALL attempt to restore the backup file to its original location
5. IF the backup restoration fails THEN the system SHALL log the error and return a recovery error indicating manual intervention may be needed

### Requirement 2: Backup File Cleanup

**User Story:** As a system administrator, I want backup files to be cleaned up after successful updates, so that disk space is not wasted with obsolete binaries.

#### Acceptance Criteria

1. WHEN the new executable has been successfully moved to the original location THEN the system SHALL attempt to delete the backup file
2. IF the backup file deletion fails THEN the system SHALL log the failure but not fail the update operation
3. IF the backup file is locked by the OS THEN the system SHALL ignore the deletion failure and allow the OS to clean it up later

### Requirement 3: Cross-Platform Compatibility

**User Story:** As a developer, I want the binary replacement logic to work consistently across all platforms, so that the update system is reliable everywhere.

#### Acceptance Criteria

1. WHEN the BinaryReplacer runs on Unix systems THEN the system SHALL use atomic rename operation (existing behavior)
2. WHEN the BinaryReplacer runs on Windows THEN the system SHALL use the rename-move-cleanup pattern (new behavior)
3. WHEN the BinaryReplacer runs on any platform THEN the system SHALL preserve the backup file for recovery purposes
4. WHEN the BinaryReplacer completes successfully on any platform THEN the system SHALL return a success result

### Requirement 4: Error Handling and Recovery

**User Story:** As a user, I want clear error messages when binary replacement fails, so that I understand what went wrong and can take corrective action.

#### Acceptance Criteria

1. WHEN the rename operation fails THEN the system SHALL return an error with the specific reason (e.g., permission denied, file not found)
2. WHEN the move operation fails THEN the system SHALL attempt rollback and return an error indicating the rollback status
3. WHEN a file operation fails due to permissions THEN the system SHALL include recovery instructions in the error message
4. WHEN the backup file cannot be deleted THEN the system SHALL log this as a non-critical warning and continue
5. IF any critical operation fails THEN the system SHALL preserve the backup file for manual recovery

### Requirement 5: Existing Test Compatibility

**User Story:** As a developer, I want all existing tests to continue passing, so that the fix doesn't introduce regressions.

#### Acceptance Criteria

1. WHEN the test suite runs THEN all existing binary replacement tests SHALL pass
2. WHEN the test suite runs on Unix systems THEN the atomic rename behavior SHALL remain unchanged
3. WHEN the test suite runs on Windows THEN the new rename-move-cleanup pattern SHALL be tested
4. WHEN tests verify backup creation THEN the backup file SHALL exist after replacement
5. WHEN tests verify rollback THEN the original file SHALL be restored if replacement fails

