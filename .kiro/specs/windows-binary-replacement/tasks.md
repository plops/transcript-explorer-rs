# Implementation Plan: Windows Binary Replacement Fix

## Overview

This implementation plan breaks down the Windows binary replacement fix into discrete coding steps. The fix modifies the `BinaryReplacer::perform_replacement()` method in `src/update/mod.rs` to use the standard Windows pattern (rename-move-cleanup) instead of the failing `fs::copy()` approach. The implementation maintains cross-platform compatibility by keeping Unix behavior unchanged while introducing Windows-specific logic.

## Tasks

- [ ] 1. Implement Windows rename-move-cleanup pattern in BinaryReplacer
  - [ ] 1.1 Modify BinaryReplacer::perform_replacement() for Windows
    - Replace the `fs::copy()` call with rename-move-cleanup sequence
    - Rename current executable to `.old` backup
    - Move new executable to original location
    - Attempt to delete backup file (ignore failures)
    - _Requirements: 1.1, 1.2, 1.3, 2.1_
  
  - [ ]* 1.2 Write property test for Windows rename-move pattern
    - **Property 1: Windows Rename-Move Pattern Completes Successfully**
    - **Validates: Requirements 1.1, 1.2, 1.3**
    - Generate random executable paths and verify new executable ends up at original location
    - Verify backup file exists after successful replacement
  
  - [ ] 1.3 Verify Unix atomic rename behavior remains unchanged
    - Ensure Unix code path still uses `fs::rename()` for atomic operation
    - No changes to Unix implementation
    - _Requirements: 3.1_
  
  - [ ]* 1.4 Write property test for Unix atomic rename
    - **Property 2: Unix Atomic Rename Behavior Unchanged**
    - **Validates: Requirements 3.1**
    - Generate random executable paths and verify atomic rename works
    - Verify no backup file created on Unix

- [ ] 2. Implement error handling for Windows replacement failures
  - [ ] 2.1 Add error handling for rename operation failure
    - Catch errors when renaming current executable to `.old`
    - Return `UpdateError::Replacement` with descriptive reason
    - _Requirements: 4.1_
  
  - [ ] 2.2 Add error handling for move operation failure
    - Catch errors when moving new executable to original location
    - Attempt rollback by restoring backup file
    - Return error with rollback status
    - _Requirements: 1.4, 4.2_
  
  - [ ]* 2.3 Write property test for error handling
    - **Property 6: Failed Replacement Returns Error with Reason**
    - **Validates: Requirements 4.1, 4.2**
    - Simulate various failure scenarios
    - Verify errors contain non-empty reason strings

- [ ] 3. Implement backup file cleanup and failure tolerance
  - [ ] 3.1 Implement graceful backup file deletion
    - Attempt to delete backup file after successful move
    - Ignore deletion failures (file may be locked by OS)
    - Log deletion failures as warnings, not errors
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [ ]* 3.2 Write property test for deletion failure tolerance
    - **Property 7: Deletion Failure Does Not Fail Update**
    - **Validates: Requirements 2.2, 2.3**
    - Simulate deletion failure
    - Verify operation succeeds despite deletion failure

- [ ] 4. Implement rollback and recovery logic
  - [ ] 4.1 Implement rollback when move operation fails
    - Restore backup file to original location if move fails
    - Handle rollback failures gracefully
    - Preserve backup file for manual recovery
    - _Requirements: 1.4, 1.5, 4.5_
  
  - [ ]* 4.2 Write property test for rollback behavior
    - **Property 8: Rollback Restores Original on Move Failure**
    - **Validates: Requirements 1.4**
    - Simulate move failure
    - Verify rollback restores original executable

- [ ] 5. Implement backup file preservation
  - [ ] 5.1 Ensure backup files are created and preserved
    - Verify backup file exists after successful replacement
    - Verify backup file exists after failed replacement
    - _Requirements: 2.1, 3.3, 4.5_
  
  - [ ]* 5.2 Write property test for backup preservation on success
    - **Property 3: Backup File Preservation on Success**
    - **Validates: Requirements 2.1, 3.3**
    - Generate random executable paths
    - Verify backup file exists after successful replacement
  
  - [ ]* 5.3 Write property test for backup preservation on failure
    - **Property 4: Backup File Preservation on Failure**
    - **Validates: Requirements 1.5, 4.5**
    - Simulate various failure scenarios
    - Verify backup file exists after failure

- [ ] 6. Implement success result handling
  - [ ] 6.1 Ensure successful replacements return Ok(())
    - Verify return type is `Result<(), UpdateError>`
    - Return `Ok(())` on successful replacement
    - _Requirements: 3.4_
  
  - [ ]* 6.2 Write property test for success result
    - **Property 5: Successful Replacement Returns Success Result**
    - **Validates: Requirements 3.4**
    - Generate random executable paths
    - Verify successful operations return `Ok(())`

- [ ] 7. Checkpoint - Verify implementation compiles and basic tests pass
  - Ensure code compiles without errors or warnings
  - Run existing unit tests to verify no regressions
  - Verify Windows-specific code compiles on Windows target
  - Ask the user if questions arise.

- [ ] 8. Run comprehensive test suite
  - [ ] 8.1 Run all unit tests for BinaryReplacer
    - Test successful Windows replacement
    - Test successful Unix replacement
    - Test rename failure handling
    - Test move failure and rollback
    - Test deletion failure handling
    - Test rollback failure handling
    - _Requirements: 5.1, 5.2, 5.4, 5.5_
  
  - [ ]* 8.2 Run all property-based tests
    - **Property 1: Windows Rename-Move Pattern**
    - **Property 2: Unix Atomic Rename**
    - **Property 3: Backup Preservation on Success**
    - **Property 4: Backup Preservation on Failure**
    - **Property 5: Success Result**
    - **Property 6: Error with Reason**
    - **Property 7: Deletion Failure Tolerance**
    - **Property 8: Rollback Restores Original**
    - Minimum 100 iterations per property test
    - _Requirements: 5.1, 5.3_

- [ ] 9. Final checkpoint - Ensure all tests pass
  - Ensure all unit tests pass
  - Ensure all property-based tests pass
  - Verify no regressions in existing functionality
  - Ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional property-based tests and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties across many inputs
- Unit tests validate specific examples and edge cases
- The fix is isolated to `src/update/mod.rs` in the `BinaryReplacer::perform_replacement()` method
- Cross-platform compatibility is maintained: Unix uses atomic rename, Windows uses rename-move-cleanup

