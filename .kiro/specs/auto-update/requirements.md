# Requirements Document: Auto-Update Functionality

## Introduction

The auto-update feature enables transcript-explorer to automatically check for and install new releases from GitHub. This feature improves user experience by keeping the application current without requiring manual downloads. The system must support multiple platforms (Linux, macOS, Windows) and architectures (x86_64, aarch64) while handling errors gracefully and providing clear user feedback.

## Glossary

- **GitHub_API**: The GitHub REST API v3 endpoint for accessing release information
- **Release**: A versioned distribution of transcript-explorer published on GitHub
- **Binary**: The compiled executable file for a specific platform and architecture
- **Platform**: The operating system (Linux, macOS, Windows)
- **Architecture**: The CPU architecture (x86_64, aarch64)
- **Version**: A semantic version string (e.g., 1.3.2) following semver conventions
- **Current_Binary**: The currently running transcript-explorer executable
- **New_Binary**: The downloaded executable from a newer release
- **Backup**: A copy of the current binary preserved before replacement
- **Interactive_Mode**: User interaction enabled with prompts and confirmations
- **Non_Interactive_Mode**: Automated operation without user prompts
- **Update_Check**: The process of querying GitHub for the latest release
- **Update_Manager**: The system component responsible for orchestrating the update process

## Requirements

### Requirement 1: Platform and Architecture Detection

**User Story:** As a user, I want the system to automatically detect my platform and architecture, so that the correct binary is downloaded without manual configuration.

#### Acceptance Criteria

1. WHEN the Update_Manager starts, THE System SHALL detect the current platform (Linux, macOS, or Windows)
2. WHEN the Update_Manager starts, THE System SHALL detect the current architecture (x86_64 or aarch64)
3. WHEN platform detection occurs, THE System SHALL use standard Rust environment variables and system APIs to determine the platform
4. IF platform or architecture cannot be determined, THEN THE System SHALL return a descriptive error indicating the unsupported configuration

### Requirement 2: GitHub Release Fetching

**User Story:** As a developer, I want the system to fetch release information from GitHub API, so that it can identify available updates.

#### Acceptance Criteria

1. WHEN an Update_Check is initiated, THE GitHub_API_Client SHALL query the GitHub API for the latest release of transcript-explorer
2. WHEN the GitHub API responds successfully, THE System SHALL parse the release information including version, assets, and metadata
3. WHEN the GitHub API is queried, THE System SHALL include appropriate HTTP headers (User-Agent) to comply with GitHub API requirements
4. IF the GitHub API request fails, THEN THE System SHALL return an error with the HTTP status code and response body
5. IF the GitHub API response is malformed, THEN THE System SHALL return a parsing error with details about the invalid structure

### Requirement 3: Version Comparison

**User Story:** As a user, I want the system to compare versions correctly, so that it only downloads updates when a newer version is available.

#### Acceptance Criteria

1. WHEN comparing versions, THE Version_Comparator SHALL parse semantic versions (major.minor.patch format)
2. WHEN comparing two versions, THE Version_Comparator SHALL determine if the remote version is newer than the Current_Binary version
3. WHEN the remote version is not newer, THE System SHALL inform the user that no update is available
4. WHEN the remote version is newer, THE System SHALL proceed with the update process
5. IF version strings are invalid or unparseable, THEN THE System SHALL return a version parsing error

### Requirement 4: Binary Asset Selection

**User Story:** As a user, I want the system to automatically select the correct binary for my platform, so that I don't need to manually choose the right download.

#### Acceptance Criteria

1. WHEN a release is fetched, THE Asset_Selector SHALL identify all available binary assets for the release
2. WHEN selecting an asset, THE Asset_Selector SHALL match the current platform and architecture to the available assets
3. WHEN a matching asset is found, THE System SHALL extract the download URL and asset metadata
4. IF no matching asset exists for the current platform/architecture, THEN THE System SHALL return an error indicating the platform is not supported in this release
5. IF multiple matching assets are found, THEN THE System SHALL select the most specific match (e.g., prefer platform-specific over generic)

### Requirement 5: Binary Download with Progress Tracking

**User Story:** As a user, I want to see download progress, so that I understand how long the update will take.

#### Acceptance Criteria

1. WHEN a binary download begins, THE Downloader SHALL display a progress bar showing download progress
2. WHEN downloading, THE Downloader SHALL report the current bytes downloaded and total bytes to download
3. WHEN the download completes successfully, THE System SHALL verify the downloaded file exists and has the expected size
4. IF the download is interrupted, THEN THE System SHALL allow the user to retry or cancel
5. IF the download fails, THEN THE System SHALL return an error with details about the failure (network error, timeout, etc.)

### Requirement 6: Binary Verification

**User Story:** As a user, I want the downloaded binary to be verified, so that I can trust it hasn't been corrupted or tampered with.

#### Acceptance Criteria

1. WHEN a binary is downloaded, THE Verifier SHALL check that the file exists and is readable
2. WHEN a binary is downloaded, THE Verifier SHALL verify the file size matches the expected size from the release metadata
3. WHEN verification succeeds, THE System SHALL proceed with the replacement process
4. IF the file size does not match, THEN THE System SHALL return a verification error and delete the corrupted file
5. IF the file is not readable, THEN THE System SHALL return a file access error

### Requirement 7: Safe Binary Replacement

**User Story:** As a user, I want the system to safely replace my current binary, so that I don't lose the ability to run the application if something goes wrong.

#### Acceptance Criteria

