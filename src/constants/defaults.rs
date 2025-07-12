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
