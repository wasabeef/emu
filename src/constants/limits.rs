//! Size limits and validation constants.

/// Minimum RAM size in MB for Android devices
pub const MIN_RAM_MB: u32 = 512;

/// Maximum RAM size in MB for Android devices
pub const MAX_RAM_MB: u32 = 8192;

/// Minimum storage size in MB for Android devices
pub const MIN_STORAGE_MB: u32 = 1024;

/// Maximum storage size in MB for Android devices
pub const MAX_STORAGE_MB: u32 = 65536;

/// Upper limit for storage validation testing
pub const STORAGE_UPPER_LIMIT_TEST: u32 = 16384;

/// Maximum device name length in characters
pub const MAX_DEVICE_NAME_LENGTH: usize = 50;

/// Maximum device type display name length for UI
pub const MAX_DEVICE_TYPE_DISPLAY_LENGTH: usize = 25;

/// Truncated device type name length
pub const TRUNCATED_DEVICE_TYPE_LENGTH: usize = 22;

/// Maximum log entries to keep in memory
pub const MAX_LOG_ENTRIES: usize = 1000;

/// Maximum notification queue size
pub const MAX_NOTIFICATIONS: usize = 10;

/// Percentage calculation multiplier
pub const PERCENTAGE_MULTIPLIER: f64 = 100.0;
