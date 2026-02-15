# Design Document: Auto-Update Functionality

## Overview

The auto-update system enables transcript-explorer to automatically detect, download, verify, and install new releases from GitHub. The design prioritizes safety, user experience, and cross-platform compatibility. The system operates in two modes: interactive (with user prompts) and non-interactive (automated). A background thread handles update checks without blocking the main application, and comprehensive error handling ensures the system remains stable even when updates fail.

## Architecture

### High-Level Flow

```
Application Startup
    ↓
Spawn Background Update Thread
    ↓
Main Application Continues
    ↓
Background Thread:
  1. Detect Platform/Architecture
  2. Query GitHub API for Latest Release
  3. Compare Versions
  4. If Update Available:
     - Download Binary
     - Verify Integrity
     - Prompt User (Interactive Mode)
     - Replace Binary
     - Verify New Binary
     - Rollback on Failure
```

### Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Main Application                         │
│  - Runs immediately on startup                              │
│  - Spawns update thread                                     │
│  - Continues normal operation                               │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│              Background Update Thread                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Platform Detector                                    │  │
│  │ - Detects OS (Linux, macOS, Windows)                │  │
│  │ - Detects Architecture (x86_64, aarch64)            │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ GitHub API Client                                    │  │
│  │ - Queries GitHub Releases API                       │  │
│  │ - Parses Release Information                        │  │
│  │ - Handles API Errors                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Version Comparator                                   │  │
│  │ - Parses Semantic Versions                          │  │
│  │ - Compares Local vs Remote                          │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Asset Selector                                       │  │
│  │ - Identifies Available Assets                       │  │
│  │ - Matches Platform/Architecture                     │  │
│  │ - Extracts Download URL                             │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ User Interaction (Interactive Mode Only)            │  │
│  │ - Prompts for Confirmation                          │  │
│  │ - Allows Cancellation                               │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Binary Downloader                                    │  │
│  │ - Downloads Binary with Progress Tracking           │  │
│  │ - Handles Download Interruptions                    │  │
│  │ - Retries on Failure                                │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Binary Verifier                                      │  │
│  │ - Checks File Existence and Readability             │  │
│  │ - Verifies File Size                                │  │
│  │ - Validates Signature (if applicable)               │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Binary Replacer                                      │  │
│  │ - Creates Backup of Current Binary                  │  │
│  │ - Replaces Binary Atomically                        │  │
│  │ - Sets Executable Permissions                       │  │
│  │ - Performs Health Check                             │  │
│  │ - Rolls Back on Failure                             │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ↓                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Error Handler & Logger                              │  │
│  │ - Categorizes Errors                                │  │
│  │ - Provides Descriptive Messages                     │  │
│  │ - Logs Results                                      │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Platform Detector

**Responsibility**: Identify the current operating system and CPU architecture.

**Interface**:
```rust
pub struct PlatformInfo {
    pub os: OperatingSystem,
    pub arch: Architecture,
}

pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
}

pub enum Architecture {
    X86_64,
    Aarch64,
}

pub fn detect_platform() -> Result<PlatformInfo, PlatformDetectionError>;
```

**Implementation Details**:
- Uses `std::env::consts::OS` and `std::env::consts::ARCH` for detection
- Maps Rust's architecture names to release asset naming conventions
- Returns descriptive errors for unsupported platforms

### 2. GitHub API Client

**Responsibility**: Query GitHub Releases API and parse release information.

**Interface**:
```rust
pub struct GitHubRelease {
    pub version: String,
    pub assets: Vec<ReleaseAsset>,
    pub published_at: DateTime<Utc>,
}

pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
}

pub struct GitHubApiClient {
    repo_owner: String,
    repo_name: String,
    http_client: reqwest::Client,
}

impl GitHubApiClient {
    pub async fn get_latest_release(&self) -> Result<GitHubRelease, ApiError>;
}
```

**Implementation Details**:
- Uses `reqwest` for HTTP requests with TLS 1.2+
- Includes User-Agent header for GitHub API compliance
- Parses JSON response into structured data
- Handles rate limiting and network errors gracefully

