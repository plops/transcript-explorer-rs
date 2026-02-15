# Self-Update Configuration Guide

## Quick Start

The self-update feature works out of the box with sensible defaults. No configuration is required to get started.

## Configuration Methods

There are three ways to configure the self-update feature, in order of precedence:

1. **Environment Variables** (highest priority)
2. **Configuration File**
3. **Built-in Defaults** (lowest priority)

## Environment Variables

Set environment variables before running transcript-explorer:

```bash
export UPDATE_ENABLED=true
export UPDATE_INTERACTIVE_MODE=true
export UPDATE_GITHUB_REPO_OWNER=plops
export UPDATE_GITHUB_REPO_NAME=transcript-explorer-rs
transcript-explorer
```

### Available Variables

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `UPDATE_ENABLED` | boolean | `true` | Enable/disable auto-update |
| `UPDATE_CHECK_INTERVAL_HOURS` | number | `24` | Hours between update checks |
| `UPDATE_INTERACTIVE_MODE` | boolean | `true` | Prompt before updating |
| `UPDATE_GITHUB_REPO_OWNER` | string | `plops` | GitHub repository owner |
| `UPDATE_GITHUB_REPO_NAME` | string | `transcript-explorer-rs` | GitHub repository name |
| `UPDATE_TEMP_DIRECTORY` | path | system temp | Directory for downloads |
| `UPDATE_BACKUP_DIRECTORY` | path | system temp | Directory for backups |

## Configuration File

Create `~/.config/transcript-explorer/update.json`:

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

### Platform-Specific Paths

**Linux:**
```
~/.config/transcript-explorer/update.json
```

**macOS:**
```
~/Library/Application Support/transcript-explorer/update.json
```

**Windows:**
```
%APPDATA%\transcript-explorer\update.json
```

## Common Configurations

### Disable Auto-Update

**Environment Variable:**
```bash
UPDATE_ENABLED=false transcript-explorer
```

**Configuration File:**
```json
{
  "enabled": false
}
```

### Non-Interactive Mode (for CI/CD)

**Environment Variable:**
```bash
UPDATE_INTERACTIVE_MODE=false transcript-explorer
```

**Configuration File:**
```json
{
  "interactive_mode": false
}
```

### Custom GitHub Repository

**Environment Variables:**
```bash
UPDATE_GITHUB_REPO_OWNER=myorg
UPDATE_GITHUB_REPO_NAME=my-fork
transcript-explorer
```

**Configuration File:**
```json
{
  "github_repo_owner": "myorg",
  "github_repo_name": "my-fork"
}
```

### Custom Directories

**Environment Variables:**
```bash
UPDATE_TEMP_DIRECTORY=/var/tmp
UPDATE_BACKUP_DIRECTORY=/var/backups/transcript-explorer
transcript-explorer
```

**Configuration File:**
```json
{
  "temp_directory": "/var/tmp",
  "backup_directory": "/var/backups/transcript-explorer"
}
```

### Check for Updates Every 12 Hours

**Environment Variable:**
```bash
UPDATE_CHECK_INTERVAL_HOURS=12 transcript-explorer
```

**Configuration File:**
```json
{
  "check_interval_hours": 12
}
```

## Validation

The configuration is validated on startup. Invalid values will:

1. Log a warning to stderr
2. Use the default value
3. Continue running normally

Example warning:
```
Warning: Failed to load update configuration: Invalid check interval: 0 (must be > 0)
Using default: 24 hours
```

## Precedence Example

If you have:

**Configuration File** (`~/.config/transcript-explorer/update.json`):
```json
{
  "enabled": true,
  "interactive_mode": false,
  "check_interval_hours": 12
}
```

**Environment Variables:**
```bash
UPDATE_ENABLED=false
UPDATE_CHECK_INTERVAL_HOURS=6
```

**Result:**
- `enabled`: `false` (from environment variable)
- `interactive_mode`: `false` (from configuration file)
- `check_interval_hours`: `6` (from environment variable)

## Troubleshooting Configuration

### Configuration Not Being Applied

1. Check the file path is correct for your platform
2. Verify the JSON is valid: `jq . ~/.config/transcript-explorer/update.json`
3. Check for typos in environment variable names (they're case-sensitive)
4. Look for warning messages in stderr when starting the application

### Invalid Configuration File

If your configuration file has invalid JSON:

```bash
# Validate JSON
jq . ~/.config/transcript-explorer/update.json

# If invalid, fix it or delete it
rm ~/.config/transcript-explorer/update.json
```

### Environment Variable Not Working

Environment variables are case-sensitive. Verify the exact name:

```bash
# Correct
UPDATE_ENABLED=false

# Incorrect (won't work)
update_enabled=false
Update_Enabled=false
```

## Advanced Configuration

### Using a Custom Repository

To use a fork or mirror of transcript-explorer:

```bash
UPDATE_GITHUB_REPO_OWNER=your-username
UPDATE_GITHUB_REPO_NAME=transcript-explorer-rs
transcript-explorer
```

The repository must have releases published on GitHub for the update check to work.

### Separate Backup Directory

To keep backups in a specific location:

```bash
UPDATE_BACKUP_DIRECTORY=/mnt/backups/transcript-explorer
transcript-explorer
```

Ensure the directory exists and is writable:

```bash
mkdir -p /mnt/backups/transcript-explorer
chmod 755 /mnt/backups/transcript-explorer
```

### Temporary Directory on Fast Storage

For faster downloads, use a temporary directory on fast storage:

```bash
UPDATE_TEMP_DIRECTORY=/mnt/nvme/tmp
transcript-explorer
```

## Default Configuration

If no configuration file or environment variables are set, these defaults are used:

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

These defaults provide a safe, user-friendly experience with interactive prompts and daily update checks.
