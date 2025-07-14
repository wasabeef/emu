//! Comprehensive tests for utils::validation module
//!
//! These tests ensure complete coverage of all validation types,
//! edge cases, and error conditions for 100% code coverage.

use emu::utils::validation::{
    validate_field, CompositeValidator, DeviceNameValidator, DevicePlatform, FieldValidator,
    NumericRangeValidator, RequiredSelectionValidator,
};

#[test]
fn test_device_name_validator_creation() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);

    // Test that validators are created properly
    assert_eq!(
        android_validator.hint(),
        "Letters, numbers, dots, dashes, and underscores only"
    );
    assert_eq!(
        ios_validator.hint(),
        "Letters, numbers, dots, dashes, and underscores only"
    );
}

#[test]
fn test_device_name_validator_android_specific() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test Android-specific rules
    assert!(validator.validate(".hidden").is_err()); // Starts with dot
    assert!(validator.validate("-device").is_err()); // Starts with hyphen

    // Test valid Android names
    assert!(validator.validate("my_device").is_ok());
    assert!(validator.validate("test123").is_ok());
    assert!(validator.validate("pixel.7.pro").is_ok());
    assert!(validator.validate("device-name").is_ok());
}

#[test]
fn test_device_name_validator_ios_specific() {
    let validator = DeviceNameValidator::new(DevicePlatform::Ios);

    // Test that iOS doesn't have the same restrictions as Android
    // (Currently no iOS-specific rules, but structure is there)
    assert!(validator.validate("my_device").is_ok());
    assert!(validator.validate("test123").is_ok());
    assert!(validator.validate("iPhone.15.Pro").is_ok());
}

#[test]
fn test_device_name_validator_all_error_cases() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test empty name
    let result = validator.validate("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Device name cannot be empty");

    // Test too long name
    let long_name = "a".repeat(51);
    let result = validator.validate(&long_name);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("50 characters or less"));

    // Test invalid characters
    let result = validator.validate("device with spaces");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device name can only contain letters, numbers, dots, dashes, and underscores"
    );

    // Test starts with dot (Android)
    let result = validator.validate(".device");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device name cannot start with '.' or '-'"
    );

    // Test starts with hyphen (Android)
    let result = validator.validate("-device");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device name cannot start with '.' or '-'"
    );
}

#[test]
fn test_device_name_validator_edge_cases() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test exactly 50 characters (should be valid)
    let max_name = "a".repeat(50);
    assert!(validator.validate(&max_name).is_ok());

    // Test exactly 51 characters (should be invalid)
    let over_max_name = "a".repeat(51);
    assert!(validator.validate(&over_max_name).is_err());

    // Test single character
    assert!(validator.validate("a").is_ok());

    // Test all valid characters
    assert!(validator.validate("abcABC123_-.").is_ok());
}

#[test]
fn test_numeric_range_validator_creation() {
    let ram_validator = NumericRangeValidator::ram_size();
    let storage_validator = NumericRangeValidator::storage_size();
    let custom_validator = NumericRangeValidator::new(1, 100, "GB");

    // Test hints
    assert_eq!(
        ram_validator.hint(),
        "Enter a number or leave empty for default"
    );
    assert_eq!(
        storage_validator.hint(),
        "Enter a number or leave empty for default"
    );
    assert_eq!(
        custom_validator.hint(),
        "Enter a number or leave empty for default"
    );
}

#[test]
fn test_numeric_range_validator_ram_size() {
    let validator = NumericRangeValidator::ram_size();

    // Test valid values
    assert!(validator.validate("").is_ok()); // Empty is allowed
    assert!(validator.validate("512").is_ok());
    assert!(validator.validate("2048").is_ok());
    assert!(validator.validate("8192").is_ok());

    // Test invalid values
    assert!(validator.validate("256").is_err()); // Too small
    assert!(validator.validate("16384").is_err()); // Too large
    assert!(validator.validate("abc").is_err()); // Not a number
    assert!(validator.validate("12.5").is_err()); // Decimal
    assert!(validator.validate("-512").is_err()); // Negative
}

#[test]
fn test_numeric_range_validator_storage_size() {
    let validator = NumericRangeValidator::storage_size();

    // Test valid values
    assert!(validator.validate("").is_ok()); // Empty is allowed
    assert!(validator.validate("1024").is_ok());
    assert!(validator.validate("2048").is_ok());
    assert!(validator.validate("65536").is_ok());

    // Test invalid values
    assert!(validator.validate("512").is_err()); // Too small (min is 1024)
    assert!(validator.validate("131072").is_err()); // Too large
    assert!(validator.validate("not_a_number").is_err());
}

