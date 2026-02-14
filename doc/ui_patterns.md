# UI Patterns & Keybindings

This document describes the user interface design philosophy and interactive patterns.

## Interface Design

The UI is divided into several areas using a hierarchical layout:
- **Navigation Layout**: The top level typically has a header, a main content area, and a status/instruction bar at the bottom. Layouts automatically scale vertically to utilize the full terminal height.
- **Filtering**: Type-to-search is implemented as a semi-modal state (`InputMode::Editing`). When active, the cursor is focused in the search bar.
- **Detail View Architecture**: Metadata is kept in a fixed-size top block, while the content (Summary/Transcript/Timestamps) uses a scrollable paragraph that can be toggled via tabs.
- **Smart Grouping**: Consecutive items with identical summaries are collapsed into a single group to reduce noise. These groups show a `[+N]` counter and can be expanded using the `Space` key.
- **Title Heuristics**: The list view applies a heuristic to skip generic text like "**Abstract:**" or "Okay, here is the..." and instead displays the first meaningful line of the summary.

## View States

### List View
Focused on scannability and high density.
- **Indicators**: A `●` indicates the presence of a vector embedding; `○` indicates absence.
- **Group Markers**: `[+N]` indicates a collapsed group of duplicate entries. `[-]` indicates an expanded group.
- **Scrolling**: Supports fine-grained selection with `j`/`k` and large jumps with `PgUp`/`PgDn`.
- **Live Search**: Filtering updates instantaneously on every keystroke, utilizing the in-memory metadata cache.

### Detail View
Focused on reading.
- **Scrolling**: Supports both single-line scroll (`j`/`k`) and page-level scroll (`PgUp`/`PgDn`).
- **Tabbed Content**: Allows switching between different data representations (Summary vs raw Transcript) without losing context.

### Similar View
Focused on discovery.
- **Color Coding**: Similarity scores are color-coded (Green > 0.9, Yellow > 0.8, Red otherwise) to provide immediate visual feedback on the quality of matches.
- **Unified Preview**: Uses the same detail pane as the main list to show source links, metadata, and **formatted markdown summaries** (using `tui-markdown`).

### Filters View
Focused on precision browsing.
- **Statistical Reference**: Displays mean, median, standard deviation, and percentiles for numeric fields to help users pick valid ranges.
- **Interactive Builder**: A multi-step state machine guides users through selecting a field, entering min/max values, or match patterns.
- **Boolean Composition**: Active filters are displayed as a hierarchical tree showing how they are combined (e.g., "ALL OF").

## Interactive Prompts

Certain actions (like encryption/decryption) require user interaction via standard CLI prompts:
- **Password Entry**: Implements **masked feedback**. As characters are typed, a `*` is echoed to the terminal. This provides visual confirmation of input without exposing sensitive information.
- **Input Interaction**: Supports backspace for correction and `Ctrl+C` for graceful cancellation.

## Keybindings Reference

### Global / List
- `f`: Enter **Filters View**.
- `/`: Enter search mode.
- `s`: Find similar items (from selection).
- `Space`: Expand/collapse group.
- `Enter`: Open detail view.
- `q` / `Esc`: Exit or back.

### Filtering (Advanced)
- `a`: Add new filter.
- `d`: Delete/Clear all filters.
- `Esc`: Cancel builder or return to List.

## Clipboard Support

On Linux, the application attempts to integrate with the system clipboard using the following priority:
1. `xclip -selection clipboard` (X11)
2. `wl-copy` (Wayland)
3. Fallback: Display the link in the status bar if no clipboard utility is found.
