# Implementation Plan: Auto-Update Functionality

## Overview

This implementation plan breaks down the auto-update feature into discrete coding tasks. The approach follows a layered architecture: first establishing core utilities (platform detection, version comparison), then building the GitHub integration layer, followed by the download and verification layer, and finally the replacement and orchestration layer. Each task builds on previous work, with property tests integrated alongside implementation to catch correctness issues early.

## Tasks

- [x] 1. Set up project structure and core types
  - Create `src/update/mod.rs` module structure
  - Define core types: `PlatformInfo`, `OperatingSystem`, `Architecture`, `UpdateError`, `UpdateResult`
  - Define `ReleaseInfo`, `ReleaseAsset`, `UpdateState` data structures
  - Set up error types with `thiserror` crate
  - Add dependencies: `reqwest`, `serde`, `serde_json`, `directories`, `indicatif`
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 2. Implement platform detection
  - [x] 2.1 Implement `PlatformDetector` with platform and architecture detection
    - Use `std::env::consts::OS` and `std::env::consts::ARCH`
    - Map Rust architecture names to release asset naming conventions
    - Return `PlatformInfo` with OS and architecture
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [ ]* 2.2 Write property test for platform detection consistency
    - **Property 1: Platform Detection Consistency**
    - **Validates: Requirements 1.1, 1.2**
    - Generate multiple detection calls, verify results are identical

- [x] 3. Implement version parsing and comparison
  - [x] 3.1 Implement `SemanticVersion` struct and parsing
    - Parse "major.minor.patch" format
    - Handle optional "v" prefix
    - Implement `Display` and `FromStr` traits
    - _Requirements: 3.1_
  
  - [ ]* 3.2 Write property test for version parsing idempotence
    - **Property 2: Version Parsing Idempotence**
    - **Validates: Requirements 3.1**
    - Generate valid version strings, verify parsing consistency
  
  - [x] 3.3 Implement `VersionComparator` with comparison logic
    - Implement `is_newer()` function
    - Compare versions lexicographically by major, minor, patch
    - _Requirements: 3.2, 3.4_
  
  - [ ]* 3.4 Write property test for version comparison transitivity
    - **Property 3: Version Comparison Transitivity**
    - **Validates: Requirements 3.2**
    - Generate version triples, verify transitivity holds

- [x] 4. Implement GitHub API client
  - [x] 4.1 Create `GitHubApiClient` struct
    - Initialize with repo owner and name
    - Create `reqwest::Client` with TLS configuration
    - _Requirements: 2.1_
  
  - [x] 4.2 Implement `get_latest_release()` method
    - Query GitHub Releases API endpoint
    - Include User-Agent header for API compliance
    - Parse JSON response into `ReleaseInfo`
    - Handle API errors and rate limiting
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [ ]* 4.3 Write property test for GitHub API header compliance
    - **Property 13: GitHub API Header Compliance**
    - **Validates: Requirements 2.3**
    - Generate API requests, verify User-Agent header present

- [-] 5. Implement asset selection
  - [ ] 5.1 Implement `AssetSelector` with platform matching
    - Build expected asset name patterns for platform/architecture
    - Search for exact matches in available assets
    - Prioritize platform-specific assets
    - _Requirements: 4.1, 4.2, 4.3_
  
  - [x] 5.2 Write property test for asset selection determinism
    - **Property 4: Asset Selection Determinism**
    - **Validates: Requirements 4.1, 4.2, 4.3**
    - Generate platform/asset combinations, verify deterministic selection

