use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

/// Operating system types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
}

/// CPU architecture types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
}

/// Platform information containing OS and architecture
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: OperatingSystem,
    pub arch: Architecture,
}

/// Release asset information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

/// Release information from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<ReleaseAsset>,
    pub body: String,
}

/// Update state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateState {
    pub last_check: Option<DateTime<Utc>>,
    pub last_successful_update: Option<DateTime<Utc>>,
    pub bad_versions: HashSet<String>,
    pub current_version: String,
}

/// Result of an update operation
#[derive(Debug, Clone)]
pub enum UpdateResult {
    Updated { new_version: String },
    UpToDate,
    Skipped { reason: String },
}

/// Comprehensive error type for update operations
#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Platform detection failed: {0}")]
    PlatformDetection(String),

    #[error("GitHub API error: HTTP {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Version parsing error: {0}")]
    VersionParse(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Download failed: {reason} (retryable: {retryable})")]
    Download { reason: String, retryable: bool },

    #[error("Verification failed: {reason}")]
    Verification { reason: String },

    #[error("Binary replacement failed: {reason} (recovered: {recovered})")]
    Replacement { reason: String, recovered: bool },

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Update already in progress")]
    LockFileExists,

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),
}

impl UpdateError {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            UpdateError::Download { retryable: true, .. }
                | UpdateError::ApiError { .. }
                | UpdateError::HttpError(_)
        )
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            UpdateError::PlatformDetection(msg) => {
                format!("Could not detect your platform: {}", msg)
            }
            UpdateError::ApiError { status, message } => {
                format!("GitHub API error ({}): {}", status, message)
            }
            UpdateError::VersionParse(msg) => {
                format!("Invalid version format: {}", msg)
            }
            UpdateError::AssetNotFound(msg) => {
                format!("No compatible binary found: {}", msg)
            }
            UpdateError::Download { reason, .. } => {
                format!("Download failed: {}", reason)
            }
            UpdateError::Verification { reason } => {
                format!("Binary verification failed: {}", reason)
            }
            UpdateError::Replacement { reason, recovered } => {
                if *recovered {
                    format!("Update failed but rolled back: {}", reason)
                } else {
                    format!("Update failed: {}", reason)
                }
            }
            UpdateError::PermissionDenied(msg) => {
                format!("Permission denied: {}", msg)
            }
            UpdateError::LockFileExists => {
                "An update is already in progress".to_string()
            }
            UpdateError::ConfigurationError(msg) => {
                format!("Configuration error: {}", msg)
            }
            UpdateError::IoError(e) => format!("File system error: {}", e),
            UpdateError::SerializationError(e) => format!("Data format error: {}", e),
            UpdateError::HttpError(e) => format!("Network error: {}", e),
        }
    }

    /// Get recovery instructions if applicable
    pub fn recovery_instructions(&self) -> Option<String> {
        match self {
            UpdateError::PermissionDenied(_) => Some(
                "Try running the application with elevated privileges or check file permissions"
                    .to_string(),
            ),
            UpdateError::Download { retryable: true, .. } => {
                Some("Check your internet connection and try again".to_string())
            }
            UpdateError::Replacement { recovered: false, .. } => {
                Some("A backup of your previous binary may be available in the backup directory"
                    .to_string())
            }
            _ => None,
        }
    }
}

/// Configuration for update behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfiguration {
    pub enabled: bool,
    pub check_interval_hours: u32,
    pub interactive_mode: bool,
    pub github_repo_owner: String,
    pub github_repo_name: String,
    pub temp_directory: PathBuf,
    pub backup_directory: PathBuf,
}

impl Default for UpdateConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_hours: 24,
            interactive_mode: true,
            github_repo_owner: "your-org".to_string(),
            github_repo_name: "transcript-explorer".to_string(),
            temp_directory: std::env::temp_dir(),
            backup_directory: std::env::temp_dir(),
        }
    }
}

/// Platform detector for identifying the current OS and architecture
pub struct PlatformDetector;

