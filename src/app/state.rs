//! Application state management

use crate::models::device_info::DynamicDeviceConfig;
use crate::models::{AndroidDevice, IosDevice};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Android,
    Ios,
}

impl Panel {
    pub fn toggle(self) -> Self {
        match self {
            Self::Android => Self::Ios,
            Self::Ios => Self::Android,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    DeviceList,
    LogArea,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    CreateDevice,
    ConfirmDelete,
    ConfirmWipe,
    Help,
}

#[derive(Debug, Clone)]
pub struct ConfirmDeleteDialog {
    pub device_name: String,
    pub device_identifier: String,
    pub platform: Panel,
}

#[derive(Debug, Clone)]
pub struct ConfirmWipeDialog {
    pub device_name: String,
    pub device_identifier: String,
    pub platform: Panel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub auto_dismiss_after: Option<std::time::Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreateDeviceField {
    ApiLevel,   // API Level first
    Category,   // Device Category second (phone/tablet/etc)
    DeviceType, // Device Type third
    RamSize,
    StorageSize,
    Name, // Name last
}

#[derive(Debug, Clone)]
pub struct DeviceCache {
    pub android_device_types: Vec<(String, String)>,
    pub android_api_levels: Vec<(String, String)>,
    pub android_device_cache: Option<Vec<(String, String)>>, // All devices for category filtering
    pub ios_device_types: Vec<(String, String)>,
    pub ios_runtimes: Vec<(String, String)>,
    pub last_updated: std::time::Instant,
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
    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() > std::time::Duration::from_secs(300) // 5分でキャッシュ無効
    }

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
}

#[derive(Debug, Clone)]
pub struct CreateDeviceForm {
    pub active_field: CreateDeviceField,
    pub name: String,
    pub device_type: String,
    pub device_type_id: String, // Internal ID for device type
    pub version: String,
    pub version_display: String, // Display text for version
    pub ram_size: String,
    pub storage_size: String,
    pub available_device_types: Vec<(String, String)>, // (id, display)
    pub available_versions: Vec<(String, String)>,     // (value, display)
    pub selected_api_level_index: usize,               // Index for API level selection
    pub selected_device_type_index: usize,             // Index for device type selection
    pub error_message: Option<String>,
    pub is_loading_cache: bool,          // バックグラウンドロード中フラグ
    pub is_creating: bool,               // デバイス作成中フラグ
    pub creation_status: Option<String>, // 作成ステータスメッセージ
    pub device_category_filter: String,  // カテゴリフィルター
    pub available_categories: Vec<String>, // 利用可能なカテゴリ
    pub selected_category_index: usize,  // カテゴリ選択インデックス
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DeviceDetails {
    pub name: String,
    pub status: String,
    pub platform: Panel,
    pub device_type: String,
    pub api_level_or_version: String,
    pub ram_size: Option<String>,
    pub storage_size: Option<String>,
    pub resolution: Option<String>,
    pub dpi: Option<String>,
    pub device_path: Option<String>,
    pub system_image: Option<String>,
    pub identifier: String, // AVD name for Android, UDID for iOS
}

pub struct AppState {
    pub active_panel: Panel,
    pub mode: Mode,
    pub android_devices: Vec<AndroidDevice>,
    pub ios_devices: Vec<IosDevice>,
    pub selected_android: usize,
    pub selected_ios: usize,
    pub is_loading: bool,
    pub device_logs: VecDeque<LogEntry>,
    pub max_log_entries: usize,
    pub create_device_form: CreateDeviceForm,
    pub confirm_delete_dialog: Option<ConfirmDeleteDialog>,
    pub confirm_wipe_dialog: Option<ConfirmWipeDialog>,
    pub notifications: VecDeque<Notification>,
    pub max_notifications: usize,
    pub log_scroll_offset: usize,
    pub log_filter_level: Option<String>,
    pub last_refresh: std::time::Instant,
    pub auto_refresh_interval: std::time::Duration,
    pub pending_device_start: Option<String>, // Device name that was just started
    pub device_cache: Arc<RwLock<DeviceCache>>, // バックグラウンドキャッシュ
    // Device operation status tracking
    pub device_operation_status: Option<String>, // Current device operation (starting, stopping, wiping)
    // New fields for log focus and display modes
    pub focused_panel: FocusedPanel,
    pub fullscreen_logs: bool,
    pub auto_scroll_logs: bool,
    pub manually_scrolled: bool,
    pub current_log_device: Option<(Panel, String)>, // Track which device's logs are being streamed
    pub log_task_handle: Option<tokio::task::JoinHandle<()>>, // Handle for log streaming task
    pub cached_device_details: Option<DeviceDetails>, // Cached detailed device information
    // Scroll offsets for device lists
    pub android_scroll_offset: usize,
    pub ios_scroll_offset: usize,
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
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
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
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn next_field_ios(&mut self) {
        // iOS doesn't have Category, RamSize, or StorageSize fields
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::DeviceType,
            CreateDeviceField::DeviceType => CreateDeviceField::Name,
            CreateDeviceField::Name => CreateDeviceField::ApiLevel,
            _ => CreateDeviceField::ApiLevel, // Fallback
        };
    }

    pub fn prev_field_ios(&mut self) {
        // iOS doesn't have Category, RamSize, or StorageSize fields
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::Name,
            CreateDeviceField::DeviceType => CreateDeviceField::ApiLevel,
            CreateDeviceField::Name => CreateDeviceField::DeviceType,
            _ => CreateDeviceField::Name, // Fallback
        };
    }

    pub fn move_selection_up(&mut self) -> bool {
        // 上下キーは選択には使わず、常にfalseを返してフィールド移動にフォールバック
        false
    }

    pub fn move_selection_down(&mut self) -> bool {
        // 上下キーは選択には使わず、常にfalseを返してフィールド移動にフォールバック
        false
    }

    pub fn update_selected_api_level(&mut self) {
        if let Some((value, display)) = self.available_versions.get(self.selected_api_level_index) {
            self.version = value.clone();
            self.version_display = display.clone();
            self.generate_placeholder_name();
        }
    }

    pub fn update_selected_category(&mut self) {
        if let Some(category) = self.available_categories.get(self.selected_category_index) {
            self.device_category_filter = category.clone();
            // カテゴリが変更されたらデバイスタイプ選択をリセット
            self.selected_device_type_index = 0;
            // デバイスタイプリストはUI側で更新される
        }
    }

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
                let words: Vec<&str> = cleaned.split_whitespace().take(3).collect();
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
                    .take(2)
                    .collect::<Vec<&str>>()
                    .join(" ")
            } else {
                format!("API {}", self.version)
            }
        } else {
            "API".to_string()
        };

        // Generate the full name with spaces preserved for display
        let full_name = format!("{} {}", device_part, api_part);
        self.name = full_name;

        // If name is empty, provide a fallback
        if self.name.trim().is_empty() {
            self.name = format!("Device API {}", self.version);
        }
    }
}

