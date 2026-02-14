# Dependency Research & Technical Findings

This document summarizes the research performed on the key dependencies during the development of `transcript-explorer-rs`.

## Turso / Libmo (Pure Rust SQLite)

### Crate Version
The project uses `turso` v0.4.0.

### Key Learnings
- **No C Dependencies**: Transitioning from `rusqlite` or `libsql` to `turso` allows for "batteries-included" Rust-only compilation, which is critical for easy cross-platform support.
- **Async API**: Unlike standard SQLite wrappers, `turso` is built around `tokio` and uses an async `connect()`, `query()`, and `execute()` pattern.
- **Rows Iteration**: As of v0.4, the `Rows` object does **not** implement `Stream` or `TryStream`. Iteration must be done via `rows.next().await?`.
- **In-Memory Caching**: For 10,000+ rows, standard `LIKE` searches on summaries take ~200ms per query. Loading all metadata (~3MB) into memory at startup provides instantaneous filtering (<1ms) and a smoother TUI experience.
- **Value Handling**: SQLite values are mapped to a `turso::Value` enum: `Integer(i64)`, `Real(f64)`, `Text(String)`, `Blob(Vec<u8>)`, and `Null`.

## Vector Search Capabilities

### SQL Integration
Turso provides native functions for vector distance:
- `vector_distance_cos(v1, v2)`: Cosine similarity.
- `vector_distance_l2(v1, v2)`: Euclidean distance.

### Matryoshka Embeddings & Dimension Mismatch
The database contains embeddings of two sizes:
- 11,244 rows at **3072** dimensions.
- 1,577 rows at **768** dimensions.

Attempting to compute `vector_distance_cos` between different dimensions results in an error. Since these are Matryoshka embeddings (Gemini), they are designed to be troncable.
- **Fix**: The application now uses `vector_slice(embedding, 0, 768)` in the SQL query for both operands, ensuring consistent 768-dimension comparisons across all entries.

## Ratatui 0.30 (TUI Framework)

### Widgets
- **StatefulWidget**: Used for `List` and `Table` to manage selection state independently of the render loop.
- **Clear**: Essential for popups (like the help screen) to overwrite existing buffer content.
- **Tabs**: Used for the detail view to switch between content types.

### Event Handling
The application uses a 250ms poll loop on `crossterm` events. This provides a balance between responsiveness to user input and terminal redraw efficiency.
- **Input Mode Transition**: Switching from `Normal` to `Editing` (for search) allows the main key dispatcher to ignore navigation shortcuts and focus on character input.
- **Crossterm Backend**: Chosen for its robust cross-platform support for terminal initialization and raw mode.
