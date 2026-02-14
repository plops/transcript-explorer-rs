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

/// Main application state.
pub struct App {
    pub db: Database,
    pub should_quit: bool,
    pub view: View,
    pub show_help: bool,

    // In-memory cache
    pub all_items: Vec<TranscriptListItem>,
    pub filtered_indices: Vec<usize>,

    // List view state
    pub list_items: Vec<TranscriptListItem>, // Current visible page
    pub list_selected: usize,         // Index within visible page
    pub list_offset: usize,           // Offset into filtered_indices
    pub page_size: usize,

    pub filter: String,
    pub input_mode: InputMode,

    // Detail view state
    pub detail: Option<TranscriptRow>,
    pub detail_tab: DetailTab,
    pub detail_scroll: u16,

    // Similar view state
    pub similar_results: Vec<SimilarResult>,
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

            list_items: Vec::new(),
            list_selected: 0,
            list_offset: 0,
            page_size: 100,

            filter: String::new(),
            input_mode: InputMode::Normal,

            detail: None,
            detail_tab: DetailTab::Summary,
            detail_scroll: 0,

            similar_results: Vec::new(),
            similar_selected: 0,
            similar_source_id: 0,
            similar_source_preview: String::new(),

            status_msg: "Loading database...".to_string(),
        }
    }

    /// Initial data load.
    pub async fn init(&mut self) -> turso::Result<()> {
        self.all_items = self.db.list_all_transcripts().await?;
        self.filtered_indices = (0..self.all_items.len()).collect();
        self.update_list_page();
        self.status_msg = format!("{} transcripts loaded", self.all_items.len());
        Ok(())
    }

    /// Update the current page of visible items based on offset.
    pub fn update_list_page(&mut self) {
        let start = self.list_offset;
        let end = (start + self.page_size).min(self.filtered_indices.len());
        self.list_items = self.filtered_indices[start..end]
            .iter()
            .map(|&i| self.all_items[i].clone())
            .collect();
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
            if new_offset < self.filtered_indices.len() {
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

    /// Open the detail view for the currently selected item.
    pub async fn open_detail(&mut self) -> turso::Result<()> {
        if let Some(item) = self.list_items.get(self.list_selected) {
            let id = item.identifier;
            if let Some(row) = self.db.get_transcript(id).await? {
                self.detail = Some(row);
                self.detail_tab = DetailTab::Summary;
                self.detail_scroll = 0;
                self.view = View::Detail;
            }
        }
        Ok(())
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
                if let Some(item) = self.list_items.get(self.list_selected) {
                    if !item.has_embedding {
                        self.status_msg = "No embedding for this entry".to_string();
                        return Ok(());
                    }
                    (item.identifier, item.summary_preview.clone())
                } else {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        };

        self.status_msg = "Computing similarities...".to_string();
        self.similar_source_id = id;
        self.similar_source_preview = preview;
        
        // This will now use vector_slice(..., 0, 768) and handles the dimension mismatch.
        self.similar_results = self.db.find_similar(id, 20).await?;
        
        self.similar_selected = 0;
        self.view = View::Similar;
        self.status_msg = format!("Found {} similar transcripts", self.similar_results.len());
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
        
        self.list_offset = 0;
        self.list_selected = 0;
        self.update_list_page();
        
        self.status_msg = format!(
            "{} results for \"{}\"",
            self.filtered_indices.len(),
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
