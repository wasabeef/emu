/// Priority values for device sorting and ordering
// Android device priorities (for sorting in device lists)
pub const ANDROID_TABLET_PRIORITY: u8 = 100;
pub const ANDROID_PIXEL_PRIORITY: u8 = 30;
pub const ANDROID_NEXUS_PRIORITY: u8 = 40;
pub const ANDROID_ONEPLUS_PRIORITY: u8 = 50;
pub const ANDROID_OTHER_BRAND_PRIORITY: u8 = 60;

// Pixel device priority calculation constants
pub const PIXEL_PRIORITY_OFFSET: u32 = 80;
pub const PIXEL_PRIORITY_MAX_BONUS: u32 = 19;
pub const PIXEL_UNVERSIONED_PRIORITY: u32 = 25;

// Phone category priority base
pub const PHONE_CATEGORY_BASE_PRIORITY: u32 = 30;

// Android alphabetical sorting base priorities
pub const ANDROID_ALPHA_LOW_PRIORITY: u8 = 70;
pub const ANDROID_ALPHA_HIGH_PRIORITY: u8 = 110;

// Android specific version priorities
pub const ANDROID_11_PRIORITY: u8 = 30;

// iOS device priorities (for sorting in device lists)
pub const IOS_IPHONE_MINI_PRIORITY: u8 = 30;
pub const IOS_IPHONE_SE_PRIORITY: u8 = 40;
pub const IOS_IPHONE_REGULAR_PRIORITY: u8 = 50;
pub const IOS_IPHONE_PLUS_PRIORITY: u8 = 60;
pub const IOS_IPHONE_PRO_PRIORITY: u8 = 70;
pub const IOS_IPHONE_PRO_MAX_PRIORITY: u8 = 80;
pub const IOS_IPAD_MINI_PRIORITY: u8 = 100;
pub const IOS_IPAD_REGULAR_PRIORITY: u8 = 110;
pub const IOS_IPAD_AIR_PRIORITY: u8 = 120;
pub const IOS_IPAD_PRO_11_PRIORITY: u8 = 130;
pub const IOS_IPAD_PRO_13_PRIORITY: u8 = 140;
pub const IOS_WATCH_PRIORITY: u8 = 150;
pub const IOS_TV_PRIORITY: u8 = 170;
pub const IOS_UNKNOWN_PRIORITY: u8 = 199;

// Detailed iOS device priority constants for device_info.rs calculations
// iPhone type priorities
pub const IOS_IPHONE_PRO_MAX_PRIORITY_VALUE: u32 = 0;
pub const IOS_IPHONE_PRO_PRIORITY_VALUE: u32 = 10;
pub const IOS_IPHONE_PLUS_MAX_PRIORITY: u32 = 20;
pub const IOS_IPHONE_MINI_PRIORITY_CALC: u32 = 30;
pub const IOS_IPHONE_SE_PRIORITY_CALC: u32 = 40;
pub const IOS_IPHONE_DEFAULT_BASE: u32 = 50;
pub const IOS_IPHONE_VERSION_OFFSET: u32 = 30;

// iPad priorities
pub const IOS_IPAD_PRO_12_9_PRIORITY: u32 = 100;
pub const IOS_IPAD_PRO_11_PRIORITY_VALUE: u32 = 110;
pub const IOS_IPAD_PRO_OTHER_PRIORITY: u32 = 120;
pub const IOS_IPAD_AIR_PRIORITY_VALUE: u32 = 130;
pub const IOS_IPAD_MINI_PRIORITY_CALC: u32 = 140;
pub const IOS_IPAD_DEFAULT_PRIORITY: u32 = 150;

// Apple TV priorities
pub const IOS_TV_4K_PRIORITY: u32 = 200;
pub const IOS_TV_DEFAULT_PRIORITY: u32 = 210;

// Apple Watch priorities
pub const IOS_WATCH_ULTRA_PRIORITY: u32 = 300;
pub const IOS_WATCH_SERIES_BASE: u32 = 310;
pub const IOS_WATCH_SERIES_OFFSET: u32 = 10;
pub const IOS_WATCH_DEFAULT_PRIORITY: u32 = 320;
pub const IOS_WATCH_SE_PRIORITY: u32 = 330;
pub const IOS_WATCH_OTHER_PRIORITY: u32 = 340;

// Unknown device priority
pub const IOS_UNKNOWN_DEVICE_PRIORITY: u32 = 999;

// Additional iOS priority constants for tests
pub const IOS_IPHONE_STANDARD_PRIORITY_VALUE: u32 = 20;
pub const IOS_IPHONE_SE_PRIORITY_VALUE: u32 = 25;
pub const IOS_IPHONE_MINI_PRIORITY_VALUE: u32 = 30;
pub const IOS_IPAD_PRO_PRIORITY_VALUE: u32 = 100;
pub const IOS_IPAD_STANDARD_PRIORITY_VALUE: u32 = 140;
pub const IOS_IPAD_MINI_PRIORITY_VALUE: u32 = 150;
pub const IOS_IPOD_PRIORITY_VALUE: u32 = 400;
pub const IOS_APPLE_TV_PRIORITY_VALUE: u32 = 500;
pub const IOS_APPLE_WATCH_PRIORITY_VALUE: u32 = 600;
pub const IOS_DEFAULT_PRIORITY_VALUE: u32 = 999;
