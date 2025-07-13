use emu::utils::validation::{
    DeviceNameValidator, DevicePlatform, NumericRangeValidator, RequiredSelectionValidator,
    CompositeValidator, FieldValidator, validate_field
};

#[test]
fn test_device_name_validation_comprehensive() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);
    
    // Valid names
    let valid_names = vec![
        "device1",
        "test_device",
        "MyDevice123",
        "pixel-8-test",
        "valid.device.name",
        "a", // Single character
        "device_with_underscores",
        "device-with-dashes",
        "device123",
        "Device_Name_123",
    ];
    
    for name in valid_names {
        assert!(android_validator.validate(name).is_ok(), 
               "Android validator should accept '{name}'");
        assert!(ios_validator.validate(name).is_ok(),
               "iOS validator should accept '{name}'");
    }
    
    // Invalid names - general
    let invalid_names = vec![
        ("", "Device name cannot be empty"),
        ("device with spaces", "Invalid character"),
        ("device;rm -rf /", "Invalid character"),
        ("device\0null", "Invalid character"),
        ("device@host", "Invalid character"),
        ("device#hash", "Invalid character"),
        ("device$var", "Invalid character"),
        ("device%mod", "Invalid character"),
        ("device^exp", "Invalid character"),
        ("device&and", "Invalid character"),
        ("device*glob", "Invalid character"),
        ("device(paren", "Invalid character"),
        ("device) paren", "Invalid character"),
        ("device[bracket", "Invalid character"),
        ("device]bracket", "Invalid character"),
        ("device{brace", "Invalid character"),
        ("device}brace", "Invalid character"),
        ("device|pipe", "Invalid character"),
        ("device\\slash", "Invalid character"),
        ("device/path", "Invalid character"),
        ("device:colon", "Invalid character"),
        ("device;semi", "Invalid character"),
        ("device\"quote", "Invalid character"),
        ("device'quote", "Invalid character"),
        ("device<less", "Invalid character"),
        ("device>greater", "Invalid character"),
        ("device?question", "Invalid character"),
        ("device=equals", "Invalid character"),
        ("device+plus", "Invalid character"),
        ("device~tilde", "Invalid character"),
        ("device`backtick", "Invalid character"),
    ];
    
    for (name, expected_error) in invalid_names {
        let result = android_validator.validate(name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains(&expected_error.to_lowercase()));
    }
}

#[test]
fn test_device_name_length_limits() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);
    
    // Test exact limit (50 characters)
    let max_length_name = "a".repeat(50);
    assert!(validator.validate(&max_length_name).is_ok());
    
    // Test over limit
    let over_length_name = "a".repeat(51);
    let result = validator.validate(&over_length_name);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("50 characters or less"));
    
    // Test very long name
    let very_long_name = "a".repeat(256);
    let result = validator.validate(&very_long_name);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("50 characters or less"));
}

#[test]
fn test_android_specific_validation() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);
    
    // Android-specific invalid names
    let android_invalid = vec![
        (".hidden", "cannot start"),
        ("-device", "cannot start"),
        ("..config", "cannot start"),
        ("--device", "cannot start"),
    ];
    
    for (name, expected_error) in android_invalid {
        let result = validator.validate(name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains(&expected_error.to_lowercase()));
    }
}

#[test]
fn test_ios_specific_validation() {
    let validator = DeviceNameValidator::new(DevicePlatform::Ios);
    
    // iOS currently has no specific restrictions beyond general ones
    // Test that it accepts the same names as Android (except Android-specific restrictions)
    let names = vec![
        ".hidden", // This should be OK for iOS
        "-device", // This should be OK for iOS
        "normal_device",
        "device-123",
    ];
    
    for name in names {
        // iOS validator should be more permissive than Android for these cases
        let ios_result = validator.validate(name);
        if name.starts_with('.') || name.starts_with('-') {
            // These should be OK for iOS but not Android
            assert!(ios_result.is_ok(), "iOS should accept '{name}'");
        } else {
            // These should be OK for both
            assert!(ios_result.is_ok(), "iOS should accept '{name}'");
        }
    }
}

