# Turso & SQLite Filtering Learnings

This document summarizes the technical approach and learnings from implementing a global filtering system in `transcript-explorer-rs`.

## In-Memory vs. Database-Level Filtering

Initially, we explored database-level filtering using Turso's `LIKE` and `GLOB` operators. However, for a high-performance TUI, we settled on **In-Memory Filtering** for several reasons:

1.  **Latency**: Immediate feedback as the user types a search query or adjusts a range filter.
2.  **Complexity**: Complex boolean logic (AND/OR/NOT) is easier to implement in Rust code than dynamic SQL generation when dealing with multi-field filters.
3.  **Consistency**: In-memory filters can easily be applied to search results (vector similarity) that are already in memory, ensuring a unified view.

## Wildcard Handling (`GLOB` style)

Users often expect `GLOB`-style wildcards (`*`, `?`). We used the `wildmatch` crate to provide case-insensitive wildcard matching that behaves like SQLite's `GLOB` but operates in memory.

```rust
// In-memory wildcard matching logic
let pattern_low = pattern.to_lowercase();
let val_low = val.to_lowercase();
WildMatch::new(&pattern_low).matches(&val_low)
```

## Statistical Confidence for Filter Ranges

To help users select meaningful ranges for numeric filters (like `cost` or `tokens`), we implemented a statistical preview:

*   **Sampling**: Instead of calculating exact stats on every refresh for large datasets, we use a sampled subset of the data.
*   **Beyond Mean/StdDev**: Standard deviation is sensitive to outliers. We included **MAD (Median Absolute Deviation)** and **Percentiles (5th and 95th)** to give a more robust "centrality" view, especially for token counts which vary wildly.

## Boolean Logic Structure

We used a recursive `enum` for the `Filter` structure, allowing for deep nesting:

```rust
pub enum Filter {
    Range { field: String, min: f64, max: f64 },
    Match { field: String, pattern: String },
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),
}
```

This makes the `matches()` implementation a clean recursive walk across the filter tree.
