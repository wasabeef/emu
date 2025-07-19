//! Form validation utilities
//!
//! This module provides a validation framework for user input,
//! ensuring consistent validation rules across the application.

use crate::constants::{
    limits::{
        DEVICE_NAME_VALIDATION_LIMIT, MEMORY_VALIDATION_BASE_MB, MEMORY_VALIDATION_HIGH_MB,
        MEMORY_VALIDATION_MAX_MB, MEMORY_VALIDATION_MIN_MB,
    },
    messages::validation::{
        DEFAULT_VALUE_HINT, DEVICE_NAME_EMPTY_ERROR, DEVICE_NAME_HINT,
        DEVICE_NAME_INVALID_CHARS_ERROR, DEVICE_NAME_INVALID_START_ERROR, NUMERIC_VALUE_HINT,
        NUMERIC_VALUE_INVALID_ERROR, REQUIRED_FIELD_HINT,
    },
    patterns::DEVICE_NAME_PATTERN,
};
use regex::Regex;
use std::sync::OnceLock;

static DEVICE_NAME_REGEX: OnceLock<Regex> = OnceLock::new();

/// A trait for field validators
pub trait FieldValidator {
    /// Validates the given value, returning an error message if invalid
    fn validate(&self, value: &str) -> Result<(), String>;

    /// Returns a hint about the expected format
    fn hint(&self) -> &str;
}

/// Validates device names according to platform requirements
pub struct DeviceNameValidator {
    platform: DevicePlatform,
}

#[derive(Debug, Clone, Copy)]
pub enum DevicePlatform {
    Android,
    Ios,
}

impl DeviceNameValidator {
    pub fn new(platform: DevicePlatform) -> Self {
        Self { platform }
    }
}

impl FieldValidator for DeviceNameValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err(DEVICE_NAME_EMPTY_ERROR.to_string());
        }

        if value.len() > DEVICE_NAME_VALIDATION_LIMIT {
            return Err(format!(
                "Device name must be {DEVICE_NAME_VALIDATION_LIMIT} characters or less"
            ));
        }

        let regex = DEVICE_NAME_REGEX
            .get_or_init(|| Regex::new(DEVICE_NAME_PATTERN).expect("Invalid device name pattern"));

        if !regex.is_match(value) {
            return Err(DEVICE_NAME_INVALID_CHARS_ERROR.to_string());
        }

        // Platform-specific validation
        match self.platform {
            DevicePlatform::Android => {
                // Android-specific rules
                if value.starts_with('.') || value.starts_with('-') {
                    return Err(DEVICE_NAME_INVALID_START_ERROR.to_string());
                }
            }
            DevicePlatform::Ios => {
                // iOS-specific rules if any
            }
        }

        Ok(())
    }

    fn hint(&self) -> &str {
        DEVICE_NAME_HINT
    }
}

/// Validates numeric values within a range (useful for RAM/Storage)
pub struct NumericRangeValidator {
    min: u32,
    max: u32,
    unit: &'static str,
}

impl NumericRangeValidator {
    pub fn new(min: u32, max: u32, unit: &'static str) -> Self {
        Self { min, max, unit }
    }

    /// Creates a validator for RAM size
    pub fn ram_size() -> Self {
        Self::new(MEMORY_VALIDATION_MIN_MB, MEMORY_VALIDATION_HIGH_MB, "MB")
    }

    /// Creates a validator for storage size
    pub fn storage_size() -> Self {
        Self::new(MEMORY_VALIDATION_BASE_MB, MEMORY_VALIDATION_MAX_MB, "MB")
    }
}

impl FieldValidator for NumericRangeValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            // Empty is allowed (uses default)
            return Ok(());
        }

        match value.parse::<u32>() {
            Ok(num) => {
                if num < self.min {
                    Err(format!(
                        "Value must be at least {min} {unit}",
                        min = self.min,
                        unit = self.unit
                    ))
                } else if num > self.max {
                    Err(format!(
                        "Value must be at most {max} {unit}",
                        max = self.max,
                        unit = self.unit
                    ))
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(NUMERIC_VALUE_INVALID_ERROR.to_string()),
        }
    }

    fn hint(&self) -> &str {
        NUMERIC_VALUE_HINT
    }
}

/// Validates that a selection has been made (not empty)
pub struct RequiredSelectionValidator {
    field_name: &'static str,
}