1. WHEN replacement begins, THE Replacer SHALL create a Backup of the Current_Binary before any modifications
2. WHEN the Backup is created, THE System SHALL store it in a safe location with a timestamp
3. WHEN the New_Binary is ready, THE Replacer SHALL replace the Current_Binary with the New_Binary
4. WHEN replacement succeeds, THE System SHALL verify the new binary is executable and in the correct location
5. IF replacement fails, THEN THE System SHALL restore the Current_Binary from the Backup and return an error
6. IF the Backup cannot be created, THEN THE System SHALL abort the update and return an error

### Requirement 8: Interactive Mode User Interaction

**User Story:** As a user, I want to be prompted before updating, so that I can choose when to update my application.

#### Acceptance Criteria

1. WHEN the Update_Manager runs in Interactive_Mode, THE System SHALL prompt the user before checking for updates
2. WHEN an update is available, THE System SHALL display the new version and ask for confirmation before downloading
3. WHEN the user confirms, THE System SHALL proceed with the download and installation
4. WHEN the user declines, THE System SHALL abort the update process
5. WHEN the download is in progress, THE System SHALL allow the user to cancel the operation

### Requirement 9: Non-Interactive Mode Operation

**User Story:** As a system administrator, I want to run updates automatically without user interaction, so that I can automate deployment workflows.

#### Acceptance Criteria

1. WHEN the Update_Manager runs in Non_Interactive_Mode, THE System SHALL skip all user prompts
2. WHEN an update is available in Non_Interactive_Mode, THE System SHALL automatically download and install it
3. WHEN the update completes in Non_Interactive_Mode, THE System SHALL log the result to standard output or a log file
4. WHEN an error occurs in Non_Interactive_Mode, THE System SHALL log the error and exit with a non-zero exit code
5. WHEN no update is available in Non_Interactive_Mode, THE System SHALL exit with a zero exit code

### Requirement 10: Error Handling and Recovery

**User Story:** As a user, I want clear error messages when something goes wrong, so that I can understand what happened and how to fix it.

#### Acceptance Criteria

1. WHEN an error occurs, THE Error_Handler SHALL return a descriptive error message indicating the error type
2. WHEN a network error occurs, THE System SHALL distinguish between timeout, connection refused, and other network errors
3. WHEN a file system error occurs, THE System SHALL distinguish between permission denied, file not found, and other file errors
4. WHEN an error occurs during replacement, THE System SHALL attempt to restore the Backup and report the recovery status
5. IF recovery fails, THEN THE System SHALL provide instructions for manual recovery

### Requirement 11: User Feedback and Logging

**User Story:** As a user, I want to see what the system is doing, so that I understand the update process.

#### Acceptance Criteria

1. WHEN the Update_Manager starts, THE System SHALL display the current version
2. WHEN checking for updates, THE System SHALL display status messages (e.g., "Checking for updates...")
3. WHEN a new version is found, THE System SHALL display the new version number
4. WHEN downloading, THE System SHALL display the download progress with a progress bar
5. WHEN the update completes, THE System SHALL display a success message with the new version
6. WHEN an error occurs, THE System SHALL display an error message with actionable information

### Requirement 12: Cross-Platform Binary Execution

**User Story:** As a user on any supported platform, I want the updated binary to be executable, so that I can run the application immediately after updating.

#### Acceptance Criteria

1. WHEN a binary is replaced on Linux or macOS, THE System SHALL set the executable permission (chmod +x)
2. WHEN a binary is replaced on Windows, THE System SHALL ensure the file is executable (Windows default)
3. WHEN the replacement completes, THE System SHALL verify the new binary is executable
4. IF the executable permission cannot be set, THEN THE System SHALL return a permission error

### Requirement 13: Concurrent Update Prevention

**User Story:** As a user, I want to prevent multiple simultaneous update processes, so that the system remains stable.

#### Acceptance Criteria

1. WHEN an update process starts, THE System SHALL create a lock file to prevent concurrent updates
2. WHEN another update is attempted while one is in progress, THE System SHALL return an error indicating an update is already in progress
3. WHEN an update completes, THE System SHALL remove the lock file
4. IF the application crashes during an update, THE System SHALL clean up the lock file on the next run

### Requirement 14: Configuration and Customization

**User Story:** As a user, I want to configure update behavior, so that I can customize how updates work.

#### Acceptance Criteria

1. WHEN the Update_Manager initializes, THE System SHALL read configuration from environment variables or a config file
2. WHERE auto-update is disabled via configuration, THE System SHALL skip all update checks
3. WHEN a custom GitHub repository is configured, THE System SHALL use that repository instead of the default
4. WHEN a custom download directory is configured, THE System SHALL use that directory for temporary files
5. IF configuration is invalid, THEN THE System SHALL use sensible defaults and log a warning

### Requirement 15: Bad Version Tracking

**User Story:** As a user, I want the system to remember failed updates, so that I don't get stuck in an update loop with broken versions.

#### Acceptance Criteria

1. WHEN a binary fails the health check, THE System SHALL mark that version as bad and persist it to a cache file
2. WHEN the Update_Manager checks for updates, THE System SHALL skip any versions marked as bad
3. WHEN a version is marked as bad, THE System SHALL store it in a system-dependent cache directory (e.g., ~/.cache on Linux, ~/Library/Caches on macOS, %APPDATA% on Windows)
4. WHEN the cache file is loaded, THE System SHALL parse the list of bad versions and use it to filter available updates
5. IF the cache file is corrupted or unreadable, THEN THE System SHALL start with an empty bad version list and log a warning
6. WHEN the application starts, THE System SHALL load the bad version list from the cache file
7. WHEN a user manually requests an update, THE System SHALL allow bypassing the bad version list with a flag or option