#[test]
fn test_device_name_validator_hint() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);
    
    let android_hint = android_validator.hint();
    let ios_hint = ios_validator.hint();
    
    assert!(!android_hint.is_empty());
    assert!(!ios_hint.is_empty());
    assert_eq!(android_hint, ios_hint); // Should be the same hint
}

#[test]
fn test_numeric_range_validator_ram() {
    let validator = NumericRangeValidator::ram_size();
    
    // Valid RAM values
    let valid_values = vec![
        "", // Empty (default)
        "512",
        "1024",
        "2048",
        "4096",
        "8192",
    ];
    
    for value in valid_values {
        assert!(validator.validate(value).is_ok(), 
               "RAM validator should accept '{value}'");
    }
    
    // Invalid RAM values
    let invalid_values = vec![
        ("256", "at least"), // Too small
        ("16384", "at most"), // Too large
        ("0", "at least"), // Zero
        ("abc", "Invalid number"),
        ("1.5", "Invalid number"), // Decimal
        ("-1024", "Invalid number"), // Negative
        ("1024MB", "Invalid number"), // With unit
        ("1,024", "Invalid number"), // With comma
    ];
    
    for (value, expected_error) in invalid_values {
        let result = validator.validate(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains(&expected_error.to_lowercase()));
    }
}

#[test]
fn test_numeric_range_validator_storage() {
    let validator = NumericRangeValidator::storage_size();
    
    // Valid storage values
    let valid_values = vec![
        "", // Empty (default)
        "1024",
        "2048",
        "4096",
        "8192",
        "16384",
        "32768",
        "65536",
    ];
    
    for value in valid_values {
        assert!(validator.validate(value).is_ok(), 
               "Storage validator should accept '{value}'");
    }
    
    // Invalid storage values
    let invalid_values = vec![
        ("512", "at least"), // Too small
        ("131072", "at most"), // Too large (if max is 65536)
        ("abc", "Invalid number"),
        ("0", "at least"),
    ];
    
    for (value, expected_error) in invalid_values {
        let result = validator.validate(value);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains(&expected_error.to_lowercase()));
    }
}

#[test]
fn test_numeric_range_validator_custom() {
    let validator = NumericRangeValidator::new(100, 1000, "units");
    
    // Valid values
    assert!(validator.validate("").is_ok());
    assert!(validator.validate("100").is_ok());
    assert!(validator.validate("500").is_ok());
    assert!(validator.validate("1000").is_ok());
    
    // Invalid values
    let result = validator.validate("99");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at least 100 units"));
    
    let result = validator.validate("1001");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("at most 1000 units"));
}

#[test]
fn test_required_selection_validator() {
    let validator = RequiredSelectionValidator::new("device type");
    
    // Valid selection
    assert!(validator.validate("pixel_8").is_ok());
    assert!(validator.validate("any_value").is_ok());
    assert!(validator.validate("a").is_ok());
    
    // Invalid selection (empty)
    let result = validator.validate("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Please select a device type"));
    
    // Test different field names
    let api_validator = RequiredSelectionValidator::new("API level");
    let result = api_validator.validate("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Please select a API level"));
}

#[test]
fn test_composite_validator() {
    let mut composite = CompositeValidator::new();
    composite = composite.with_validator(Box::new(DeviceNameValidator::new(DevicePlatform::Android)));
    composite = composite.with_validator(Box::new(RequiredSelectionValidator::new("device name")));
    
    // Valid value passes all validators
    assert!(composite.validate("valid_device").is_ok());
    
    // Empty fails required validator
    let result = composite.validate("");
    assert!(result.is_err());
    
    // Invalid characters fail device name validator
    let result = composite.validate("invalid device");
    assert!(result.is_err());
}

