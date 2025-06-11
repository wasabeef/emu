//! Legacy device configuration module (deprecated).
//!
//! This module contains legacy functions that have been replaced by the dynamic
//! device configuration system in `device_info::DynamicDeviceConfig`. These functions
//! are maintained only for backward compatibility and will be removed in a future version.
//!
//! # Migration Guide
//!
//! Replace usage of these functions with their dynamic equivalents:
//! - `get_device_priority()` → `DynamicDeviceConfig::get_device_priority()`
//! - `get_device_skin_info()` → `DynamicDeviceConfig::get_device_skin_info()`
//! - `parse_device_name()` → `DynamicDeviceConfig::parse_device_name()`
//!
//! The dynamic system provides better compatibility with future Android SDK updates
//! by discovering device information at runtime rather than using hardcoded values.

pub use crate::models::device_info::{test_constants, FALLBACK_ANDROID_DEVICES};

use crate::models::device_info::DynamicDeviceConfig;

/// Legacy function for calculating device priority.
///
/// **Deprecated**: Use `DynamicDeviceConfig::get_device_priority()` instead.
///
/// # Arguments
/// * `_device_id` - Device identifier (unused in legacy implementation)
///
/// # Returns
/// Always returns 999 (lowest priority) and logs a warning.
pub fn get_device_priority(_device_id: &str) -> u32 {
    // This is a temporary fallback that should be replaced with dynamic calls
    log::warn!("Using legacy get_device_priority - should use DynamicDeviceConfig");
    999 // Default priority
}

/// Legacy function for retrieving device skin information.
///
/// **Deprecated**: Use `DynamicDeviceConfig::get_device_skin_info()` instead.
///
/// # Arguments
/// * `_device_id` - Device identifier (unused in legacy implementation)
///
/// # Returns
/// Always returns `(None, None)` and logs a warning.
pub fn get_device_skin_info(_device_id: &str) -> (Option<String>, Option<String>) {
    // This is a temporary fallback that should be replaced with dynamic calls
    log::warn!("Using legacy get_device_skin_info - should use DynamicDeviceConfig");
    (None, None)
}

/// Legacy function for parsing device display names.
///
/// **Deprecated**: Use `DynamicDeviceConfig::parse_device_name()` instead.
///
/// # Arguments
/// * `device_type` - Device type string to parse
///
/// # Returns
/// Basic parsing that removes parentheses and takes first 3 words.
/// Logs a warning about using the legacy function.
pub fn parse_device_name(device_type: &str) -> Vec<String> {
    // This is a temporary fallback that should be replaced with dynamic calls
    log::warn!("Using legacy parse_device_name - should use DynamicDeviceConfig");

    // Basic fallback parsing
    device_type
        .replace("(", "")
        .replace(")", "")
        .split_whitespace()
        .take(3)
        .map(|s| s.to_string())
        .collect()
}

/// Creates a new instance of the dynamic device configuration system.
///
/// This is a convenience function for creating a `DynamicDeviceConfig` instance.
///
/// # Returns
/// A new `DynamicDeviceConfig` instance ready for use.
pub fn create_dynamic_config() -> DynamicDeviceConfig {
    DynamicDeviceConfig::new()
}
