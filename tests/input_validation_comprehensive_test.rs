//! Comprehensive input validation tests for Emu
//!
//! This module contains extensive tests for all input validation scenarios
//! across the application, ensuring robust protection against invalid data
//! and edge cases in user input handling.

use emu::utils::validation::{
    CompositeValidator, DeviceNameValidator, DevicePlatform, FieldValidator, NumericRangeValidator,
    RequiredSelectionValidator,
};

/// Test device name validation with comprehensive edge cases
#[test]
fn test_device_name_validation_comprehensive() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);

    // Valid device names
    let valid_names = vec![
        "my_device",
        "test-device-123",
        "pixel.7.pro",
        "device_2024",
        "AVD-test",
        "emulator.test",
    ];

    for name in valid_names {
        assert!(
            android_validator.validate(name).is_ok(),
            "Valid name rejected: {name}"
        );
        assert!(
            ios_validator.validate(name).is_ok(),
            "Valid name rejected on iOS: {name}"
        );
    }

    // Invalid device names
    let long_name_1 = "a".repeat(51);
    let long_name_2 = "x".repeat(100);
    let long_name_3 = "device_".repeat(20);

    let invalid_names = vec![
        // Empty and whitespace
        "",
        " ",
        "   ",
        "\t",
        "\n",
        "\r\n",
        // Special characters
        "device!",
        "device@home",
        "device#1",
        "device$money",
        "device%percent",
        "device^caret",
        "device&and",
        "device*star",
        "device(paren)",
        "device[bracket]",
        "device{brace}",
        "device|pipe",
        "device\\backslash",
        "device/slash",
        "device:colon",
        "device;semicolon",
        "device<less",
        "device>greater",
        "device?question",
        "device\"quote",
        "device'apostrophe",
        // Control characters
        "device\0null",
        "device\x01soh",
        "device\x1bescape",
        "device\x7fdelete",
        // Unicode edge cases
        "device🚀rocket",
        "device♠spade",
        "device™trademark",
        "device©copyright",
        // Length violations
        &long_name_1.as_str(), // Too long (assuming 50 char limit)
        &long_name_2.as_str(),
        &long_name_3.as_str(),
        // Path-like patterns
        "../device",
        "./device",
        "device/../other",
        "/etc/device",
        "C:\\device",
        // Starting with invalid characters (Android)
        ".hidden_device",
        "-device",
        // "_device", // Underscore start is actually valid
    ];

    for name in &invalid_names {
        let android_result = android_validator.validate(name);
        assert!(
            android_result.is_err(),
            "Invalid name accepted on Android: '{}' - {:?}",
            name.escape_default(),
            android_result
        );
    }
}

/// Test numeric range validation for RAM and storage
#[test]
fn test_numeric_range_validation_comprehensive() {
    let ram_validator = NumericRangeValidator::ram_size();
    let storage_validator = NumericRangeValidator::storage_size();

    // Test RAM validation (typical range: 512-8192 MB)
    let ram_tests = vec![
        // Valid values
        ("512", true),
        ("1024", true),
        ("2048", true),
        ("4096", true),
        ("8192", true),
        ("", true), // Empty allowed (uses default)
        // Invalid values
        ("0", false),
        ("256", false),    // Too small
        ("16384", false),  // Too large
        ("-1024", false),  // Negative
        ("1024.5", false), // Decimal
        ("2GB", false),    // With unit
        ("abc", false),    // Non-numeric
        ("1e3", false),    // Scientific notation
        ("0x400", false),  // Hex
        ("1,024", false),  // With separator
        ("1 024", false),  // With space
        ("+2048", true),   // With plus sign (Rust parse accepts it)
        ("١٠٢٤", false),   // Arabic numerals
        ("1024\0", false), // With null
        ("10 24", false),  // Space in middle
    ];

    for (input, should_pass) in ram_tests {
        let result = ram_validator.validate(input);
        assert_eq!(
            result.is_ok(),
            should_pass,
            "RAM validation failed for '{}': {:?}",
            input.escape_default(),
            result
        );
    }

    // Test storage validation (typical range: 1024-65536 MB)
    let storage_tests = vec![
        // Valid values
        ("1024", true),
        ("2048", true),
        ("4096", true),
        ("8192", true),
        ("16384", true),
        ("32768", true),
        ("", true), // Empty allowed
        // Invalid values
        ("512", false),          // Too small
        ("131072", false),       // Too large
        ("storage", false),      // Text
        ("10.5GB", false),       // Unit and decimal
        ("\u{200B}4096", false), // Zero-width space prefix
        ("4096\u{200B}", false), // Zero-width space suffix
    ];

    for (input, should_pass) in storage_tests {
        let result = storage_validator.validate(input);
        assert_eq!(
            result.is_ok(),
            should_pass,
            "Storage validation failed for '{}': {:?}",
            input.escape_default(),
            result
        );
    }
}