#[test]
fn test_composite_validator_default() {
    let composite1 = CompositeValidator::new();
    let composite2 = CompositeValidator::default();
    
    // Both should behave the same for empty validation
    assert_eq!(
        composite1.validate("test").is_ok(),
        composite2.validate("test").is_ok()
    );
}

#[test]
fn test_composite_validator_hint() {
    let composite = CompositeValidator::new();
    
    // Empty composite should have default hint
    let hint = composite.hint();
    assert!(!hint.is_empty());
    
    // Composite with validators should use first validator's hint
    let composite_with_validator = CompositeValidator::new()
        .with_validator(Box::new(DeviceNameValidator::new(DevicePlatform::Android)));
    
    let hint_with_validator = composite_with_validator.hint();
    assert!(!hint_with_validator.is_empty());
}

#[test]
fn test_validate_field_helper() {
    let validator = DeviceNameValidator::new(DevicePlatform::Android);
    
    // Valid field
    let result = validate_field("Device Name", "valid_device", &validator);
    assert!(result.is_ok());
    
    // Invalid field
    let result = validate_field("Device Name", "", &validator);
    assert!(result.is_err());
    assert!(result.unwrap_err().starts_with("Device Name:"));
    
    // Test error message formatting
    let result = validate_field("RAM", "invalid", &NumericRangeValidator::ram_size());
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.starts_with("RAM:"));
    assert!(error.contains("Invalid number"));
}

#[test]
fn test_validator_traits() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    let numeric_validator = NumericRangeValidator::ram_size();
    let required_validator = RequiredSelectionValidator::new("test");
    
    // All validators implement FieldValidator trait
    fn test_validator(validator: &dyn FieldValidator) {
        let _ = validator.validate("test");
        let _ = validator.hint();
    }
    
    test_validator(&device_validator);
    test_validator(&numeric_validator);
    test_validator(&required_validator);
}

#[test]
fn test_device_platform_debug() {
    let android = DevicePlatform::Android;
    let ios = DevicePlatform::Ios;
    
    let android_debug = format!("{android:?}");
    let ios_debug = format!("{ios:?}");
    
    assert_eq!(android_debug, "Android");
    assert_eq!(ios_debug, "Ios");
}

#[test]
fn test_device_platform_clone_copy() {
    let original = DevicePlatform::Android;
    let cloned = original.clone();
    let copied = original;
    
    // Should all be equal
    assert_eq!(original, cloned);
    assert_eq!(original, copied);
    assert_eq!(cloned, copied);
}

#[test]
fn test_validator_edge_cases() {
    let device_validator = DeviceNameValidator::new(DevicePlatform::Android);
    
    // Test with only valid characters at boundaries
    assert!(device_validator.validate("a").is_ok()); // Single char
    assert!(device_validator.validate("_").is_ok()); // Single underscore
    assert!(device_validator.validate("1").is_ok()); // Single digit
    
    // Test mixed valid characters
    assert!(device_validator.validate("a1_b2-c3.d4").is_ok());
    
    // Test edge cases for numeric validator
    let numeric_validator = NumericRangeValidator::new(0, u32::MAX, "test");
    assert!(numeric_validator.validate("0").is_ok());
    assert!(numeric_validator.validate(&u32::MAX.to_string()).is_ok());
}

#[test]
fn test_validation_error_messages_consistency() {
    let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
    
    // Error messages should be consistent and informative
    let empty_error = android_validator.validate("").unwrap_err();
    assert!(!empty_error.is_empty());
    assert!(empty_error.to_lowercase().contains("empty") || 
            empty_error.to_lowercase().contains("required"));
    
    let invalid_char_error = android_validator.validate("invalid char").unwrap_err();
    assert!(!invalid_char_error.is_empty());
    assert!(invalid_char_error.to_lowercase().contains("invalid") || 
            invalid_char_error.to_lowercase().contains("character"));
    
    let start_error = android_validator.validate(".invalid").unwrap_err();
    assert!(!start_error.is_empty());
    assert!(start_error.to_lowercase().contains("start") || 
            start_error.to_lowercase().contains("begin"));
}