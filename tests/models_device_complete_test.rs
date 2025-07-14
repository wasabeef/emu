//! Comprehensive tests for models::device module
//!
//! These tests ensure complete coverage of all device models, traits,
//! and status enumerations with edge cases and serialization.

use anyhow::Result;
use emu::models::device::{AndroidDevice, Device, DeviceStatus, IosDevice};

#[tokio::test]
async fn test_android_device_default() -> Result<()> {
    let device = AndroidDevice::default();

    assert_eq!(device.name, "");
    assert_eq!(device.device_type, "");
    assert_eq!(device.api_level, 0);
    assert_eq!(device.status, DeviceStatus::Stopped);
    assert!(!device.is_running);
    assert_eq!(device.ram_size, "2048");
    assert_eq!(device.storage_size, "512M");

    Ok(())
}

#[tokio::test]
async fn test_android_device_trait_implementation() -> Result<()> {
    let device = AndroidDevice {
        name: "TestDevice".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "4096".to_string(),
        storage_size: "16384M".to_string(),
    };

    // Test Device trait methods
    assert_eq!(device.id(), "TestDevice");
    assert_eq!(device.name(), "TestDevice");
    assert_eq!(device.status(), &DeviceStatus::Running);
    assert!(device.is_running());

    Ok(())
}

#[tokio::test]
async fn test_ios_device_trait_implementation() -> Result<()> {
    let device = IosDevice {
        name: "iPhone 15".to_string(),
        udid: "ABC123-DEF456-GHI789".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: true,
    };

    // Test Device trait methods
    assert_eq!(device.id(), "ABC123-DEF456-GHI789");
    assert_eq!(device.name(), "iPhone 15");
    assert_eq!(device.status(), &DeviceStatus::Running);
    assert!(device.is_running());

    Ok(())
}

#[tokio::test]
async fn test_device_status_all_variants() -> Result<()> {
    let statuses = vec![
        DeviceStatus::Running,
        DeviceStatus::Stopped,
        DeviceStatus::Starting,
        DeviceStatus::Stopping,
        DeviceStatus::Creating,
        DeviceStatus::Error,
        DeviceStatus::Unknown,
    ];

    // Test each status can be created and compared
    for status in &statuses {
        let device = AndroidDevice {
            status: *status,
            ..AndroidDevice::default()
        };
        assert_eq!(device.status(), status);
    }

    // Test status equality
    assert_eq!(DeviceStatus::Running, DeviceStatus::Running);
    assert_ne!(DeviceStatus::Running, DeviceStatus::Stopped);

    Ok(())
}

#[tokio::test]
async fn test_device_status_debug_formatting() -> Result<()> {
    assert_eq!(format!("{:?}", DeviceStatus::Running), "Running");
    assert_eq!(format!("{:?}", DeviceStatus::Stopped), "Stopped");
    assert_eq!(format!("{:?}", DeviceStatus::Starting), "Starting");
    assert_eq!(format!("{:?}", DeviceStatus::Stopping), "Stopping");
    assert_eq!(format!("{:?}", DeviceStatus::Creating), "Creating");
    assert_eq!(format!("{:?}", DeviceStatus::Error), "Error");
    assert_eq!(format!("{:?}", DeviceStatus::Unknown), "Unknown");

    Ok(())
}

#[tokio::test]
async fn test_android_device_serialization() -> Result<()> {
    let device = AndroidDevice {
        name: "Serialization_Test".to_string(),
        device_type: "pixel_8".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "8192".to_string(),
        storage_size: "32768M".to_string(),
    };

    // Test serialization
    let json = serde_json::to_string(&device)?;
    assert!(json.contains("Serialization_Test"));
    assert!(json.contains("pixel_8"));
    assert!(json.contains("34"));
    assert!(json.contains("Running"));

    // Test deserialization
    let deserialized: AndroidDevice = serde_json::from_str(&json)?;
    assert_eq!(deserialized.name, device.name);
    assert_eq!(deserialized.device_type, device.device_type);
    assert_eq!(deserialized.api_level, device.api_level);
    assert_eq!(deserialized.status, device.status);
    assert_eq!(deserialized.is_running, device.is_running);
    assert_eq!(deserialized.ram_size, device.ram_size);
    assert_eq!(deserialized.storage_size, device.storage_size);

    Ok(())
}

