use crate::db::{Database, SimilarResult, TranscriptListItem, TranscriptRow};

/// Which view is currently active.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    List,
    Detail,
    Similar,
}

/// Which tab is selected in the detail view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Summary,
    Transcript,
    Timestamps,
}

impl DetailTab {
    pub fn next(self) -> Self {
        match self {
            Self::Summary => Self::Transcript,
            Self::Transcript => Self::Timestamps,
            Self::Timestamps => Self::Summary,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Summary => Self::Timestamps,
            Self::Transcript => Self::Summary,
            Self::Timestamps => Self::Transcript,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Transcript => "Transcript",
            Self::Timestamps => "Timestamps",
        }
    }

    pub const ALL: [DetailTab; 3] = [Self::Summary, Self::Transcript, Self::Timestamps];
}

/// Input mode for the filter bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// A group of consecutive identical entries.
#[derive(Debug, Clone)]
pub struct TranscriptGroup {
    pub items: Vec<TranscriptListItem>,
    pub expanded: bool,
}

/// A group of consecutive identical similarity results.
#[derive(Debug, Clone)]
pub struct SimilarGroup {
    pub items: Vec<SimilarResult>,
    pub expanded: bool,
}

pub const LIST_OVERHEAD: u16 = 9;

/// Main application state.
pub struct App {
    pub db: Database,
    pub should_quit: bool,
    pub view: View,
    pub show_help: bool,

    // In-memory cache
    pub all_items: Vec<TranscriptListItem>,
    pub filtered_indices: Vec<usize>,

    // Grouped items for the list display
    pub grouped_items: Vec<TranscriptGroup>,
    
    // List view state
    pub list_items: Vec<TranscriptGroup>, // Current visible page of groups
    pub list_selected: usize,              // Index within visible page
    pub list_offset: usize,                // Offset into grouped_items
    pub page_size: usize,

    pub filter: String,
    pub input_mode: InputMode,

    // Detail view state
    pub detail: Option<TranscriptRow>,
    pub detail_tab: DetailTab,
    pub detail_scroll: u16,

    // Similar view state
    pub similar_results: Vec<SimilarResult>,
    pub grouped_similar_results: Vec<SimilarGroup>,
    pub similar_selected: usize,
    pub similar_source_id: i64,
    pub similar_source_preview: String,

    // Status message
    pub status_msg: String,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            should_quit: false,
            view: View::List,
            show_help: false,

            all_items: Vec::new(),
            filtered_indices: Vec::new(),
            grouped_items: Vec::new(),

            list_items: Vec::new(),
            list_selected: 0,
            list_offset: 0,
            page_size: 20, // Initial default, will be updated on first render/resize

            filter: String::new(),
            input_mode: InputMode::Normal,

            detail: None,
            detail_tab: DetailTab::Summary,
            detail_scroll: 0,

            similar_results: Vec::new(),
            grouped_similar_results: Vec::new(),
            similar_selected: 0,
            similar_source_id: 0,
            similar_source_preview: String::new(),

