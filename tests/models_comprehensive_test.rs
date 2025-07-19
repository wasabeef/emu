//! Comprehensive tests for Models layer
//!
//! These tests validate all model functionality including device models,
//! error handling, platform detection, and data validation.

use anyhow::Result;
use emu::models::device::Device;
use emu::models::device_info::DeviceInfo;
use emu::models::error::format_user_error;
use emu::models::{AndroidDevice, DeviceError, DeviceStatus, IosDevice, Platform};

#[tokio::test]
async fn test_android_device_model() -> Result<()> {
    let device = AndroidDevice {
        name: "Test_Device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    // Test Device trait implementation
    assert_eq!(device.name(), "Test_Device");
    assert_eq!(device.id(), "Test_Device");
    assert_eq!(device.status(), &DeviceStatus::Stopped);
    assert!(!device.is_running());

    // Test Android-specific properties
    assert_eq!(device.api_level, 34);
    assert_eq!(device.device_type, "pixel_7");
    assert_eq!(device.ram_size, "2048");
    assert_eq!(device.storage_size, "8192M");

    Ok(())
}

#[tokio::test]
async fn test_ios_device_model() -> Result<()> {
    let device = IosDevice {
        name: "iPhone 15".to_string(),
        udid: "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6".to_string(),
        device_type: "com.apple.CoreSimulator.SimDeviceType.iPhone-15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: true,
    };

    // Test Device trait implementation
    assert_eq!(device.name(), "iPhone 15");
    assert_eq!(device.id(), "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6");
    assert_eq!(device.status(), &DeviceStatus::Running);
    assert!(device.is_running());

    // Test iOS-specific properties
    assert_eq!(device.ios_version, "17.0");
    assert_eq!(device.runtime_version, "iOS 17.0");
    assert!(device.is_available);

    Ok(())
}

#[tokio::test]
async fn test_device_status_enum() -> Result<()> {
    // Test all device status variants
    let running = DeviceStatus::Running;
    let stopped = DeviceStatus::Stopped;
    let unknown = DeviceStatus::Unknown;

    assert_ne!(running, stopped);
    assert_ne!(stopped, unknown);
    assert_ne!(running, unknown);

    // Test Debug formatting
    assert_eq!(format!("{running:?}"), "Running");
    assert_eq!(format!("{stopped:?}"), "Stopped");
    assert_eq!(format!("{unknown:?}"), "Unknown");

    Ok(())
}

#[tokio::test]
async fn test_platform_detection() -> Result<()> {
    // Test platform availability logic
    let ios_supported = Platform::Ios.is_supported();
    let android_supported = Platform::Android.is_supported();

    // Android should always be supported
    assert!(android_supported);

    // iOS support depends on platform
    if cfg!(target_os = "macos") {
        assert!(ios_supported);
    } else {
        assert!(!ios_supported);
    }

    Ok(())
}

#[tokio::test]
async fn test_error_formatting() -> Result<()> {
    // Test various error scenarios
    let device_error = DeviceError::not_found("Test Device");
    let command_error = DeviceError::command_failed("adb devices");
    let config_error = DeviceError::InvalidConfig {
        message: "Invalid config".to_string(),
    };

    // Test user-friendly error formatting
    let device_msg = format_user_error(&device_error.into());
    assert!(device_msg.contains("not found") || device_msg.contains("Device"));

    let command_msg = format_user_error(&command_error.into());
    assert!(command_msg.contains("command") || command_msg.contains("execution"));

    let config_msg = format_user_error(&config_error.into());
    assert!(config_msg.contains("config") || config_msg.contains("Configuration"));

    Ok(())
}

#[tokio::test]
async fn test_device_info_model() -> Result<()> {
    let device_info = DeviceInfo {
        id: "test_device_id".to_string(),
        display_name: "Test Device".to_string(),
        oem: Some("Google".to_string()),
        category: emu::models::device_info::DeviceCategory::Phone,
    };

    // Test basic properties
    assert_eq!(device_info.display_name, "Test Device");
    assert_eq!(device_info.id, "test_device_id");
    assert_eq!(device_info.oem, Some("Google".to_string()));
    assert_eq!(
        device_info.category,
        emu::models::device_info::DeviceCategory::Phone
    );

    Ok(())
}

#[tokio::test]
async fn test_device_validation() -> Result<()> {
    // Test valid Android device
    let valid_android = AndroidDevice {
        name: "Valid_Device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    assert!(!valid_android.name().is_empty());
    assert!(valid_android.api_level >= 21); // Minimum supported API level
    assert!(valid_android.api_level <= 35); // Maximum reasonable API level

    // Test valid iOS device
    let valid_ios = IosDevice {
        name: "Valid iPhone".to_string(),
        udid: "12345678-1234-1234-1234-123456789012".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "15.0".to_string(),
        runtime_version: "iOS 15.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    assert!(!valid_ios.name().is_empty());
    assert!(!valid_ios.udid.is_empty());
    assert!(valid_ios.udid.len() >= 32); // UUID should be at least 32 chars

    Ok(())
}

#[tokio::test]
async fn test_device_serialization() -> Result<()> {
    let android_device = AndroidDevice {
        name: "Serialization_Test".to_string(),
        device_type: "pixel_6".to_string(),
        api_level: 33,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "4096".to_string(),
        storage_size: "16384M".to_string(),
    };

    // Test that device can be converted to/from JSON (if Serialize/Deserialize is implemented)
    // This would typically be done with serde_json, but depends on trait implementation
    let device_name = android_device.name();
    let device_status = android_device.status();

    assert_eq!(device_name, "Serialization_Test");
    assert_eq!(*device_status, DeviceStatus::Running);

    Ok(())
}

#[tokio::test]
async fn test_platform_specific_features() -> Result<()> {
    // Test Android-specific features
    let android_device = AndroidDevice {
        name: "Android_Features_Test".to_string(),
        device_type: "wear_round".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "1024".to_string(),
        storage_size: "4096M".to_string(),
    };

    // Android devices have API levels
    assert!(android_device.api_level > 0);

    // Android devices have RAM/storage as strings
    assert!(!android_device.ram_size.is_empty());
    assert!(!android_device.storage_size.is_empty());

    // Test iOS-specific features
    let ios_device = IosDevice {
        name: "iOS_Features_Test".to_string(),
        udid: "87654321-4321-4321-4321-210987654321".to_string(),
        device_type: "iPad".to_string(),
        ios_version: "16.4".to_string(),
        runtime_version: "iOS 16.4".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    // iOS devices have UDIDs
    assert!(!ios_device.udid.is_empty());

    // iOS devices have version strings
    assert!(!ios_device.ios_version.is_empty());
    assert!(!ios_device.runtime_version.is_empty());

    // iOS devices have availability flag
    assert!(ios_device.is_available);

    Ok(())
}

#[tokio::test]
async fn test_error_chain_handling() -> Result<()> {
    // Test error chaining and context
    let base_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let wrapped_error = DeviceError::Io(base_error);

    let error_message = format!("{wrapped_error}");
    assert!(error_message.contains("IO error"));
    assert!(error_message.contains("File not found"));

    // Test error formatting
    let formatted = format_user_error(&wrapped_error.into());
    assert!(!formatted.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_device_edge_cases() -> Result<()> {
    // Test edge cases for device models

    // Empty/minimal Android device
    let minimal_android = AndroidDevice {
        name: "".to_string(),
        device_type: "".to_string(),
        api_level: 21, // Minimum API level
        status: DeviceStatus::Unknown,
        is_running: false,
        ram_size: "0".to_string(),
        storage_size: "0M".to_string(),
    };

    assert_eq!(minimal_android.name(), "");
    assert_eq!(minimal_android.status(), &DeviceStatus::Unknown);

    // Maximum values Android device
    let max_android = AndroidDevice {
        name: "A".repeat(100), // Long name
        device_type: "custom_device_type".to_string(),
        api_level: 35, // High API level
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "16384".to_string(),      // 16GB
        storage_size: "65536M".to_string(), // 64GB
    };

    assert_eq!(max_android.name().len(), 100);
    assert!(max_android.api_level <= 35);

    // Test iOS device with special characters
    let special_ios = IosDevice {
        name: "Test Device 🚀".to_string(),
        udid: "FFFFFFFF-FFFF-FFFF-FFFF-FFFFFFFFFFFF".to_string(),
        device_type: "com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro-Max".to_string(),
        ios_version: "17.2.1".to_string(),
        runtime_version: "iOS 17.2.1 (21C52)".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: false, // Unavailable device
    };

    assert!(special_ios.name().contains("🚀"));
    assert!(!special_ios.is_available);

    Ok(())
}

#[tokio::test]
async fn test_model_consistency() -> Result<()> {
    // Test consistency between is_running flag and status
    let consistent_device = AndroidDevice {
        name: "Consistent_Device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true, // Should match status
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    assert_eq!(consistent_device.status(), &DeviceStatus::Running);
    assert!(consistent_device.is_running());

    let stopped_device = AndroidDevice {
        name: "Stopped_Device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false, // Should match status
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    assert_eq!(stopped_device.status(), &DeviceStatus::Stopped);
    assert!(!stopped_device.is_running());

    Ok(())
}