/// Test required field validation
#[test]
fn test_required_field_validation() {
    let device_type_validator = RequiredSelectionValidator::new("device type");
    let api_level_validator = RequiredSelectionValidator::new("API level");

    // Empty values should fail
    assert!(device_type_validator.validate("").is_err());
    assert!(api_level_validator.validate("").is_err());

    // Non-empty values (including whitespace) should pass
    // The validator only checks for empty string, not trimmed content
    assert!(device_type_validator.validate(" ").is_ok());
    assert!(device_type_validator.validate("\t").is_ok());
    assert!(device_type_validator.validate("\n").is_ok());

    // Any non-empty value should pass
    assert!(device_type_validator.validate("phone").is_ok());
    assert!(device_type_validator.validate("tablet").is_ok());
    assert!(api_level_validator.validate("30").is_ok());
    assert!(api_level_validator.validate("34").is_ok());
}

/// Test composite validation with multiple rules
#[test]
fn test_composite_validation() {
    // Create a composite validator for a field that must be:
    // 1. Non-empty (required)
    // 2. Within numeric range
    let composite = CompositeValidator::new()
        .with_validator(Box::new(RequiredSelectionValidator::new("value")))
        .with_validator(Box::new(NumericRangeValidator::new(100, 1000, "units")));

    // Test various inputs
    assert!(composite.validate("").is_err()); // Empty fails required check
    assert!(composite.validate("500").is_ok()); // Valid
    assert!(composite.validate("50").is_err()); // Too small
    assert!(composite.validate("2000").is_err()); // Too large
    assert!(composite.validate("abc").is_err()); // Non-numeric
}

/// Test validation error messages
#[test]
fn test_validation_error_messages() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ram_validator = NumericRangeValidator::ram_size();
    let required_validator = RequiredSelectionValidator::new("device type");

    // Test empty device name error
    let err = device_validator.validate("").unwrap_err();
    assert!(err.contains("empty") || err.contains("required"));

    // Test special character error
    let err = device_validator.validate("device!").unwrap_err();
    assert!(err.contains("character") || err.contains("invalid") || err.contains("letters"));

    // Test numeric range error
    let err = ram_validator.validate("100").unwrap_err();
    assert!(err.contains("at least") || err.contains("minimum"));

    // Test required field error
    let err = required_validator.validate("").unwrap_err();
    assert!(err.contains("select") || err.contains("required"));
}

/// Test hint messages
#[test]
fn test_validation_hints() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ram_validator = NumericRangeValidator::ram_size();
    let required_validator = RequiredSelectionValidator::new("device type");

    // All validators should provide helpful hints
    assert!(!device_validator.hint().is_empty());
    assert!(!ram_validator.hint().is_empty());
    assert!(!required_validator.hint().is_empty());
}

/// Test boundary conditions for numeric validation
#[test]
fn test_numeric_boundary_validation() {
    // Test exact boundaries
    let validator = NumericRangeValidator::new(100, 200, "units");

    assert!(validator.validate("99").is_err()); // Just below minimum
    assert!(validator.validate("100").is_ok()); // Exact minimum
    assert!(validator.validate("200").is_ok()); // Exact maximum
    assert!(validator.validate("201").is_err()); // Just above maximum

    // Test edge cases around parsing
    assert!(validator.validate("0100").is_ok()); // Leading zeros
    assert!(validator.validate(" 150 ").is_err()); // Spaces
    assert!(validator.validate("1_5_0").is_err()); // Underscores
}

/// Test internationalization and locale-specific inputs
#[test]
fn test_internationalization_validation() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let numeric_validator = NumericRangeValidator::new(1000, 9999, "units");

    // Non-ASCII but valid characters
    let i18n_device_names = vec![
        // These should typically be rejected for device names
        ("gerät", false),      // German
        ("dispositivo", true), // Spanish (ASCII)
        ("устройство", false), // Russian
        ("デバイス", false),   // Japanese
        ("设备", false),       // Chinese
        ("جهاز", false),       // Arabic
    ];

    for (name, should_pass) in i18n_device_names {
        let result = device_validator.validate(name);
        assert_eq!(
            result.is_ok(),
            should_pass,
            "i18n validation mismatch for '{name}': {result:?}"
        );
    }

    // Locale-specific number formats (should all fail)
    let i18n_numbers = vec![
        "1,234",          // Thousand separator
        "1.234",          // European thousand separator
        "١٢٣٤",           // Arabic numerals
        "一千二百三十四", // Chinese numerals
        "1 234",          // Space separator
        "1'234",          // Swiss separator
    ];

    for number in i18n_numbers {
        assert!(
            numeric_validator.validate(number).is_err(),
            "Locale-specific number accepted: '{number}'"
        );
    }
}

