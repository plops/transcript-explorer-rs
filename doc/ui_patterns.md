# UI Patterns & Keybindings

This document describes the user interface design philosophy and interactive patterns.

## Interface Design

The UI is divided into several areas using a hierarchical layout:
- **Navigation Layout**: The top level typically has a header, a main content area, and a status/instruction bar at the bottom.
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

## Clipboard Support

On Linux, the application attempts to integrate with the system clipboard using the following priority:
1. `xclip -selection clipboard` (X11)
2. `wl-copy` (Wayland)
3. Fallback: Display the link in the status bar if no clipboard utility is found.
