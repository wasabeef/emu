//! Error message constants used throughout the application.

// Android manager error contexts
pub const ERR_LIST_ANDROID_DEVICES: &str = "Failed to list Android devices";
pub const ERR_LIST_ANDROID_AVDS: &str = "Failed to list Android AVDs";
pub const ERR_READ_AVD_CONFIG: &str = "Failed to read existing AVD configuration";
pub const ERR_WRITE_AVD_CONFIG: &str = "Failed to write updated AVD configuration";
pub const ERR_STOP_EMULATOR: &str = "Failed to stop emulator {}";
pub const ERR_DELETE_ANDROID_AVD: &str = "Failed to delete Android AVD '{}'";

// iOS manager error contexts
pub const ERR_LIST_DEVICE_TYPES: &str = "Failed to list device types";
pub const ERR_PARSE_DEVICE_TYPES_JSON: &str = "Failed to parse device types JSON";
pub const ERR_LIST_RUNTIMES: &str = "Failed to list runtimes";
pub const ERR_PARSE_RUNTIMES_JSON: &str = "Failed to parse runtimes JSON";
pub const ERR_LIST_IOS_DEVICES: &str = "Failed to list iOS devices";
pub const ERR_PARSE_SIMCTL_JSON: &str = "Failed to parse simctl JSON output";
pub const ERR_GET_DEVICE_STATUS: &str = "Failed to get device status";
pub const ERR_PARSE_DEVICE_STATUS: &str = "Failed to parse device status";

// Command execution errors
pub const ERR_EXECUTE_COMMAND: &str = "Failed to execute command";
pub const ERR_SPAWN_COMMAND: &str = "Failed to spawn command";
