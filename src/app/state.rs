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

use crate::constants::{
    defaults::{DEFAULT_RAM_MB, DEFAULT_STORAGE_MB},
    limits::{MAX_WORDS_IN_API_DISPLAY, MAX_WORDS_IN_DEVICE_NAME},
    timeouts::{
        CACHE_EXPIRATION_TIME, CACHE_INVALIDATION_OFFSET_SECS, DEFAULT_AUTO_REFRESH_INTERVAL,
        FAST_REFRESH_INTERVAL_SECS, NOTIFICATION_AUTO_DISMISS_TIME,
    },
    MAX_LOG_ENTRIES, MAX_NOTIFICATIONS,
};
use crate::models::device_info::DynamicDeviceConfig;
use crate::models::{AndroidDevice, ApiLevel, InstallProgress, IosDevice};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

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

/// Types of notifications that can be displayed to the user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    /// Success notification (green)
    Success,
    /// Error notification (red)
    Error,
    /// Warning notification (yellow)
    Warning,
    /// Info notification (blue)
    Info,
}

/// Represents a notification message displayed to the user.
/// Notifications can auto-dismiss after a duration or be persistent.
#[derive(Debug, Clone)]
pub struct Notification {
    /// The notification message text
    pub message: String,
    /// The type/severity of the notification
    pub notification_type: NotificationType,
    /// When the notification was created
    pub timestamp: chrono::DateTime<chrono::Local>,
    /// Optional auto-dismiss duration. None means persistent.
    pub auto_dismiss_after: Option<std::time::Duration>,
}

/// Fields in the device creation form.
/// The order represents the navigation flow in the form.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreateDeviceField {
    /// API Level selection (Android) or iOS version selection
    ApiLevel,
    /// Device category selection (phone/tablet/tv/wear/etc) - Android only
    Category,
    /// Specific device type selection
    DeviceType,
    /// RAM size in MB - Android only
    RamSize,
    /// Storage size in MB - Android only
    StorageSize,
    /// Custom device name (final field)
    Name,
}

/// Cache for device creation options to avoid repeated API calls.
/// This cache stores available device types, API levels, and runtimes.
/// It expires after 5 minutes to ensure fresh data.
#[derive(Debug, Clone)]
pub struct DeviceCache {
    /// Available Android device types as (id, display_name) tuples
    pub android_device_types: Vec<(String, String)>,
    /// Available Android API levels as (api_level, display_name) tuples
    pub android_api_levels: Vec<(String, String)>,
    /// Complete list of Android devices for category filtering
    pub android_device_cache: Option<Vec<(String, String)>>,
    /// Available iOS device types as (identifier, display_name) tuples
    pub ios_device_types: Vec<(String, String)>,
    /// Available iOS runtime versions as (identifier, display_name) tuples
    pub ios_runtimes: Vec<(String, String)>,
    /// Timestamp of last cache update
    pub last_updated: std::time::Instant,
    /// Flag indicating if cache is currently being loaded
    pub is_loading: bool,
}

impl Default for DeviceCache {
    fn default() -> Self {
        Self {
            android_device_types: Vec::new(),
            android_api_levels: Vec::new(),
            android_device_cache: None,
            ios_device_types: Vec::new(),
            ios_runtimes: Vec::new(),
            last_updated: std::time::Instant::now(),
            is_loading: false,
        }
    }
}

