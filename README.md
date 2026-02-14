# Transcript Explorer

A cross-platform Rust Terminal User Interface (TUI) for browsing, searching, and exploring YouTube transcript summaries stored in a SQLite database.

### Main View
![Main Explorer View](screenshot/1.png)
*Filtering thousands of transcript summaries with sub-millisecond responsiveness.*

### Similarity Search
![Vector Similarity View](screenshot/2.png)
*Discovering related content using semantic vector similarity directly in SQL.*

## Features

- **Blazing Fast**: Built with Rust and Turso (pure Rust SQLite engine).
- **Instantaneous Search**: Real-time, in-memory filtering of thousands of entries as you type.
- **Smart Grouping**: Consecutive identical summaries are automatically collapsed into groups for a cleaner browsing experience.
- **Detailed View**: Metadata header with costs/tokens, and tabs for Summary, Transcript, and Timestamps.
- **Vector Similarity**: Find related transcripts using built-in vector similarity search (cosine distance).
- **Portable**: Small, self-contained binary with no C dependencies.
- **Clipboard Integration**: Yank source links directly to your system clipboard.

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) 1.91.0 or newer.

### Build from source
```bash
git clone https://github.com/user/transcript-explorer-rs.git
cd transcript-explorer-rs
cargo build --release
```

## Usage

Run the binary by providing the path to your `summaries.db` file:

```bash
./target/release/transcript-explorer --db /path/to/your/summaries.db
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate lists / scroll content |
| `PgUp`/`PgDn` | Page up/down in lists and content |
| `Space` | Expand/Collapse duplicate group |
| `Enter` | Open detail view / select result |
| `/` | Focus filter bar (live search) |
| `s` | Find similar transcripts (vector search) |
| `Tab` / `1-3` | Switch detail tabs (Summary, Transcript, Timestamps) |
| `y` | Yank source link to clipboard |
| `?` | Toggle help overlay |
| `Esc` | Back / cancel / clear filter |
| `q` | Quit |

## Documentation

- [Architecture & Design](doc/architecture.md)
- [UI Patterns](doc/ui_patterns.md)
- [Release Process](doc/release_process.md)
- [Specification & Research](doc/spec/)

## License

MIT / Apache-2.0
