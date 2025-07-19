//! Unit tests for patterns.rs module
//!
//! This module contains comprehensive tests for the patterns.rs constants module,
//! focusing on the lazy_static Regex patterns and their runtime behavior.

use emu::constants::patterns::*;
use regex::Regex;

#[cfg(test)]
mod regex_pattern_tests {
    use super::*;

    #[test]
    fn test_api_level_config_regex() {
        // Test that the pattern is properly initialized
        assert!(API_LEVEL_CONFIG.is_match("system-images/android-34/"), 
               "Should match API level 34 config");
        assert!(API_LEVEL_CONFIG.is_match("system-images/android-33/"), 
               "Should match API level 33 config");
        assert!(API_LEVEL_CONFIG.is_match("system-images/android-29/"), 
               "Should match API level 29 config");
        
        // Test capture groups
        let caps = API_LEVEL_CONFIG.captures("image.sysdir.1=system-images/android-34/").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "34", "Should capture API level 34");
        
        let caps = API_LEVEL_CONFIG.captures("image.sysdir.1=system-images/android-33/").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "33", "Should capture API level 33");
    }

    #[test]
    fn test_api_level_target_regex() {
        // Test target pattern matching
        assert!(API_LEVEL_TARGET.is_match("target=android-34"), 
               "Should match target android-34");
        assert!(API_LEVEL_TARGET.is_match("target=android-33"), 
               "Should match target android-33");
        assert!(API_LEVEL_TARGET.is_match("target=android-29"), 
               "Should match target android-29");
        
        // Test capture groups
        let caps = API_LEVEL_TARGET.captures("target=android-34").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "34", "Should capture API level 34");
        
        let caps = API_LEVEL_TARGET.captures("target=android-28").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "28", "Should capture API level 28");
    }

    #[test]
    fn test_api_level_based_on_regex() {
        // Test "Based on" pattern matching
        assert!(API_LEVEL_BASED_ON.is_match("Based on: Android 14"), 
               "Should match 'Based on: Android 14'");
        assert!(API_LEVEL_BASED_ON.is_match("Based on: Android 13"), 
               "Should match 'Based on: Android 13'");
        assert!(API_LEVEL_BASED_ON.is_match("Based on:  Android  12"), 
               "Should match with extra spaces");
        
        // Test capture groups
        let caps = API_LEVEL_BASED_ON.captures("Based on: Android 14").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "14", "Should capture Android version 14");
        
        let caps = API_LEVEL_BASED_ON.captures("Based on: Android 13.1").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "13.1", "Should capture Android version 13.1");
    }

    #[test]
    fn test_api_level_generic_regex() {
        // Test generic API level pattern
        assert!(API_LEVEL_GENERIC.is_match("API level 34"), 
               "Should match 'API level 34'");
        assert!(API_LEVEL_GENERIC.is_match("android-33"), 
               "Should match 'android-33'");
        assert!(API_LEVEL_GENERIC.is_match("API level 29"), 
               "Should match 'API level 29'");
        
        // Test capture groups
        let caps = API_LEVEL_GENERIC.captures("API level 34").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "34", "Should capture API level 34");
        
        let caps = API_LEVEL_GENERIC.captures("android-33").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "33", "Should capture API level 33");
    }

    #[test]
    fn test_name_pattern_regex() {
        // Test name pattern matching
        assert!(NAME_PATTERN.is_match("Name: MyDevice"), 
               "Should match 'Name: MyDevice'");
        assert!(NAME_PATTERN.is_match("Name:  Test Device  "), 
               "Should match with extra spaces");
        
        // Test capture groups
        let caps = NAME_PATTERN.captures("Name: MyDevice").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "MyDevice", "Should capture device name");
        
        let caps = NAME_PATTERN.captures("Name:  Test Device  ").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), " Test Device  ", "Should capture with spaces");
    }

    #[test]
    fn test_path_pattern_regex() {
        // Test path pattern matching
        assert!(PATH_PATTERN.is_match("Path: /home/user/.android/avd/device.avd"), 
               "Should match path");
        assert!(PATH_PATTERN.is_match("Path: C:\\Users\\user\\.android\\avd\\device.avd"), 
               "Should match Windows path");
        
        // Test capture groups
        let caps = PATH_PATTERN.captures("Path: /home/user/.android/avd/device.avd").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/home/user/.android/avd/device.avd", "Should capture path");
    }

    #[test]
    fn test_target_pattern_regex() {
        // Test target pattern matching
        assert!(TARGET_PATTERN.is_match("Target: android-34"), 
               "Should match target");
        assert!(TARGET_PATTERN.is_match("Target: Google APIs"), 
               "Should match Google APIs target");
        
        // Test capture groups
        let caps = TARGET_PATTERN.captures("Target: android-34").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "android-34", "Should capture target");
    }

    #[test]
    fn test_tag_abi_pattern_regex() {
        // Test tag/ABI pattern matching
        assert!(TAG_ABI_PATTERN.is_match("Tag/ABI: google_apis/x86_64"), 
               "Should match tag/ABI");
        assert!(TAG_ABI_PATTERN.is_match("Tag/ABI: default/arm64-v8a"), 
               "Should match ARM64 ABI");
        
        // Test capture groups
        let caps = TAG_ABI_PATTERN.captures("Tag/ABI: google_apis/x86_64").unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "google_apis/x86_64", "Should capture tag/ABI");
    }

    #[test]
    fn test_emulator_serial_regex() {
        // Test emulator serial pattern matching
        assert!(EMULATOR_SERIAL.is_match("emulator-5554"), 
               "Should match emulator-5554");
        assert!(EMULATOR_SERIAL.is_match("emulator-5556"), 
               "Should match emulator-5556");
        assert!(EMULATOR_SERIAL.is_match("Something emulator-5554 something"), 
               "Should match within text");
        
        // Test that it doesn't match invalid patterns
        assert!(!EMULATOR_SERIAL.is_match("emulator-"), 
               "Should not match incomplete serial");
        assert!(!EMULATOR_SERIAL.is_match("emulator-abc"), 
               "Should not match non-numeric serial");
        
        // Test find functionality
        let text = "Device emulator-5554 is running";
        let found = EMULATOR_SERIAL.find(text).unwrap();
        assert_eq!(found.as_str(), "emulator-5554", "Should find emulator serial");
    }

    #[test]
    fn test_system_image_package_regex() {
        // Test system image package pattern matching
        let package = "system-images;android-34;google_apis;x86_64";
        assert!(SYSTEM_IMAGE_PACKAGE.is_match(package), 
               "Should match system image package");
        
        // Test capture groups
        let caps = SYSTEM_IMAGE_PACKAGE.captures(package).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "34", "Should capture API level");
        assert_eq!(caps.get(2).unwrap().as_str(), "google_apis", "Should capture tag");
        assert_eq!(caps.get(3).unwrap().as_str(), "x86_64", "Should capture ABI");
        
        // Test another package format
        let package2 = "system-images;android-33;default;arm64-v8a";
        let caps2 = SYSTEM_IMAGE_PACKAGE.captures(package2).unwrap();
        assert_eq!(caps2.get(1).unwrap().as_str(), "33", "Should capture API level 33");
        assert_eq!(caps2.get(2).unwrap().as_str(), "default", "Should capture default tag");
        assert_eq!(caps2.get(3).unwrap().as_str(), "arm64-v8a", "Should capture ARM64 ABI");
    }
}

