//! Application state management module for the Emu TUI application.
//!
//! This module contains all state management structures and logic for the terminal user interface.
//! It manages device lists, UI panels, modal dialogs, notifications, logs, and device creation forms.
//! The state is designed to be thread-safe with async access patterns using Arc<RwLock<>> for
//! concurrent operations.
//!
//! # Architecture
//!
//! The module follows a centralized state pattern where `AppState` contains all application state.
//! State updates are performed through methods that ensure consistency and thread safety.
//! Background operations use async tasks with proper synchronization through RwLock.

mod api_levels;
mod cache;
mod details;
mod forms;
mod navigation;
mod notifications;

use crate::constants::{
    timeouts::{DEFAULT_AUTO_REFRESH_INTERVAL, FAST_REFRESH_INTERVAL_SECS},
    MAX_LOG_ENTRIES, MAX_NOTIFICATIONS,
};
use crate::models::{AndroidDevice, IosDevice};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use self::api_levels::ApiLevelManagementState;
pub use self::cache::DeviceCache;
pub use self::forms::{CreateDeviceField, CreateDeviceForm};
pub use self::notifications::{Notification, NotificationType};
pub use crate::models::DeviceDetails;

/// Represents the two main device panels in the UI.
/// The application displays Android and iOS devices in separate panels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    /// Android device panel showing AVDs (Android Virtual Devices)
    Android,
    /// iOS device panel showing simulators (macOS only)
    Ios,
}

impl Panel {
    /// Toggles between Android and iOS panels.
    /// Returns the opposite panel from the current one.
    pub fn toggle(self) -> Self {
        match self {
            Self::Android => Self::Ios,
            Self::Ios => Self::Android,
        }
    }
}

/// Represents which UI panel currently has focus.
/// Used for keyboard navigation between device list and log area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    /// The device list panel (Android or iOS) has focus
    DeviceList,
    /// The log area panel has focus
    LogArea,
}

/// Application modes representing different UI states.
/// Each mode corresponds to a different screen or modal dialog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Normal mode - viewing device lists
    Normal,
    /// Device creation modal is active
    CreateDevice,
    /// Delete confirmation dialog is active
    ConfirmDelete,
    /// Wipe data confirmation dialog is active
    ConfirmWipe,
    /// API level management dialog is active
    ManageApiLevels,
    /// Help screen is displayed
    Help,
}

/// Data for the delete confirmation dialog.
/// Stores information about the device to be deleted.
#[derive(Debug, Clone)]
pub struct ConfirmDeleteDialog {
    /// Display name of the device
    pub device_name: String,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub device_identifier: String,
    /// Platform of the device being deleted
    pub platform: Panel,
}

/// Data for the wipe data confirmation dialog.
/// Stores information about the device whose data will be wiped.
#[derive(Debug, Clone)]
pub struct ConfirmWipeDialog {
    /// Display name of the device
    pub device_name: String,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub device_identifier: String,
    /// Platform of the device being wiped
    pub platform: Panel,
}

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

