//! User-facing messages and text constants.
//!
//! This module centralizes all strings displayed to users, making it easier to:
//! - Maintain consistent messaging
//! - Update text without searching the codebase
//! - Potentially support internationalization in the future

/// Error messages displayed when operations fail
pub mod errors {
    // SDK and environment errors
    pub const ANDROID_SDK_NOT_FOUND: &str =
        "Android SDK not found. Please set ANDROID_HOME or ANDROID_SDK_ROOT";
    pub const TOOL_NOT_FOUND: &str = "Tool '{}' not found in Android SDK";
    pub const DEVICE_NOT_FOUND: &str = "Device '{}' not found";
    pub const NO_DEVICE_DEFINITIONS: &str =
        "No Android device definitions found. Please check your Android SDK installation.";
    pub const NO_API_LEVELS: &str =
        "No API levels found. Please check your Android SDK installation.";

    // Operation errors
    pub const DEVICE_START_FAILED: &str = "Failed to start device '{}': {}";
    pub const DEVICE_STOP_FAILED: &str = "Failed to stop device '{}': {}";
    pub const DEVICE_CREATE_FAILED: &str = "Failed to create device '{}': {}";
    pub const DEVICE_DELETE_FAILED: &str = "Failed to delete device '{}': {}";
    pub const DEVICE_WIPE_FAILED: &str = "Failed to wipe device '{}': {}";

    // Generic errors
    pub const COMMAND_EXECUTION_FAILED: &str = "Command execution failed";
    pub const CONFIGURATION_ERROR: &str = "Configuration error: {}";
    pub const FILE_ACCESS_ERROR: &str = "File access error occurred";
    pub const DATA_PARSING_FAILED: &str = "Data parsing failed";
    pub const PATTERN_MATCHING_ERROR: &str = "Pattern matching error occurred";

    // Platform-specific
    pub const IOS_NOT_AVAILABLE: &str = "iOS manager not available (only available on macOS)";
    pub const IOS_SIMULATOR_NOT_AVAILABLE: &str = "iOS simulator not available on this platform.";
}

/// Success and notification messages
pub mod notifications {
    // Device operations
    pub const DEVICE_STARTING: &str = "Starting device '{}'...";
    pub const DEVICE_START_SUCCESS: &str = "Device '{}' is now running!";
    pub const DEVICE_STOPPED: &str = "Device '{}' stopped";
    pub const DEVICE_CREATED: &str = "Device '{}' created successfully";
    pub const DEVICE_DELETED: &str = "Device '{}' deleted successfully";
    pub const DEVICE_WIPED: &str = "Device '{}' wiped successfully";

    // System operations
    pub const LOGS_CLEARED: &str = "Logs cleared";
    pub const SYSTEM_IMAGE_INSTALLED: &str = "System image installed successfully";
    pub const SYSTEM_IMAGE_UNINSTALLED: &str = "System image uninstalled successfully";

    // Status operations
    pub const STOPPING_DEVICE: &str = "Stopping device '{}'...";
    pub const WIPING_DEVICE: &str = "Wiping device '{}'...";
    pub const CREATING_DEVICE: &str = "Creating device '{}'...";
    pub const DELETING_DEVICE: &str = "Deleting device '{}'...";
}

/// UI labels and static text
pub mod ui {
    // Window titles
    pub const APP_TITLE: &str = " 🦤 Emu - Device Manager";
    pub const APP_TITLE_FULLSCREEN: &str = " 🦤 Emu - Device Manager [FULLSCREEN LOGS]";

    // Panel titles
    pub const ANDROID_DEVICES_TITLE: &str = "🤖 Android Devices";
    pub const IOS_DEVICES_TITLE: &str = "🍎 iOS Simulators";
    pub const DEVICE_DETAILS_TITLE: &str = "Device Details";
    pub const LOGS_TITLE: &str = "📋 Logs";

    // Dialog titles
    pub const CONFIRM_DELETE_TITLE: &str = "Confirm Delete";
    pub const CONFIRM_WIPE_TITLE: &str = "Confirm Wipe";
    pub const CREATE_DEVICE_TITLE: &str = "Create New Device";
    pub const API_LEVEL_MANAGEMENT_TITLE: &str = "📦 Android System Images ({}/{} installed)";