impl DeviceCache {
    /// Checks if the cache is stale (older than 5 minutes).
    /// Returns true if the cache should be refreshed.
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() > CACHE_EXPIRATION_TIME
    }

    /// Updates the Android device cache with new data.
    /// Resets the loading flag and updates the timestamp.
    pub fn update_android_cache(
        &mut self,
        device_types: Vec<(String, String)>,
        api_levels: Vec<(String, String)>,
    ) {
        self.android_device_types = device_types;
        self.android_api_levels = api_levels;
        self.last_updated = std::time::Instant::now();
        self.is_loading = false;
    }

    /// Updates the iOS device cache with new data.
    /// Resets the loading flag and updates the timestamp.
    pub fn update_ios_cache(
        &mut self,
        device_types: Vec<(String, String)>,
        runtimes: Vec<(String, String)>,
    ) {
        self.ios_device_types = device_types;
        self.ios_runtimes = runtimes;
        self.last_updated = std::time::Instant::now();
        self.is_loading = false;
    }

    /// Invalidates the Android cache by clearing API levels and marking as stale.
    /// This forces a cache refresh on the next device creation.
    pub fn invalidate_android_cache(&mut self) {
        self.android_api_levels.clear();
        self.last_updated = std::time::Instant::now()
            - std::time::Duration::from_secs(CACHE_INVALIDATION_OFFSET_SECS);
    }

    /// Invalidates the iOS cache by clearing runtimes and marking as stale.
    /// This forces a cache refresh on the next device creation.
    pub fn invalidate_ios_cache(&mut self) {
        self.ios_runtimes.clear();
        self.last_updated = std::time::Instant::now()
            - std::time::Duration::from_secs(CACHE_INVALIDATION_OFFSET_SECS);
    }
}

/// Form state for creating new devices.
/// Manages all fields, selections, and validation for device creation.
#[derive(Debug, Clone)]
pub struct CreateDeviceForm {
    /// Currently active/focused field in the form
    pub active_field: CreateDeviceField,
    /// User-entered device name
    pub name: String,
    /// Display name of selected device type
    pub device_type: String,
    /// Internal ID of selected device type (used for API calls)
    pub device_type_id: String,
    /// Selected API level or runtime version value
    pub version: String,
    /// Display text for the selected version
    pub version_display: String,
    /// RAM size in MB (Android only)
    pub ram_size: String,
    /// Storage size in MB (Android only)
    pub storage_size: String,
    /// Available device types as (id, display_name) tuples
    pub available_device_types: Vec<(String, String)>,
    /// Available API levels/versions as (value, display_name) tuples
    pub available_versions: Vec<(String, String)>,
    /// Currently selected index in the API level list
    pub selected_api_level_index: usize,
    /// Currently selected index in the device type list
    pub selected_device_type_index: usize,
    /// Error message to display if validation fails
    pub error_message: Option<String>,
    /// Flag indicating background cache loading is in progress
    pub is_loading_cache: bool,
    /// Flag indicating device creation is in progress
    pub is_creating: bool,
    /// Status message during device creation
    pub creation_status: Option<String>,
    /// Current device category filter (all/phone/tablet/tv/wear/etc)
    pub device_category_filter: String,
    /// List of available device categories
    pub available_categories: Vec<String>,
    /// Currently selected category index
    pub selected_category_index: usize,
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

/// Detailed information about a device.
/// Used in the device details panel to show comprehensive device information.
#[derive(Debug, Clone)]
pub struct DeviceDetails {
    /// Device display name
    pub name: String,
    /// Current status (Running/Stopped/Booted/Shutdown)
    pub status: String,
    /// Platform the device belongs to
    pub platform: Panel,
    /// Device type (e.g., "Pixel 4", "iPhone 15")
    pub device_type: String,
    /// API level (Android) or iOS version
    pub api_level_or_version: String,
    /// RAM size in MB (Android only)
    pub ram_size: Option<String>,
    /// Storage size in MB (Android only)
    pub storage_size: Option<String>,
    /// Screen resolution (e.g., "1080x1920")
    pub resolution: Option<String>,
    /// DPI value (Android) or scale factor (iOS)
    pub dpi: Option<String>,
    /// Full path to device files on disk
    pub device_path: Option<String>,
    /// System image path or identifier (Android only)
    pub system_image: Option<String>,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub identifier: String,
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

impl Default for CreateDeviceForm {
    fn default() -> Self {
        Self {
            active_field: CreateDeviceField::ApiLevel, // Start with API Level
            name: String::new(),
            device_type: String::new(),
            device_type_id: String::new(),
            version: String::new(),
            version_display: String::new(),
            ram_size: DEFAULT_RAM_MB.to_string(),
            storage_size: DEFAULT_STORAGE_MB.to_string(),
            available_device_types: vec![],
            available_versions: vec![],
            selected_api_level_index: 0,
            selected_device_type_index: 0,
            error_message: None,
            is_loading_cache: false,
            is_creating: false,
            creation_status: None,
            device_category_filter: "all".to_string(),
            available_categories: vec![
                "all".to_string(),
                "phone".to_string(),
                "tablet".to_string(),
                "wear".to_string(),
                "tv".to_string(),
                "automotive".to_string(),
                "desktop".to_string(),
            ],
            selected_category_index: 0,
        }
    }
}

impl CreateDeviceForm {
    /// Creates a new form with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a form configured for Android device creation.
    /// Initializes with Android-specific fields and defaults.
    pub fn for_android() -> Self {
        let mut form = Self::new();
        // Initialize category selection
        form.update_selected_category();
        // This will be populated from avdmanager list device
        form.available_device_types = vec![];
        form.device_type = String::new();
        form.device_type_id = String::new();
        form
    }