/// Main application state containing all UI and data state.
/// This is the central state store for the entire application.
pub struct AppState {
    /// Currently active device panel (Android or iOS)
    pub active_panel: Panel,
    /// Current application mode/screen
    pub mode: Mode,
    /// List of Android devices (AVDs)
    pub android_devices: Vec<AndroidDevice>,
    /// List of iOS devices (simulators)
    pub ios_devices: Vec<IosDevice>,
    /// Index of selected Android device
    pub selected_android: usize,
    /// Index of selected iOS device
    pub selected_ios: usize,
    /// Flag indicating device list is being loaded
    pub is_loading: bool,
    /// Queue of device log entries (limited by max_log_entries)
    pub device_logs: VecDeque<LogEntry>,
    /// Maximum number of log entries to keep in memory
    pub max_log_entries: usize,
    /// Form state for device creation
    pub create_device_form: CreateDeviceForm,
    /// Active delete confirmation dialog data
    pub confirm_delete_dialog: Option<ConfirmDeleteDialog>,
    /// Active wipe confirmation dialog data
    pub confirm_wipe_dialog: Option<ConfirmWipeDialog>,
    /// Queue of user notifications
    pub notifications: VecDeque<Notification>,
    /// Maximum number of notifications to display
    pub max_notifications: usize,
    /// Current scroll position in the log view
    pub log_scroll_offset: usize,
    /// Optional log level filter (DEBUG/INFO/WARN/ERROR)
    pub log_filter_level: Option<String>,
    /// Timestamp of last device list refresh
    pub last_refresh: std::time::Instant,
    /// Interval for automatic device list refresh
    pub auto_refresh_interval: std::time::Duration,
    /// Name of device that was just started (triggers faster refresh)
    pub pending_device_start: Option<String>,
    /// Shared cache for device creation options
    pub device_cache: Arc<RwLock<DeviceCache>>,
    /// Current device operation status message
    pub device_operation_status: Option<String>,
    /// Which panel currently has keyboard focus
    pub focused_panel: FocusedPanel,
    /// Flag for fullscreen log display mode
    pub fullscreen_logs: bool,
    /// Flag for automatic log scrolling
    pub auto_scroll_logs: bool,
    /// Flag indicating user has manually scrolled logs
    pub manually_scrolled: bool,
    /// Device whose logs are currently being streamed
    pub current_log_device: Option<(Panel, String)>,
    /// Handle to the background log streaming task
    pub log_task_handle: Option<tokio::task::JoinHandle<()>>,
    /// Cached device details for the details panel
    pub cached_device_details: Option<DeviceDetails>,
    /// Scroll offset for Android device list
    pub android_scroll_offset: usize,
    /// Scroll offset for iOS device list
    pub ios_scroll_offset: usize,
    /// API level management dialog state (when dialog is open)
    pub api_level_management: Option<ApiLevelManagementState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_panel: Panel::Android,
            mode: Mode::Normal,
            android_devices: Vec::new(),
            ios_devices: Vec::new(),
            selected_android: 0,
            selected_ios: 0,
            is_loading: true, // Start in loading state
            device_logs: VecDeque::new(),
            max_log_entries: MAX_LOG_ENTRIES,
            create_device_form: CreateDeviceForm::default(),
            confirm_delete_dialog: None,
            confirm_wipe_dialog: None,
            notifications: VecDeque::new(),
            max_notifications: MAX_NOTIFICATIONS,
            log_scroll_offset: 0,
            log_filter_level: None,
            last_refresh: std::time::Instant::now(),
            auto_refresh_interval: DEFAULT_AUTO_REFRESH_INTERVAL, // 3-second refresh
            pending_device_start: None,
            device_cache: Arc::new(RwLock::new(DeviceCache::default())),
            device_operation_status: None,
            focused_panel: FocusedPanel::DeviceList,
            fullscreen_logs: false,
            auto_scroll_logs: true,
            manually_scrolled: false,
            current_log_device: None,
            log_task_handle: None,
            cached_device_details: None,
            android_scroll_offset: 0,
            ios_scroll_offset: 0,
            api_level_management: None,
        }
    }
}

impl AppState {
    /// Creates a new AppState with default values.
    pub fn new() -> Self {
        Self::default()
    }

    // --- Mode predicates ---

    /// Returns true if the app is in normal mode.
    pub fn is_normal_mode(&self) -> bool {
        self.mode == Mode::Normal
    }

    /// Returns true if the app is in device creation mode.
    pub fn is_create_mode(&self) -> bool {
        self.mode == Mode::CreateDevice
    }

    /// Returns true if the app is in help mode.
    pub fn is_help_mode(&self) -> bool {
        self.mode == Mode::Help
    }

    /// Returns true if the app is in confirm delete mode.
    pub fn is_confirm_delete_mode(&self) -> bool {
        self.mode == Mode::ConfirmDelete
    }

    /// Returns true if the app is in confirm wipe mode.
    pub fn is_confirm_wipe_mode(&self) -> bool {
        self.mode == Mode::ConfirmWipe
    }

