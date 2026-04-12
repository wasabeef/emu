use super::AppState;

/// Represents a single log entry from device output.
/// Used for displaying device logs in the UI.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp when the log was captured (HH:MM:SS format)
    pub timestamp: String,
    /// Log level (DEBUG, INFO, WARN, ERROR, etc)
    pub level: String,
    /// The actual log message content
    pub message: String,
}

impl AppState {
    /// Adds a new log entry to the device log queue.
    /// Automatically manages log rotation when max_log_entries is exceeded.
    /// Handles auto-scrolling if enabled and user hasn't manually scrolled.
    pub fn add_log(&mut self, level: String, message: String) {
        use chrono::Local;

        let timestamp = Local::now().format("%H:%M:%S").to_string();
        self.device_logs.push_back(LogEntry {
            timestamp,
            level,
            message,
        });

        while self.device_logs.len() > self.max_log_entries {
            self.device_logs.pop_front();
        }

        if self.auto_scroll_logs && !self.manually_scrolled {
            let total_logs = self.device_logs.len();
            self.log_scroll_offset = total_logs.saturating_sub(1);
        }
    }

    /// Clears all device logs from memory.
    pub fn clear_logs(&mut self) {
        self.device_logs.clear();
    }

    /// Scrolls logs up by one line.
    /// Sets manually_scrolled flag to disable auto-scroll.
    pub fn scroll_logs_up(&mut self) {
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset -= 1;
            self.manually_scrolled = true;
        }
    }

    /// Scrolls logs down by one line.
    /// Sets manually_scrolled flag to disable auto-scroll.
    pub fn scroll_logs_down(&mut self) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        if self.log_scroll_offset < max_offset {
            self.log_scroll_offset += 1;
            self.manually_scrolled = true;
        }
    }

    /// Scrolls logs up by a full page.
    pub fn scroll_logs_page_up(&mut self, page_size: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(page_size);
        self.manually_scrolled = true;
    }

    /// Scrolls logs down by a full page.
    pub fn scroll_logs_page_down(&mut self, page_size: usize) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        self.log_scroll_offset = (self.log_scroll_offset + page_size).min(max_offset);
        self.manually_scrolled = true;
    }

    /// Resets log scroll position to the top.
    pub fn reset_log_scroll(&mut self) {
        self.log_scroll_offset = 0;
    }

    /// Sets or clears the log level filter.
    /// Resets scroll position when filter changes.
    pub fn toggle_log_filter(&mut self, level: Option<String>) {
        self.log_filter_level = level;
        self.reset_log_scroll();
    }

    /// Toggles fullscreen log display mode.
    pub fn toggle_fullscreen_logs(&mut self) {
        self.fullscreen_logs = !self.fullscreen_logs;
    }

    /// Toggles automatic log scrolling.
    /// When enabled, logs automatically scroll to show new entries.
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll_logs = !self.auto_scroll_logs;
        if self.auto_scroll_logs {
            self.manually_scrolled = false;
        }
    }

    /// Scrolls logs to the very top.
    /// Sets manually_scrolled to prevent auto-scroll.
    pub fn scroll_logs_to_top(&mut self) {
        self.log_scroll_offset = 0;
        self.manually_scrolled = true;
    }

    /// Scrolls logs to the very bottom.
    /// Clears manually_scrolled to re-enable auto-scroll.
    pub fn scroll_logs_to_bottom(&mut self) {
        let total_logs = self.get_filtered_logs().len();
        self.log_scroll_offset = total_logs.saturating_sub(1);
        self.manually_scrolled = false;
    }

    /// Scrolls logs up by half a page.
    pub fn scroll_logs_half_page_up(&mut self, page_size: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(page_size / 2);
        self.manually_scrolled = true;
    }

    /// Scrolls logs down by half a page.
    pub fn scroll_logs_half_page_down(&mut self, page_size: usize) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        self.log_scroll_offset = (self.log_scroll_offset + page_size / 2).min(max_offset);
        self.manually_scrolled = true;
    }

    /// Returns filtered log entries based on current log level filter.
    /// If no filter is set, returns all logs.
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        if let Some(ref filter_level) = self.log_filter_level {
            self.device_logs
                .iter()
                .filter(|entry| entry.level == *filter_level)
                .collect()
        } else {
            self.device_logs.iter().collect()
        }
    }

    /// Updates the status of a specific Android device without full refresh.
    /// Used for optimized device state updates during start/stop operations.
    pub fn update_single_android_device_status(&mut self, device_name: &str, is_running: bool) {
        if let Some(device) = self
            .android_devices
            .iter_mut()
            .find(|d| d.name == device_name)
        {
            device.is_running = is_running;

            if let Some(ref mut cached) = self.cached_device_details {
                if cached.identifier == device_name {
                    cached.status = if is_running {
                        "Running".to_string()
                    } else {
                        "Stopped".to_string()
                    };
                }
            }
        }
    }

    /// Updates the status of a specific iOS device without full refresh.
    /// Used for optimized device state updates during start/stop operations.
    pub fn update_single_ios_device_status(&mut self, device_udid: &str, is_running: bool) {
        if let Some(device) = self.ios_devices.iter_mut().find(|d| d.udid == device_udid) {
            device.is_running = is_running;

            if let Some(ref mut cached) = self.cached_device_details {
                if cached.identifier == device_udid {
                    cached.status = if is_running {
                        "Booted".to_string()
                    } else {
                        "Shutdown".to_string()
                    };
                }
            }
        }
    }
}
