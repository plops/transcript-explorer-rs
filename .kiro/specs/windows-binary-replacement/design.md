# Design Document: Windows Binary Replacement Fix

## Overview

This design addresses the critical Windows file locking issue in the auto-update system. The current implementation uses `fs::copy()` on Windows, which fails with `AccessDenied` because Windows prevents overwriting executing binaries. The fix implements the standard Windows pattern: rename the running executable to a backup, move the new executable to the original location, and optionally clean up the backup.

The solution maintains cross-platform compatibility by keeping the Unix atomic rename behavior unchanged while introducing the rename-move-cleanup pattern for Windows only.

## Architecture

### Current State (Broken)

```
Windows Binary Replacement (Current - BROKEN)
┌─────────────────────────────────────────┐
│ BinaryReplacer::perform_replacement()   │
├─────────────────────────────────────────┤
│ fs::copy(new_path, current_path)        │
│ ❌ FAILS: AccessDenied (file locked)    │
└─────────────────────────────────────────┘
```

### Proposed State (Fixed)

```
Windows Binary Replacement (Proposed - FIXED)
┌──────────────────────────────────────────────────────────┐
│ BinaryReplacer::perform_replacement()                    │
├──────────────────────────────────────────────────────────┤
│ #[cfg(windows)]                                          │
│ 1. fs::rename(current_path, backup_path)                │
│    ✓ Rename app.exe → app.exe.old                       │
│                                                          │
│ 2. fs::rename(new_path, current_path)                   │
│    ✓ Move new_app.exe → app.exe                         │
│                                                          │
│ 3. fs::remove_file(backup_path) [optional]              │
│    ✓ Delete app.exe.old (ignore if locked)              │
│                                                          │
│ #[cfg(unix)]                                             │
│ fs::rename(new_path, current_path) [unchanged]          │
│ ✓ Atomic rename (existing behavior)                     │
└──────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### BinaryReplacer Component

**Location:** `src/update/mod.rs` - `BinaryReplacer` struct

**Responsibility:** Replace the running executable with a new version, handling platform-specific constraints.

**Key Methods:**

1. **`replace_binary()`** (existing)
   - Public entry point
   - Orchestrates: backup creation → replacement → health check → rollback if needed
   - No changes required

2. **`perform_replacement()`** (modified)
   - Private method handling platform-specific replacement logic
   - **Current behavior (Unix):** Atomic rename
   - **New behavior (Windows):** Rename-move-cleanup pattern
   - **Error handling:** Return `UpdateError::Replacement` on failure

3. **`create_backup()`** (existing)
   - Creates backup of current executable
   - No changes required

4. **`rollback()`** (existing)
   - Restores backup if replacement fails
   - No changes required

### Error Handling

The existing `UpdateError::Replacement` variant is sufficient:

```rust
Replacement {
    reason: String,
    recovered: bool,
}
```

**New error scenarios:**
- Rename current executable fails → `reason: "Failed to rename current executable: {error}"`
- Move new executable fails → `reason: "Failed to move new executable: {error}"`, `recovered: true/false` based on rollback success
- Backup deletion fails → Logged as warning, not returned as error

## Data Models

No new data models required. The fix operates entirely within the existing `BinaryReplacer` struct using standard file system operations.

**Existing structures used:**
- `std::path::Path` - File paths
- `std::path::PathBuf` - Owned file paths
- `UpdateError` - Error reporting

## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.


### Property 1: Windows Rename-Move Pattern Completes Successfully

*For any* Windows system with a current executable and a new executable, performing binary replacement SHALL result in the new executable at the original location and a backup file created.

**Validates: Requirements 1.1, 1.2, 1.3**

### Property 2: Unix Atomic Rename Behavior Unchanged

*For any* Unix system with a current executable and a new executable, performing binary replacement SHALL use atomic rename and result in the new executable at the original location without creating a backup file.

**Validates: Requirements 3.1**

### Property 3: Backup File Preservation on Success

*For any* platform and any successful binary replacement, a backup file SHALL exist after the operation completes (either as `.old` on Windows or as a separate backup on Unix).

**Validates: Requirements 2.1, 3.3**

### Property 4: Backup File Preservation on Failure

*For any* platform and any failed binary replacement, the backup file SHALL exist after the operation fails, enabling manual recovery.

**Validates: Requirements 1.5, 4.5**

### Property 5: Successful Replacement Returns Success Result

*For any* platform and any successful binary replacement, the operation SHALL return `Ok(())` indicating success.

**Validates: Requirements 3.4**

### Property 6: Failed Replacement Returns Error with Reason

*For any* platform and any failed binary replacement, the operation SHALL return an `UpdateError::Replacement` with a non-empty reason string describing the failure.

**Validates: Requirements 4.1, 4.2**

### Property 7: Deletion Failure Does Not Fail Update

*For any* Windows system where backup file deletion fails, the overall binary replacement operation SHALL still return success if the new executable is in place.

**Validates: Requirements 2.2, 2.3**

### Property 8: Rollback Restores Original on Move Failure

*For any* Windows system where the move operation fails, the rollback operation SHALL restore the backup file to the original location, resulting in the original executable being available.

**Validates: Requirements 1.4**

## Error Handling

### Failure Scenarios and Recovery

1. **Rename Current Executable Fails**
   - Error: `UpdateError::Replacement { reason: "Failed to rename current executable: {error}", recovered: false }`
   - Recovery: No rollback needed (original file still exists)
   - User Action: Check file permissions, disk space, or try again

2. **Move New Executable Fails**
   - Error: `UpdateError::Replacement { reason: "Failed to move new executable: {error}", recovered: true/false }`
   - Recovery: Attempt to restore backup file to original location
   - User Action: If recovery failed, manually restore from backup or retry update

3. **Backup Deletion Fails**
   - Logged as warning, not returned as error
   - Update operation succeeds
   - Backup file remains on disk for manual cleanup or OS cleanup on next reboot

4. **Rollback Fails**
   - Error: `UpdateError::Replacement { reason: "Failed to restore backup: {error}", recovered: false }`
   - User Action: Manual intervention required - restore backup file manually

### Error Messages

All error messages SHALL include:
- Specific operation that failed (rename, move, delete, rollback)
- Underlying OS error message
- Recovery instructions when applicable

## Testing Strategy

### Unit Tests

Unit tests verify specific examples and edge cases:

1. **Successful Windows Replacement**
   - Create mock current and new executables
   - Verify rename, move, and cleanup operations
   - Verify backup file exists after operation

2. **Successful Unix Replacement**
   - Create mock current and new executables
   - Verify atomic rename operation
   - Verify no backup file created

3. **Rename Failure Handling**
   - Simulate rename failure (permission denied)
   - Verify error is returned with reason
   - Verify new executable is not moved

4. **Move Failure and Rollback**
   - Simulate move failure
   - Verify rollback is attempted
   - Verify backup file is restored

5. **Deletion Failure Handling**
   - Simulate deletion failure
   - Verify operation still succeeds
   - Verify backup file remains

6. **Rollback Failure Handling**
   - Simulate both move and rollback failures
   - Verify appropriate error is returned
   - Verify backup file is preserved

### Property-Based Tests

Property-based tests verify universal properties across many generated inputs:

1. **Property 1: Windows Rename-Move Pattern**
   - Generate random executable paths
   - Verify new executable ends up at original location
   - Verify backup file exists
   - **Feature: windows-binary-replacement, Property 1: Windows Rename-Move Pattern Completes Successfully**

2. **Property 2: Unix Atomic Rename**
   - Generate random executable paths
   - Verify new executable ends up at original location
   - Verify no backup file created
   - **Feature: windows-binary-replacement, Property 2: Unix Atomic Rename Behavior Unchanged**

3. **Property 3: Backup Preservation on Success**
   - Generate random executable paths
   - Verify backup file exists after successful replacement
   - **Feature: windows-binary-replacement, Property 3: Backup File Preservation on Success**

4. **Property 4: Backup Preservation on Failure**
   - Generate random executable paths
   - Simulate various failure scenarios
   - Verify backup file exists after failure
   - **Feature: windows-binary-replacement, Property 4: Backup File Preservation on Failure**

5. **Property 5: Success Result**
   - Generate random executable paths
   - Verify successful operations return `Ok(())`
   - **Feature: windows-binary-replacement, Property 5: Successful Replacement Returns Success Result**

6. **Property 6: Error with Reason**
   - Generate random failure scenarios
   - Verify errors contain non-empty reason strings
   - **Feature: windows-binary-replacement, Property 6: Failed Replacement Returns Error with Reason**

7. **Property 7: Deletion Failure Tolerance**
   - Generate random executable paths
   - Simulate deletion failure
   - Verify operation succeeds despite deletion failure
   - **Feature: windows-binary-replacement, Property 7: Deletion Failure Does Not Fail Update**

8. **Property 8: Rollback Restores Original**
   - Generate random executable paths
   - Simulate move failure
   - Verify rollback restores original executable
   - **Feature: windows-binary-replacement, Property 8: Rollback Restores Original on Move Failure**

### Test Configuration

- Minimum 100 iterations per property test
- Each property test tagged with feature name and property number
- Both unit tests and property tests required for comprehensive coverage
- Unit tests catch concrete bugs; property tests verify general correctness