#[test]
fn test_numeric_range_validator_custom_range() {
    let validator = NumericRangeValidator::new(10, 100, "GB");

    // Test valid values
    assert!(validator.validate("").is_ok()); // Empty is allowed
    assert!(validator.validate("10").is_ok());
    assert!(validator.validate("50").is_ok());
    assert!(validator.validate("100").is_ok());

    // Test invalid values
    let result = validator.validate("5");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 10 GB"));

    let result = validator.validate("101");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at most 100 GB"));
}

#[test]
fn test_numeric_range_validator_error_messages() {
    let validator = NumericRangeValidator::new(512, 8192, "MB");

    // Test minimum error message
    let result = validator.validate("256");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Value must be at least 512 MB");

    // Test maximum error message
    let result = validator.validate("16384");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Value must be at most 8192 MB");

    // Test invalid number error message
    let result = validator.validate("invalid");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Please enter a valid number");
}

#[test]
fn test_numeric_range_validator_boundary_values() {
    let validator = NumericRangeValidator::new(100, 1000, "units");

    // Test exact boundary values
    assert!(validator.validate("100").is_ok()); // Minimum
    assert!(validator.validate("1000").is_ok()); // Maximum

    // Test just outside boundaries
    assert!(validator.validate("99").is_err()); // Below minimum
    assert!(validator.validate("1001").is_err()); // Above maximum
}

#[test]
fn test_required_selection_validator_creation() {
    let validator = RequiredSelectionValidator::new("device type");

    // Test hint
    assert_eq!(validator.hint(), "Required field");
}

#[test]
fn test_required_selection_validator_validation() {
    let validator = RequiredSelectionValidator::new("API level");

    // Test valid selection
    assert!(validator.validate("30").is_ok());
    assert!(validator.validate("Android 14").is_ok());
    assert!(validator.validate("any_value").is_ok());

    // Test invalid selection (empty)
    let result = validator.validate("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Please select a API level");
}

#[test]
fn test_required_selection_validator_different_fields() {
    let device_validator = RequiredSelectionValidator::new("device type");
    let api_validator = RequiredSelectionValidator::new("API level");

    // Test different field names in error messages
    let result = device_validator.validate("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Please select a device type");

    let result = api_validator.validate("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Please select a API level");
}

#[test]
fn test_composite_validator_creation() {
    let validator = CompositeValidator::new();

    // Test initial state
    assert_eq!(validator.hint(), "Enter a value");
    assert!(validator.validate("test").is_ok());
}

#[test]
fn test_composite_validator_default() {
    let validator = CompositeValidator::default();

    // Test default implementation
    assert_eq!(validator.hint(), "Enter a value");
    assert!(validator.validate("test").is_ok());
}

#[test]
fn test_composite_validator_with_single_validator() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let composite = CompositeValidator::new().with_validator(Box::new(device_validator));

    // Test that composite inherits first validator's hint
    assert_eq!(
        composite.hint(),
        "Letters, numbers, dots, dashes, and underscores only"
    );

    // Test validation
    assert!(composite.validate("valid_name").is_ok());
    assert!(composite.validate("invalid name").is_err());
}

#[test]
fn test_composite_validator_with_multiple_validators() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let required_validator = RequiredSelectionValidator::new("device name");

    let composite = CompositeValidator::new()
        .with_validator(Box::new(required_validator))
        .with_validator(Box::new(device_validator));

    // Test that all validators are applied
    assert!(composite.validate("").is_err()); // Fails required check
    assert!(composite.validate("invalid name").is_err()); // Fails device name check
    assert!(composite.validate("valid_name").is_ok()); // Passes all checks
}

#[test]
fn test_composite_validator_early_exit() {
    let required_validator = RequiredSelectionValidator::new("field");
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);

    let composite = CompositeValidator::new()
        .with_validator(Box::new(required_validator))
        .with_validator(Box::new(device_validator));

    // Test that validation stops at first error
    let result = composite.validate("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Please select a field");
}

