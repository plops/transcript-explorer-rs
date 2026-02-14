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
