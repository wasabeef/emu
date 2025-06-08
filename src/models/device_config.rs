//! Legacy device configuration - use device_info::DynamicDeviceConfig instead
//! This module is kept for backward compatibility only
pub use crate::models::device_info::{test_constants, FALLBACK_ANDROID_DEVICES};

use crate::models::device_info::DynamicDeviceConfig;

/// Legacy function for device priority - will be removed
/// Use DynamicDeviceConfig::get_device_priority instead
pub fn get_device_priority(_device_id: &str) -> u32 {
    // This is a temporary fallback that should be replaced with dynamic calls
    log::warn!("Using legacy get_device_priority - should use DynamicDeviceConfig");
    999 // Default priority
}

/// Legacy function for device skin info - will be removed  
/// Use DynamicDeviceConfig::get_device_skin_info instead
pub fn get_device_skin_info(_device_id: &str) -> (Option<String>, Option<String>) {
    // This is a temporary fallback that should be replaced with dynamic calls
    log::warn!("Using legacy get_device_skin_info - should use DynamicDeviceConfig");
    (None, None)
}

/// Legacy function for parsing device name - will be removed
/// Use DynamicDeviceConfig::parse_device_name instead  
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

/// Create a dynamic device config instance
pub fn create_dynamic_config() -> DynamicDeviceConfig {
    DynamicDeviceConfig::new()
}
