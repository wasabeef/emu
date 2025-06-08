//! Application constants and configuration values
//!
//! This module defines constants for Android SDK and iOS tools integration.
//! These constants represent command names, arguments, and path structures that
//! are stable across Android SDK versions.
//!
//! # Android SDK Command-Line Tools
//!
//! ## Tool Discovery and Path Resolution
//!
//! Android SDK tools are located in multiple possible paths:
//! 1. `$ANDROID_HOME/cmdline-tools/latest/bin/` (recommended)
//! 2. `$ANDROID_HOME/tools/bin/` (legacy)
//! 3. `$ANDROID_HOME/emulator/` (emulator-specific)
//!
//! Environment variables checked in order:
//! 1. `ANDROID_HOME` (standard)
//! 2. `ANDROID_SDK_ROOT` (alternative)
//!
//! ## avdmanager Command Structure
//!
//! ### Subcommands:
//! - `list avd`: List all created AVDs
//! - `list device`: List available device definitions
//! - `list targets`: List available platform targets
//! - `create avd`: Create new AVD
//! - `delete avd`: Delete existing AVD
//!
//! ### Common Arguments:
//! - `--name {name}`: Specify AVD name
//! - `--device {device_id}`: Specify device type
//! - `--package {package}`: Specify system image package
//! - `--skin {skin_name}`: Specify device skin
//! - `--force`: Overwrite existing AVD
//!
//! ## sdkmanager Command Structure
//!
//! ### Key Arguments:
//! - `--list`: List installed and available packages
//! - `--verbose`: Include detailed package information
//! - `--include_obsolete`: Include obsolete packages
//! - `--licenses`: Accept SDK licenses
//!
//! ### Package Types:
//! - `platforms;android-{level}`: Android platform packages
//! - `system-images;android-{level};{tag};{abi}`: System image packages
//! - `build-tools;{version}`: Build tools packages
//! - `emulator`: Android Emulator package
//!
//! ## ADB Command Integration
//!
//! ### Device Management:
//! - `adb devices`: List connected devices/emulators
//! - `adb -s {serial} shell {command}`: Execute shell command on specific device
//! - `adb -s {serial} emu {command}`: Send command to emulator console
//!
//! ### Property Queries:
//! - `getprop ro.boot.qemu.avd_name`: Get AVD name from boot properties
//! - `getprop ro.kernel.qemu.avd_name`: Get AVD name from kernel properties
//!
//! ### Emulator Control:
//! - `adb -s {serial} emu kill`: Terminate emulator
//! - `adb -s {serial} emu avd name`: Get AVD name via console
//!
//! ## Emulator Launch Arguments
//!
//! ### Required Arguments:
//! - `-avd {name}`: Specify AVD to launch
//!
//! ### Performance Arguments:
//! - `-no-audio`: Disable audio (reduces resource usage)
//! - `-no-snapshot-save`: Skip snapshot saving on exit
//! - `-no-boot-anim`: Skip boot animation for faster startup
//! - `-netfast`: Use faster network emulation
//! - `-wipe-data`: Perform cold boot with data wipe
//!
//! ## Configuration File Patterns
//!
//! ### AVD Directory Structure:
//! ```text
//! ~/.android/avd/
//! ├── {AVD_NAME}.avd/
//! │   ├── config.ini          # Main configuration
//! │   ├── userdata.img        # User data partition
//! │   └── snapshots/          # Saved snapshots
//! └── {AVD_NAME}.ini          # AVD metadata
//! ```
//!
//! ### config.ini Key Patterns:
//! - `image.sysdir.1=`: System image path (contains API level)
//! - `target=android-{level}`: Target API level
//! - `avd.ini.displayname=`: Human-readable name
//! - `hw.device.name=`: Device type identifier
//! - `hw.ramSize=`: RAM allocation in MB
//!
//! ## Regular Expression Patterns
//!
//! These patterns are used for parsing command outputs:
//!
//! ### API Level Extraction:
//! - `image\.sysdir\.1=system-images/android-(\d+)/?`: From config.ini
//! - `target=android-(\d+)`: From target configuration
//! - `Based on:\s*Android\s*([\d.]+)`: From avdmanager output
//! - `(?:API level |android-)(\d+)`: Generic API level pattern
//!
//! ### Device Information:
//! - `Name:\s*(.+)`: Device/AVD name extraction
//! - `Path:\s*(.+)`: Path extraction
//! - `Target:\s*(.+)`: Target information
//! - `Tag/ABI:\s*(.+)`: System image tag and ABI
//!
//! ## Error Response Patterns
//!
//! ### avdmanager Error Indicators:
//! - "Error:" prefix indicates command failure
//! - "Warning:" prefix indicates non-fatal issues
//! - License-related: "licenses have not been accepted"
//! - Package-related: "package path is not valid"
//!
//! ### ADB Error Indicators:
//! - "error" in response: Command execution failed
//! - "KO" response: Command not supported by emulator
//! - "unknown command": Command not recognized
//! - Empty response: Property not set or device offline
//!
//! ## System Image Package Format
//!
//! System images follow the pattern:
//! `system-images;android-{API_LEVEL};{TAG};{ABI}`
//!
//! ### Common Tags:
//! - `default`: Basic Android system image
//! - `google_apis`: Google APIs included
//! - `google_apis_playstore`: Google APIs + Play Store
//! - `android-tv`: Android TV system image
//! - `android-wear`: Wear OS system image
//!
//! ### Common ABIs:
//! - `arm64-v8a`: ARM 64-bit (recommended for M1/M2 Macs)
//! - `x86_64`: Intel 64-bit (for Intel/AMD processors)
//! - `x86`: Intel 32-bit (legacy)
//! - `armeabi-v7a`: ARM 32-bit (legacy)
//!

