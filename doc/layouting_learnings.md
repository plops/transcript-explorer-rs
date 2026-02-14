# Ratatui TUI Layouting Learnings

This document summarizes the approach taken to implement a responsive, dual-view preview system using Ratatui.

## Responsive Layouts based on Terminal State

TUIs are often constrained by terminal height. We implemented a **Responsive Layout Heuristic** where the UI structure changes based on the available area:

```rust
let preview_height = if area.height > 60 {
    Constraint::Percentage(50) // High-res mode: Large preview
} else {
    Constraint::Length(10)      // Low-res mode: Compact preview
};
```

This ensures that on small terminal windows, the list/navigation space is preserved, while on large windows, the user gets a comprehensive "full page" preview without having to switch views.

## Component Reusability

To keep the codebase dry, we extracted the preview logic into a dedicated module `src/ui/preview.rs`. This allowed us to:

1.  **Shared Render Logic**: Use the exact SAME layout and styling for previews in both the main `List` view and the vector `Similar` results view.
2.  **Mocking for Consistency**: In the similarity view, we convert `SimilarResult` objects into temporary `TranscriptListItem`-like structures so `render_preview` can handle them uniformly.

## Nested Split Constraints

Complex UIs like the Filter Configuration screen benefit from a hierarchy of `Layout` splits:

*   **Vertical Split**: Separates header, main content, and footer status bar.
*   **Horizontal Split (within content)**: Separates statistical reference data (left) from the active filter tree (right).

Using `Constraint::Min(0)` or `Constraint::Percentage` helps in creating layouts that don't crash when terminal windows are resized to extremely small dimensions.

## Style Tokens

Consistent styling (e.g., `Color::DarkGray` for labels, `Color::White` for values) was maintained across modules by using shared components and following a consistent pattern in `render_preview`.