- [ ] 6. Implement bad version tracking
  - [ ] 6.1 Implement `BadVersionTracker` struct
    - Use `directories` crate for cross-platform cache paths
    - Load bad versions from cache file on initialization
    - Implement `mark_bad()` to add version to bad list
    - Implement `is_bad()` to check if version is bad
    - Implement `save()` to persist bad versions to cache
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.6_
  
  - [ ] 6.2 Implement cache file handling
    - Use JSON format for cache file
    - Handle corrupted cache files gracefully
    - Fall back to empty list on load errors
    - Log warnings for cache issues
    - _Requirements: 15.5_
  
  - [ ]* 6.3 Write property test for bad version persistence
    - **Property 16: Bad Version Persistence**
    - **Validates: Requirements 15.1, 15.2, 15.3, 15.6**
    - Generate bad version marks, verify persistence across restarts
  
  - [ ]* 6.4 Write property test for bad version cache loading
    - **Property 17: Bad Version Cache Loading**
    - **Validates: Requirements 15.4, 15.6**
    - Generate cache files, verify bad versions are loaded
  
  - [ ]* 6.5 Write property test for bad version cache fallback
    - **Property 18: Bad Version Cache Fallback**
    - **Validates: Requirements 15.5**
    - Generate corrupted cache files, verify fallback to empty list

- [x] 7. Implement binary downloader
  - [x] 7.1 Create `BinaryDownloader` struct
    - Use `reqwest` for HTTP downloads
    - Stream response body to disk
    - Implement progress callback mechanism
    - _Requirements: 5.1, 5.2_
  
  - [x] 7.2 Implement `download_binary()` method
    - Download from provided URL
    - Report progress via callback
    - Implement retry logic with exponential backoff
    - Clean up partial downloads on failure
    - _Requirements: 5.1, 5.2, 5.4, 5.5_
  
  - [ ]* 7.3 Write property test for download progress reporting
    - **Property 5: Downloaded File Size Verification**
    - **Validates: Requirements 5.3, 6.2**
    - Generate downloads, verify file sizes match metadata

- [x] 8. Implement binary verifier
  - [x] 8.1 Create `BinaryVerifier` struct
    - Check file existence and readability
    - Verify file size matches expected size
    - _Requirements: 6.1, 6.2_
  
  - [x] 8.2 Implement `verify_binary()` method
    - Return verification result with file size
    - Delete corrupted files automatically
    - Return detailed error information
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [ ]* 8.3 Write unit tests for verification edge cases
    - Test missing files
    - Test unreadable files
    - Test size mismatches
    - _Requirements: 6.4, 6.5_

- [ ] 9. Implement binary replacer with rollback
  - [ ] 9.1 Create `BinaryReplacer` struct
    - Implement backup creation with timestamps
    - Implement atomic binary replacement
    - _Requirements: 7.1, 7.2, 7.3_
  
  - [ ] 9.2 Implement `replace_binary()` method
    - Create timestamped backup before replacement
    - Replace binary atomically (platform-specific)
    - Set executable permissions (chmod +x on Unix)
    - Run health check on new binary
    - Implement rollback on health check failure
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 12.1, 12.2, 12.3, 12.4_
  
  - [ ]* 9.3 Write property test for backup creation
    - **Property 6: Backup Creation Before Replacement**
    - **Validates: Requirements 7.1, 7.2**
    - Generate replacement operations, verify backup exists
  
  - [ ]* 9.4 Write property test for executable permissions
    - **Property 7: Executable Permission Preservation**
    - **Validates: Requirements 12.1, 12.3**
    - Generate replacements on Unix, verify permissions
  
  - [ ]* 9.5 Write property test for rollback on health check failure
    - **Property 14: Rollback on Health Check Failure**
    - **Validates: Requirements 7.5, 10.4**
    - Generate failed health checks, verify rollback occurs

- [ ] 10. Implement lock file management
  - [ ] 10.1 Create `LockFileManager` struct
    - Create lock file on update start
    - Check for existing lock file
    - Remove lock file on completion
    - Clean up stale lock files on startup
    - _Requirements: 13.1, 13.2, 13.3, 13.4_
  
  - [ ]* 10.2 Write property test for lock file cleanup
    - **Property 8: Lock File Cleanup**
    - **Validates: Requirements 13.1, 13.3**
    - Generate update operations, verify lock file cleanup

- [ ] 11. Implement configuration management
  - [ ] 11.1 Create `UpdateConfiguration` struct
    - Read from environment variables
    - Read from config file (if exists)
    - Implement sensible defaults
    - _Requirements: 14.1, 14.2, 14.3, 14.4_
  
  - [ ] 11.2 Implement configuration loading
    - Load from environment variables first
    - Fall back to config file
    - Fall back to defaults
    - Log warnings for invalid configuration
    - _Requirements: 14.1, 14.5_
  
  - [ ]* 11.3 Write property test for configuration fallback
    - **Property 12: Configuration Fallback**
    - **Validates: Requirements 14.5**
    - Generate invalid configs, verify defaults used