impl Notification {
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: Some(std::time::Duration::from_secs(5)),
        }
    }

    pub fn success(message: String) -> Self {
        Self::new(message, NotificationType::Success)
    }

    pub fn error(message: String) -> Self {
        Self::new(message, NotificationType::Error)
    }

    pub fn warning(message: String) -> Self {
        Self::new(message, NotificationType::Warning)
    }

    pub fn info(message: String) -> Self {
        Self::new(message, NotificationType::Info)
    }

    pub fn persistent(message: String, notification_type: NotificationType) -> Self {
        Self {
            message,
            notification_type,
            timestamp: chrono::Local::now(),
            auto_dismiss_after: None,
        }
    }

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
            is_loading: true, // 初期状態はローディング中
            device_logs: VecDeque::new(),
            max_log_entries: 1000,
            create_device_form: CreateDeviceForm::default(),
            confirm_delete_dialog: None,
            confirm_wipe_dialog: None,
            notifications: VecDeque::new(),
            max_notifications: 10,
            log_scroll_offset: 0,
            log_filter_level: None,
            last_refresh: std::time::Instant::now(),
            auto_refresh_interval: std::time::Duration::from_secs(3), // 3秒毎
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
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Android => Panel::Ios,
            Panel::Ios => Panel::Android,
        };
    }

    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android > 0 {
                        self.selected_android -= 1;
                    } else {
                        // 一番上から上に行くと一番下にループ
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
                        // 一番上から上に行くと一番下にループ
                        self.selected_ios = self.ios_devices.len() - 1;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android < self.android_devices.len() - 1 {
                        self.selected_android += 1;
                    } else {
                        // 一番下から下に行くと一番上にループ
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
                        // 一番下から下に行くと一番上にループ
                        self.selected_ios = 0;
                    }
                    // Update scroll offset to keep selection visible
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    // Helper methods to update scroll offsets
    fn update_android_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    fn update_ios_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    // Methods to get the current scroll offset calculated by render
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

    pub fn clear_logs(&mut self) {
        self.device_logs.clear();
    }

    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push_back(notification);

        while self.notifications.len() > self.max_notifications {
            self.notifications.pop_front();
        }
    }

    pub fn add_success_notification(&mut self, message: String) {
        self.add_notification(Notification::success(message));
    }

    pub fn add_error_notification(&mut self, message: String) {
        self.add_notification(Notification::error(message));
    }

    pub fn add_warning_notification(&mut self, message: String) {
        self.add_notification(Notification::warning(message));
    }

    pub fn add_info_notification(&mut self, message: String) {
        self.add_notification(Notification::info(message));
    }

    pub fn dismiss_expired_notifications(&mut self) {
        self.notifications.retain(|n| !n.should_dismiss());
    }

    pub fn dismiss_all_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn dismiss_notification(&mut self, index: usize) {
        if index < self.notifications.len() {
            self.notifications.remove(index);
        }
    }

    pub fn scroll_logs_up(&mut self) {
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset -= 1;
            self.manually_scrolled = true;
        }
    }

    pub fn scroll_logs_down(&mut self) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        if self.log_scroll_offset < max_offset {
            self.log_scroll_offset += 1;
            self.manually_scrolled = true;
        }
    }

    pub fn scroll_logs_page_up(&mut self, page_size: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(page_size);
        self.manually_scrolled = true;
    }

    pub fn scroll_logs_page_down(&mut self, page_size: usize) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        self.log_scroll_offset = (self.log_scroll_offset + page_size).min(max_offset);
        self.manually_scrolled = true;
    }

    pub fn reset_log_scroll(&mut self) {
        self.log_scroll_offset = 0;
    }

    pub fn toggle_log_filter(&mut self, level: Option<String>) {
        self.log_filter_level = level;
        self.reset_log_scroll();
    }

    // Tab navigation is now handled directly in app/mod.rs for three-way cycling

    pub fn toggle_fullscreen_logs(&mut self) {
        self.fullscreen_logs = !self.fullscreen_logs;
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll_logs = !self.auto_scroll_logs;
        if self.auto_scroll_logs {
            self.manually_scrolled = false;
        }
    }

    /// Get details for the currently selected device (uses cached details if available)
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
                if let Some(device) = self.android_devices.get(self.selected_android) {
                    Some(DeviceDetails {
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
                } else {
                    None
                }
            }
            Panel::Ios => {
                if let Some(device) = self.ios_devices.get(self.selected_ios) {
                    Some(DeviceDetails {
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
                } else {
                    None
                }
            }
        }
    }

    /// Update cached device details
    pub fn update_cached_device_details(&mut self, details: DeviceDetails) {
        self.cached_device_details = Some(details);
    }

    /// Clear cached device details
    pub fn clear_cached_device_details(&mut self) {
        self.cached_device_details = None;
    }

    /// Smart cache clearing - only clear if switching to different platform
    pub fn smart_clear_cached_device_details(&mut self, new_panel: Panel) {
        if let Some(ref cached) = self.cached_device_details {
            if cached.platform != new_panel {
                self.cached_device_details = None;
            }
        }
    }

    /// Get Android version name for API level (simplified version)
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

    pub fn scroll_logs_to_top(&mut self) {
        self.log_scroll_offset = 0;
        self.manually_scrolled = true;
    }

    pub fn scroll_logs_to_bottom(&mut self) {
        let total_logs = self.get_filtered_logs().len();
        self.log_scroll_offset = total_logs.saturating_sub(1);
        self.manually_scrolled = false;
    }

    pub fn scroll_logs_half_page_up(&mut self, page_size: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(page_size / 2);
        self.manually_scrolled = true;
    }

    pub fn scroll_logs_half_page_down(&mut self, page_size: usize) {
        let max_offset = self.device_logs.len().saturating_sub(1);
        self.log_scroll_offset = (self.log_scroll_offset + page_size / 2).min(max_offset);
        self.manually_scrolled = true;
    }

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

    pub fn should_auto_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.auto_refresh_interval
            || self.pending_device_start.is_some()
    }

    pub fn mark_refreshed(&mut self) {
        self.last_refresh = std::time::Instant::now();
    }

    pub fn set_pending_device_start(&mut self, device_name: String) {
        self.pending_device_start = Some(device_name);
        // Refresh more frequently when device is starting
        self.auto_refresh_interval = std::time::Duration::from_secs(1);
    }

    pub fn clear_pending_device_start(&mut self) {
        self.pending_device_start = None;
        // Return to normal refresh interval
        self.auto_refresh_interval = std::time::Duration::from_secs(3);
    }

    pub fn get_pending_device_start(&self) -> Option<&String> {
        self.pending_device_start.as_ref()
    }

    /// Set device operation status
    pub fn set_device_operation_status(&mut self, status: String) {
        self.device_operation_status = Some(status);
    }

    /// Clear device operation status
    pub fn clear_device_operation_status(&mut self) {
        self.device_operation_status = None;
    }

    /// Get current device operation status
    pub fn get_device_operation_status(&self) -> Option<&String> {
        self.device_operation_status.as_ref()
    }

    /// バックグラウンドでデバイス情報キャッシュを更新開始
    pub async fn start_background_cache_update(&mut self) {
        // キャッシュがstaleか、初回ロードの場合のみ更新
        {
            let cache = self.device_cache.read().await;
            if !cache.is_stale() && !cache.android_device_types.is_empty() {
                return; // キャッシュが新しく、データが存在する場合はスキップ
            }
        }

        // ロード中フラグを設定
        {
            let mut cache = self.device_cache.write().await;
            if cache.is_loading {
                return; // すでにロード中の場合はスキップ
            }
            cache.is_loading = true;
        }

        self.create_device_form.is_loading_cache = true;
    }

    /// キャッシュからデータを取得してフォームに設定
    pub async fn populate_form_from_cache(&mut self, platform: Panel) {
        let cache = self.device_cache.read().await;

        match platform {
            Panel::Android => {
                if !cache.android_device_types.is_empty() && !cache.android_api_levels.is_empty() {
                    self.create_device_form.available_device_types =
                        cache.android_device_types.clone();
                    self.create_device_form.available_versions = cache.android_api_levels.clone();

                    // カテゴリフィルタ用に全デバイスもキャッシュに保存
                    if cache.android_device_cache.is_none() {
                        drop(cache);
                        let mut cache = self.device_cache.write().await;
                        cache.android_device_cache = Some(cache.android_device_types.clone());
                        drop(cache);
                        let _cache = self.device_cache.read().await;
                    }

                    // 初期選択を設定
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

                    // 初期選択を設定
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

    /// キャッシュが利用可能かチェック
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