### 3. Version Comparator

**Responsibility**: Parse and compare semantic versions.

**Interface**:
```rust
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub fn parse_version(version_str: &str) -> Result<SemanticVersion, VersionParseError>;

pub fn is_newer(remote: &SemanticVersion, local: &SemanticVersion) -> bool;
```

**Implementation Details**:
- Parses versions in "major.minor.patch" format
- Handles optional "v" prefix (e.g., "v1.3.2")
- Compares versions lexicographically by major, then minor, then patch
- Returns clear errors for invalid version strings

### 4. Asset Selector

**Responsibility**: Match platform/architecture to available release assets.

**Interface**:
```rust
pub struct AssetMatch {
    pub asset: ReleaseAsset,
    pub specificity: u32,
}

pub fn select_asset(
    platform: &PlatformInfo,
    assets: &[ReleaseAsset],
) -> Result<ReleaseAsset, AssetSelectionError>;
```

**Implementation Details**:
- Builds expected asset name patterns for the platform/architecture
- Searches for exact matches in available assets
- Prioritizes platform-specific assets over generic ones
- Returns error if no matching asset is found

### 5. Binary Downloader

**Responsibility**: Download binary with progress tracking and error handling.

**Interface**:
```rust
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}

pub trait ProgressCallback: Send {
    fn on_progress(&self, progress: DownloadProgress);
}

pub async fn download_binary(
    url: &str,
    destination: &Path,
    progress_callback: Option<Box<dyn ProgressCallback>>,
) -> Result<(), DownloadError>;
```

**Implementation Details**:
- Uses `reqwest` for HTTP downloads
- Streams response body to disk to minimize memory usage
- Reports progress via callback for UI integration
- Implements retry logic with exponential backoff
- Cleans up partial downloads on failure

### 6. Binary Verifier

**Responsibility**: Verify downloaded binary integrity.

**Interface**:
```rust
pub struct VerificationResult {
    pub is_valid: bool,
    pub file_size: u64,
    pub expected_size: u64,
}

pub fn verify_binary(
    path: &Path,
    expected_size: u64,
) -> Result<VerificationResult, VerificationError>;
```

**Implementation Details**:
- Checks file existence and readability
- Verifies file size matches expected size from metadata
- Optionally validates cryptographic signatures (future enhancement)
- Deletes corrupted files automatically
- Returns detailed error information

### 7. Binary Replacer

**Responsibility**: Safely replace current binary with new version.

**Interface**:
```rust
pub struct ReplacementResult {
    pub success: bool,
    pub new_version: String,
    pub rolled_back: bool,
}

pub async fn replace_binary(
    current_path: &Path,
    new_path: &Path,
    new_version: &str,
) -> Result<ReplacementResult, ReplacementError>;
```

**Implementation Details**:
- Creates timestamped backup before replacement
- Performs atomic replacement (platform-specific)
- Sets executable permissions (chmod +x on Unix, default on Windows)
- Runs health check on new binary
- Automatically rolls back on health check failure
- Restores backup if replacement fails

### 8. Update Manager

**Responsibility**: Orchestrate the entire update process.

**Interface**:
```rust
pub struct UpdateConfig {
    pub interactive_mode: bool,
    pub auto_update_enabled: bool,
    pub github_repo_owner: String,
    pub github_repo_name: String,
    pub temp_dir: PathBuf,
}

pub struct UpdateManager {
    config: UpdateConfig,
    platform: PlatformInfo,
}

impl UpdateManager {
    pub async fn check_and_update(&self) -> Result<UpdateResult, UpdateError>;
}

pub enum UpdateResult {
    Updated { new_version: String },
    UpToDate,
    Skipped { reason: String },
}
```

**Implementation Details**:
- Coordinates all components in sequence
- Handles mode-specific behavior (interactive vs non-interactive)
- Manages lock file for concurrent update prevention
- Logs all operations and results
- Provides user feedback at each step