#[cfg(test)]
mod regex_initialization_tests {
    use super::*;

    #[test]
    fn test_all_regex_patterns_initialize() {
        // Test that all lazy_static regex patterns initialize without panicking
        let _ = &*API_LEVEL_CONFIG;
        let _ = &*API_LEVEL_TARGET;
        let _ = &*API_LEVEL_BASED_ON;
        let _ = &*API_LEVEL_GENERIC;
        let _ = &*NAME_PATTERN;
        let _ = &*PATH_PATTERN;
        let _ = &*TARGET_PATTERN;
        let _ = &*TAG_ABI_PATTERN;
        let _ = &*EMULATOR_SERIAL;
        let _ = &*SYSTEM_IMAGE_PACKAGE;
    }

    #[test]
    fn test_regex_patterns_are_valid() {
        // Test that patterns can be used for matching
        assert!(API_LEVEL_CONFIG.as_str().len() > 0, "API_LEVEL_CONFIG should have a pattern");
        assert!(API_LEVEL_TARGET.as_str().len() > 0, "API_LEVEL_TARGET should have a pattern");
        assert!(API_LEVEL_BASED_ON.as_str().len() > 0, "API_LEVEL_BASED_ON should have a pattern");
        assert!(API_LEVEL_GENERIC.as_str().len() > 0, "API_LEVEL_GENERIC should have a pattern");
        assert!(NAME_PATTERN.as_str().len() > 0, "NAME_PATTERN should have a pattern");
        assert!(PATH_PATTERN.as_str().len() > 0, "PATH_PATTERN should have a pattern");
        assert!(TARGET_PATTERN.as_str().len() > 0, "TARGET_PATTERN should have a pattern");
        assert!(TAG_ABI_PATTERN.as_str().len() > 0, "TAG_ABI_PATTERN should have a pattern");
        assert!(EMULATOR_SERIAL.as_str().len() > 0, "EMULATOR_SERIAL should have a pattern");
        assert!(SYSTEM_IMAGE_PACKAGE.as_str().len() > 0, "SYSTEM_IMAGE_PACKAGE should have a pattern");
    }
}

