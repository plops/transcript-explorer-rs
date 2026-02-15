# Self-Update Feature

## Overview

The self-update feature enables transcript-explorer to automatically check for and install new releases from GitHub. This keeps the application current without requiring manual downloads. The system supports multiple platforms (Linux, macOS, Windows) and architectures (x86_64, aarch64) while handling errors gracefully and providing clear user feedback.

## How It Works

### Background Update Check

When transcript-explorer starts, it spawns a background thread that:

1. Detects your platform and architecture
2. Queries the GitHub API for the latest release
3. Compares the remote version with your current version
4. If a newer version is available, prompts you (in interactive mode) or automatically downloads it (in non-interactive mode)
5. Verifies the downloaded binary
6. Safely replaces your current binary with a backup
7. Runs a health check on the new binary
8. Rolls back if anything goes wrong

The background thread runs independently, so your application continues to work normally during the update process.

## Configuration

### Environment Variables

You can customize update behavior using environment variables:

```bash
# Enable/disable auto-update (default: true)
UPDATE_ENABLED=true

# Check interval in hours (default: 24)
UPDATE_CHECK_INTERVAL_HOURS=24

# Interactive mode - prompt before updating (default: true)
UPDATE_INTERACTIVE_MODE=true

# GitHub repository owner (default: plops)
UPDATE_GITHUB_REPO_OWNER=plops

# GitHub repository name (default: transcript-explorer-rs)
UPDATE_GITHUB_REPO_NAME=transcript-explorer-rs

# Temporary directory for downloads (default: system temp dir)
UPDATE_TEMP_DIRECTORY=/tmp

# Backup directory for old binaries (default: system temp dir)
UPDATE_BACKUP_DIRECTORY=/tmp
```

### Configuration File

You can also create a configuration file at `~/.config/transcript-explorer/update.json`:

```json
{
  "enabled": true,
  "check_interval_hours": 24,
  "interactive_mode": true,
  "github_repo_owner": "plops",
  "github_repo_name": "transcript-explorer-rs",
  "temp_directory": "/tmp",
  "backup_directory": "/tmp"
}
```

Environment variables override configuration file settings.

## Modes

### Interactive Mode (Default)

In interactive mode, you'll be prompted before checking for updates and before downloading:

```
Check for updates? (y/n): y
New version available: 1.4.0
Download and install version 1.4.0? (y/n): y
Downloading update...
Download progress: [==============================] 100.0% (5242880/5242880)
Verifying binary...
Installing update...
Update successful! New version: 1.4.0
```

### Non-Interactive Mode

In non-interactive mode, updates are downloaded and installed automatically without prompts:

```bash
UPDATE_INTERACTIVE_MODE=false transcript-explorer
```

All results are logged to stdout/stderr for automation workflows.

## Safety Features

### Backup Creation

Before replacing your binary, the system creates a timestamped backup:

```
~/.cache/transcript-explorer/backups/
├── transcript-explorer.1.3.2.20250215T143022Z
├── transcript-explorer.1.3.1.20250214T091500Z
└── transcript-explorer.1.3.0.20250213T182345Z
```

If something goes wrong during the update, the backup is automatically restored.

### Health Check

After replacing the binary, the system runs a health check to ensure the new binary works correctly. If the health check fails, the system automatically rolls back to the previous version.

### Bad Version Tracking

If a version fails the health check, it's marked as "bad" and won't be offered again:

```
~/.cache/transcript-explorer/bad_versions.json
```

This prevents update loops with broken releases.

### Lock File

The system uses a lock file to prevent concurrent update processes:

```
~/.cache/transcript-explorer/update.lock
```

If the application crashes during an update, the lock file is cleaned up on the next run.

## Troubleshooting

### Update Check Failed

If you see "Background update check failed", check:

1. **Internet connection** - Verify you can reach GitHub
2. **GitHub API rate limiting** - GitHub limits API requests to 60 per hour for unauthenticated requests
3. **Repository configuration** - Verify `UPDATE_GITHUB_REPO_OWNER` and `UPDATE_GITHUB_REPO_NAME` are correct

### Update Stuck

If an update appears stuck:

1. Check if a lock file exists: `~/.cache/transcript-explorer/update.lock`
2. If the lock file is stale (older than 1 hour), you can safely delete it
3. Restart the application

### Rollback to Previous Version

If you need to use a previous version:

1. Find the backup: `ls ~/.cache/transcript-explorer/backups/`
2. Copy the backup to your binary location: `cp ~/.cache/transcript-explorer/backups/transcript-explorer.1.3.2.* /path/to/transcript-explorer`
3. Make it executable: `chmod +x /path/to/transcript-explorer`

### Disable Auto-Update

To disable auto-update:

```bash
UPDATE_ENABLED=false transcript-explorer
```

Or set it in your configuration file.

## Platform-Specific Notes

### Linux

- Backups are stored in `~/.cache/transcript-explorer/`
- Executable permissions are preserved with `chmod +x`
- The system uses standard Linux paths for temporary files

### macOS

- Backups are stored in `~/Library/Caches/transcript-explorer/`
- Executable permissions are preserved with `chmod +x`
- The system uses standard macOS paths for temporary files

### Windows

- Backups are stored in `%APPDATA%\transcript-explorer\cache\`
- Executable permissions are handled by Windows (no chmod needed)
- The system uses standard Windows paths for temporary files

## Architecture Support

The self-update feature supports:

- **Linux**: x86_64, aarch64
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Windows**: x86_64

The system automatically detects your platform and architecture and downloads the correct binary.

## Error Handling

The system distinguishes between different error types and provides appropriate recovery suggestions:

### Network Errors

- **Timeout**: Check your internet connection and try again
- **Connection refused**: Verify GitHub is accessible
- **DNS resolution failed**: Check your DNS settings

### File System Errors

- **Permission denied**: Run with elevated privileges or check file permissions
- **File not found**: Ensure the file exists and the path is correct
- **Insufficient disk space**: Free up disk space and try again

### GitHub API Errors

- **404 Not Found**: Verify the repository owner and name are correct
- **Rate limit exceeded**: Wait an hour and try again
- **Server error (5xx)**: GitHub may be experiencing issues; try again later

## Development

For developers working on the self-update feature:

- Implementation: `src/update/mod.rs`
- Specification: `.kiro/specs/auto-update/`
- Tests: Run `cargo test --bin transcript-explorer update`

The feature includes comprehensive unit tests and property-based tests to ensure correctness across all platforms and scenarios.