### 9. Error Handler

**Responsibility**: Categorize and report errors with actionable information.

**Interface**:
```rust
pub enum UpdateError {
    PlatformDetection(String),
    ApiError { status: u16, message: String },
    VersionParse(String),
    AssetNotFound(String),
    Download { reason: String, retryable: bool },
    Verification { reason: String },
    Replacement { reason: String, recovered: bool },
    PermissionDenied(String),
    LockFileExists,
    ConfigurationError(String),
}

impl UpdateError {
    pub fn is_retryable(&self) -> bool;
    pub fn user_message(&self) -> String;
    pub fn recovery_instructions(&self) -> Option<String>;
}
```

**Implementation Details**:
- Distinguishes between different error categories
- Provides user-friendly error messages
- Suggests recovery actions when applicable
- Logs detailed error information for debugging

### 10. Bad Version Tracker

**Responsibility**: Track and persist versions that fail health checks.

**Interface**:
```rust
pub struct BadVersionTracker {
    cache_path: PathBuf,
    bad_versions: HashSet<String>,
}

impl BadVersionTracker {
    pub fn load() -> Result<Self, LoadError>;
    pub fn mark_bad(&mut self, version: String) -> Result<(), MarkBadError>;
    pub fn is_bad(&self, version: &str) -> bool;
    pub fn save(&self) -> Result<(), SaveError>;
    pub fn clear(&mut self) -> Result<(), ClearError>;
}
```

**Implementation Details**:
- Uses `directories` crate for cross-platform cache paths
- Persists bad versions to JSON file in cache directory
- Loads bad versions on startup
- Handles corrupted cache files gracefully
- Provides method to bypass bad version list for manual updates

## Data Models

### Release Information
```rust
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<AssetInfo>,
    pub body: String, // Release notes
}

pub struct AssetInfo {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}
```

### Update State
```rust
pub struct UpdateState {
    pub last_check: Option<DateTime<Utc>>,
    pub last_successful_update: Option<DateTime<Utc>>,
    pub bad_versions: HashSet<String>, // Versions that failed health check
    pub current_version: String,
}

impl UpdateState {
    pub fn load() -> Result<Self, StateLoadError>;
    pub fn save(&self) -> Result<(), StateSaveError>;
    pub fn mark_bad(&mut self, version: String) -> Result<(), MarkBadError>;
    pub fn is_bad(&self, version: &str) -> bool;
}
```

### Configuration
```rust
pub struct UpdateConfiguration {
    pub enabled: bool,
    pub check_interval_hours: u32,
    pub interactive_mode: bool,
    pub github_repo_owner: String,
    pub github_repo_name: String,
    pub temp_directory: PathBuf,
    pub backup_directory: PathBuf,
}
```

## Correctness Properties

A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.

### Property 1: Platform Detection Consistency
*For any* system, the detected platform and architecture should remain constant across multiple detection calls.
**Validates: Requirements 1.1, 1.2**

### Property 2: Version Parsing Idempotence
*For any* valid semantic version string, parsing it multiple times should produce identical results.
**Validates: Requirements 3.1**

### Property 3: Version Comparison Transitivity
*For any* three versions A, B, C where A < B and B < C, then A < C should hold.
**Validates: Requirements 3.2**

### Property 4: Asset Selection Determinism
*For any* platform/architecture combination and set of available assets, selecting an asset multiple times should return the same asset.
**Validates: Requirements 4.1, 4.2, 4.3**

### Property 5: Downloaded File Size Verification
*For any* downloaded binary, the file size on disk should match the expected size from release metadata.
**Validates: Requirements 5.3, 6.2**

### Property 6: Backup Creation Before Replacement
*For any* binary replacement operation, a backup of the original binary should exist before the replacement occurs.
**Validates: Requirements 7.1, 7.2**

### Property 7: Executable Permission Preservation
*For any* replaced binary on Unix systems, the executable permission (chmod +x) should be set after replacement.
**Validates: Requirements 12.1, 12.3**

