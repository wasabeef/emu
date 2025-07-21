//! Complete coverage test for constants/patterns.rs
//!
//! This test specifically targets the lazy_static regex patterns to ensure they are initialized

use emu::constants::patterns::*;

#[cfg(test)]
mod constants_patterns_coverage_tests {
    use super::*;

    #[test]
    fn test_api_level_config_regex() {
        // Test API_LEVEL_CONFIG regex initialization and usage
        let test_string = "image.sysdir.1=system-images/android-34/google_apis/x86_64/";
        let result = API_LEVEL_CONFIG.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "34");
    }

    #[test]
    fn test_api_level_target_regex() {
        // Test API_LEVEL_TARGET regex initialization and usage
        let test_string = "target=android-33";
        let result = API_LEVEL_TARGET.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "33");
    }

    #[test]
    fn test_api_level_based_on_regex() {
        // Test API_LEVEL_BASED_ON regex initialization and usage
        let test_string = "Based on: Android 12.0";
        let result = API_LEVEL_BASED_ON.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "12.0");
    }

    #[test]
    fn test_api_level_generic_regex() {
        // Test API_LEVEL_GENERIC regex initialization and usage
        let test_string1 = "API level 31";
        let test_string2 = "android-29";

        let result1 = API_LEVEL_GENERIC.captures(test_string1);
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().get(1).unwrap().as_str(), "31");

        let result2 = API_LEVEL_GENERIC.captures(test_string2);
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().get(1).unwrap().as_str(), "29");
    }

    #[test]
    fn test_name_pattern_regex() {
        // Test NAME_PATTERN regex initialization and usage
        let test_string = "Name: MyDevice";
        let result = NAME_PATTERN.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "MyDevice");
    }

    #[test]
    fn test_path_pattern_regex() {
        // Test PATH_PATTERN regex initialization and usage
        let test_string = "Path: /home/user/.android/avd/device.avd";
        let result = PATH_PATTERN.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(
            captures.get(1).unwrap().as_str(),
            "/home/user/.android/avd/device.avd"
        );
    }

    #[test]
    fn test_target_pattern_regex() {
        // Test TARGET_PATTERN regex initialization and usage
        let test_string = "Target: Google APIs";
        let result = TARGET_PATTERN.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "Google APIs");
    }

    #[test]
    fn test_tag_abi_pattern_regex() {
        // Test TAG_ABI_PATTERN regex initialization and usage
        let test_string = "Tag/ABI: google_apis/x86_64";
        let result = TAG_ABI_PATTERN.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "google_apis/x86_64");
    }

    #[test]
    fn test_emulator_serial_regex() {
        // Test EMULATOR_SERIAL regex initialization and usage
        let test_string = "emulator-5554";
        let result = EMULATOR_SERIAL.find(test_string);
        assert!(result.is_some());

        let matched = result.unwrap();
        assert_eq!(matched.as_str(), "emulator-5554");
    }

    #[test]
    fn test_system_image_package_regex() {
        // Test SYSTEM_IMAGE_PACKAGE regex initialization and usage
        let test_string = "system-images;android-34;google_apis;x86_64";
        let result = SYSTEM_IMAGE_PACKAGE.captures(test_string);
        assert!(result.is_some());

        let captures = result.unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "34");
        assert_eq!(captures.get(2).unwrap().as_str(), "google_apis");
        assert_eq!(captures.get(3).unwrap().as_str(), "x86_64");
    }

    #[test]
    fn test_device_name_pattern_string() {
        // Test DEVICE_NAME_PATTERN string constant
        let pattern = regex::Regex::new(DEVICE_NAME_PATTERN).unwrap();

        // Test valid names
        assert!(pattern.is_match("ValidDevice123"));
        assert!(pattern.is_match("device_name-123"));
        assert!(pattern.is_match("device.name"));
        assert!(pattern.is_match("123"));
        assert!(pattern.is_match("test-device_123.avd"));

        // Test invalid names
        assert!(!pattern.is_match("invalid device"));
        assert!(!pattern.is_match("device@name"));
        assert!(!pattern.is_match("device#name"));
        assert!(!pattern.is_match("device/name"));
        assert!(!pattern.is_match("device name"));
    }

    #[test]
    fn test_error_patterns_constants() {
        // Test error pattern constants
        use emu::constants::patterns::errors::*;

        // Test that error constants are non-empty
        assert!(!ERROR_PREFIX.is_empty());
        assert!(!WARNING_PREFIX.is_empty());
        assert!(!LICENSE_NOT_ACCEPTED.is_empty());
        assert!(!PACKAGE_PATH_INVALID.is_empty());
        assert!(!ADB_ERROR.is_empty());
        assert!(!ADB_KO.is_empty());
        assert!(!ADB_UNKNOWN_COMMAND.is_empty());

        // Test specific values
        assert_eq!(ERROR_PREFIX, "Error:");
        assert_eq!(WARNING_PREFIX, "Warning:");
        assert_eq!(ADB_ERROR, "error");
        assert_eq!(ADB_KO, "KO");

        // Test usage in string matching
        let error_message = "Error: Something went wrong";
        assert!(error_message.starts_with(ERROR_PREFIX));

        let warning_message = "Warning: This is a warning";
        assert!(warning_message.starts_with(WARNING_PREFIX));
    }

    #[test]
    fn test_all_regex_patterns_initialized() {
        // Test that all lazy_static regex patterns are properly initialized
        let patterns = vec![
            (&*API_LEVEL_CONFIG, "API_LEVEL_CONFIG"),
            (&*API_LEVEL_TARGET, "API_LEVEL_TARGET"),
            (&*API_LEVEL_BASED_ON, "API_LEVEL_BASED_ON"),
            (&*API_LEVEL_GENERIC, "API_LEVEL_GENERIC"),
            (&*NAME_PATTERN, "NAME_PATTERN"),
            (&*PATH_PATTERN, "PATH_PATTERN"),
            (&*TARGET_PATTERN, "TARGET_PATTERN"),
            (&*TAG_ABI_PATTERN, "TAG_ABI_PATTERN"),
            (&*EMULATOR_SERIAL, "EMULATOR_SERIAL"),
            (&*SYSTEM_IMAGE_PACKAGE, "SYSTEM_IMAGE_PACKAGE"),
        ];

        for (pattern, name) in patterns {
            // Just accessing the pattern should trigger lazy initialization
            let _pattern_str = pattern.as_str();
            assert!(
                !_pattern_str.is_empty(),
                "Pattern {name} should not be empty"
            );
        }
    }

    #[test]
    fn test_regex_patterns_compilation() {
        // Test that all regex patterns compile correctly
        let _config = &*API_LEVEL_CONFIG;
        let _target = &*API_LEVEL_TARGET;
        let _based_on = &*API_LEVEL_BASED_ON;
        let _generic = &*API_LEVEL_GENERIC;
        let _name = &*NAME_PATTERN;
        let _path = &*PATH_PATTERN;
        let _target_pattern = &*TARGET_PATTERN;
        let _tag_abi = &*TAG_ABI_PATTERN;
        let _serial = &*EMULATOR_SERIAL;
        let _system_image = &*SYSTEM_IMAGE_PACKAGE;

        // All patterns should be accessible without panicking
    }

    #[test]
    fn test_regex_patterns_edge_cases() {
        // Test edge cases for regex patterns

        // Empty string should not match
        assert!(!API_LEVEL_CONFIG.is_match(""));
        assert!(!NAME_PATTERN.is_match(""));
        assert!(!EMULATOR_SERIAL.is_match(""));

        // Multiple matches
        let multiple_emulators = "emulator-5554 emulator-5556";
        let matches: Vec<_> = EMULATOR_SERIAL.find_iter(multiple_emulators).collect();
        assert_eq!(matches.len(), 2);

        // Case sensitivity
        assert!(!API_LEVEL_CONFIG.is_match("IMAGE.SYSDIR.1=system-images/android-34/"));
        assert!(API_LEVEL_CONFIG.is_match("image.sysdir.1=system-images/android-34/"));
    }
}
