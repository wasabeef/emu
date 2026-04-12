use crate::models::{ApiLevel, InstallProgress};

/// State for API level management dialog.
#[derive(Debug, Clone)]
pub struct ApiLevelManagementState {
    /// List of available API levels
    pub api_levels: Vec<ApiLevel>,
    /// Currently selected API level index
    pub selected_index: usize,
    /// Whether data is being loaded
    pub is_loading: bool,
    /// Current installation progress
    pub install_progress: Option<InstallProgress>,
    /// Package ID being installed/uninstalled
    pub installing_package: Option<String>,
    /// Error message to display
    pub error_message: Option<String>,
    /// Scroll offset for the API level list
    pub scroll_offset: usize,
}

impl Default for ApiLevelManagementState {
    fn default() -> Self {
        Self {
            api_levels: Vec::new(),
            selected_index: 0,
            is_loading: true,
            install_progress: None,
            installing_package: None,
            error_message: None,
            scroll_offset: 0,
        }
    }
}

impl ApiLevelManagementState {
    /// Creates a new API level management state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves selection up.
    pub fn move_up(&mut self) {
        if !self.api_levels.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.api_levels.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Moves selection down.
    pub fn move_down(&mut self) {
        if !self.api_levels.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.api_levels.len();
        }
    }

    /// Returns true if an install or uninstall operation is currently in progress.
    pub fn is_busy(&self) -> bool {
        self.install_progress.is_some() || self.installing_package.is_some()
    }

    /// Gets the currently selected API level.
    pub fn get_selected_api_level(&self) -> Option<&ApiLevel> {
        self.api_levels.get(self.selected_index)
    }

    /// Calculates scroll offset to keep selected item visible.
    pub fn get_scroll_offset(&self, available_height: usize) -> usize {
        if self.api_levels.is_empty() || available_height == 0 {
            return 0;
        }

        let total_items = self.api_levels.len();
        let selected = self.selected_index;
        let preferred_offset = selected.saturating_sub(available_height / 2);
        let max_offset = total_items.saturating_sub(available_height);

        preferred_offset.min(max_offset)
    }
}