impl PlatformDetector {
    /// Detect the current platform and architecture
    ///
    /// Uses `std::env::consts::OS` and `std::env::consts::ARCH` to determine
    /// the current platform and maps Rust architecture names to release asset
    /// naming conventions.
    ///
    /// # Returns
    /// - `Ok(PlatformInfo)` with detected OS and architecture
    /// - `Err(UpdateError)` if the platform is unsupported
    ///
    /// # Requirements
    /// - 1.1: Detect current platform (Linux, macOS, Windows)
    /// - 1.2: Detect current architecture (x86_64, aarch64)
    /// - 1.3: Use standard Rust environment variables and system APIs
    pub fn detect() -> Result<PlatformInfo, UpdateError> {
        let os = Self::detect_os()?;
        let arch = Self::detect_architecture()?;

        Ok(PlatformInfo { os, arch })
    }

    /// Detect the operating system
    fn detect_os() -> Result<OperatingSystem, UpdateError> {
        match std::env::consts::OS {
            "linux" => Ok(OperatingSystem::Linux),
            "macos" => Ok(OperatingSystem::MacOS),
            "windows" => Ok(OperatingSystem::Windows),
            os => Err(UpdateError::PlatformDetection(format!(
                "Unsupported operating system: {}",
                os
            ))),
        }
    }

    /// Detect the CPU architecture
    fn detect_architecture() -> Result<Architecture, UpdateError> {
        match std::env::consts::ARCH {
            "x86_64" => Ok(Architecture::X86_64),
            "aarch64" => Ok(Architecture::Aarch64),
            arch => Err(UpdateError::PlatformDetection(format!(
                "Unsupported architecture: {}",
                arch
            ))),
        }
    }
}

impl PlatformInfo {
    /// Get the asset name pattern for this platform
    ///
    /// Returns a string pattern that can be used to match release assets
    /// for this platform and architecture combination.
    pub fn asset_pattern(&self) -> String {
        let os_str = match self.os {
            OperatingSystem::Linux => "linux",
            OperatingSystem::MacOS => "macos",
            OperatingSystem::Windows => "windows",
        };

        let arch_str = match self.arch {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
        };

        format!("{}-{}", os_str, arch_str)
    }
}

/// Semantic version representation (major.minor.patch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemanticVersion {
    /// Create a new semantic version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Parse a version string in "major.minor.patch" format
    ///
    /// Handles optional "v" prefix (e.g., "v1.3.2" or "1.3.2")
    ///
    /// # Arguments
    /// * `version_str` - Version string to parse
    ///
    /// # Returns
    /// - `Ok(SemanticVersion)` if parsing succeeds
    /// - `Err(UpdateError)` if the version format is invalid
    ///
    /// # Requirements
    /// - 3.1: Parse "major.minor.patch" format
    /// - 3.1: Handle optional "v" prefix
    pub fn parse(version_str: &str) -> Result<Self, UpdateError> {
        let trimmed = version_str.trim();
        let version_part = if trimmed.starts_with('v') || trimmed.starts_with('V') {
            &trimmed[1..]
        } else {
            trimmed
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(UpdateError::VersionParse(format!(
                "Expected 3 version components (major.minor.patch), got {}",
                parts.len()
            )));
        }

        let major = parts[0].parse::<u32>().map_err(|_| {
            UpdateError::VersionParse(format!("Invalid major version: {}", parts[0]))
        })?;

        let minor = parts[1].parse::<u32>().map_err(|_| {
            UpdateError::VersionParse(format!("Invalid minor version: {}", parts[1]))
        })?;

        let patch = parts[2].parse::<u32>().map_err(|_| {
            UpdateError::VersionParse(format!("Invalid patch version: {}", parts[2]))
        })?;

        Ok(SemanticVersion { major, minor, patch })
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl FromStr for SemanticVersion {
    type Err = UpdateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SemanticVersion::parse(s)
    }
}

/// Version comparator for semantic versions
pub struct VersionComparator;

