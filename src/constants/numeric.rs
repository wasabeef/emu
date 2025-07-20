/// Numeric constants for calculations and conversions
// Unit conversion constants (correct values)
pub const BYTES_PER_KB: u64 = 1024;
pub const BYTES_PER_MB: u64 = 1024 * 1024;
pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;

// Version parsing divisors
pub const VERSION_MAJOR_DIVISOR: f32 = 10.0;
pub const VERSION_MINOR_DIVISOR: f32 = 100.0;
pub const VERSION_PATCH_DIVISOR: f32 = 10000.0;

// Default version value
pub const VERSION_DEFAULT: f32 = 0.0;

// iOS device batch processing
pub const IOS_DEVICE_PARSE_BATCH_SIZE: usize = 10;

// Screen size thresholds (inches)
pub const SCREEN_SIZE_PHONE_MEDIUM: u32 = 5;
pub const SCREEN_SIZE_PHONE_LARGE: u32 = 6;
pub const SCREEN_SIZE_MEDIUM_TABLET: u32 = 10;
pub const SCREEN_SIZE_LARGE_TABLET: u32 = 11;
pub const SCREEN_SIZE_EXTRA_LARGE_TABLET: u32 = 12;

// Device category priority base values
pub const PHONE_CATEGORY_PRIORITY_BASE: u32 = 0;
pub const FOLDABLE_CATEGORY_PRIORITY_BASE: u32 = 20;
pub const TABLET_CATEGORY_PRIORITY_BASE: u32 = 100;

// Version calculation constants
pub const VERSION_PRIORITY_BASE: u32 = 100;
pub const MAX_VERSION_FOR_PRIORITY: u32 = 99;
pub const VERSION_INCREMENT: u32 = 1;

// Priority calculation constants
pub const INITIAL_CATEGORY_PRIORITY: u32 = 0;
pub const INVALID_VERSION: u32 = 0;
pub const OEM_BONUS_WEIGHT_FULL: u32 = 2;
pub const OEM_BONUS_WEIGHT_HALF: u32 = 2;

// Regex group indices
pub const REGEX_GROUP_FIRST: usize = 1;

// Priority category offsets
pub const FOLDABLE_PRIORITY_OFFSET: u32 = 0;
pub const PHONE_PRIORITY_OFFSET: u32 = 100;
pub const TABLET_PRIORITY_OFFSET: u32 = 200;
pub const WEAR_PRIORITY_OFFSET: u32 = 300;
pub const TV_PRIORITY_OFFSET: u32 = 400;
pub const AUTOMOTIVE_PRIORITY_OFFSET: u32 = 500;
pub const UNKNOWN_PRIORITY_OFFSET: u32 = 900;
pub const UNKNOWN_DEVICE_PRIORITY: u32 = 999;

// OEM priority values
pub const SAMSUNG_OEM_PRIORITY: u32 = 10;
pub const ONEPLUS_OEM_PRIORITY: u32 = 20;
pub const XIAOMI_OEM_PRIORITY: u32 = 30;
pub const ASUS_OEM_PRIORITY: u32 = 35;
pub const OPPO_OEM_PRIORITY: u32 = 40;
pub const NOKIA_OEM_PRIORITY: u32 = 45;
pub const HUAWEI_OEM_PRIORITY: u32 = 50;
pub const MOTOROLA_OEM_PRIORITY: u32 = 55;
pub const LENOVO_OEM_PRIORITY: u32 = 60;
pub const SONY_OEM_PRIORITY: u32 = 65;
pub const LG_OEM_PRIORITY: u32 = 70;
pub const HTC_OEM_PRIORITY: u32 = 75;
pub const OEM_GENERIC_PRIORITY: u32 = 100;

// iOS iPad priority ranges
pub const IPAD_PRIORITY_MIN: u32 = 100;
pub const IPAD_PRIORITY_MAX: u32 = 199;

// Device category fallback priority
pub const DEVICE_CATEGORY_FALLBACK_PRIORITY: u32 = 50;
pub const DEVICE_CATEGORY_PHONE_FALLBACK: u32 = 150;
pub const DEVICE_CATEGORY_TABLET_FALLBACK: u32 = 250;
pub const DEVICE_CATEGORY_WEAR_FALLBACK: u32 = 350;
pub const DEVICE_CATEGORY_TV_FALLBACK: u32 = 450;
pub const DEVICE_CATEGORY_AUTOMOTIVE_FALLBACK: u32 = 550;
pub const DEVICE_CATEGORY_UNKNOWN_FALLBACK: u32 = 999;