### Property 8: Lock File Cleanup
*For any* completed update operation, the lock file should be removed after the operation completes (success or failure).
**Validates: Requirements 13.1, 13.3**

### Property 9: Interactive Mode Prompts
*For any* interactive mode update check where an update is available, the user should be prompted for confirmation before downloading.
**Validates: Requirements 8.1, 8.2**

### Property 10: Non-Interactive Mode Automation
*For any* non-interactive mode update check where an update is available, the system should automatically download and install without prompts.
**Validates: Requirements 9.1, 9.2**

### Property 11: Error Message Descriptiveness
*For any* error that occurs, the error message should contain information about the error type and context.
**Validates: Requirements 10.1, 10.2, 10.3**

### Property 12: Configuration Fallback
*For any* invalid configuration value, the system should use a sensible default and log a warning.
**Validates: Requirements 14.5**

### Property 13: GitHub API Header Compliance
*For any* GitHub API request, the request should include a User-Agent header.
**Validates: Requirements 2.3**

### Property 14: Rollback on Health Check Failure
*For any* binary replacement where the health check fails, the original binary should be restored from backup.
**Validates: Requirements 7.5, 10.4**

### Property 15: Version Comparison Correctness
*For any* pair of versions where the remote version is newer, the system should proceed with the update process.
**Validates: Requirements 3.4**

### Property 16: Bad Version Persistence
*For any* version marked as bad, the bad version list should persist across application restarts and the version should be skipped on subsequent update checks.
**Validates: Requirements 15.1, 15.2, 15.3, 15.6**

### Property 17: Bad Version Cache Loading
*For any* application startup, the bad version list should be loaded from the cache file and used to filter available updates.
**Validates: Requirements 15.4, 15.6**

### Property 18: Bad Version Cache Fallback
*For any* corrupted or unreadable cache file, the system should start with an empty bad version list and log a warning.
**Validates: Requirements 15.5**

## Error Handling

### Error Categories

1. **Platform Detection Errors**
   - Unsupported OS or architecture
   - Cannot determine platform information
   - Action: Abort update, display error message

2. **Network Errors**
   - Connection timeout
   - Connection refused
   - DNS resolution failure
   - Action: Log error, abort silently in background thread

3. **API Errors**
   - HTTP 4xx/5xx responses
   - Malformed JSON response
   - Missing required fields
   - Action: Log error, abort silently, retry on next check

4. **Version Errors**
   - Invalid version format
   - Cannot parse semantic version
   - Action: Log error, treat as no update available

5. **Asset Errors**
   - No matching asset for platform
   - Multiple conflicting assets
   - Action: Log error, abort update

6. **Download Errors**
   - Network interruption
   - Timeout during download
   - Insufficient disk space
   - Action: Retry up to 3 times, then abort

7. **Verification Errors**
   - File size mismatch
   - File not readable
   - Signature verification failure
   - Action: Delete corrupted file, abort update

8. **Replacement Errors**
   - Cannot create backup
   - Cannot replace binary
   - Cannot set permissions
   - Action: Restore backup if possible, abort update

9. **Permission Errors**
   - Cannot write to binary location
   - Cannot set executable permission
   - Action: Abort update, display permission error

10. **Lock File Errors**
    - Lock file already exists
    - Cannot create lock file
    - Action: Abort update, display "update in progress" message

### Recovery Strategies

- **Automatic Rollback**: If health check fails, automatically restore backup
- **Backup Preservation**: Keep backup for manual recovery if needed
- **Lock File Cleanup**: Remove stale lock files on startup
- **Partial Download Cleanup**: Remove incomplete downloads
- **Configuration Fallback**: Use defaults for invalid configuration

## Testing Strategy

### Unit Testing

Unit tests verify specific examples, edge cases, and error conditions:

1. **Platform Detection Tests**
   - Test detection on each supported platform
   - Test unsupported platform handling
   - Test architecture detection

2. **Version Parsing Tests**
   - Test valid semantic versions
   - Test versions with "v" prefix
   - Test invalid version formats
   - Test edge cases (0.0.0, 999.999.999)

