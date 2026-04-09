//! Regular expression patterns for parsing command outputs.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Pattern to extract Android API level from config.ini
    pub static ref API_LEVEL_CONFIG: Regex =
        Regex::new(r"image\.sysdir\.1=system-images/android-(\d+)/?").unwrap();

    /// Pattern to extract target API level
    pub static ref API_LEVEL_TARGET: Regex =
        Regex::new(r"target=android-(\d+)").unwrap();

    /// Pattern to extract API level from avdmanager output
    pub static ref API_LEVEL_BASED_ON: Regex =
        Regex::new(r"Based on:\s*Android\s*([\d.]+)").unwrap();

    /// Generic API level pattern
    pub static ref API_LEVEL_GENERIC: Regex =
        Regex::new(r"(?:API level |android-)(\d+)").unwrap();

    /// Pattern to extract device/AVD name
    pub static ref NAME_PATTERN: Regex =
        Regex::new(r"Name:\s*(.+)").unwrap();

    /// Pattern to extract path
    pub static ref PATH_PATTERN: Regex =
        Regex::new(r"Path:\s*(.+)").unwrap();

    /// Pattern to extract target information
    pub static ref TARGET_PATTERN: Regex =
        Regex::new(r"Target:\s*(.+)").unwrap();

    /// Pattern to extract Tag/ABI
    pub static ref TAG_ABI_PATTERN: Regex =
        Regex::new(r"Tag/ABI:\s*(.+)").unwrap();

    /// Pattern to extract emulator serial
    pub static ref EMULATOR_SERIAL: Regex =
        Regex::new(r"emulator-\d+").unwrap();

    /// Pattern to parse system image package format
    pub static ref SYSTEM_IMAGE_PACKAGE: Regex =
        Regex::new(r"system-images;android-(\d+);([^;]+);([^;]+)").unwrap();
}

/// Pattern for validating device names
pub const DEVICE_NAME_PATTERN: &str = r"^[a-zA-Z0-9_.-]+$";

/// Error patterns in command outputs
pub mod errors {
    /// avdmanager error indicators
    pub const ERROR_PREFIX: &str = "Error:";
    pub const WARNING_PREFIX: &str = "Warning:";
    pub const LICENSE_NOT_ACCEPTED: &str = "licenses have not been accepted";
    pub const PACKAGE_PATH_INVALID: &str = "package path is not valid";

    /// ADB error indicators
    pub const ADB_ERROR: &str = "error";
    pub const ADB_KO: &str = "KO";
    pub const ADB_UNKNOWN_COMMAND: &str = "unknown command";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_image_regex() {
        let caps = SYSTEM_IMAGE_PACKAGE
            .captures("system-images;android-34;google_apis;arm64-v8a")
            .unwrap();
        assert_eq!(&caps[1], "34");
        assert_eq!(&caps[2], "google_apis");
        assert_eq!(&caps[3], "arm64-v8a");
    }

    #[test]
    fn test_api_level_regex_variants() {
        assert!(API_LEVEL_CONFIG.is_match("image.sysdir.1=system-images/android-34/"));
        assert!(API_LEVEL_TARGET.is_match("target=android-33"));
        assert!(API_LEVEL_BASED_ON.is_match("Based on: Android 14"));
        assert!(API_LEVEL_GENERIC.is_match("API level 34"));
    }

    #[test]
    fn test_emulator_serial_regex() {
        assert!(EMULATOR_SERIAL.is_match("emulator-5554"));
        assert!(!EMULATOR_SERIAL.is_match("device-5554"));
    }

    #[test]
    fn test_device_name_pattern_valid() {
        let re = regex::Regex::new(DEVICE_NAME_PATTERN).unwrap();
        assert!(re.is_match("Pixel_7_API34"));
        assert!(!re.is_match("invalid name!"));
    }
}

/// Character patterns for text processing
pub mod text_patterns {
    /// Inch measurement indicator
    pub const INCH_INDICATOR: &str = "\"";

    /// Memory specification brackets
    pub const MEMORY_OPEN_BRACKET: &str = "(";
    pub const MEMORY_CLOSE_BRACKET: &str = ")";

    /// Apple device prefixes
    pub const APPLE_DEVICE_PREFIX_I: &str = "i";
    pub const APPLE_DEVICE_IPHONE: &str = "iPhone";
    pub const APPLE_DEVICE_IPAD: &str = "iPad";
    pub const APPLE_DEVICE_IPOD: &str = "iPod";

    /// Chip identification prefixes
    pub const CHIP_PREFIX_M: &str = "m";
    pub const CHIP_PREFIX_A: &str = "a";
}
