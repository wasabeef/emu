//! Tests for managers/common.rs
//!
//! Tests focus on device name sanitization functions and utility methods
//! that are critical for safe filesystem and command-line operations.

use emu::managers::common::{
    format_device_name, sanitize_device_name, sanitize_device_name_for_command,
};

#[cfg(test)]
mod format_device_name_tests {
    use super::*;

    #[test]
    fn test_short_name_unchanged() {
        let result = format_device_name("short", 20);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_exact_length_unchanged() {
        let result = format_device_name("exactly20characters", 20);
        assert_eq!(result, "exactly20characters");
    }

    #[test]
    fn test_long_name_truncated() {
        let result = format_device_name("this_is_a_very_long_device_name", 15);
        assert_eq!(result, "this_is_a_ve...");
    }

    #[test]
    fn test_very_short_max_length() {
        let result = format_device_name("long_name", 3);
        assert_eq!(result, "...");
    }

    #[test]
    fn test_empty_name() {
        let result = format_device_name("", 10);
        assert_eq!(result, "");
    }

    #[test]
    fn test_unicode_characters() {
        let result = format_device_name("デバイス名前", 8);
        assert_eq!(result, "デバイス...");
    }
}

#[cfg(test)]
mod sanitize_device_name_tests {
    use super::*;

    #[test]
    fn test_alphanumeric_preserved() {
        let result = sanitize_device_name("Device123");
        assert_eq!(result, "Device123");
    }

    #[test]
    fn test_allowed_special_chars_preserved() {
        let result = sanitize_device_name("Test-Device_v1.0");
        assert_eq!(result, "Test-Device_v1.0");
    }

    #[test]
    fn test_spaces_replaced() {
        let result = sanitize_device_name("My Device Name");
        assert_eq!(result, "My_Device_Name");
    }

    #[test]
    fn test_unsafe_chars_replaced() {
        let result = sanitize_device_name("Device@#$%^&*()");
        assert_eq!(result, "Device__________");
    }

    #[test]
    fn test_mixed_safe_unsafe_chars() {
        let result = sanitize_device_name("API-Level@30#test");
        assert_eq!(result, "API-Level_30_test");
    }

    #[test]
    fn test_empty_string() {
        let result = sanitize_device_name("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_only_special_chars() {
        let result = sanitize_device_name("@#$%^&*()");
        assert_eq!(result, "__________");
    }

    #[test]
    fn test_unicode_replacement() {
        let result = sanitize_device_name("デバイス");
        assert_eq!(result, "____");
    }
}

#[cfg(test)]
mod sanitize_device_name_for_command_tests {
    use super::*;

    #[test]
    fn test_quotes_removed() {
        let result = sanitize_device_name_for_command("'Device Name'");
        assert_eq!(result, "DeviceName");
    }

    #[test]
    fn test_spaces_removed() {
        let result = sanitize_device_name_for_command("My Device");
        assert_eq!(result, "MyDevice");
    }

    #[test]
    fn test_tabs_newlines_removed() {
        let result = sanitize_device_name_for_command("Device\tName\nTest");
        assert_eq!(result, "DeviceNameTest");
    }

    #[test]
    fn test_screen_size_format() {
        let result = sanitize_device_name_for_command("2.7\" QVGA API 36");
        assert_eq!(result, "2.7QVGAAPI36");
    }

    #[test]
    fn test_complex_device_name() {
        let result = sanitize_device_name_for_command("Pixel 4 API 30 (Google Play)");
        assert_eq!(result, "Pixel4API30GooglePlay");
    }

    #[test]
    fn test_leading_trailing_special_chars_trimmed() {
        let result = sanitize_device_name_for_command("_Device_Name_");
        assert_eq!(result, "Device_Name");
    }

    #[test]
    fn test_only_special_chars_empty() {
        let result = sanitize_device_name_for_command("___");
        assert_eq!(result, "");
    }

    #[test]
    fn test_mixed_quotes_and_spaces() {
        let result = sanitize_device_name_for_command("\"Galaxy Nexus\" API 30");
        assert_eq!(result, "GalaxyNexusAPI30");
    }

    #[test]
    fn test_preserve_alphanumeric_hyphen_underscore_dot() {
        let result = sanitize_device_name_for_command("device-name_v1.0");
        assert_eq!(result, "device-name_v1.0");
    }

    #[test]
    fn test_empty_string() {
        let result = sanitize_device_name_for_command("");
        assert_eq!(result, "");
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_very_long_name_sanitization() {
        let long_name = "a".repeat(1000);
        let result = sanitize_device_name(&long_name);
        assert_eq!(result, long_name); // All 'a' characters are valid
    }

    #[test]
    fn test_format_then_sanitize() {
        let original = "Very Long Device Name With Special@Chars";
        let formatted = format_device_name(original, 20);
        let sanitized = sanitize_device_name(&formatted);
        assert_eq!(sanitized, "Very_Long_Device_...");
    }

    #[test]
    fn test_sanitize_then_format() {
        let original = "Device@Name#Test";
        let sanitized = sanitize_device_name(original);
        let formatted = format_device_name(&sanitized, 10);
        assert_eq!(formatted, "Device_Nam...");
    }

    #[test]
    fn test_command_sanitize_edge_cases() {
        // Test various edge cases for command sanitization
        let test_cases = vec![
            ("", ""),
            ("   ", ""),
            ("abc", "abc"),
            ("'\"abc\"'", "abc"),
            ("__abc__", "abc"),
            ("123abc", "123abc"),
            ("abc123", "abc123"),
            ("a_b-c.d", "a_b-c.d"),
        ];

        for (input, expected) in test_cases {
            let result = sanitize_device_name_for_command(input);
            assert_eq!(result, expected, "Failed for input: '{}'", input);
        }
    }
}