//! File paths, extensions, and directory structures.

/// Android SDK paths and directory structures
pub mod android {
    pub const CMDLINE_TOOLS_LATEST_BIN: &str = "cmdline-tools/latest/bin";
    pub const TOOLS_BIN: &str = "tools/bin";
    pub const EMULATOR_DIR: &str = "emulator";
    pub const AVD_DIR: &str = ".android";
    pub const AVD_SUBDIR: &str = "avd";
    pub const CONFIG_INI: &str = "config.ini";
    pub const SKINS_DIR: &str = "skins";
    pub const PLATFORMS_DIR: &str = "platforms";
    pub const SYSTEM_IMAGES_DIR: &str = "system-images";
}

/// File extensions
pub const AVD_EXTENSION: &str = ".avd";
pub const INI_EXTENSION: &str = ".ini";
pub const LOG_EXTENSION: &str = ".log";

/// Configuration file names
pub const CONFIG_FILE: &str = "config.ini";
pub const HARDWARE_FILE: &str = "hardware-qemu.ini";
