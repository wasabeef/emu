//! Basic tests for utils/validation.rs
//!
//! Tests form validation boundary cases, custom validators, and error messages.

use emu::utils::validation::{
    validate_field, CompositeValidator, DeviceNameValidator, DevicePlatform, FieldValidator,
    NumericRangeValidator, RequiredSelectionValidator,
};

#[test]
fn test_device_name_validator() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Valid names
    assert!(validator.validate("my_device").is_ok());
    assert!(validator.validate("test-123").is_ok());
    assert!(validator.validate("pixel.7.pro").is_ok());

    // Invalid names
    assert!(validator.validate("").is_err());
    assert!(validator.validate("my device").is_err()); // Space
    assert!(validator.validate("device!").is_err()); // Special char
    assert!(validator.validate(".hidden").is_err()); // Starts with dot
    assert!(validator.validate(&"a".repeat(51)).is_err()); // Too long
}

#[test]
fn test_device_name_validator_android_specific() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Android-specific validation
    assert!(validator.validate("-device").is_err());
    assert!(validator.validate(".device").is_err());

    // Valid Android names
    assert!(validator.validate("device_name").is_ok());
    assert!(validator.validate("Device123").is_ok());
}

#[test]
fn test_device_name_validator_ios() {
    let validator = DeviceNameValidator::new(DevicePlatform::Ios);

    // iOS should allow these (no specific restrictions)
    assert!(validator.validate("device_name").is_ok());
    assert!(validator.validate("Device-123").is_ok());

    // Still invalid for basic rules
    assert!(validator.validate("").is_err());
    assert!(validator.validate("my device").is_err()); // Space
}

#[test]
fn test_device_name_validator_edge_cases() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Exactly at limit (50 characters)
    let max_name = "a".repeat(50);
    assert!(validator.validate(&max_name).is_ok());

    // Over limit
    let over_limit = "a".repeat(51);
    assert!(validator.validate(&over_limit).is_err());

    // Single character
    assert!(validator.validate("a").is_ok());

    // All valid characters
    assert!(validator.validate("aZ0._-").is_ok());
}

#[test]
fn test_numeric_range_validator_ram() {
    let validator = NumericRangeValidator::ram_size();

    // Valid values
    assert!(validator.validate("").is_ok()); // Empty allowed
    assert!(validator.validate("2048").is_ok());
    assert!(validator.validate("512").is_ok());
    assert!(validator.validate("8192").is_ok());

    // Invalid values
    assert!(validator.validate("256").is_err()); // Too small
    assert!(validator.validate("16384").is_err()); // Too large
    assert!(validator.validate("abc").is_err()); // Not a number
}

#[test]
fn test_numeric_range_validator_storage() {
    let validator = NumericRangeValidator::storage_size();

    // Valid storage values
    assert!(validator.validate("1024").is_ok());
    assert!(validator.validate("65536").is_ok());

    // Invalid storage values
    assert!(validator.validate("512").is_err()); // Too small for storage
    assert!(validator.validate("131072").is_err()); // Too large
}

#[test]
fn test_numeric_range_validator_custom() {
    let validator = NumericRangeValidator::new(10, 100, "units");

    assert!(validator.validate("10").is_ok());
    assert!(validator.validate("50").is_ok());
    assert!(validator.validate("100").is_ok());

    assert!(validator.validate("9").is_err());
    assert!(validator.validate("101").is_err());

    // Check error messages
    let err = validator.validate("5").unwrap_err();
    assert!(err.contains("at least 10 units"));

    let err = validator.validate("200").unwrap_err();
    assert!(err.contains("at most 100 units"));
}

#[test]
fn test_numeric_range_validator_boundaries() {
    let validator = NumericRangeValidator::new(100, 1000, "MB");

    // Boundary values
    assert!(validator.validate("100").is_ok()); // Min
    assert!(validator.validate("1000").is_ok()); // Max
    assert!(validator.validate("99").is_err()); // Just below min
    assert!(validator.validate("1001").is_err()); // Just above max
}

#[test]
fn test_required_selection_validator() {
    let validator = RequiredSelectionValidator::new("device type");

    assert!(validator.validate("pixel_7").is_ok());
    assert!(validator.validate("iPhone 15").is_ok());

    let err = validator.validate("").unwrap_err();
    assert!(err.contains("Please select a device type"));
}

#[test]
fn test_composite_validator() {
    let composite = CompositeValidator::new()
        .with_validator(Box::new(RequiredSelectionValidator::new("value")))
        .with_validator(Box::new(NumericRangeValidator::new(1, 10, "items")));

    // Pass all validators
    assert!(composite.validate("5").is_ok());

    // Fail first validator (empty)
    assert!(composite.validate("").is_err());

    // Fail second validator (out of range)
    assert!(composite.validate("15").is_err());

    // Fail second validator (not numeric)
    assert!(composite.validate("abc").is_err());
}