    /// Returns true if the app is in API level management mode.
    pub fn is_api_level_mode(&self) -> bool {
        self.mode == Mode::ManageApiLevels
    }

    // --- Panel predicates ---

    /// Returns true if the Android panel is active.
    pub fn is_android_panel(&self) -> bool {
        self.active_panel == Panel::Android
    }

    /// Returns true if the iOS panel is active.
    pub fn is_ios_panel(&self) -> bool {
        self.active_panel == Panel::Ios
    }

    // --- Device accessors ---

    /// Returns the number of Android devices.
    pub fn android_device_count(&self) -> usize {
        self.android_devices.len()
    }

    /// Returns the number of iOS devices.
    pub fn ios_device_count(&self) -> usize {
        self.ios_devices.len()
    }

    /// Returns the currently selected Android device, if any.
    pub fn selected_android_device(&self) -> Option<&AndroidDevice> {
        self.android_devices.get(self.selected_android)
    }

    /// Returns the currently selected iOS device, if any.
    pub fn selected_ios_device(&self) -> Option<&IosDevice> {
        self.ios_devices.get(self.selected_ios)
    }

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

        // Auto-scroll to bottom if enabled and not manually scrolling
        if self.auto_scroll_logs && !self.manually_scrolled {
            let total_logs = self.device_logs.len();
            self.log_scroll_offset = total_logs.saturating_sub(1);
        }
    }

    /// Clears all device logs from memory.
    pub fn clear_logs(&mut self) {
        self.device_logs.clear();
    }

    /// Adds a notification to the queue.
    /// Automatically removes oldest notifications when max_notifications is exceeded.
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_back(notification);

        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
    }

    /// Adds a success notification with green color.
    pub fn add_success_notification(&mut self, message: String) {
        self.add_notification(Notification::success(message));
    }

    /// Adds an error notification with red color.
    pub fn add_error_notification(&mut self, message: String) {
        self.add_notification(Notification::error(message));
    }

    /// Adds a warning notification with yellow color.
    pub fn add_warning_notification(&mut self, message: String) {
        self.add_notification(Notification::warning(message));
    }

    /// Adds an info notification with blue color.
    pub fn add_info_notification(&mut self, message: String) {
        self.add_notification(Notification::info(message));
    }

    /// Removes notifications that have exceeded their auto-dismiss duration.
    pub fn dismiss_expired_notifications(&mut self) {
        self.notifications.retain(|n| !n.should_dismiss());
    }

    /// Clears all notifications from the queue.
    pub fn dismiss_all_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Removes a specific notification by index.
    pub fn dismiss_notification(&mut self, index: usize) {
        if index < self.notifications.len() {
            self.notifications.remove(index);
        }
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

    // Tab navigation is now handled directly in app/mod.rs for three-way cycling

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
        let logs: Vec<&LogEntry> = if let Some(ref filter_level) = self.log_filter_level {
            self.device_logs
                .iter()
                .filter(|entry| entry.level == *filter_level)
                .collect()
        } else {
            self.device_logs.iter().collect()
        };
        logs
    }

    /// Checks if device list should be automatically refreshed.
    /// Returns true if refresh interval elapsed or device start is pending.
    pub fn should_auto_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.auto_refresh_interval
            || self.pending_device_start.is_some()
    }

    /// Updates the last refresh timestamp to current time.
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = std::time::Instant::now();
    }

    /// Sets a device as pending start, triggering faster refresh.
    /// Reduces refresh interval to 1 second for quicker status updates.
    pub fn set_pending_device_start(&mut self, device_name: String) {
        self.pending_device_start = Some(device_name);
        // Refresh more frequently when device is starting
        self.auto_refresh_interval = std::time::Duration::from_secs(FAST_REFRESH_INTERVAL_SECS);
    }

    /// Clears pending device start and returns to normal refresh interval.
    pub fn clear_pending_device_start(&mut self) {
        self.pending_device_start = None;
        // Return to normal refresh interval
        self.auto_refresh_interval = DEFAULT_AUTO_REFRESH_INTERVAL;
    }

    /// Gets the name of device pending start, if any.
    pub fn get_pending_device_start(&self) -> Option<&String> {
        self.pending_device_start.as_ref()
    }

    /// Sets the current device operation status message.
    /// Used to display progress for long-running operations.
    pub fn set_device_operation_status(&mut self, status: String) {
        self.device_operation_status = Some(status);
    }

    /// Clears the device operation status message.
    pub fn clear_device_operation_status(&mut self) {
        self.device_operation_status = None;
    }

    /// Gets the current device operation status message.
    pub fn get_device_operation_status(&self) -> Option<&String> {
        self.device_operation_status.as_ref()
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

            // Update cached device details if they match this device
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

            // Update cached device details if they match this device
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

    /// Initiates background cache update if cache is stale or empty.
    /// Checks if update is needed and sets loading flags.
    /// This is an async operation that coordinates with background tasks.
    pub async fn start_background_cache_update(&mut self) {
        // Check if cache update is needed
        {
            let cache = self.device_cache.read().await;
            if !cache.is_stale() && !cache.android_device_types.is_empty() {
                return; // Cache is fresh and has data
            }
        }

        // Set loading flag
        {
            let mut cache = self.device_cache.write().await;
            if cache.is_loading {
                return; // Already loading
            }
            cache.is_loading = true;
        }

        self.create_device_form.is_loading_cache = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_toggle() {
        assert_eq!(Panel::Android.toggle(), Panel::Ios);
        assert_eq!(Panel::Ios.toggle(), Panel::Android);
    }

    #[test]
    fn test_api_level_management_state_new() {
        let state = ApiLevelManagementState::new();
        assert_eq!(state.selected_index, 0);
        assert!(state.api_levels.is_empty());
        assert!(state.is_loading);
        assert!(state.install_progress.is_none());
        assert!(state.installing_package.is_none());
        assert!(state.error_message.is_none());
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_notification_creation() {
        let notification = Notification {
            message: "Test message".to_string(),
            notification_type: NotificationType::Success,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: Some(std::time::Duration::from_secs(5)),
        };
        assert_eq!(notification.message, "Test message");
        assert!(notification.auto_dismiss_after.is_some());

        let persistent = Notification {
            message: "Error".to_string(),
            notification_type: NotificationType::Error,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: None,
        };
        assert!(persistent.auto_dismiss_after.is_none());
    }

    #[test]
    fn test_device_cache_default_and_staleness() {
        let cache = DeviceCache::default();
        assert!(cache.android_device_types.is_empty());
        assert!(cache.ios_device_types.is_empty());
        assert!(!cache.is_loading);
        assert!(!cache.is_stale());

        // Simulate stale cache
        let stale = DeviceCache {
            last_updated: std::time::Instant::now() - std::time::Duration::from_secs(301),
            ..Default::default()
        };
        assert!(stale.is_stale());
    }

    #[test]
    fn test_device_cache_update_android() {
        let mut cache = DeviceCache {
            is_loading: true,
            ..Default::default()
        };
        let types = vec![("pixel_7".to_string(), "Pixel 7".to_string())];
        let levels = vec![("34".to_string(), "Android 14".to_string())];
        cache.update_android_cache(types.clone(), levels.clone());

        assert_eq!(cache.android_device_types, types);
        assert_eq!(cache.android_api_levels, levels);
        assert!(!cache.is_loading);
    }

    #[test]
    fn test_device_cache_ios_data() {
        let cache = DeviceCache {
            ios_device_types: vec![("iPhone15,2".to_string(), "iPhone 15".to_string())],
            ios_runtimes: vec![("iOS-17-0".to_string(), "iOS 17.0".to_string())],
            ..Default::default()
        };
        assert_eq!(cache.ios_device_types.len(), 1);
        assert_eq!(cache.ios_runtimes.len(), 1);
    }
}
