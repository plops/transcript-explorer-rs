# Self-Update Architecture

## Component Overview

The self-update feature is implemented as a modular system in `src/update/mod.rs` with the following components:

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

## Core Components

### PlatformDetector

Detects the current operating system and CPU architecture.

**Key Methods:**
- `detect() -> Result<PlatformInfo, UpdateError>` - Detects platform and architecture
- `asset_pattern() -> String` - Generates asset name pattern for the platform

**Implementation:**
- Uses `std::env::consts::OS` and `std::env::consts::ARCH`
- Maps Rust architecture names to release asset naming conventions
- Returns descriptive errors for unsupported platforms

### GitHubApiClient

Queries the GitHub Releases API and parses release information.

**Key Methods:**
- `new(owner: String, name: String) -> Result<Self, UpdateError>` - Creates a new client
- `get_latest_release() -> Result<ReleaseInfo, UpdateError>` - Fetches the latest release

**Implementation:**
- Uses `reqwest` for HTTP requests with TLS 1.2+
- Includes User-Agent header for GitHub API compliance
- Parses JSON response into structured data
- Handles rate limiting and network errors gracefully

### SemanticVersion

Parses and compares semantic versions.

**Key Methods:**
- `parse(version_str: &str) -> Result<SemanticVersion, UpdateError>` - Parses a version string
- `is_newer(remote: &SemanticVersion, local: &SemanticVersion) -> bool` - Compares versions

**Implementation:**
- Parses versions in "major.minor.patch" format
- Handles optional "v" prefix (e.g., "v1.3.2")
- Compares versions lexicographically by major, then minor, then patch

### AssetSelector

Matches platform/architecture to available release assets.

**Key Methods:**
- `select_asset(platform: &PlatformInfo, assets: &[ReleaseAsset]) -> Result<ReleaseAsset, UpdateError>` - Selects the best matching asset

**Implementation:**
- Builds expected asset name patterns for the platform/architecture
- Searches for exact matches in available assets
- Prioritizes platform-specific assets over generic ones

### BinaryDownloader

Downloads binary with progress tracking and error handling.

**Key Methods:**
- `new() -> Result<Self, UpdateError>` - Creates a new downloader
- `download_binary(url: &str, destination: &Path, progress_callback: Option<Box<dyn ProgressCallback>>) -> Result<(), DownloadError>` - Downloads a binary

**Implementation:**
- Uses `reqwest` for HTTP downloads
- Streams response body to disk to minimize memory usage
- Reports progress via callback for UI integration
- Implements retry logic with exponential backoff
- Cleans up partial downloads on failure

### BinaryVerifier

Verifies downloaded binary integrity.

**Key Methods:**
- `verify_binary(path: &Path, expected_size: u64) -> Result<VerificationResult, VerificationError>` - Verifies a binary

**Implementation:**
- Checks file existence and readability
- Verifies file size matches expected size from metadata
- Deletes corrupted files automatically
- Returns detailed error information

### BinaryReplacer

Safely replaces current binary with new version.

**Key Methods:**
- `replace_binary(current_path: &Path, new_path: &Path, new_version: &str) -> Result<ReplacementResult, ReplacementError>` - Replaces the binary

**Implementation:**
- Creates timestamped backup before replacement
- Performs atomic replacement (platform-specific)
- Sets executable permissions (chmod +x on Unix, default on Windows)
- Runs health check on new binary
- Automatically rolls back on health check failure

### BadVersionTracker

Tracks versions that have failed health checks.

**Key Methods:**
- `load() -> Result<Self, UpdateError>` - Loads bad versions from cache
- `mark_bad(version: String) -> Result<(), UpdateError>` - Marks a version as bad
- `is_bad(version: &str) -> bool` - Checks if a version is bad
- `save() -> Result<(), UpdateError>` - Persists bad versions to cache

**Implementation:**
- Uses `directories` crate for cross-platform cache paths
- Persists bad versions to JSON file in cache directory
- Handles corrupted cache files gracefully
- Provides method to bypass bad version list for manual updates

### LockFileManager

Prevents concurrent update processes.