/// Test SQL injection patterns in input validation
#[test]
fn test_sql_injection_patterns() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);

    let sql_patterns = vec![
        "device'; DROP TABLE devices--",
        "device' OR '1'='1",
        "device'); DELETE FROM users--",
        "device\" OR 1=1--",
        "device`; DROP DATABASE--",
        "device'||'",
        "device' UNION SELECT * FROM passwords--",
    ];

    for pattern in sql_patterns {
        assert!(
            device_validator.validate(pattern).is_err(),
            "SQL injection pattern accepted: {pattern}"
        );
    }
}

/// Test XSS patterns in input validation
#[test]
fn test_xss_patterns() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);

    let xss_patterns = vec![
        "<script>alert('xss')</script>",
        "device<img src=x onerror=alert(1)>",
        "device\"><script>alert(1)</script>",
        "device&lt;script&gt;alert(1)&lt;/script&gt;",
        "javascript:alert(1)",
        "device onmouseover=alert(1)",
    ];

    for pattern in xss_patterns {
        assert!(
            device_validator.validate(pattern).is_err(),
            "XSS pattern accepted: {pattern}"
        );
    }
}

/// Test LDAP injection patterns
#[test]
fn test_ldap_injection_patterns() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);

    let ldap_patterns = vec![
        "device)(cn=*",
        "device)(|(cn=*",
        "device*)(uid=*",
        "device\\",
        "device)(password=*)",
    ];

    for pattern in ldap_patterns {
        assert!(
            device_validator.validate(pattern).is_err(),
            "LDAP injection pattern accepted: {pattern}"
        );
    }
}

/// Test comprehensive edge case validation
#[test]
fn test_comprehensive_edge_cases() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let numeric_validator = NumericRangeValidator::new(1, 100, "percent");

    // Mixed valid/invalid characters
    assert!(device_validator.validate("device-123_test.avd").is_ok());
    assert!(device_validator.validate("device 123").is_err()); // Space
    assert!(device_validator.validate("device\t123").is_err()); // Tab

    // Numeric edge cases
    assert!(numeric_validator.validate("00001").is_ok()); // Leading zeros
    assert!(numeric_validator.validate("+50").is_ok()); // Plus sign (accepted by parse)
    assert!(numeric_validator.validate("50.0").is_err()); // Decimal
    assert!(numeric_validator.validate("5e1").is_err()); // Scientific

    // Empty vs whitespace
    assert!(numeric_validator.validate("").is_ok()); // Empty is allowed
    assert!(numeric_validator.validate(" ").is_err()); // Space is not empty
    assert!(numeric_validator.validate("\u{00A0}").is_err()); // Non-breaking space
}

/// Test validation performance with large inputs
#[test]
fn test_validation_performance() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test with increasingly large inputs
    let sizes = vec![100, 1000, 10000, 100000];

    for size in sizes {
        let large_input = "a".repeat(size);
        let start = std::time::Instant::now();
        let result = device_validator.validate(&large_input);
        let duration = start.elapsed();

        // Validation should complete quickly even for large inputs
        assert!(
            duration.as_millis() < 100,
            "Validation too slow for size {size}: {duration:?}"
        );
        assert!(result.is_err()); // Should fail due to length
    }
}

/// Integration test combining multiple validators
#[test]
fn test_validation_integration() {
    // Simulate complete device creation validation
    let name_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ram_validator = NumericRangeValidator::ram_size();
    let storage_validator = NumericRangeValidator::storage_size();
    let device_type_validator = RequiredSelectionValidator::new("device type");
    let api_level_validator = RequiredSelectionValidator::new("API level");

    // Valid device configuration
    assert!(name_validator.validate("test_device").is_ok());
    assert!(ram_validator.validate("2048").is_ok());
    assert!(storage_validator.validate("8192").is_ok());
    assert!(device_type_validator.validate("phone").is_ok());
    assert!(api_level_validator.validate("30").is_ok());

    // Invalid configurations
    assert!(name_validator.validate("test device!").is_err());
    assert!(ram_validator.validate("100MB").is_err());
    assert!(storage_validator.validate("unlimited").is_err());
    assert!(device_type_validator.validate("").is_err());
    assert!(api_level_validator.validate(" ").is_ok()); // Non-empty passes
}