    /// Creates a form configured for iOS device creation.
    /// Skips Android-specific fields like Category, RAM, and Storage.
    pub fn for_ios() -> Self {
        let mut form = Self::new();
        // Initialize category selection (though not used for iOS)
        form.update_selected_category();
        // Device types will be populated dynamically
        form.available_device_types = vec![];
        form.device_type_id = String::new();
        form.device_type = String::new();
        // Start with API Level for iOS (no category field)
        form.active_field = CreateDeviceField::ApiLevel;
        form
    }

    /// Moves focus to the next field in the form (Android version).
    /// Cycles through all fields in order: ApiLevel -> Category -> DeviceType -> RamSize -> StorageSize -> Name.
    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::Category,
            CreateDeviceField::Category => CreateDeviceField::DeviceType,
            CreateDeviceField::DeviceType => CreateDeviceField::RamSize,
            CreateDeviceField::RamSize => CreateDeviceField::StorageSize,
            CreateDeviceField::StorageSize => CreateDeviceField::Name,
            CreateDeviceField::Name => CreateDeviceField::ApiLevel,
        };
    }

    /// Moves focus to the previous field in the form (Android version).
    /// Cycles through fields in reverse order.
    pub fn prev_field(&mut self) {
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::Name,
            CreateDeviceField::Category => CreateDeviceField::ApiLevel,
            CreateDeviceField::DeviceType => CreateDeviceField::Category,
            CreateDeviceField::RamSize => CreateDeviceField::DeviceType,
            CreateDeviceField::StorageSize => CreateDeviceField::RamSize,
            CreateDeviceField::Name => CreateDeviceField::StorageSize,
        };
    }

    /// Moves focus to the next field in the form (iOS version).
    /// Only cycles through iOS-relevant fields: ApiLevel -> DeviceType -> Name.
    pub fn next_field_ios(&mut self) {
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::DeviceType,
            CreateDeviceField::DeviceType => CreateDeviceField::Name,
            CreateDeviceField::Name => CreateDeviceField::ApiLevel,
            _ => CreateDeviceField::ApiLevel, // Fallback
        };
    }

    /// Moves focus to the previous field in the form (iOS version).
    /// Only cycles through iOS-relevant fields in reverse order.
    pub fn prev_field_ios(&mut self) {
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::Name,
            CreateDeviceField::DeviceType => CreateDeviceField::ApiLevel,
            CreateDeviceField::Name => CreateDeviceField::DeviceType,
            _ => CreateDeviceField::Name, // Fallback
        };
    }

    /// Placeholder method that always returns false.
    /// Selection is handled through field navigation instead.
    pub fn move_selection_up(&mut self) -> bool {
        false
    }

    /// Placeholder method that always returns false.
    /// Selection is handled through field navigation instead.
    pub fn move_selection_down(&mut self) -> bool {
        false
    }

    /// Updates the selected API level based on the current index.
    /// Also regenerates the placeholder device name.
    pub fn update_selected_api_level(&mut self) {
        if let Some((value, display)) = self.available_versions.get(self.selected_api_level_index) {
            self.version = value.clone();
            self.version_display = display.clone();
            self.generate_placeholder_name();
        }
    }

    /// Updates the selected device category based on the current index.
    /// Resets device type selection when category changes.
    pub fn update_selected_category(&mut self) {
        if let Some(category) = self.available_categories.get(self.selected_category_index) {
            self.device_category_filter = category.clone();
            // Reset device type selection when category changes
            self.selected_device_type_index = 0;
            // Device type list will be updated by UI
        }
    }

    /// Updates the selected device type based on the current index.
    /// Also regenerates the placeholder device name.
    pub fn update_selected_device_type(&mut self) {
        if let Some((id, display)) = self
            .available_device_types
            .get(self.selected_device_type_index)
        {
            self.device_type_id = id.clone();
            self.device_type = display.clone();
            self.generate_placeholder_name();
        }
    }

    /// Generates a placeholder name based on selected device type and API level.
    /// Uses DynamicDeviceConfig for intelligent parsing of device names.
    /// Falls back to simple concatenation if parsing fails.
    pub fn generate_placeholder_name(&mut self) {
        // Use DynamicDeviceConfig for dynamic parsing instead of hardcoded patterns
        let device_config = DynamicDeviceConfig::new();
        let device_part = if !self.device_type.is_empty() {
            // Use dynamic device config parsing
            let parsed_parts = device_config.parse_device_name(&self.device_type);
            if !parsed_parts.is_empty() {
                parsed_parts.join(" ")
            } else {
                // Basic fallback parsing without hardcoded patterns
                let cleaned = self
                    .device_type
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
                    .trim()
                    .to_string();

                // Take first 2-3 meaningful words, keep spaces
                let words: Vec<&str> = cleaned
                    .split_whitespace()
                    .take(MAX_WORDS_IN_DEVICE_NAME)
                    .collect();
                if words.is_empty() {
                    "Device".to_string()
                } else {
                    words.join(" ")
                }
            }
        } else {
            "Device".to_string()
        };

        let api_part = if !self.version_display.is_empty() {
            // For iOS: "iOS 17.0" -> "iOS 17"
            // For Android: "API 34 - Android 14" -> "API 34"
            if self.version_display.starts_with("iOS") {
                // Simplify iOS version for name (iOS 17.0 -> iOS 17)
                let parts: Vec<&str> = self.version_display.split('.').collect();
                if let Some(major) = parts.first() {
                    major.to_string()
                } else {
                    self.version_display.clone()
                }
            } else if self.version_display.starts_with("API") {
                self.version_display
                    .split(' ')
                    .take(MAX_WORDS_IN_API_DISPLAY)
                    .collect::<Vec<&str>>()
                    .join(" ")
            } else {
                format!("API {}", self.version)
            }
        } else {
            "API".to_string()
        };

        // Generate the full name with spaces preserved for display
        let full_name = format!("{device_part} {api_part}");
        self.name = full_name;

        // If name is empty, provide a fallback
        if self.name.trim().is_empty() {
            self.name = format!("Device API {}", self.version);
        }
    }
}