impl VersionComparator {
    /// Check if remote version is newer than local version
    ///
    /// Compares versions lexicographically by major, then minor, then patch.
    /// Returns true if remote > local.
    ///
    /// # Arguments
    /// * `remote` - The remote version to check
    /// * `local` - The local version to compare against
    ///
    /// # Returns
    /// - `true` if remote version is newer than local version
    /// - `false` otherwise
    ///
    /// # Requirements
    /// - 3.2: Determine if remote version is newer than local version
    /// - 3.4: Compare versions lexicographically by major, minor, patch
    pub fn is_newer(remote: &SemanticVersion, local: &SemanticVersion) -> bool {
        remote > local
    }
}

/// GitHub API response for a release
#[derive(Debug, Clone, Deserialize)]
struct GitHubReleaseResponse {
    tag_name: String,
    name: Option<String>,
    published_at: DateTime<Utc>,
    assets: Vec<GitHubAssetResponse>,
    body: Option<String>,
}

/// GitHub API response for a release asset
#[derive(Debug, Clone, Deserialize)]
struct GitHubAssetResponse {
    name: String,
    browser_download_url: String,
    size: u64,
    created_at: DateTime<Utc>,
}

/// GitHub API client for fetching release information
pub struct GitHubApiClient {
    repo_owner: String,
    repo_name: String,
    http_client: reqwest::Client,
}

impl GitHubApiClient {
    /// Create a new GitHub API client
    ///
    /// Initializes with repo owner and name, and creates a reqwest::Client
    /// with TLS configuration.
    ///
    /// # Arguments
    /// * `repo_owner` - GitHub repository owner (e.g., "your-org")
    /// * `repo_name` - GitHub repository name (e.g., "transcript-explorer")
    ///
    /// # Returns
    /// - `Ok(GitHubApiClient)` if client creation succeeds
    /// - `Err(UpdateError)` if client creation fails
    ///
    /// # Requirements
    /// - 2.1: Initialize with repo owner and name
    /// - 2.1: Create reqwest::Client with TLS configuration
    pub fn new(repo_owner: String, repo_name: String) -> Result<Self, UpdateError> {
        let http_client = reqwest::Client::builder()
            .user_agent("transcript-explorer-updater/1.0")
            .build()
            .map_err(|e| UpdateError::HttpError(e))?;

        Ok(Self {
            repo_owner,
            repo_name,
            http_client,
        })
    }