3. **Version Comparison Tests**
   - Test newer version detection
   - Test equal version handling
   - Test older version handling
   - Test transitivity property

4. **Asset Selection Tests**
   - Test exact asset matching
   - Test platform-specific matching
   - Test missing asset handling
   - Test multiple matching assets

5. **Error Handling Tests**
   - Test network error categorization
   - Test file system error handling
   - Test permission error handling
   - Test recovery instructions

6. **Configuration Tests**
   - Test valid configuration loading
   - Test invalid configuration fallback
   - Test environment variable override
   - Test config file parsing

### Property-Based Testing

Property-based tests verify universal properties across many generated inputs:

1. **Platform Detection Property Test**
   - **Property 1**: Platform detection consistency
   - Generate multiple detection calls, verify results are identical

2. **Version Parsing Property Test**
   - **Property 2**: Version parsing idempotence
   - Generate valid version strings, verify parsing is consistent

3. **Version Comparison Property Test**
   - **Property 3**: Version comparison transitivity
   - Generate version triples, verify transitivity holds

4. **Asset Selection Property Test**
   - **Property 4**: Asset selection determinism
   - Generate platform/asset combinations, verify deterministic selection

5. **File Verification Property Test**
   - **Property 5**: Downloaded file size verification
   - Generate files with various sizes, verify size matching

6. **Backup Creation Property Test**
   - **Property 6**: Backup creation before replacement
   - Generate replacement operations, verify backup exists

7. **Permission Setting Property Test**
   - **Property 7**: Executable permission preservation
   - Generate replacement operations on Unix, verify permissions

8. **Lock File Property Test**
   - **Property 8**: Lock file cleanup
   - Generate update operations, verify lock file cleanup

9. **Interactive Mode Property Test**
   - **Property 9**: Interactive mode prompts
   - Generate interactive updates, verify prompts occur

10. **Non-Interactive Mode Property Test**
    - **Property 10**: Non-interactive mode automation
    - Generate non-interactive updates, verify no prompts

11. **Error Message Property Test**
    - **Property 11**: Error message descriptiveness
    - Generate errors, verify messages contain context

12. **Configuration Fallback Property Test**
    - **Property 12**: Configuration fallback
    - Generate invalid configs, verify defaults used

13. **GitHub API Header Property Test**
    - **Property 13**: GitHub API header compliance
    - Generate API requests, verify User-Agent header present

14. **Rollback Property Test**
    - **Property 14**: Rollback on health check failure
    - Generate failed health checks, verify rollback occurs

15. **Version Comparison Flow Property Test**
    - **Property 15**: Version comparison correctness
    - Generate version pairs, verify update flow starts for newer versions

16. **Bad Version Persistence Property Test**
    - **Property 16**: Bad version persistence
    - Generate bad version marks, verify persistence across restarts

17. **Bad Version Cache Loading Property Test**
    - **Property 17**: Bad version cache loading
    - Generate cache files, verify bad versions are loaded and used

18. **Bad Version Cache Fallback Property Test**
    - **Property 18**: Bad version cache fallback
    - Generate corrupted cache files, verify fallback to empty list

### Test Configuration

- Minimum 100 iterations per property test
- Each property test tagged with feature name and property number
- Tag format: `Feature: auto-update, Property N: [property description]`
- Unit tests focus on specific examples and edge cases
- Property tests focus on universal correctness across inputs

## Testing Notes

### Running Tests

Since transcript-explorer is a binary-only project (no library target), use the following command to run tests from the project root directory:

```bash
# From project root
cargo test --bin transcript-explorer <test_name>
```

Examples:
- Run all tests: `cargo test --bin transcript-explorer`
- Run specific test module: `cargo test --bin transcript-explorer update`
- Run specific test: `cargo test --bin transcript-explorer test_semantic_version_parse_valid`

**Important**: Always run cargo commands from the project root directory (where `Cargo.toml` is located).

Do NOT use `cargo test --lib` as this project has no library target.