            status_msg: "Loading database...".to_string(),
        }
    }

    /// Initial data load.
    pub async fn init(&mut self) -> turso::Result<()> {
        self.all_items = self.db.list_all_transcripts().await?;
        self.apply_filter();
        self.status_msg = format!("{} transcripts loaded", self.all_items.len());
        Ok(())
    }

    /// Update the current page of visible items based on offset.
    pub fn update_list_page(&mut self) {
        let start = self.list_offset;
        let end = (start + self.page_size).min(self.grouped_items.len());
        self.list_items = self.grouped_items[start..end].to_vec();
    }

    /// Update page size based on terminal height.
    pub fn update_page_size(&mut self, terminal_height: u16) {
        let new_size = terminal_height.saturating_sub(LIST_OVERHEAD) as usize;
        self.page_size = new_size.max(1);
        self.update_list_page();
    }

    /// Move selection down in the list.
    pub fn list_next(&mut self) {
        if self.list_items.is_empty() {
            return;
        }
        if self.list_selected + 1 < self.list_items.len() {
            self.list_selected += 1;
        } else {
            // Next page
            let new_offset = self.list_offset + self.page_size;
            if new_offset < self.grouped_items.len() {
                self.list_offset = new_offset;
                self.list_selected = 0;
                self.update_list_page();
            }
        }
    }

    /// Move selection up in the list.
    pub fn list_prev(&mut self) {
        if self.list_selected > 0 {
            self.list_selected -= 1;
        } else if self.list_offset > 0 {
            // Prev page
            self.list_offset = self.list_offset.saturating_sub(self.page_size);
            self.update_list_page();
            self.list_selected = self.list_items.len().saturating_sub(1);
        }
    }

    pub fn list_page_down(&mut self) {
        let new_offset = self.list_offset + self.page_size;
        if new_offset < self.grouped_items.len() {
            self.list_offset = new_offset;
            self.update_list_page();
            self.list_selected = 0;
        } else {
            // Go to end
            let last_page_start = (self.grouped_items.len().saturating_sub(1) / self.page_size) * self.page_size;
            self.list_offset = last_page_start;
            self.update_list_page();
            self.list_selected = self.list_items.len().saturating_sub(1);
        }
    }

    pub fn list_page_up(&mut self) {
        if self.list_offset > 0 {
            self.list_offset = self.list_offset.saturating_sub(self.page_size);
            self.update_list_page();
            self.list_selected = 0;
        } else {
            self.list_selected = 0;
        }
    }

    /// Open the detail view for the currently selected item.
    pub async fn open_detail(&mut self) -> turso::Result<()> {
        match self.view {
            View::List => {
                if let Some(group) = self.list_items.get(self.list_selected) {
                    if let Some(item) = group.items.first() {
                        let id = item.identifier;
                        if let Some(row) = self.db.get_transcript(id).await? {
                            self.detail = Some(row);
                            self.detail_tab = DetailTab::Summary;
                            self.detail_scroll = 0;
                            self.view = View::Detail;
                        }
                    }
                }
            }
            View::Similar => {
                if let Some(group) = self.grouped_similar_results.get(self.similar_selected) {
                    if let Some(item) = group.items.first() {
                        let id = item.identifier;
                        if let Some(row) = self.db.get_transcript(id).await? {
                            self.detail = Some(row);
                            self.detail_tab = DetailTab::Summary;
                            self.detail_scroll = 0;
                            self.view = View::Detail;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Toggle expansion of the currently selected group.
    pub fn toggle_expand(&mut self) {
        match self.view {
            View::List => {
                if let Some(group) = self.list_items.get_mut(self.list_selected) {
                    group.expanded = !group.expanded;
                    // Sync back to grouped_items
                    if let Some(g) = self.grouped_items.get_mut(self.list_offset + self.list_selected) {
                        g.expanded = group.expanded;
                    }
                }
            }
            View::Similar => {
                if let Some(group) = self.grouped_similar_results.get_mut(self.similar_selected) {
                    group.expanded = !group.expanded;
                }
            }
            _ => {}
        }
    }

    /// Open detail for a specific identifier (used from similar view).
    pub async fn open_detail_by_id(&mut self, id: i64) -> turso::Result<()> {
        if let Some(row) = self.db.get_transcript(id).await? {
            self.detail = Some(row);
            self.detail_tab = DetailTab::Summary;
            self.detail_scroll = 0;
            self.view = View::Detail;
        }
        Ok(())
    }

    /// Open the similar view for the currently viewed/selected transcript.
    pub async fn open_similar(&mut self) -> turso::Result<()> {
        let (id, preview) = match &self.view {
            View::Detail => {
                if let Some(ref d) = self.detail {
                    (d.identifier, d.summary.chars().take(80).collect::<String>())
                } else {
                    return Ok(());
                }
            }
            View::List => {
                if let Some(group) = self.list_items.get(self.list_selected) {
                    if let Some(item) = group.items.first() {
                        if !item.has_embedding {
                            self.status_msg = "No embedding for this entry".to_string();
                            return Ok(());
                        }
                        (item.identifier, item.summary_preview.clone())
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        };

        self.status_msg = "Computing similarities...".to_string();
        self.similar_source_id = id;
        self.similar_source_preview = preview;
        
        self.similar_results = self.db.find_similar(id, 20).await?;
        
        // Grouping logic for similarity results
        self.grouped_similar_results.clear();
        if !self.similar_results.is_empty() {
            let mut current_group: Vec<SimilarResult> = Vec::new();
            for item in &self.similar_results {
                if let Some(last) = current_group.last() {
                    if last.summary_preview == item.summary_preview {
                        current_group.push(item.clone());
                    } else {
                        self.grouped_similar_results.push(SimilarGroup {
                            items: current_group,
                            expanded: false,
                        });
                        current_group = vec![item.clone()];
                    }
                } else {
                    current_group.push(item.clone());
                }
            }
            if !current_group.is_empty() {
                self.grouped_similar_results.push(SimilarGroup {
                    items: current_group,
                    expanded: false,
                });
            }
        }
        
        self.similar_selected = 0;
        self.view = View::Similar;
        self.status_msg = format!("Found {} similar transcripts ({} groups)", self.similar_results.len(), self.grouped_similar_results.len());
        Ok(())
    }

    /// Apply filter and reset list.
    pub fn apply_filter(&mut self) {
        let filter = self.filter.to_lowercase();
        self.filtered_indices.clear();
        
        if filter.is_empty() {
            self.filtered_indices = (0..self.all_items.len()).collect();
        } else {
            for (i, item) in self.all_items.iter().enumerate() {
                if item.summary_preview.to_lowercase().contains(&filter) 
                   || item.host.to_lowercase().contains(&filter)
                   || item.original_source_link.to_lowercase().contains(&filter) {
                    self.filtered_indices.push(i);
                }
            }
        }
        
        // Grouping logic
        self.grouped_items.clear();
        if !self.filtered_indices.is_empty() {
            let mut current_group: Vec<TranscriptListItem> = Vec::new();
            
            for &idx in &self.filtered_indices {
                let item = &self.all_items[idx];
                if let Some(last) = current_group.last() {
                    // Group if summary is near-identical (just heuristic)
                    if last.summary_preview == item.summary_preview {
                        current_group.push(item.clone());
                    } else {
                        self.grouped_items.push(TranscriptGroup {
                            items: current_group,
                            expanded: false,
                        });
                        current_group = vec![item.clone()];
                    }
                } else {
                    current_group.push(item.clone());
                }
            }
            if !current_group.is_empty() {
                self.grouped_items.push(TranscriptGroup {
                    items: current_group,
                    expanded: false,
                });
            }
        }

        self.list_offset = 0;
        self.list_selected = 0;
        self.update_list_page();
        
        self.status_msg = format!(
            "{} groups found for \"{}\"",
            self.grouped_items.len(),
            if self.filter.is_empty() { "all" } else { &self.filter }
        );
    }

    pub fn scroll_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    pub fn scroll_page_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(20);
    }

    pub fn scroll_page_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(20);
    }
}

/// Helper to get a meaningful title prefix by skipping generic leads.
pub fn get_display_title(preview: &str) -> String {
    let lines: Vec<&str> = preview.lines().collect();
    if lines.is_empty() {
        return "No summary".to_string();
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Heuristic: skip lines that are just generic titles
        let lower = trimmed.to_lowercase();
        if lower.starts_with("**abstract:**") {
            // Find the first sentence after the bold
            if let Some(idx) = trimmed.find("**:") {
                let rest = &trimmed[idx + 3..].trim();
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
            continue;
        }
        
        if lower.starts_with("okay, here is the abstract") {
            continue;
        }

        return trimmed.to_string();
    }

    preview.lines().next().unwrap_or("").to_string()
}
