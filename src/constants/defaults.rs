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
