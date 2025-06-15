//! iOS-specific constants and identifiers.

/// iOS device status constants
pub const IOS_DEVICE_STATUS_BOOTED: &str = "Booted";
pub const IOS_DEVICE_STATUS_SHUTDOWN: &str = "Shutdown";
pub const IOS_DEVICE_STATUS_CREATING: &str = "Creating";

/// iOS simulator runtime identifier prefix
pub const IOS_RUNTIME_PREFIX: &str = "com.apple.CoreSimulator.SimRuntime.iOS-";

/// iOS device type identifier prefix
pub const IOS_DEVICE_TYPE_PREFIX: &str = "com.apple.CoreSimulator.SimDeviceType.";

/// iOS display name replacements
pub const IOS_INCH_REPLACEMENT: &str = "\"";
pub const IOS_INCH_PATTERN: &str = " inch";

/// Simulator app commands
pub const SIMULATOR_QUIT_COMMAND: &str = "tell application \"Simulator\" to quit";
pub const SIMULATOR_APP_NAME: &str = "Simulator";
pub const SIMULATOR_OPEN_FLAG: &str = "-a";

/// iOS error messages for graceful handling
pub const IOS_ALREADY_BOOTED_ERROR: &str = "Unable to boot device in current state: Booted";
pub const IOS_ALREADY_SHUTDOWN_ERROR: &str = "Unable to shutdown device in current state: Shutdown";

/// Screen size patterns for device priority calculation
pub const SCREEN_SIZE_PATTERNS: &[&str] = &["5", "6", "10", "11", "12", "13", "15", "17"];

/// Device priority ranges
pub const IPHONE_PRIORITY_BASE: u32 = 0;
pub const IPAD_PRIORITY_BASE: u32 = 100;
pub const APPLE_TV_PRIORITY_BASE: u32 = 200;
pub const APPLE_WATCH_PRIORITY_BASE: u32 = 300;

/// Device type priority offsets
pub const PRIORITY_PRO_MAX: u32 = 0;
pub const PRIORITY_PRO: u32 = 10;
pub const PRIORITY_PLUS_MAX: u32 = 20;
pub const PRIORITY_MINI: u32 = 30;
pub const PRIORITY_SE: u32 = 40;
pub const PRIORITY_REGULAR_OFFSET: u32 = 50;

/// iPad specific priorities
pub const IPAD_PRO_12_9_PRIORITY: u32 = 100;
pub const IPAD_PRO_11_PRIORITY: u32 = 110;
pub const IPAD_AIR_PRIORITY: u32 = 130;
pub const IPAD_MINI_PRIORITY: u32 = 140;
pub const IPAD_REGULAR_PRIORITY: u32 = 150;

/// Apple TV priorities
pub const APPLE_TV_4K_PRIORITY: u32 = 200;
pub const APPLE_TV_HD_PRIORITY: u32 = 210;

/// Apple Watch priorities
pub const APPLE_WATCH_ULTRA_PRIORITY: u32 = 300;
pub const APPLE_WATCH_SERIES_OFFSET: u32 = 310;
pub const APPLE_WATCH_SE_PRIORITY: u32 = 330;

/// Version calculation constants
pub const VERSION_PRIORITY_RANGE: u32 = 50;
pub const SERIES_PRIORITY_RANGE: u32 = 25;