/// Command line tools and executables
pub mod commands {
    pub const ADB: &str = "adb";
    pub const AVDMANAGER: &str = "avdmanager";
    pub const EMULATOR: &str = "emulator";
    pub const SDKMANAGER: &str = "sdkmanager";
    pub const XCRUN: &str = "xcrun";
    pub const SIMCTL: &str = "simctl";
}

/// Environment variable names
pub mod env_vars {
    pub const ANDROID_HOME: &str = "ANDROID_HOME";
    pub const ANDROID_SDK_ROOT: &str = "ANDROID_SDK_ROOT";
    pub const HOME: &str = "HOME";
    pub const RUST_LOG: &str = "RUST_LOG";
    pub const ANDROID_EMULATOR_LOG_ENABLE: &str = "ANDROID_EMULATOR_LOG_ENABLE";
    pub const ANDROID_AVD_VERBOSE: &str = "ANDROID_AVD_VERBOSE";
    pub const ANDROID_VERBOSE: &str = "ANDROID_VERBOSE";
}

/// Android SDK paths and directory structures
pub mod android_paths {
    pub const CMDLINE_TOOLS_LATEST_BIN: &str = "cmdline-tools/latest/bin";
    pub const TOOLS_BIN: &str = "tools/bin";
    pub const EMULATOR_DIR: &str = "emulator";
    pub const AVD_DIR: &str = ".android";
    pub const AVD_SUBDIR: &str = "avd";
    pub const CONFIG_INI: &str = "config.ini";
    pub const SKINS_DIR: &str = "skins";
}

/// ADB commands and arguments
pub mod adb_commands {
    pub const DEVICES: &str = "devices";
    pub const SHELL: &str = "shell";
    pub const GETPROP: &str = "getprop";
    pub const EMU: &str = "emu";
    pub const AVD: &str = "avd";
    pub const NAME: &str = "name";
    pub const KILL: &str = "kill";
    pub const LOGCAT: &str = "logcat";

    // System properties
    pub const PROP_AVD_NAME: &str = "ro.boot.qemu.avd_name";
    pub const PROP_KERNEL_AVD_NAME: &str = "ro.kernel.qemu.avd_name";
}

/// iOS Simulator commands and arguments  
pub mod ios_commands {
    pub const LIST: &str = "list";
    pub const DEVICES: &str = "devices";
    pub const RUNTIMES: &str = "runtimes";
    pub const BOOT: &str = "boot";
    pub const SHUTDOWN: &str = "shutdown";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
    pub const ERASE: &str = "erase";
    pub const SPAWN: &str = "spawn";
    pub const LOG: &str = "log";
    pub const STREAM: &str = "stream";
    pub const JSON_FLAG: &str = "--json";
}

/// Emulator startup arguments
pub mod emulator_args {
    pub const NO_AUDIO: &str = "-no-audio";
    pub const NO_SNAPSHOT_SAVE: &str = "-no-snapshot-save";
    pub const NO_BOOT_ANIM: &str = "-no-boot-anim";
    pub const NETFAST: &str = "-netfast";
    pub const WIPE_DATA: &str = "-wipe-data";
    pub const AVD_FLAG: &str = "-avd";
    pub const FORCE_FLAG: &str = "-force";
}