#[test]
fn test_composite_validator_chaining() {
    let composite = CompositeValidator::new()
        .with_validator(Box::new(RequiredSelectionValidator::new("test")))
        .with_validator(Box::new(DeviceNameValidator::new(DevicePlatform::Android)))
        .with_validator(Box::new(NumericRangeValidator::new(1, 100, "test")));

    // Test chaining multiple validators
    assert!(composite.validate("").is_err()); // First validator fails
    assert!(composite.validate("invalid name").is_err()); // Second validator fails
    assert!(composite.validate("valid_name").is_err()); // Third validator fails (not numeric)
    assert!(composite.validate("50").is_ok()); // All validators pass
}

#[test]
fn test_validate_field_helper_function() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test successful validation
    let result = validate_field("Device Name", "valid_name", &validator);
    assert!(result.is_ok());

    // Test validation with error
    let result = validate_field("Device Name", "invalid name", &validator);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device Name: Device name can only contain letters, numbers, dots, dashes, and underscores"
    );

    // Test with empty value
    let result = validate_field("Device Name", "", &validator);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device Name: Device name cannot be empty"
    );
}

#[test]
fn test_validate_field_different_field_names() {
    let validator = RequiredSelectionValidator::new("selection");

    // Test different field names
    let result = validate_field("API Level", "", &validator);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "API Level: Please select a selection");

    let result = validate_field("Device Type", "", &validator);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Device Type: Please select a selection"
    );
}

#[test]
fn test_device_platform_debug() {
    let android = DevicePlatform::Android;
    let ios = DevicePlatform::Ios;

    // Test debug formatting
    let android_debug = format!("{android:?}");
    let ios_debug = format!("{ios:?}");

    assert_eq!(android_debug, "Android");
    assert_eq!(ios_debug, "Ios");
}

#[test]
fn test_device_platform_clone_copy() {
    let android = DevicePlatform::Android;
    let ios = DevicePlatform::Ios;

    // Test clone
    let android_clone = android;
    let ios_clone = ios;

    assert_eq!(android_clone as u8, android as u8);
    assert_eq!(ios_clone as u8, ios as u8);

    // Test copy
    let android_copy = android;
    let ios_copy = ios;

    assert_eq!(android_copy as u8, android as u8);
    assert_eq!(ios_copy as u8, ios as u8);
}

#[test]
fn test_validation_with_unicode_characters() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);

    // Test various Unicode characters (should be invalid)
    assert!(validator.validate("デバイス").is_err()); // Japanese
    assert!(validator.validate("устройство").is_err()); // Russian
    assert!(validator.validate("device™").is_err()); // Trademark symbol
    assert!(validator.validate("device→test").is_err()); // Arrow

    // Test valid ASCII characters
    assert!(validator.validate("device123").is_ok());
    assert!(validator.validate("test_device").is_ok());
}

#[test]
fn test_validation_regex_initialization() {
    let validator1 = DeviceNameValidator::new(DevicePlatform::Android);
    let validator2 = DeviceNameValidator::new(DevicePlatform::Ios);

    // Test that regex is properly initialized and reused
    assert!(validator1.validate("test_device").is_ok());
    assert!(validator2.validate("test_device").is_ok());

    // Test that regex catches invalid patterns
    assert!(validator1.validate("invalid!").is_err());
    assert!(validator2.validate("invalid!").is_err());
}

#[test]
fn test_numeric_validator_overflow_edge_cases() {
    let validator = NumericRangeValidator::new(1, 1000, "units");

    // Test very large numbers (should be parsed and checked against max)
    assert!(validator.validate("999999999999999999999999").is_err());

    // Test zero
    assert!(validator.validate("0").is_err()); // Below minimum

    // Test whitespace
    assert!(validator.validate("  123  ").is_err()); // Trimming not handled

    // Test scientific notation
    assert!(validator.validate("1e3").is_err()); // Not valid u32
}

#[test]
fn test_all_validator_combinations() {
    // Test all validator types together
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let numeric_validator = NumericRangeValidator::ram_size();
    let required_validator = RequiredSelectionValidator::new("test");

    // Test device validator
    assert!(device_validator.validate("valid_device").is_ok());
    assert!(device_validator.validate("").is_err());

    // Test numeric validator
    assert!(numeric_validator.validate("2048").is_ok());
    assert!(numeric_validator.validate("").is_ok());
    assert!(numeric_validator.validate("abc").is_err());

    // Test required validator
    assert!(required_validator.validate("selected").is_ok());
    assert!(required_validator.validate("").is_err());
}