    // Field labels
    pub const NAME_LABEL: &str = "Name:";
    pub const API_LEVEL_LABEL: &str = "API Level:";
    pub const CATEGORY_LABEL: &str = "Category:";
    pub const DEVICE_TYPE_LABEL: &str = "Device Type:";
    pub const RAM_SIZE_LABEL: &str = "RAM Size (MB):";
    pub const STORAGE_SIZE_LABEL: &str = "Storage Size (MB):";
    pub const STATUS_LABEL: &str = "Status: ";
    pub const VERSION_LABEL: &str = "Version: ";
    pub const TYPE_LABEL: &str = "Type: ";
    pub const PATH_LABEL: &str = "📂 Path:";
    pub const SYSTEM_IMAGE_LABEL: &str = "🏷️  System Image:";
    pub const RAM_LABEL: &str = "🧠 RAM: ";
    pub const STORAGE_LABEL: &str = "💾 Storage: ";
    pub const RESOLUTION_LABEL: &str = "📱 Resolution: ";
    pub const DPI_LABEL: &str = "🎯 DPI: ";

    // Messages
    pub const NO_DEVICE_SELECTED: &str = "No device selected";
    pub const TERMINAL_TOO_SMALL: &str = "Terminal too small";
    pub const LOADING: &str = "Loading...";
    pub const NO_LOGS: &str = "No logs available. Start a device to see logs.";

    // Confirmation messages
    pub const DELETE_ANDROID_CONFIRM: &str = "Are you sure you want to delete this Android device?\n\n🤖 {}\n\nThis action cannot be undone.";
    pub const DELETE_IOS_CONFIRM: &str = "Are you sure you want to delete this iOS simulator?\n\n🍎 {}\n\nThis action cannot be undone.";
    pub const WIPE_ANDROID_CONFIRM: &str = "Are you sure you want to wipe this Android device?\n\n🤖 {}\n\nThis will erase all data and reset to factory state.";
    pub const WIPE_IOS_CONFIRM: &str = "Are you sure you want to wipe this iOS simulator?\n\n🍎 {}\n\nThis will erase all data and reset to factory state.";

    // Instructions
    pub const API_LEVEL_INSTRUCTIONS: &str =
        "✅ Green = Installed  📦 Gray = Available  Select and press Enter/d";

    // Shortcuts
    pub const CONFIRM_SHORTCUTS: &str = "✅ [Y]es   ❌ [N]o / [Esc] Cancel";
}

/// Device status values
pub mod status {
    pub const RUNNING: &str = "Running";
    pub const STOPPED: &str = "Stopped";
    pub const BOOTED: &str = "Booted";
    pub const SHUTDOWN: &str = "Shutdown";
    pub const STARTING: &str = "Starting";
    pub const STOPPING: &str = "Stopping";
}

/// Log level strings
pub mod log_levels {
    pub const ERROR: &str = "ERROR";
    pub const WARN: &str = "WARN";
    pub const INFO: &str = "INFO";
    pub const DEBUG: &str = "DEBUG";
    pub const VERBOSE: &str = "VERBOSE";
}

/// Device categories
pub mod categories {
    pub const ALL: &str = "All Devices";
    pub const PHONE: &str = "Phone";
    pub const TABLET: &str = "Tablet";
    pub const WEAR: &str = "Wear";
    pub const TV: &str = "TV";
    pub const AUTOMOTIVE: &str = "Automotive";
    pub const DESKTOP: &str = "Desktop";
}

/// Format patterns for common string formations
pub mod formats {
    pub const SIZE_MB: &str = "{} MB";
    pub const RESOLUTION: &str = "{}x{}";
    pub const API_LEVEL: &str = "API {}";
    pub const API_VERSION: &str = "API {} ({})";
    pub const DEVICE_WITH_OEM: &str = "{} ({})";
    pub const DPI_FORMAT: &str = "{} DPI";
}

/// Platform names
pub mod platforms {
    pub const ANDROID_DEVICE: &str = "Android device";
    pub const IOS_SIMULATOR: &str = "iOS simulator";
}
