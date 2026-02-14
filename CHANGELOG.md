# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.2] - 2026-02-14

### Added
- Rich markdown formatting in the preview pane using `tui-markdown`.
- Masked password feedback for CLI prompts when decrypting databases.
- Comprehensive UI tour walkthrough documentation in `doc/ui_tour.md`.
- Dynamic status line messages and cursor positioning in the filter widget.
- Optimization level `1` for the development profile to improve performance during debugging.

### Changed
- Refactored preview pane layout for better separation of metadata and summary content.
- Updated documentation for "Smart Grouping" and architectural patterns.

### Fixed
- Duration calculation to support ISO-like timestamps in the database.

## [1.3.1] - 2026-02-14

### Added
- Preview pane to similarity search view with keyboard shortcuts for YouTube links.
- Database download functionality with cross-platform caching.
- `--password` CLI argument to provide decryption passwords directly.

### Fixed
- `ExcessiveWork` error in debug builds by increasing `max_work_factor` for `age` decryption.
- Reliability issues by simplifying Brotli compression to single-stream.
