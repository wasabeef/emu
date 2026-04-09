//! Default values used throughout the application.

use std::time::Duration;

/// Default RAM size in MB for new Android devices
pub const DEFAULT_RAM_MB: u32 = 2048;

/// Default storage size in MB for new Android devices
pub const DEFAULT_STORAGE_MB: u32 = 8192;

/// Default ABI for the current architecture
pub fn default_abi() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "arm64-v8a"
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        "x86_64" // fallback
    }
}

/// Default GPU mode for emulators
pub const DEFAULT_GPU_MODE: &str = "auto";

/// Default skin for devices without specific skin
pub const DEFAULT_SKIN: &str = "pixel_5";

/// Cache expiration time (5 minutes)
pub const CACHE_EXPIRATION: Duration = Duration::from_secs(300);

/// Device refresh interval (5 seconds)
pub const DEVICE_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Default notification display duration (3 seconds)
pub const NOTIFICATION_DURATION: Duration = Duration::from_secs(3);

/// Default API levels to install (in descending order of preference)
pub const DEFAULT_API_LEVELS: &[u32] = &[35, 34, 33, 32, 31, 30, 29, 28];

/// Default log level for the application
pub const DEFAULT_LOG_LEVEL: &str = "info";

/// Environment variable value to disable Android logging
pub const ANDROID_LOGGING_DISABLED_VALUE: &str = "0";

/// Test device constants for debug builds
pub const TEST_DEVICE_NAME_BASE: &str = "test_debug_device";
pub const TEST_DEVICE_NAME_33: &str = "test_debug_device_33";
pub const TEST_DEVICE_TYPE: &str = "phone";
pub const TEST_API_LEVEL_34: &str = "34";
pub const TEST_API_LEVEL_33: &str = "33";

/// Test hardware specifications
pub const TEST_RAM_SIZE_DEFAULT: &str = "2048";
pub const TEST_RAM_SIZE_HIGH: &str = "4096";
pub const TEST_STORAGE_SIZE_DEFAULT: &str = "8192M";
pub const TEST_STORAGE_SIZE_SIMPLE: &str = "8192";
pub const TEST_STORAGE_SIZE_GB: &str = "8G";

/// Test device types and models
pub const TEST_ANDROID_DEVICE_TYPE: &str = "pixel_4";
pub const TEST_IOS_DEVICE_TYPE: &str = "iPhone 14";
pub const TEST_IOS_VERSION: &str = "16.0";
pub const TEST_IOS_RUNTIME: &str = "iOS 16.0";
pub const TEST_IOS_VERSION_17: &str = "17.0";
pub const TEST_IOS_RUNTIME_17: &str = "iOS 17.0";

/// Test API levels (numeric)
pub const TEST_API_LEVEL_30: u32 = 30;
pub const TEST_API_LEVEL_34_NUM: u32 = 34;
pub const TEST_API_LEVEL_33_NUM: u32 = 33;

/// Default fallback values when information is unavailable
pub const UNKNOWN_VALUE: &str = "Unknown";
pub const NO_DEVICE_SELECTED: &str = "No device selected";
pub const DEFAULT_DEVICE_CATEGORY: &str = "all";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::limits::*;
    use crate::constants::progress::*;
    use crate::constants::ui_layout::*;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_memory_limits_consistency() {
        assert!(MIN_RAM_MB < MAX_RAM_MB);
        assert!(MIN_STORAGE_MB < MAX_STORAGE_MB);
        assert!((MIN_RAM_MB..=MAX_RAM_MB).contains(&DEFAULT_RAM_MB));
        assert!((MIN_STORAGE_MB..=MAX_STORAGE_MB).contains(&DEFAULT_STORAGE_MB));
        assert!(DEFAULT_STORAGE_MB >= DEFAULT_RAM_MB);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_memory_validation_tiers_ordered() {
        assert!(MEMORY_VALIDATION_MIN_MB <= MEMORY_VALIDATION_BASE_MB);
        assert!(MEMORY_VALIDATION_BASE_MB <= MEMORY_VALIDATION_HIGH_MB);
        assert!(MEMORY_VALIDATION_HIGH_MB <= MEMORY_VALIDATION_MAX_MB);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_duration_relationships() {
        assert!(CACHE_EXPIRATION > DEVICE_REFRESH_INTERVAL);
        assert!(DEVICE_REFRESH_INTERVAL > NOTIFICATION_DURATION);
    }

    #[test]
    fn test_api_levels_descending() {
        for pair in DEFAULT_API_LEVELS.windows(2) {
            assert!(pair[0] > pair[1], "API levels must be in descending order");
        }
    }

    #[test]
    fn test_panel_percentages_sum() {
        assert_eq!(
            ANDROID_PANEL_PERCENTAGE + IOS_PANEL_PERCENTAGE + DEVICE_DETAILS_PANEL_PERCENTAGE,
            100
        );
        assert_eq!(
            ANDROID_PANEL_PERCENTAGE + IOS_PANEL_PERCENTAGE,
            DEVICE_PANELS_PERCENTAGE
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_progress_phases_ascending() {
        assert!(DOWNLOAD_PHASE_START_PERCENTAGE < EXTRACT_PHASE_START_PERCENTAGE);
        assert!(EXTRACT_PHASE_START_PERCENTAGE < INSTALL_PHASE_START_PERCENTAGE);
        assert!(INSTALL_PHASE_START_PERCENTAGE < COMPLETION_THRESHOLD_PERCENTAGE);
        assert!(COMPLETION_THRESHOLD_PERCENTAGE <= PROGRESS_PHASE_100_PERCENT);
    }

    #[test]
    fn test_default_abi_returns_valid() {
        let abi = default_abi();
        assert!(!abi.is_empty());
        assert!(
            abi == "x86_64" || abi == "arm64-v8a",
            "Unexpected ABI: {abi}"
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_dialog_size_ordering() {
        assert!(DIALOG_WIDTH_SMALL < DIALOG_WIDTH_MEDIUM);
        assert!(DIALOG_WIDTH_MEDIUM < DIALOG_WIDTH_LARGE);
        assert!(DIALOG_HEIGHT_SMALL < DIALOG_HEIGHT_MEDIUM);
        assert!(DIALOG_HEIGHT_MEDIUM < DIALOG_HEIGHT_LARGE);
    }
}