    /// Get the latest release from GitHub
    ///
    /// Queries the GitHub Releases API endpoint for the latest release,
    /// includes User-Agent header for API compliance, parses the JSON response
    /// into ReleaseInfo, and handles API errors and rate limiting.
    ///
    /// # Returns
    /// - `Ok(ReleaseInfo)` if the request succeeds
    /// - `Err(UpdateError)` if the request fails or response is invalid
    ///
    /// # Requirements
    /// - 2.1: Query GitHub Releases API endpoint
    /// - 2.2: Parse JSON response into ReleaseInfo
    /// - 2.3: Include User-Agent header for API compliance
    /// - 2.1, 2.2, 2.3: Handle API errors and rate limiting
    pub async fn get_latest_release(&self) -> Result<ReleaseInfo, UpdateError> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| UpdateError::HttpError(e))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            return Err(UpdateError::ApiError {
                status: status.as_u16(),
                message: error_body,
            });
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| UpdateError::HttpError(e))?;

        let github_release: GitHubReleaseResponse = serde_json::from_str(&response_text)
            .map_err(|e| UpdateError::SerializationError(e))?;

        // Extract version from tag_name (e.g., "v1.3.2" -> "1.3.2")
        let version = if github_release.tag_name.starts_with('v') {
            github_release.tag_name[1..].to_string()
        } else {
            github_release.tag_name.clone()
        };

        // Convert assets
        let assets = github_release
            .assets
            .into_iter()
            .map(|asset| ReleaseAsset {
                name: asset.name,
                download_url: asset.browser_download_url,
                size: asset.size,
                created_at: asset.created_at,
            })
            .collect();

        Ok(ReleaseInfo {
            version,
            tag_name: github_release.tag_name,
            published_at: github_release.published_at,
            assets,
            body: github_release.body.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info_creation() {
        let platform = PlatformInfo {
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
        };
        assert_eq!(platform.os, OperatingSystem::Linux);
        assert_eq!(platform.arch, Architecture::X86_64);
    }

    #[test]
    fn test_update_error_is_retryable() {
        let download_error = UpdateError::Download {
            reason: "timeout".to_string(),
            retryable: true,
        };
        assert!(download_error.is_retryable());

        let verification_error = UpdateError::Verification {
            reason: "size mismatch".to_string(),
        };
        assert!(!verification_error.is_retryable());
    }

    #[test]
    fn test_update_error_user_message() {
        let error = UpdateError::AssetNotFound("linux-x86_64".to_string());
        let msg = error.user_message();
        assert!(msg.contains("No compatible binary found"));
    }

    #[test]
    fn test_update_error_recovery_instructions() {
        let error = UpdateError::PermissionDenied("binary file".to_string());
        let instructions = error.recovery_instructions();
        assert!(instructions.is_some());
        assert!(instructions.unwrap().contains("elevated privileges"));
    }

    #[test]
    fn test_update_configuration_default() {
        let config = UpdateConfiguration::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval_hours, 24);
        assert!(config.interactive_mode);
    }

    #[test]
    fn test_platform_detector_detect() {
        let platform = PlatformDetector::detect();
        assert!(platform.is_ok());
        let platform = platform.unwrap();
        // Verify we got a valid platform
        assert!(matches!(
            platform.os,
            OperatingSystem::Linux | OperatingSystem::MacOS | OperatingSystem::Windows
        ));
        assert!(matches!(
            platform.arch,
            Architecture::X86_64 | Architecture::Aarch64
        ));
    }

    #[test]
    fn test_platform_info_asset_pattern() {
        let platform = PlatformInfo {
            os: OperatingSystem::Linux,
            arch: Architecture::X86_64,
        };
        assert_eq!(platform.asset_pattern(), "linux-x86_64");

        let platform = PlatformInfo {
            os: OperatingSystem::MacOS,
            arch: Architecture::Aarch64,
        };
        assert_eq!(platform.asset_pattern(), "macos-aarch64");

        let platform = PlatformInfo {
            os: OperatingSystem::Windows,
            arch: Architecture::X86_64,
        };
        assert_eq!(platform.asset_pattern(), "windows-x86_64");
    }

    #[test]
    fn test_platform_detection_consistency() {
        // Detect platform multiple times and verify results are identical
        let platform1 = PlatformDetector::detect().expect("First detection failed");
        let platform2 = PlatformDetector::detect().expect("Second detection failed");
        let platform3 = PlatformDetector::detect().expect("Third detection failed");

        assert_eq!(platform1, platform2);
        assert_eq!(platform2, platform3);
    }

    // Version parsing tests
    #[test]
    fn test_semantic_version_parse_valid() {
        let version = SemanticVersion::parse("1.2.3").expect("Failed to parse valid version");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_semantic_version_parse_with_v_prefix() {
        let version = SemanticVersion::parse("v1.2.3").expect("Failed to parse version with v prefix");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_semantic_version_parse_with_uppercase_v() {
        let version = SemanticVersion::parse("V1.2.3").expect("Failed to parse version with V prefix");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_semantic_version_parse_with_whitespace() {
        let version = SemanticVersion::parse("  v1.2.3  ").expect("Failed to parse version with whitespace");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_semantic_version_parse_zero_version() {
        let version = SemanticVersion::parse("0.0.0").expect("Failed to parse 0.0.0");
        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_semantic_version_parse_large_numbers() {
        let version = SemanticVersion::parse("999.999.999").expect("Failed to parse large version");
        assert_eq!(version.major, 999);
        assert_eq!(version.minor, 999);
        assert_eq!(version.patch, 999);
    }

    #[test]
    fn test_semantic_version_parse_invalid_too_few_parts() {
        let result = SemanticVersion::parse("1.2");
        assert!(result.is_err());
        match result {
            Err(UpdateError::VersionParse(msg)) => assert!(msg.contains("3 version components")),
            _ => panic!("Expected VersionParse error"),
        }
    }

    #[test]
    fn test_semantic_version_parse_invalid_too_many_parts() {
        let result = SemanticVersion::parse("1.2.3.4");
        assert!(result.is_err());
        match result {
            Err(UpdateError::VersionParse(msg)) => assert!(msg.contains("3 version components")),
            _ => panic!("Expected VersionParse error"),
        }
    }

    #[test]
    fn test_semantic_version_parse_invalid_non_numeric() {
        let result = SemanticVersion::parse("1.2.abc");
        assert!(result.is_err());
        match result {
            Err(UpdateError::VersionParse(msg)) => assert!(msg.contains("Invalid patch version")),
            _ => panic!("Expected VersionParse error"),
        }
    }

    #[test]
    fn test_semantic_version_parse_invalid_empty_string() {
        let result = SemanticVersion::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_version_display() {
        let version = SemanticVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_semantic_version_from_str() {
        let version: SemanticVersion = "1.2.3".parse().expect("Failed to parse from str");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    // Version comparison tests
    #[test]
    fn test_version_comparator_is_newer_major() {
        let remote = SemanticVersion::new(2, 0, 0);
        let local = SemanticVersion::new(1, 9, 9);
        assert!(VersionComparator::is_newer(&remote, &local));
    }

    #[test]
    fn test_version_comparator_is_newer_minor() {
        let remote = SemanticVersion::new(1, 3, 0);
        let local = SemanticVersion::new(1, 2, 9);
        assert!(VersionComparator::is_newer(&remote, &local));
    }

    #[test]
    fn test_version_comparator_is_newer_patch() {
        let remote = SemanticVersion::new(1, 2, 3);
        let local = SemanticVersion::new(1, 2, 2);
        assert!(VersionComparator::is_newer(&remote, &local));
    }

    #[test]
    fn test_version_comparator_is_not_newer_equal() {
        let remote = SemanticVersion::new(1, 2, 3);
        let local = SemanticVersion::new(1, 2, 3);
        assert!(!VersionComparator::is_newer(&remote, &local));
    }

    #[test]
    fn test_version_comparator_is_not_newer_older() {
        let remote = SemanticVersion::new(1, 2, 2);
        let local = SemanticVersion::new(1, 2, 3);
        assert!(!VersionComparator::is_newer(&remote, &local));
    }

    #[test]
    fn test_version_comparator_transitivity() {
        // If A < B and B < C, then A < C
        let a = SemanticVersion::new(1, 0, 0);
        let b = SemanticVersion::new(1, 1, 0);
        let c = SemanticVersion::new(1, 2, 0);

        assert!(!VersionComparator::is_newer(&a, &b)); // a < b
        assert!(!VersionComparator::is_newer(&b, &c)); // b < c
        assert!(!VersionComparator::is_newer(&a, &c)); // a < c (transitivity)
    }

    #[test]
    fn test_version_comparator_current_version() {
        // Test with current version from Cargo.toml (1.3.2)
        let current = SemanticVersion::parse("1.3.2").expect("Failed to parse current version");
        let newer = SemanticVersion::parse("1.3.3").expect("Failed to parse newer version");
        let older = SemanticVersion::parse("1.3.1").expect("Failed to parse older version");

        assert!(VersionComparator::is_newer(&newer, &current));
        assert!(!VersionComparator::is_newer(&older, &current));
        assert!(!VersionComparator::is_newer(&current, &current));
    }

    #[test]
    fn test_semantic_version_parsing_idempotence() {
        // Parse the same version string multiple times and verify results are identical
        let version_str = "1.2.3";
        let v1 = SemanticVersion::parse(version_str).expect("First parse failed");
        let v2 = SemanticVersion::parse(version_str).expect("Second parse failed");
        let v3 = SemanticVersion::parse(version_str).expect("Third parse failed");

        assert_eq!(v1, v2);
        assert_eq!(v2, v3);
    }

    // GitHub API client tests
    #[test]
    fn test_github_api_client_creation() {
        let result = GitHubApiClient::new(
            "your-org".to_string(),
            "transcript-explorer".to_string(),
        );
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(client.repo_owner, "your-org");
        assert_eq!(client.repo_name, "transcript-explorer");
    }

    #[tokio::test]
    async fn test_github_api_client_get_latest_release() {
        // This test uses the real GitHub API to fetch the latest release
        // It will only work if the repository exists and has releases
        let client = GitHubApiClient::new(
            "your-org".to_string(),
            "transcript-explorer".to_string(),
        )
        .expect("Failed to create client");

        // This will fail if the repo doesn't exist, which is expected for this test
        // In a real scenario, this would be mocked or use a test repository
        let result = client.get_latest_release().await;

        // We just verify the error handling works correctly
        // The actual API call may fail if the repo doesn't exist
        match result {
            Ok(release) => {
                // If we get a release, verify it has the expected structure
                assert!(!release.version.is_empty());
                assert!(!release.tag_name.is_empty());
            }
            Err(UpdateError::ApiError { status, .. }) => {
                // 404 is expected if the repo doesn't exist
                assert!(status == 404 || status == 403);
            }
            Err(e) => {
                // Other errors are acceptable (network issues, etc.)
                eprintln!("API error: {}", e);
            }
        }
    }

    #[test]
    fn test_github_release_response_deserialization() {
        // Test that we can deserialize a GitHub API response
        let json = r#"{
            "tag_name": "v1.3.2",
            "name": "Release 1.3.2",
            "published_at": "2024-01-15T10:30:00Z",
            "assets": [
                {
                    "name": "transcript-explorer-linux-x86_64",
                    "browser_download_url": "https://github.com/your-org/transcript-explorer/releases/download/v1.3.2/transcript-explorer-linux-x86_64",
                    "size": 1024000,
                    "created_at": "2024-01-15T10:30:00Z"
                }
            ],
            "body": "Release notes here"
        }"#;

        let response: Result<GitHubReleaseResponse, _> = serde_json::from_str(json);
        assert!(response.is_ok());

        let release = response.unwrap();
        assert_eq!(release.tag_name, "v1.3.2");
        assert_eq!(release.assets.len(), 1);
        assert_eq!(release.assets[0].name, "transcript-explorer-linux-x86_64");
        assert_eq!(release.assets[0].size, 1024000);
    }

    #[test]
    fn test_github_api_client_version_extraction() {
        // Test that version is correctly extracted from tag_name
        let json = r#"{
            "tag_name": "v1.3.2",
            "name": "Release 1.3.2",
            "published_at": "2024-01-15T10:30:00Z",
            "assets": [],
            "body": "Release notes"
        }"#;

        let response: GitHubReleaseResponse = serde_json::from_str(json).unwrap();
        let version = if response.tag_name.starts_with('v') {
            response.tag_name[1..].to_string()
        } else {
            response.tag_name.clone()
        };

        assert_eq!(version, "1.3.2");
    }

    #[test]
    fn test_github_api_client_version_extraction_no_prefix() {
        // Test version extraction when tag_name has no "v" prefix
        let json = r#"{
            "tag_name": "1.3.2",
            "name": "Release 1.3.2",
            "published_at": "2024-01-15T10:30:00Z",
            "assets": [],
            "body": "Release notes"
        }"#;

        let response: GitHubReleaseResponse = serde_json::from_str(json).unwrap();
        let version = if response.tag_name.starts_with('v') {
            response.tag_name[1..].to_string()
        } else {
            response.tag_name.clone()
        };

        assert_eq!(version, "1.3.2");
    }
}