- [ ] 12. Implement error handling and user feedback
  - [ ] 12.1 Implement `ErrorHandler` with error categorization
    - Distinguish network errors (timeout, connection refused, DNS)
    - Distinguish file system errors (permission, not found)
    - Provide descriptive error messages
    - Suggest recovery actions
    - _Requirements: 10.1, 10.2, 10.3, 10.5_
  
  - [ ] 12.2 Implement user feedback system
    - Display current version on startup
    - Display status messages during checks
    - Display new version when found
    - Display download progress with progress bar
    - Display success message on completion
    - Display error messages with actionable information
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6_
  
  - [ ]* 12.3 Write property test for error message descriptiveness
    - **Property 11: Error Message Descriptiveness**
    - **Validates: Requirements 10.1, 10.2, 10.3**
    - Generate errors, verify messages contain context

- [ ] 13. Implement interactive mode
  - [ ] 13.1 Create `InteractiveMode` handler
    - Prompt user before checking for updates
    - Display new version and ask for confirmation
    - Allow user to cancel during download
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  
  - [ ]* 13.2 Write property test for interactive mode prompts
    - **Property 9: Interactive Mode Prompts**
    - **Validates: Requirements 8.1, 8.2**
    - Generate interactive updates, verify prompts occur

- [ ] 14. Implement non-interactive mode
  - [ ] 14.1 Create `NonInteractiveMode` handler
    - Skip all user prompts
    - Automatically download and install updates
    - Log results to stdout or log file
    - Exit with appropriate exit codes
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  
  - [ ]* 14.2 Write property test for non-interactive mode automation
    - **Property 10: Non-Interactive Mode Automation**
    - **Validates: Requirements 9.1, 9.2**
    - Generate non-interactive updates, verify no prompts

- [ ] 15. Implement UpdateManager orchestration
  - [ ] 15.1 Create `UpdateManager` struct
    - Coordinate all components
    - Manage update flow
    - Handle mode-specific behavior
    - Manage lock file lifecycle
    - _Requirements: 1.1, 1.2, 2.1, 3.1, 3.2, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 13.1, 14.1, 15.1_
  
  - [ ] 15.2 Implement `check_and_update()` method
    - Detect platform and architecture
    - Query GitHub API
    - Compare versions
    - Check bad version list
    - Select asset
    - Prompt user (interactive mode)
    - Download binary
    - Verify binary
    - Replace binary
    - Handle errors and rollback
    - _Requirements: All_
  
  - [ ] 15.3 Implement background thread spawning
    - Spawn update thread on application startup
    - Ensure main application continues
    - Handle thread completion gracefully
    - _Requirements: 8.1, 9.1_

- [ ] 16. Checkpoint - Ensure all tests pass
  - Ensure all unit tests pass
  - Ensure all property tests pass (minimum 100 iterations each)
  - Verify no compilation warnings
  - Ask the user if questions arise

- [ ] 17. Integration and wiring
  - [ ] 17.1 Integrate UpdateManager into main application
    - Spawn update thread on startup
    - Pass configuration to UpdateManager
    - Handle update results
    - Display user feedback
    - _Requirements: All_
  
  - [ ]* 17.2 Write integration tests
    - Test end-to-end update flow
    - Test error recovery
    - Test rollback scenarios
    - _Requirements: All_

- [ ] 18. Final checkpoint - Ensure all tests pass
  - Ensure all unit tests pass
  - Ensure all property tests pass
  - Ensure all integration tests pass
  - Verify cross-platform compatibility
  - Ask the user if questions arise

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests are integrated alongside implementation to catch issues early
- Checkpoints ensure incremental validation
- The reference implementation (rs-example-self-update) provides guidance on patterns and best practices
- Bad version tracking prevents update loops with broken releases
- Cross-platform support requires testing on Linux, macOS (x86_64 and aarch64), and Windows