impl Notification {
    /// Creates a new notification with the specified message and type.
    /// Sets auto-dismiss to 5 seconds by default.
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: Some(NOTIFICATION_AUTO_DISMISS_TIME),
        }
    }

    /// Creates a success notification (green) with 5-second auto-dismiss.
    pub fn success(message: String) -> Self {
        Self::new(message, NotificationType::Success)
    }

    /// Creates an error notification (red) with 5-second auto-dismiss.
    pub fn error(message: String) -> Self {
        Self::new(message, NotificationType::Error)
    }

    /// Creates a warning notification (yellow) with 5-second auto-dismiss.
    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    /// Creates an info notification (blue) with 5-second auto-dismiss.
    pub fn info(message: String) -> Self {
        Self::new(message, NotificationType::Info)
    }

    /// Creates a persistent notification that won't auto-dismiss.
    /// User must manually dismiss or clear all notifications.
    pub fn persistent(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: None,
        }
    }

    /// Checks if this notification should be automatically dismissed.
    /// Returns true if the auto-dismiss duration has elapsed.
    pub fn should_dismiss(&self) -> bool {
        if let Some(duration) = self.auto_dismiss_after {
            chrono::Local::now().signed_duration_since(self.timestamp)
                > chrono::Duration::from_std(duration).unwrap_or_default()
        } else {
            false
        }
    }
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

    /// Switches between Android and iOS panels.
    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Android => Panel::Ios,
            Panel::Ios => Panel::Android,
        };
    }

    /// Moves selection up in the current device list.
    /// Wraps around from top to bottom when reaching the first item.
    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android > 0 {
                        self.selected_android -= 1;
                    } else {
                        // Wrap from top to bottom
                        self.selected_android = self.android_devices.len() - 1;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_android_scroll_offset();
                }
            }
            Panel::Ios => {
                if !self.ios_devices.is_empty() {
                    if self.selected_ios > 0 {
                        self.selected_ios -= 1;
                    } else {
                        // Wrap from top to bottom
                        self.selected_ios = self.ios_devices.len() - 1;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    /// Moves selection down in the current device list.
    /// Wraps around from bottom to top when reaching the last item.
    pub fn move_down(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android < self.android_devices.len() - 1 {
                        self.selected_android += 1;
                    } else {
                        // Wrap from bottom to top
                        self.selected_android = 0;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_android_scroll_offset();
                }
            }
            Panel::Ios => {
                if !self.ios_devices.is_empty() {
                    if self.selected_ios < self.ios_devices.len() - 1 {
                        self.selected_ios += 1;
                    } else {
                        // Wrap from bottom to top
                        self.selected_ios = 0;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    /// Moves device selection by a specified number of steps.
    /// Positive steps move down/right, negative steps move up/left.
    /// Handles wrapping at list boundaries.
    pub fn move_by_steps(&mut self, steps: i32) {
        if steps == 0 {
            return;
        }

        match self.active_panel {
            Panel::Android => {
                let device_count = self.android_devices.len();
                if device_count == 0 {
                    return;
                }

                // Calculate new position with wrapping
                let current = self.selected_android as i32;
                let new_pos = if steps > 0 {
                    // Moving down
                    let raw_pos = current + steps;
                    (raw_pos % device_count as i32) as usize
                } else {
                    // Moving up
                    let raw_pos = current + steps;
                    if raw_pos < 0 {
                        let wrapped = device_count as i32 + (raw_pos % device_count as i32);
                        (wrapped % device_count as i32) as usize
                    } else {
                        raw_pos as usize
                    }
                };

                self.selected_android = new_pos;
                self.update_android_scroll_offset();
            }
            Panel::Ios => {
                let device_count = self.ios_devices.len();
                if device_count == 0 {
                    return;
                }

                // Calculate new position with wrapping
                let current = self.selected_ios as i32;
                let new_pos = if steps > 0 {
                    // Moving down
                    let raw_pos = current + steps;
                    (raw_pos % device_count as i32) as usize
                } else {
                    // Moving up
                    let raw_pos = current + steps;
                    if raw_pos < 0 {
                        let wrapped = device_count as i32 + (raw_pos % device_count as i32);
                        (wrapped % device_count as i32) as usize
                    } else {
                        raw_pos as usize
                    }
                };

                self.selected_ios = new_pos;
                self.update_ios_scroll_offset();
            }
        }
    }

    /// Helper method to update Android scroll offset.
    /// Currently empty as scroll offset is calculated dynamically during rendering.
    fn update_android_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    /// Helper method to update iOS scroll offset.
    /// Currently empty as scroll offset is calculated dynamically during rendering.
    fn update_ios_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    /// Calculates the appropriate scroll offset for the Android device list.
    /// Ensures the selected item is visible within the available height.
    pub fn get_android_scroll_offset(&self, available_height: usize) -> usize {
        if self.android_devices.len() <= available_height || available_height == 0 {
            return 0;
        }

        let selected = self.selected_android;
        let current_offset = self.android_scroll_offset;

        if selected < current_offset {
            selected
        } else if selected >= current_offset + available_height {
            selected.saturating_sub(available_height.saturating_sub(1))
        } else {
            current_offset
        }
    }

    /// Calculates the appropriate scroll offset for the iOS device list.
    /// Ensures the selected item is visible within the available height.
    pub fn get_ios_scroll_offset(&self, available_height: usize) -> usize {
        if self.ios_devices.len() <= available_height || available_height == 0 {
            return 0;
        }

        let selected = self.selected_ios;
        let current_offset = self.ios_scroll_offset;

        if selected < current_offset {
            selected
        } else if selected >= current_offset + available_height {
            selected.saturating_sub(available_height.saturating_sub(1))
        } else {
            current_offset
        }
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

    /// Gets details for the currently selected device.
    /// Returns cached details if available and matching current selection.
    /// Falls back to generating basic details from device data if cache miss.
    pub fn get_selected_device_details(&self) -> Option<DeviceDetails> {
        // Return cached details if available and matches current selection
        if let Some(ref cached) = self.cached_device_details {
            let current_identifier = match self.active_panel {
                Panel::Android => self
                    .android_devices
                    .get(self.selected_android)
                    .map(|d| d.name.clone()),
                Panel::Ios => self
                    .ios_devices
                    .get(self.selected_ios)
                    .map(|d| d.udid.clone()),
            };

            if Some(cached.identifier.clone()) == current_identifier {
                return Some(cached.clone());
            }
        }

        // Fall back to basic details
        match self.active_panel {
            Panel::Android => {
                self.android_devices
                    .get(self.selected_android)
                    .map(|device| DeviceDetails {
                        name: device.name.clone(),
                        status: if device.is_running {
                            "Running".to_string()
                        } else {
                            "Stopped".to_string()
                        },
                        platform: Panel::Android,
                        device_type: device.device_type.clone(),
                        api_level_or_version: format!(
                            "API {} (Android {})",
                            device.api_level,
                            self.get_android_version_name(device.api_level)
                        ),
                        ram_size: None,     // Will be filled by manager
                        storage_size: None, // Will be filled by manager
                        resolution: None,   // Will be filled by manager
                        dpi: None,          // Will be filled by manager
                        device_path: None,  // Will be filled by manager
                        system_image: None, // Will be filled by manager
                        identifier: device.name.clone(),
                    })
            }
            Panel::Ios => {
                self.ios_devices
                    .get(self.selected_ios)
                    .map(|device| DeviceDetails {
                        name: device.name.clone(),
                        status: if device.is_running {
                            "Booted".to_string()
                        } else {
                            "Shutdown".to_string()
                        },
                        platform: Panel::Ios,
                        device_type: device.device_type.clone(),
                        api_level_or_version: device.runtime_version.clone(),
                        ram_size: None,     // iOS simulators don't expose RAM info
                        storage_size: None, // iOS simulators don't expose storage info
                        resolution: None,   // Will be filled by manager
                        dpi: None,          // iOS uses scale factor instead
                        device_path: None,  // Will be filled by manager
                        system_image: None, // iOS doesn't use system images
                        identifier: device.udid.clone(),
                    })
            }
        }
    }

    /// Updates the cached device details with new information.
    /// This cache is used to avoid repeated expensive device queries.
    pub fn update_cached_device_details(&mut self, details: DeviceDetails) {
        self.cached_device_details = Some(details);
    }

    /// Clears all cached device details.
    pub fn clear_cached_device_details(&mut self) {
        self.cached_device_details = None;
    }

    /// Intelligently clears cached device details only when switching platforms.
    /// Preserves cache when staying on the same platform.
    pub fn smart_clear_cached_device_details(&mut self, new_panel: Panel) {
        if let Some(ref cached) = self.cached_device_details {
            if cached.platform != new_panel {
                self.cached_device_details = None;
            }
        }
    }

    /// Maps Android API level to Android version name.
    /// Returns a human-readable version string like "14" for API 34.
    fn get_android_version_name(&self, api_level: u32) -> String {
        match api_level {
            35 => "15".to_string(),
            34 => "14".to_string(),
            33 => "13".to_string(),
            32 => "12L".to_string(),
            31 => "12".to_string(),
            30 => "11".to_string(),
            29 => "10".to_string(),
            28 => "9".to_string(),
            27 => "8.1".to_string(),
            26 => "8.0".to_string(),
            25 => "7.1".to_string(),
            24 => "7.0".to_string(),
            23 => "6.0".to_string(),
            _ => "Unknown".to_string(),
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

    /// Populates the device creation form from cached data.
    /// Updates available device types and versions based on platform.
    /// Also handles initial selection and category filtering setup.
    pub async fn populate_form_from_cache(&mut self, platform: Panel) {
        let cache = self.device_cache.read().await;

        match platform {
            Panel::Android => {
                if !cache.android_device_types.is_empty() && !cache.android_api_levels.is_empty() {
                    self.create_device_form.available_device_types =
                        cache.android_device_types.clone();
                    self.create_device_form.available_versions = cache.android_api_levels.clone();

                    // Store all devices for category filtering
                    if cache.android_device_cache.is_none() {
                        drop(cache);
                        let mut cache = self.device_cache.write().await;
                        cache.android_device_cache = Some(cache.android_device_types.clone());
                        drop(cache);
                        let _cache = self.device_cache.read().await;
                    }

                    // Set initial selections
                    if self.create_device_form.selected_device_type_index
                        < self.create_device_form.available_device_types.len()
                    {
                        self.create_device_form.update_selected_device_type();
                    }
                    if self.create_device_form.selected_api_level_index
                        < self.create_device_form.available_versions.len()
                    {
                        self.create_device_form.update_selected_api_level();
                    }
                }
            }
            Panel::Ios => {
                if !cache.ios_device_types.is_empty() && !cache.ios_runtimes.is_empty() {
                    self.create_device_form.available_device_types = cache.ios_device_types.clone();
                    self.create_device_form.available_versions = cache.ios_runtimes.clone();

                    // Set initial selections
                    if self.create_device_form.selected_device_type_index
                        < self.create_device_form.available_device_types.len()
                    {
                        self.create_device_form.update_selected_device_type();
                    }
                    if self.create_device_form.selected_api_level_index
                        < self.create_device_form.available_versions.len()
                    {
                        self.create_device_form.update_selected_api_level();
                    }
                }
            }
        }

        self.create_device_form.is_loading_cache = false;
    }

    /// Checks if device creation cache is available for the given platform.
    /// Returns true if both device types and API levels/runtimes are cached.
    pub async fn is_cache_available(&self, platform: Panel) -> bool {
        let cache = self.device_cache.read().await;
        match platform {
            Panel::Android => {
                !cache.android_device_types.is_empty() && !cache.android_api_levels.is_empty()
            }
            Panel::Ios => !cache.ios_device_types.is_empty() && !cache.ios_runtimes.is_empty(),
        }
    }
}

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

        // Keep selected item in the middle of the visible area when possible
        let preferred_offset = selected.saturating_sub(available_height / 2);
        let max_offset = total_items.saturating_sub(available_height);

        preferred_offset.min(max_offset)
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
}