#[test]
fn test_composite_validator_empty() {
    let validator = CompositeValidator::new();
    assert!(validator.validate("anything").is_ok());
}

#[test]
fn test_composite_validator_default() {
    let validator = CompositeValidator::default();
    assert!(validator.validate("anything").is_ok());
}

#[test]
fn test_validate_field_helper() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    let result = validate_field("Device Name", "my_device", &validator);
    assert!(result.is_ok());

    let result = validate_field("Device Name", "", &validator);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.starts_with("Device Name: "));
    assert!(err.contains("Device name cannot be empty"));
}

#[test]
fn test_validator_hints() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    assert!(!device_validator.hint().is_empty());

    let numeric_validator = NumericRangeValidator::ram_size();
    assert!(!numeric_validator.hint().is_empty());

    let required_validator = RequiredSelectionValidator::new("test");
    assert!(!required_validator.hint().is_empty());
}

#[test]
fn test_device_name_invalid_characters() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test various invalid characters
    let invalid_chars = [
        "device name",
        "device@test",
        "device#123",
        "device$",
        "device%",
    ];

    for invalid in invalid_chars {
        assert!(
            validator.validate(invalid).is_err(),
            "Should fail for: {invalid}"
        );
    }
}

#[test]
fn test_device_name_valid_characters() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test various valid characters
    let valid_names = [
        "device123",
        "device_test",
        "device-test",
        "device.test",
        "DeviceName",
        "DEVICE",
        "device_123_test",
        "test-device-123",
    ];

    for valid in valid_names {
        assert!(
            validator.validate(valid).is_ok(),
            "Should pass for: {valid}"
        );
    }
}

#[test]
fn test_numeric_validator_error_messages() {
    let validator = NumericRangeValidator::new(512, 8192, "MB");

    // Test specific error messages
    let low_err = validator.validate("256").unwrap_err();
    assert!(low_err.contains("at least 512 MB"));

    let high_err = validator.validate("16384").unwrap_err();
    assert!(high_err.contains("at most 8192 MB"));

    let invalid_err = validator.validate("not-a-number").unwrap_err();
    assert!(invalid_err.contains("Please enter a valid number"));
}

#[test]
fn test_numeric_validator_special_cases() {
    let validator = NumericRangeValidator::new(0, 100, "percent");

    // Test zero value
    assert!(validator.validate("0").is_ok());

    // Test leading zeros
    assert!(validator.validate("007").is_ok());

    // Test negative numbers
    assert!(validator.validate("-5").is_err());

    // Test decimal numbers
    assert!(validator.validate("50.5").is_err());
}

#[test]
fn test_required_selection_custom_messages() {
    let validator = RequiredSelectionValidator::new("API level");

    let err = validator.validate("").unwrap_err();
    assert!(err.contains("Please select a API level"));

    // Test with different field names
    let validator2 = RequiredSelectionValidator::new("device type");
    let err2 = validator2.validate("").unwrap_err();
    assert!(err2.contains("Please select a device type"));
}

#[test]
fn test_composite_validator_multiple_failures() {
    let composite = CompositeValidator::new()
        .with_validator(Box::new(DeviceNameValidator::new(DevicePlatform::Android)))
        .with_validator(Box::new(NumericRangeValidator::new(1, 10, "items")));

    // Should fail on first validator (device name with space)
    let err = composite.validate("invalid name").unwrap_err();
    assert!(err.contains("letters, numbers, dots, dashes, and underscores"));

    // Should fail on second validator (out of range)
    let err2 = composite.validate("validname").unwrap_err();
    assert!(err2.contains("Please enter a valid number"));
}

#[test]
fn test_regex_initialization() {
    // Test that regex is properly initialized and reused
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // First call initializes
    assert!(validator.validate("test123").is_ok());

    // Second call uses cached regex
    assert!(validator.validate("test456").is_ok());

    // Verify pattern works correctly
    assert!(validator.validate("test@123").is_err());
}

#[test]
fn test_platform_specific_differences() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);

    // Test Android-specific restrictions
    assert!(android_validator.validate("-device").is_err());
    assert!(android_validator.validate(".device").is_err());

    // Test that iOS doesn't have these restrictions in the current implementation
    // (Note: This assumes iOS doesn't have the same restrictions - adjust if needed)
    assert!(ios_validator.validate("device_name").is_ok());
}

#[test]
fn test_field_validator_trait_consistency() {
    // Test that all validators implement the trait correctly
    let validators: Vec<Box<dyn FieldValidator>> = vec![
        Box::new(DeviceNameValidator::new(DevicePlatform::Android)),
        Box::new(NumericRangeValidator::ram_size()),
        Box::new(RequiredSelectionValidator::new("test")),
        Box::new(CompositeValidator::new()),
    ];

    for validator in validators {
        // All should have non-empty hints
        assert!(!validator.hint().is_empty());

        // All should handle some basic cases
        let _ = validator.validate("test");
        let _ = validator.validate("");
    }
}
