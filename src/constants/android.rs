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

/// Screen size detection patterns
pub const SCREEN_SIZE_5_INCH: &str = "5";
pub const SCREEN_SIZE_6_INCH: &str = "6";
pub const SCREEN_SIZE_10_INCH: &str = "10";
pub const SCREEN_SIZE_11_INCH: &str = "11";
pub const SCREEN_SIZE_12_INCH: &str = "12";
pub const SCREEN_SIZE_13_INCH: &str = "13";
pub const SCREEN_SIZE_15_INCH: &str = "15";
pub const SCREEN_SIZE_17_INCH: &str = "17";
pub const INCH_KEYWORD: &str = "inch";

/// Android API version mappings (latest versions)
pub const API_35_VERSION: &str = "15";
pub const API_34_VERSION: &str = "14";
pub const API_33_VERSION: &str = "13";
pub const API_32_VERSION: &str = "12L";
pub const API_31_VERSION: &str = "12";
pub const API_30_VERSION: &str = "11";
pub const API_29_VERSION: &str = "10";
pub const API_28_VERSION: &str = "9";
pub const API_27_VERSION: &str = "8.1";
pub const API_26_VERSION: &str = "8.0";

/// Default storage fallback value
pub const DEFAULT_STORAGE_FALLBACK: &str = "512M";

/// Command execution timeouts
pub const AVD_CREATION_TIMEOUT_SECS: u64 = 2;
pub const DEVICE_STATUS_CHECK_TIMEOUT_SECS: u64 = 2;

/// Default API levels to use when system images are not available
pub const ANDROID_DEFAULT_API_LEVELS: [u32; 8] = [35, 34, 33, 32, 31, 30, 29, 28];