#[cfg(test)]
mod string_constant_tests {
    use super::*;

    #[test]
    fn test_device_name_pattern_constant() {
        assert_eq!(DEVICE_NAME_PATTERN, r"^[a-zA-Z0-9_.-]+$", "DEVICE_NAME_PATTERN should match expected regex");
        
        // Test that the pattern string itself is valid
        let regex = Regex::new(DEVICE_NAME_PATTERN).unwrap();
        assert!(regex.is_match("test_device"), "Should match valid device name");
        assert!(regex.is_match("test-device"), "Should match device name with dash");
        assert!(regex.is_match("test.device"), "Should match device name with dot");
        assert!(regex.is_match("TestDevice123"), "Should match alphanumeric device name");
        assert!(!regex.is_match("test device"), "Should not match device name with space");
        assert!(!regex.is_match("test@device"), "Should not match device name with @ symbol");
    }
}

#[cfg(test)]
mod error_pattern_tests {
    use super::*;

    #[test]
    fn test_error_pattern_constants() {
        // Test error pattern constants
        assert_eq!(errors::ERROR_PREFIX, "Error:", "ERROR_PREFIX should be 'Error:'");
        assert_eq!(errors::WARNING_PREFIX, "Warning:", "WARNING_PREFIX should be 'Warning:'");
        assert_eq!(errors::LICENSE_NOT_ACCEPTED, "licenses have not been accepted", 
                  "LICENSE_NOT_ACCEPTED should match expected text");
        assert_eq!(errors::PACKAGE_PATH_INVALID, "package path is not valid", 
                  "PACKAGE_PATH_INVALID should match expected text");
        assert_eq!(errors::ADB_ERROR, "error", "ADB_ERROR should be 'error'");
        assert_eq!(errors::ADB_KO, "KO", "ADB_KO should be 'KO'");
        assert_eq!(errors::ADB_UNKNOWN_COMMAND, "unknown command", 
                  "ADB_UNKNOWN_COMMAND should match expected text");
        
        // Test that error patterns are not empty
        assert!(!errors::ERROR_PREFIX.is_empty(), "ERROR_PREFIX should not be empty");
        assert!(!errors::WARNING_PREFIX.is_empty(), "WARNING_PREFIX should not be empty");
        assert!(!errors::LICENSE_NOT_ACCEPTED.is_empty(), "LICENSE_NOT_ACCEPTED should not be empty");
        assert!(!errors::PACKAGE_PATH_INVALID.is_empty(), "PACKAGE_PATH_INVALID should not be empty");
        assert!(!errors::ADB_ERROR.is_empty(), "ADB_ERROR should not be empty");
        assert!(!errors::ADB_KO.is_empty(), "ADB_KO should not be empty");
        assert!(!errors::ADB_UNKNOWN_COMMAND.is_empty(), "ADB_UNKNOWN_COMMAND should not be empty");
    }
}

#[cfg(test)]
mod regex_performance_tests {
    use super::*;

    #[test]
    fn test_regex_patterns_performance() {
        // Test that regex patterns perform reasonably well
        let test_text = "system-images/android-34/google_apis/x86_64";
        
        // Test multiple matches don't cause performance issues
        for _ in 0..100 {
            let _ = API_LEVEL_CONFIG.is_match(test_text);
            let _ = API_LEVEL_TARGET.is_match("target=android-34");
            let _ = EMULATOR_SERIAL.is_match("emulator-5554");
        }
    }

    #[test]
    fn test_regex_reuse() {
        // Test that regex patterns can be reused efficiently
        let pattern = &*API_LEVEL_CONFIG;
        
        // Multiple uses of the same pattern should work
        assert!(pattern.is_match("system-images/android-34/"));
        assert!(pattern.is_match("system-images/android-33/"));
        assert!(pattern.is_match("system-images/android-29/"));
        
        // Test capture groups work consistently
        let caps1 = pattern.captures("image.sysdir.1=system-images/android-34/").unwrap();
        let caps2 = pattern.captures("image.sysdir.1=system-images/android-33/").unwrap();
        
        assert_eq!(caps1.get(1).unwrap().as_str(), "34");
        assert_eq!(caps2.get(1).unwrap().as_str(), "33");
    }
}