**Key Methods:**
- `new() -> Result<Self, UpdateError>` - Creates a new lock manager
- `acquire_lock() -> Result<(), UpdateError>` - Acquires the lock
- `release_lock() -> Result<(), UpdateError>` - Releases the lock

**Implementation:**
- Creates lock file in cache directory
- Checks for existing lock file
- Cleans up stale lock files on startup

### UpdateConfiguration

Manages update configuration from environment variables and config files.

**Key Methods:**
- `load() -> Result<Self, UpdateError>` - Loads configuration
- `validate() -> Result<(), UpdateError>` - Validates configuration
- `apply_env_overrides()` - Applies environment variable overrides

**Implementation:**
- Reads from environment variables first
- Falls back to config file
- Falls back to defaults
- Logs warnings for invalid configuration

### UpdateManager

Orchestrates the entire update process.

**Key Methods:**
- `new(config: UpdateConfiguration) -> Result<Self, UpdateError>` - Creates a new manager
- `check_and_update() -> Result<UpdateResult, UpdateError>` - Performs the update check and update
- `spawn_background_thread() -> JoinHandle<()>` - Spawns a background thread

**Implementation:**
- Coordinates all components in sequence
- Handles mode-specific behavior (interactive vs non-interactive)
- Manages lock file for concurrent update prevention
- Logs all operations and results
- Provides user feedback at each step

## Data Models

### PlatformInfo

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
```

### ReleaseInfo

```rust
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<ReleaseAsset>,
    pub body: String,
}

pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}
```

### UpdateResult

```rust
pub enum UpdateResult {
    Updated { new_version: String },
    UpToDate,
    Skipped { reason: String },
}
```

### UpdateError

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
    // ... more variants
}
```

## Error Handling

The system distinguishes between different error categories:

1. **Network Errors** - Connection timeout, connection refused, DNS resolution failure
2. **File System Errors** - Permission denied, file not found, insufficient disk space
3. **GitHub API Errors** - HTTP 4xx/5xx responses, malformed JSON
4. **Version Errors** - Invalid version format
5. **Asset Errors** - No matching asset for platform
6. **Download Errors** - Network interruption, timeout, insufficient disk space
7. **Verification Errors** - File size mismatch, file not readable
8. **Replacement Errors** - Cannot create backup, cannot replace binary, cannot set permissions
9. **Permission Errors** - Cannot write to binary location
10. **Lock File Errors** - Lock file already exists

## Testing

The feature includes comprehensive testing:

- **142 unit tests** covering all components
- **46 property-based tests** verifying universal properties
- **41 integration/mode tests** for end-to-end scenarios

Run tests with:
```bash
cargo test --bin transcript-explorer update
```

## Integration Points

### Main Application

The update manager is integrated into `src/main.rs`:

1. Configuration is loaded on startup
2. UpdateManager is created with the configuration
3. Background thread is spawned
4. Main application continues normally

### Error Handling

Errors during initialization are logged to stderr but don't block the application:

```rust
match update::UpdateConfiguration::load() {
    Ok(config) => {
        match update::UpdateManager::new(config) {
            Ok(manager) => {
                Some(manager.spawn_background_thread())
            }
            Err(e) => {
                eprintln!("Warning: Failed to initialize update manager: {}", e.user_message());
                None
            }
        }
    }
    Err(e) => {
        eprintln!("Warning: Failed to load update configuration: {}", e.user_message());
        None
    }
}
```

## Performance Considerations

- **Background Thread**: Update checks run in a separate thread with their own tokio runtime
- **Memory Usage**: Binary downloads are streamed to disk to minimize memory usage
- **Network**: Retry logic with exponential backoff prevents overwhelming the network
- **Disk I/O**: Atomic replacement minimizes the window where the binary is in an inconsistent state

## Security Considerations

- **HTTPS Only**: All GitHub API requests use HTTPS with TLS 1.2+
- **Backup Preservation**: Original binary is backed up before replacement
- **Health Check**: New binary is verified before committing to the update
- **Atomic Replacement**: Binary replacement is atomic to prevent partial updates
- **Rollback**: Automatic rollback on health check failure
- **Bad Version Tracking**: Prevents update loops with broken releases