/// SDKManager arguments
pub mod sdkmanager_args {
    pub const LIST: &str = "--list";
    pub const VERBOSE: &str = "--verbose";
    pub const INCLUDE_OBSOLETE: &str = "--include_obsolete";
}

/// AVDManager arguments
pub mod avdmanager_args {
    pub const LIST: &str = "list";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
    pub const AVD: &str = "avd";
    pub const TARGETS: &str = "targets";
    pub const DEVICE: &str = "device";
    pub const PACKAGE: &str = "--package";
    pub const NAME: &str = "--name";
    pub const DEVICE_FLAG: &str = "--device";
    pub const SKIN_FLAG: &str = "--skin";
    pub const FORCE: &str = "--force";
}

/// Android packages and system image configurations
pub mod android_packages {
    pub const DEFAULT_TAG: &str = "google_apis_playstore";
    pub const DEFAULT_ABI: &str = "arm64-v8a";
    pub const GOOGLE_APIS_TAG: &str = "google_apis";
    pub const SYSTEM_IMAGE_FORMAT: &str = "system-images;android-{};{};{}";
    pub const SYSTEM_IMAGES_PREFIX: &str = "system-images/android-";
    pub const TARGET_PREFIX: &str = "target=android-";
}

/// Regular expression patterns
pub mod regex_patterns {
    pub const IMAGE_SYSDIR_PATTERN: &str = r"image\.sysdir\.1=system-images/android-(\d+)/?";
    pub const TARGET_PATTERN: &str = r"target=android-(\d+)";
    pub const BASED_ON_PATTERN: &str = r"Based on:\s*Android\s*([\d.]+)";
    pub const API_LEVEL_PATTERN: &str = r"(?:API level |android-)(\d+)";
    pub const AVD_NAME_PATTERN: &str = r"Name:\s*(.+)";
    pub const AVD_PATH_PATTERN: &str = r"Path:\s*(.+)";
    pub const AVD_TARGET_PATTERN: &str = r"Target:\s*(.+)";
    pub const AVD_ABI_PATTERN: &str = r"Tag/ABI:\s*(.+)";
    pub const AVD_DEVICE_PATTERN: &str = r"Device:\s*(.+)";
}

/// Default configuration values
pub mod defaults {
    pub const DEFAULT_RAM_SIZE: &str = "2048";
    pub const DEFAULT_STORAGE_SIZE: &str = "8192";
    pub const DEFAULT_THEME: &str = "dark";
    pub const DEFAULT_LOG_LEVEL: &str = "info";

    // UI defaults
    pub const NOTIFICATION_AUTO_DISMISS_SECS: u64 = 5;
    pub const MAX_LOG_ENTRIES: usize = 1000;
    pub const MAX_NOTIFICATIONS: usize = 10;
    pub const AUTO_REFRESH_INTERVAL_SECS: u64 = 3;
    pub const FAST_REFRESH_INTERVAL_SECS: u64 = 1;
}

/// Status and message constants
pub mod status_messages {
    pub const UNKNOWN: &str = "Unknown";
    pub const GENERIC: &str = "Generic";
    pub const ERROR: &str = "error";
    pub const KO: &str = "KO";
    pub const OK: &str = "OK";
    pub const DEVICE: &str = "Device";
    pub const GENERIC_DEVICE: &str = "Generic Device";
}

/// Legacy Android API level mapping functions - DEPRECATED
/// Use AndroidManager::get_dynamic_android_version_name() for dynamic retrieval from SDK
pub fn get_android_version_name(api_level: u32) -> &'static str {
    // This function is deprecated - use AndroidManager::get_dynamic_android_version_name() instead
    log::warn!("DEPRECATED: get_android_version_name() called for API {}. Use AndroidManager::get_dynamic_android_version_name() for SDK-based dynamic lookup", api_level);

    // Emergency fallback only - should not be used in production
    "Unknown Android (use dynamic lookup)"
}

pub fn android_version_to_api_level(version: u32) -> u32 {
    // This function is deprecated - use AndroidManager::get_available_api_levels() instead
    log::warn!("DEPRECATED: android_version_to_api_level() called for version {}. Use AndroidManager::get_available_api_levels() for SDK-based dynamic lookup", version);

    // Emergency fallback only - should not be used in production
    0
}

/// File extensions and formats
pub mod file_formats {
    pub const AVD_EXTENSION: &str = ".avd";
    pub const INI_EXTENSION: &str = ".ini";
    pub const TOML_EXTENSION: &str = ".toml";
    pub const JSON_EXTENSION: &str = ".json";
}
