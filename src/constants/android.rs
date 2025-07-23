//! Android-specific constants and identifiers.

/// Android emulator serial number prefix
pub const EMULATOR_SERIAL_PREFIX: &str = "emulator-";

/// ADB device state strings
pub const ADB_DEVICE_STATE: &str = "device";
pub const ADB_OFFLINE_STATE: &str = "offline";

/// Android device categories and their priority ranges
pub const PHONE_CATEGORY_MIN: u32 = 100;
pub const PHONE_CATEGORY_MAX: u32 = 199;
pub const TABLET_CATEGORY_MIN: u32 = 200;
pub const TABLET_CATEGORY_MAX: u32 = 299;
pub const WEAR_CATEGORY_MIN: u32 = 300;
pub const WEAR_CATEGORY_MAX: u32 = 399;
pub const TV_CATEGORY_MIN: u32 = 400;
pub const TV_CATEGORY_MAX: u32 = 499;
pub const AUTOMOTIVE_CATEGORY_MIN: u32 = 500;
pub const AUTOMOTIVE_CATEGORY_MAX: u32 = 599;
pub const UNKNOWN_CATEGORY_MIN: u32 = 800;
pub const UNKNOWN_CATEGORY_MAX: u32 = 899;

/// Device priority calculation constants
pub const BASE_PRIORITY_OFFSET: u32 = 100;
pub const PRIORITY_VERSION_DIVIDER: u32 = 25;

/// Screen size detection patterns - now handled dynamically in size check methods
pub const INCH_KEYWORD: &str = "inch";

// API version mappings removed - now fetched dynamically from SDK

/// Default storage fallback value
pub const DEFAULT_STORAGE_FALLBACK: &str = "512M";

/// Command execution timeouts
pub const AVD_CREATION_TIMEOUT_SECS: u64 = 2;
pub const DEVICE_STATUS_CHECK_TIMEOUT_SECS: u64 = 2;

/// Default API levels range when system images are not available
/// Shows latest 8 API levels based on current max
pub const DEFAULT_API_LEVELS_COUNT: usize = 8;
pub const DEFAULT_MIN_API_LEVEL: u32 = 21; // Android 5.0

/// Android emulator port configuration
pub const EMULATOR_PORT_BASE: u16 = 5554;
pub const EMULATOR_PORT_INCREMENT: u16 = 2;
