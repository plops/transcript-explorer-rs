# Transcript Explorer RS — Design Specification

A cross-platform Rust TUI application for browsing, searching, and exploring ~12,000 YouTube transcript summaries stored in a SQLite database.

## Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Database | **`turso` v0.4** (pure Rust SQLite) | No C compiler needed → trivial cross-compilation to Linux/macOS/Windows |
| TUI Framework | **`ratatui`** + **`crossterm`** backend | De-facto Rust TUI standard; crossterm provides cross-platform terminal I/O |
| Async Runtime | **`tokio`** | Required by `turso` crate; also drives event loop |
| Vector Search | Turso built-in `vector_distance_cos()` | Cosine similarity over the `embedding` blobs already in the DB |
| CLI Args | **`clap`** (derive) | Accept DB file path, version info |

## Database Schema (read-only)

Table: `items`
```
(
  identifier        INTEGER PRIMARY KEY,
  model              TEXT,
  transcript         TEXT,
  host               TEXT,
  original_source_link TEXT,
  include_comments   BOOLEAN,
  include_timestamps BOOLEAN,
  include_glossary   BOOLEAN,
  output_language    TEXT,
  summary            TEXT,
  summary_done       BOOLEAN,
  summary_input_tokens  INTEGER,
  summary_output_tokens INTEGER,
  summary_timestamp_start TEXT,
  summary_timestamp_end   TEXT,
  timestamps         TEXT,
  timestamps_done    BOOLEAN,
  timestamps_input_tokens  INTEGER,
  timestamps_output_tokens INTEGER,
  timestamps_timestamp_start TEXT,
  timestamps_timestamp_end   TEXT,
  timestamped_summary_in_youtube_format TEXT,
  cost               FLOAT,
  embedding          BLOB,   -- vector for similarity search (f32 LE bytes)
  embedding_model    TEXT,
  full_embedding     BLOB
)
```

## Application Architecture

- **`main.rs`**: Entry point, terminal setup/restoration, and event loop.
- **`app.rs`**: State management (active view, selection, in-memory metadata cache, filtering).
- **`db.rs`**: Turso-specific database queries. Uses `vector_slice` for all similarity comparisons to handle mismatched embedding dimensions.
- **`ui/`**: Modular view rendering for List, Detail, Similar, and Help states.

## Navigation & Workflows

1. **Browsing**: Scroll through all entries. In-memory metadata caching ensures high responsiveness. Includes **PageUp/Down** support.
2. **Filtering**: Real-time, synchronous in-memory filtering that updates **after every keystroke**.
3. **Similarity**: Semantic search using `vector_distance_cos` combined with `vector_slice(..., 0, 768)` to support Matryoshka embeddings.
4. **Detail**: Multi-pane view for technical summaries and full transcripts.
5. **UI Refinements**:
   - **Title Heuristics**: Skip generic prefixes like "**Abstract:**" when extracting list titles.
   - **Entry Collapsing**: Automatically collapse consecutive identical entries in both the **Main List** and **Similarity View**, with a toggle to expand (`Space`).
   - **Live Search**: Integrated real-time filtering updates.

## CI/CD and Automation

- **GitHub Actions**: Automated release pipeline triggered on version tags (`v*`).
- **Cross-Platform Support**: Automated builds for:
  - Linux (x86_64-unknown-linux-gnu)
  - macOS (x86_64-apple-darwin, aarch64-apple-darwin)
  - Windows (x86_64-pc-windows-msvc)
- **Releases**: Assets (.tar.gz and .zip) automatically attached to GitHub Releases.
