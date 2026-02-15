use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
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
}
