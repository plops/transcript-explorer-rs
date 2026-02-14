# Database Maintenance & Preparation

To share the datasets used by Transcript Explorer publicly, it is often necessary to reduce the file size and remove non-essential or sensitive information.

## Cleanup Tool (`tools/cleanup_db.py`)

The cleanup script is designed to take a full production database (typically containing full transcripts and high-resolution embeddings) and produce a lightweight version suitable for demonstration.

### Features

- **Error Removal**: Automatically deletes rows where the `summary` indicates a processing error (e.g., "Error: resource exhausted").
- **Privacy & Size Optimization**: Empties the `transcript`, `timestamps`, and `timestamped_summary_in_youtube_format` columns while keeping the short `summary` and metadata.
- **Embedding Truncation**: Truncates 3072-dimensional embeddings to 768 dimensions (Matryoshka embeddings). This significantly reduces file size while maintaining high-quality similarity search results, as the application is optimized for 768-dimensional vectors.
- **Disk Space Recovery**: Performs a `VACUUM` on the resulting file to reclaim free space.

### Prerequisites

- [uv](https://github.com/astral-sh/uv) (Astral's Python package manager)

### Usage

The script is located in the `tools/` directory.

```bash
cd tools
uv run python cleanup_db.py path/to/original.db path/to/cleaned.db
```

### Technical Implementation

The tool uses `sqlite-minutils` for efficient database interaction. It processes rows in batches to minimize memory overhead and preserves the original database schema, including all necessary metadata columns used by the Rust application.