impl RequiredSelectionValidator {
    pub fn new(field_name: &'static str) -> Self {
        Self { field_name }
    }
}

impl FieldValidator for RequiredSelectionValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.is_empty() {
            Err(format!(
                "Please select a {field_name}",
                field_name = self.field_name
            ))
        } else {
            Ok(())
        }
    }

    fn hint(&self) -> &str {
        REQUIRED_FIELD_HINT
    }
}

/// Composite validator that runs multiple validators
pub struct CompositeValidator {
    validators: Vec<Box<dyn FieldValidator>>,
}

impl CompositeValidator {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn with_validator(mut self, validator: Box<dyn FieldValidator>) -> Self {
        self.validators.push(validator);
        self
    }
}

impl Default for CompositeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldValidator for CompositeValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        for validator in &self.validators {
            validator.validate(value)?;
        }
        Ok(())
    }

    fn hint(&self) -> &str {
        self.validators
            .first()
            .map(|v| v.hint())
            .unwrap_or(DEFAULT_VALUE_HINT)
    }
}

/// Helper function to validate a form field and return a formatted error
pub fn validate_field(
    field_name: &str,
    value: &str,
    validator: &dyn FieldValidator,
) -> Result<(), String> {
    validator
        .validate(value)
        .map_err(|e| format!("{field_name}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_numeric_range_validator() {
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
    fn test_device_name_validator_hint() {
        let validator = DeviceNameValidator::new(DevicePlatform::Android);
        assert_eq!(validator.hint(), DEVICE_NAME_HINT);
    }

    #[test]
    fn test_device_name_validator_platform_specific() {
        let android_validator = DeviceNameValidator::new(DevicePlatform::Android);
        let ios_validator = DeviceNameValidator::new(DevicePlatform::Ios);

        // Android-specific validation
        assert!(android_validator.validate("-device").is_err());
        assert!(android_validator.validate(".device").is_err());

        // iOS should allow these (no specific restrictions)
        assert!(ios_validator.validate("device_name").is_ok());
        assert!(ios_validator.validate("Device-123").is_ok());
    }

    #[test]
    fn test_device_name_validator_edge_cases() {
        let validator = DeviceNameValidator::new(DevicePlatform::Android);

        // Exactly at limit
        let max_name = "a".repeat(DEVICE_NAME_VALIDATION_LIMIT);
        assert!(validator.validate(&max_name).is_ok());

        // Over limit
        let over_limit = "a".repeat(DEVICE_NAME_VALIDATION_LIMIT + 1);
        assert!(validator.validate(&over_limit).is_err());

        // Single character
        assert!(validator.validate("a").is_ok());

        // All valid characters
        assert!(validator.validate("aZ0._-").is_ok());
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
    fn test_numeric_range_validator_hint() {
        let validator = NumericRangeValidator::ram_size();
        assert_eq!(validator.hint(), NUMERIC_VALUE_HINT);
    }

    #[test]
    fn test_required_selection_validator() {
        let validator = RequiredSelectionValidator::new("device type");

        assert!(validator.validate("pixel_7").is_ok());
        assert!(validator.validate("iPhone 15").is_ok());

        let err = validator.validate("").unwrap_err();
        assert!(err.contains("Please select a device type"));

        assert_eq!(validator.hint(), REQUIRED_FIELD_HINT);
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
    fn test_composite_validator_hint() {
        let composite = CompositeValidator::new()
            .with_validator(Box::new(DeviceNameValidator::new(DevicePlatform::Android)));

        assert_eq!(composite.hint(), DEVICE_NAME_HINT);

        let empty_composite = CompositeValidator::new();
        assert_eq!(empty_composite.hint(), DEFAULT_VALUE_HINT);
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
        assert!(err.contains(DEVICE_NAME_EMPTY_ERROR));
    }

    #[test]
    fn test_regex_initialization() {
        // Ensure regex is initialized properly
        let validator = DeviceNameValidator::new(DevicePlatform::Android);

        // First call initializes
        assert!(validator.validate("test123").is_ok());

        // Second call uses cached regex
        assert!(validator.validate("test456").is_ok());

        // Verify pattern works correctly
        assert!(validator.validate("test@123").is_err());
    }

    #[test]
    fn test_default_composite_validator() {
        let validator = CompositeValidator::default();
        assert!(validator.validate("anything").is_ok());
        assert_eq!(validator.hint(), DEFAULT_VALUE_HINT);
    }
}
