# Dependency Research & Technical Findings

This document summarizes the research performed on the key dependencies during the development of `transcript-explorer-rs`.

## Turso / Libmo (Pure Rust SQLite)

### Crate Version
The project uses `turso` v0.4.0.

### Key Learnings
- **No C Dependencies**: Transitioning from `rusqlite` or `libsql` to `turso` allows for "batteries-included" Rust-only compilation, which is critical for easy cross-platform support.
- **Async API**: Unlike standard SQLite wrappers, `turso` is built around `tokio` and uses an async `connect()`, `query()`, and `execute()` pattern.
- **Rows Iteration**: As of v0.4, the `Rows` object does **not** implement `Stream` or `TryStream`. Iteration must be done via `rows.next().await?` which returns `Option<Row>`.
- **Value Handling**: SQLite values are mapped to a `turso::Value` enum: `Integer(i64)`, `Real(f64)`, `Text(String)`, `Blob(Vec<u8>)`, and `Null`.

## Vector Search Capabilities

### SQL Integration
Turso provides native functions for vector distance:
- `vector_distance_cos(v1, v2)`: Cosine similarity.
- `vector_distance_l2(v1, v2)`: Euclidean distance.

### Data Format
The embeddings in the provided database were stored using Python's `numpy.float32.tobytes()`. 
- **Format**: Raw 32-bit floats in Little-Endian byte order.
- **Interpretation**: Turso treats these as `BLOB`s when computing distances.
- **Query Pattern**: Similarity search is most efficient when calculated in a single SQL query using a subquery to fetch the source vector:
  ```sql
  SELECT ... vector_distance_cos(t.embedding, s.embedding) 
  FROM items t, (SELECT embedding FROM items WHERE id = ?) s
  ```

## Ratatui 0.30 (TUI Framework)

### Widgets
- **StatefulWidget**: Used for `List` and `Table` to manage selection state independently of the render loop.
- **Clear**: Essential for popups (like the help screen) to overwrite existing buffer content.
- **Tabs**: Used for the detail view to switch between content types.

### Event Handling
The application uses a 250ms poll loop on `crossterm` events. This provides a balance between responsiveness to user input and terminal redraw efficiency.
- **Input Mode Transition**: Switching from `Normal` to `Editing` (for search) allows the main key dispatcher to ignore navigation shortcuts and focus on character input.
- **Crossterm Backend**: Chosen for its robust cross-platform support for terminal initialization and raw mode.
