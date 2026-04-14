use super::{AppState, Panel};
use crate::constants::{
    defaults::{DEFAULT_RAM_MB, DEFAULT_STORAGE_MB},
    limits::{MAX_WORDS_IN_API_DISPLAY, MAX_WORDS_IN_DEVICE_NAME},
};
use crate::models::device_info::DynamicDeviceConfig;

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

impl Default for CreateDeviceForm {
    fn default() -> Self {
        Self {
            active_field: CreateDeviceField::ApiLevel,
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
        form.update_selected_category();
        form.available_device_types = vec![];
        form.device_type = String::new();
        form.device_type_id = String::new();
        form
    }

    /// Creates a form configured for iOS device creation.
    /// Skips Android-specific fields like Category, RAM, and Storage.
    pub fn for_ios() -> Self {
        let mut form = Self::new();
        form.update_selected_category();
        form.available_device_types = vec![];
        form.device_type_id = String::new();
        form.device_type = String::new();
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
            _ => CreateDeviceField::ApiLevel,
        };
    }

    /// Moves focus to the previous field in the form (iOS version).
    /// Only cycles through iOS-relevant fields in reverse order.
    pub fn prev_field_ios(&mut self) {
        self.active_field = match self.active_field {
            CreateDeviceField::ApiLevel => CreateDeviceField::Name,
            CreateDeviceField::DeviceType => CreateDeviceField::ApiLevel,
            CreateDeviceField::Name => CreateDeviceField::DeviceType,
            _ => CreateDeviceField::Name,
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
            self.selected_device_type_index = 0;
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
        let device_config = DynamicDeviceConfig::new();
        let device_part = if !self.device_type.is_empty() {
            let parsed_parts = device_config.parse_device_name(&self.device_type);
            if !parsed_parts.is_empty() {
                parsed_parts.join(" ")
            } else {
                let cleaned = self
                    .device_type
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect::<String>()
                    .trim()
                    .to_string();

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
            if self.version_display.starts_with("iOS") {
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

        let full_name = format!("{device_part} {api_part}");
        self.name = full_name;

        if self.name.trim().is_empty() {
            self.name = format!("Device API {}", self.version);
        }
    }
}

impl AppState {
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

                    if cache.android_device_cache.is_none() {
                        drop(cache);
                        let mut cache = self.device_cache.write().await;
                        cache.android_device_cache = Some(cache.android_device_types.clone());
                        drop(cache);
                        let _cache = self.device_cache.read().await;
                    }

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
                !cache.is_stale()
                    && !cache.android_device_types.is_empty()
                    && !cache.android_api_levels.is_empty()
            }
            Panel::Ios => {
                !cache.is_stale()
                    && !cache.ios_device_types.is_empty()
                    && !cache.ios_runtimes.is_empty()
            }
        }
    }
}
