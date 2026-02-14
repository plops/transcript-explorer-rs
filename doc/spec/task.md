# Transcript Explorer RS — Task Tracking (History)

## Phase 1: Planning & Research
- [x] Research Turso Rust API (v0.4)
- [x] Research Ratatui 0.30 architecture
- [x] Write design specification
- [x] Inspect actual database schema and embedding format (`f32` Little-Endian)

## Phase 2: Core Implementation
- [x] Scaffold project and dependencies
- [x] Implement Database layer (`src/db.rs`) with Turso async API
- [x] Implement App state machine (`src/app.rs`) with pagination
- [x] Implement List View UI
- [x] Implement Detail View UI (tabbed)
- [x] Implement Vector Similarity View UI
- [x] Add Help popup and status bar notifications

## Phase 3: Verification & Polish
- [x] Fix compilation errors related to Turso `Rows` API (non-stream)
- [x] Verify application startup and data loading (13,000+ entries)
- [x] Implement clipboard integration (yank link)
- [x] Final documentation (Root README, `doc/`, `doc/spec/`)

## Phase 4: UI Refinements & Better Search
- [x] Implement PageUp/Down in list view
- [x] Enable live-search (update on every keystroke)
- [x] Implement title heuristic (skip "Abstract:", etc.)
- [x] Implement collapsing of consecutive duplicates
- [x] Add toggle for uncollapsing groups

## Phase 5: Dynamic Scaling & Optimization
- [x] Implement dynamic `page_size` based on terminal window height
- [x] Implement `Resize` event handling to update layout on-the-fly
- [x] Update documentation to reflect dynamic TUI behavior

## Phase 6: Database Preparation & Maintenance Tools
- [x] Initialize `tools/` directory with `astral uv` project
- [x] Implement `cleanup_db.py` to strip transcripts and large text
- [x] Implement embedding truncation to 768 dimensions (Matryoshka optimization)
- [x] Add filtering for error entries in summaries
- [x] Add `doc/database_maintenance.md` and reference in root `README.md`

## Phase 7: Compression and Encryption
- [x] Integrate `age` for encryption and `brotli` for compression
- [x] Add CLI subcommands (`encrypt`, `decrypt`, `run`)
- [x] Verify round-trip encryption/decryption

## Phase 8: Performance Optimization
- [x] Implement multithreaded compression (initial implementation)
- [x] Add performance metrics to CLI output

## Phase 9: Reliability & Robustness
- [x] Switch to single-stream Brotli processing for maximum compatibility
- [x] Fix database panic caused by multi-stream data loss
- [x] Remove `rayon` dependency to simplify the codebase
