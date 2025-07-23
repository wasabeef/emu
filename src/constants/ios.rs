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

// Screen size patterns and priority calculation constants removed - now handled dynamically
