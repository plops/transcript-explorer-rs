# Architecture & Implementation Details

This document provides a technical overview of the `transcript-explorer-rs` application.

## System Components

### 1. Database Layer (`src/db.rs`)
The application uses the `turso` crate, which is a pure Rust implementation of a SQLite-compatible database engine.
- **Async I/O**: All database operations are asynchronous, leveraging the `turso` crate's native async support.
- **Vector Search**: Leverages Turso's built-in `vector_distance_cos` function for high-performance similarity search directly in SQL.
- **Read-Only**: The application is designed to open the database in a read-only fashion to avoid file locks or corruption when browsing existing data.

### 2. Application State (`src/app.rs`)
The `App` struct acts as the central state machine.
- **In-Memory Caching**: Loads all transcript metadata into memory at startup (~3MB for 13k rows). This ensures that filtering by summary, host, or source link is instantaneous and synchronous.
- **Smart Grouping**: Identifies consecutive entries with identical summaries and automatically collapses them into groups to improve list scannability.
- **Dynamic Pagination**: While search is in-memory, the UI displays results based on available terminal height (dynamic `page_size`). This ensures maximum use of screen real estate.
- **Synchronous Filtering**: Manages the live filter string and performs local character-matching, avoiding the overhead of database queries during active typing.

### 3. UI Layer (`src/ui/`)
Built using `ratatui` with the `crossterm` backend.
- **Immediate Mode**: The UI is redrawn on every tick (250ms) or event, ensuring a responsive feel.
- **Custom Widgets**: Utilizes `List`, `Paragraph`, `Tabs`, and `Clear` widgets to build a multi-pane interface.
- **Layouts**: Uses constraint-based layouts to ensure the app scales properly across various terminal sizes.

### 4. Maintenance Tools (`tools/`)
The project includes specialized tools for database preparation and sharing.
- **Cleanup Script (`cleanup_db.py`)**: A Python-based utility used to prepare datasets for public distribution. It removes error entries, strips large transcript data, and truncates high-dimensional embeddings to 768 dimensions to optimize for similarity search while reducing disk footprint.
30: 
### 5. Download & Caching (`src/main.rs`)
Automatically manages the database availability for a seamless first-run experience.
- **HTTPS Download**: Uses `reqwest` and `indicatif` to download the latest encrypted database from the server with a terminal progress bar.
- **Project Directories**: Uses the `directories` crate to resolve cross-platform cache paths, ensuring data is stored in the correct locations for Linux, macOS, and Windows.
- **Temporary Decryption**: Encrypted databases are decrypted on-the-fly to volatile temporary files using `tempfile`, which are automatically cleaned up on exit.

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant EventLoop
    participant AppState
    participant Database

    User->>EventLoop: Keyboard Input
    EventLoop->>AppState: handle_key(Event)
    AppState->>AppState: filter_items_locally(filter)
    EventLoop->>User: redraw(Frame)
    User->>AppState: Select Entry
    AppState->>Database: get_transcript(id)
    Database-->>AppState: TranscriptRow
    AppState-->>EventLoop: updated state
    EventLoop->>User: redraw(Frame)
```

## Vector Search Implementation

The application performs semantic similarity search by querying the `embedding` column (BLOB) stored in the database.
1. The user selects a transcript with an embedding (marked with `●`).
2. The app executes a SQL query that uses `vector_distance_cos` between the source and target.
3. **Matryoshka Support**: Because the database contains mixed embedding sizes (3072 and 768 dimensions), the query uses `vector_slice(embedding, 0, 768)` on both operands to ensure compatibility.
4. The results are displayed with similarity scores (calculated as `1.0 - distance`).