#[tokio::test]
async fn test_ios_device_serialization() -> Result<()> {
    let device = IosDevice {
        name: "Test iPhone".to_string(),
        udid: "12345678-1234-1234-1234-123456789012".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.2".to_string(),
        runtime_version: "iOS 17.2 (21C62)".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    // Test serialization
    let json = serde_json::to_string(&device)?;
    assert!(json.contains("Test iPhone"));
    assert!(json.contains("12345678-1234-1234-1234-123456789012"));
    assert!(json.contains("17.2"));

    // Test deserialization
    let deserialized: IosDevice = serde_json::from_str(&json)?;
    assert_eq!(deserialized.name, device.name);
    assert_eq!(deserialized.udid, device.udid);
    assert_eq!(deserialized.device_type, device.device_type);
    assert_eq!(deserialized.ios_version, device.ios_version);
    assert_eq!(deserialized.runtime_version, device.runtime_version);
    assert_eq!(deserialized.status, device.status);
    assert_eq!(deserialized.is_running, device.is_running);
    assert_eq!(deserialized.is_available, device.is_available);

    Ok(())
}

#[tokio::test]
async fn test_device_status_serialization() -> Result<()> {
    let statuses = vec![
        DeviceStatus::Running,
        DeviceStatus::Stopped,
        DeviceStatus::Starting,
        DeviceStatus::Stopping,
        DeviceStatus::Creating,
        DeviceStatus::Error,
        DeviceStatus::Unknown,
    ];

    for status in statuses {
        // Test serialization
        let json = serde_json::to_string(&status)?;

        // Test deserialization
        let deserialized: DeviceStatus = serde_json::from_str(&json)?;
        assert_eq!(deserialized, status);
    }

    Ok(())
}

#[tokio::test]
async fn test_android_device_clone() -> Result<()> {
    let device = AndroidDevice {
        name: "Clone_Test".to_string(),
        device_type: "pixel_fold".to_string(),
        api_level: 33,
        status: DeviceStatus::Creating,
        is_running: false,
        ram_size: "12288".to_string(),
        storage_size: "64G".to_string(),
    };

    let cloned = device.clone();

    assert_eq!(cloned.name, device.name);
    assert_eq!(cloned.device_type, device.device_type);
    assert_eq!(cloned.api_level, device.api_level);
    assert_eq!(cloned.status, device.status);
    assert_eq!(cloned.is_running, device.is_running);
    assert_eq!(cloned.ram_size, device.ram_size);
    assert_eq!(cloned.storage_size, device.storage_size);

    Ok(())
}

#[tokio::test]
async fn test_ios_device_clone() -> Result<()> {
    let device = IosDevice {
        name: "iPad Pro Clone Test".to_string(),
        udid: "FEDCBA98-7654-3210-FEDC-BA9876543210".to_string(),
        device_type: "iPad Pro 12.9".to_string(),
        ios_version: "16.6".to_string(),
        runtime_version: "iOS 16.6 (20G75)".to_string(),
        status: DeviceStatus::Error,
        is_running: false,
        is_available: false,
    };

    let cloned = device.clone();

    assert_eq!(cloned.name, device.name);
    assert_eq!(cloned.udid, device.udid);
    assert_eq!(cloned.device_type, device.device_type);
    assert_eq!(cloned.ios_version, device.ios_version);
    assert_eq!(cloned.runtime_version, device.runtime_version);
    assert_eq!(cloned.status, device.status);
    assert_eq!(cloned.is_running, device.is_running);
    assert_eq!(cloned.is_available, device.is_available);

    Ok(())
}

#[tokio::test]
async fn test_android_device_debug_formatting() -> Result<()> {
    let device = AndroidDevice {
        name: "Debug_Test".to_string(),
        device_type: "tv_4k".to_string(),
        api_level: 31,
        status: DeviceStatus::Starting,
        is_running: false,
        ram_size: "1024".to_string(),
        storage_size: "2048M".to_string(),
    };

    let debug_output = format!("{device:?}");
    assert!(debug_output.contains("AndroidDevice"));
    assert!(debug_output.contains("Debug_Test"));
    assert!(debug_output.contains("tv_4k"));
    assert!(debug_output.contains("31"));
    assert!(debug_output.contains("Starting"));

    Ok(())
}

#[tokio::test]
async fn test_ios_device_debug_formatting() -> Result<()> {
    let device = IosDevice {
        name: "Debug iOS".to_string(),
        udid: "DEBUG-1234-5678-9012-DEBUGDEVICE01".to_string(),
        device_type: "iPhone 12 mini".to_string(),
        ios_version: "15.7".to_string(),
        runtime_version: "iOS 15.7 (19H12)".to_string(),
        status: DeviceStatus::Stopping,
        is_running: true,
        is_available: true,
    };

    let debug_output = format!("{device:?}");
    assert!(debug_output.contains("IosDevice"));
    assert!(debug_output.contains("Debug iOS"));
    assert!(debug_output.contains("DEBUG-1234-5678-9012-DEBUGDEVICE01"));
    assert!(debug_output.contains("iPhone 12 mini"));
    assert!(debug_output.contains("15.7"));
    assert!(debug_output.contains("Stopping"));

    Ok(())
}

#[tokio::test]
async fn test_device_status_copy_clone() -> Result<()> {
    let status = DeviceStatus::Running;

    // Test Copy trait
    let copied = status;
    assert_eq!(copied, DeviceStatus::Running);
    assert_eq!(status, DeviceStatus::Running); // Original still available

    // Test Clone trait
    let cloned = status;
    assert_eq!(cloned, DeviceStatus::Running);

    Ok(())
}

#[tokio::test]
async fn test_device_edge_cases() -> Result<()> {
    // Test with empty strings
    let empty_device = AndroidDevice {
        name: String::new(),
        device_type: String::new(),
        api_level: 0,
        status: DeviceStatus::Unknown,
        is_running: false,
        ram_size: String::new(),
        storage_size: String::new(),
    };

    assert_eq!(empty_device.id(), "");
    assert_eq!(empty_device.name(), "");
    assert_eq!(empty_device.status(), &DeviceStatus::Unknown);
    assert!(!empty_device.is_running());

    // Test with very long strings
    let long_name = "A".repeat(1000);
    let long_device = AndroidDevice {
        name: long_name.clone(),
        device_type: "B".repeat(500),
        api_level: u32::MAX,
        status: DeviceStatus::Error,
        is_running: true,
        ram_size: "C".repeat(100),
        storage_size: "D".repeat(100),
    };

    assert_eq!(long_device.id(), &long_name);
    assert_eq!(long_device.name(), &long_name);

    Ok(())
}
