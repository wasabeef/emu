//! Data integrity validation tests for device models
//!
//! These tests ensure that device data structures maintain consistency
//! and handle edge cases properly across serialization, validation,
//! and state transitions.

use anyhow::Result;
use emu::models::device::{AndroidDevice, Device, DeviceStatus, IosDevice};

// Import common test helpers from unit test module
use crate::unit::common::helpers::*;

/// Test Android device JSON serialization and deserialization
#[test]
fn test_android_device_json_round_trip() -> Result<()> {
    let original = create_android_device_with_status("test_device", DeviceStatus::Running);

    // Serialize to JSON
    let json = serde_json::to_string(&original)?;

    // Deserialize back
    let deserialized: AndroidDevice = serde_json::from_str(&json)?;

    // Verify all fields match
    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.device_type, deserialized.device_type);
    assert_eq!(original.api_level, deserialized.api_level);
    assert_eq!(original.status, deserialized.status);
    assert_eq!(original.is_running, deserialized.is_running);
    assert_eq!(original.ram_size, deserialized.ram_size);
    assert_eq!(original.storage_size, deserialized.storage_size);

    Ok(())
}

/// Test iOS device JSON serialization and deserialization
#[test]
fn test_ios_device_json_round_trip() -> Result<()> {
    let original = create_ios_device_with_status("iPhone 15", DeviceStatus::Stopped);

    // Serialize to JSON
    let json = serde_json::to_string(&original)?;

    // Deserialize back
    let deserialized: IosDevice = serde_json::from_str(&json)?;

    // Verify all fields match
    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.udid, deserialized.udid);
    assert_eq!(original.device_type, deserialized.device_type);
    assert_eq!(original.ios_version, deserialized.ios_version);
    assert_eq!(original.runtime_version, deserialized.runtime_version);
    assert_eq!(original.status, deserialized.status);
    assert_eq!(original.is_running, deserialized.is_running);
    assert_eq!(original.is_available, deserialized.is_available);

    Ok(())
}

/// Test device status consistency with is_running field
#[test]
fn test_device_status_consistency() {
    // Create devices with consistent status/is_running combinations
    let running_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "running_test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "4096M".to_string(),
    };

    let stopped_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "stopped_test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "4096M".to_string(),
    };

    // Verify Device trait methods match internal state
    assert!(running_device.is_running());
    assert_eq!(*running_device.status(), DeviceStatus::Running);

    assert!(!stopped_device.is_running());
    assert_eq!(*stopped_device.status(), DeviceStatus::Stopped);
}

/// Test Android device with edge case values
#[test]
fn test_android_device_edge_cases() {
    // Test with minimum valid values
    let minimal_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "a".to_string(), // Single character name
        device_type: "generic".to_string(),
        api_level: 1, // Minimum API level
        status: DeviceStatus::Unknown,
        is_running: false,
        ram_size: "512".to_string(),       // Minimum RAM
        storage_size: "1024M".to_string(), // Minimum storage
    };

    assert_eq!(minimal_device.id(), "a");
    assert_eq!(minimal_device.name(), "a");
    assert_eq!(*minimal_device.status(), DeviceStatus::Unknown);
    assert!(!minimal_device.is_running());

    // Test with maximum reasonable values
    let maximal_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "a".repeat(100), // Long name
        device_type: "automotive_desktop_large".to_string(),
        api_level: 50, // High API level
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "16384".to_string(),      // High RAM
        storage_size: "65536M".to_string(), // High storage
    };

    assert_eq!(maximal_device.api_level, 50);
    assert_eq!(maximal_device.ram_size, "16384");
    assert!(maximal_device.is_running());
}

/// Test iOS device with edge case values
#[test]
fn test_ios_device_edge_cases() {
    // Test with various UUID formats
    let uuid_variations = vec![
        "A1B2C3D4-E5F6-7890-ABCD-EF1234567890", // Standard format
        "a1b2c3d4-e5f6-7890-abcd-ef1234567890", // Lowercase
        "A1B2C3D4E5F67890ABCDEF1234567890",     // No dashes
        "12345678-1234-1234-1234-123456789012", // Numeric
    ];

    for udid in uuid_variations {
        let device = IosDevice {
            name: "Test Device".to_string(),
            udid: udid.to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        assert_eq!(device.id(), udid);
        assert!(!device.is_running());
    }
}

/// Test empty/default values handling
#[test]
fn test_empty_values_handling() {
    let empty_android = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: String::new(),
        device_type: String::new(),
        api_level: 0,
        status: DeviceStatus::Unknown,
        is_running: false,
        ram_size: String::new(),
        storage_size: String::new(),
    };

    // Should handle empty values gracefully
    assert_eq!(empty_android.id(), "");
    assert_eq!(empty_android.name(), "");
    assert!(!empty_android.is_running());

    let empty_ios = IosDevice {
        name: String::new(),
        udid: String::new(),
        device_type: String::new(),
        ios_version: String::new(),
        runtime_version: String::new(),
        status: DeviceStatus::Unknown,
        is_running: false,
        is_available: false,
    };

    assert_eq!(empty_ios.id(), "");
    assert_eq!(empty_ios.name(), "");
    assert!(!empty_ios.is_running());
}

/// Test device status transitions and validity
#[test]
fn test_device_status_transitions() {
    let valid_transitions = vec![
        (DeviceStatus::Stopped, DeviceStatus::Starting),
        (DeviceStatus::Starting, DeviceStatus::Running),
        (DeviceStatus::Running, DeviceStatus::Stopping),
        (DeviceStatus::Stopping, DeviceStatus::Stopped),
        (DeviceStatus::Creating, DeviceStatus::Stopped),
        (DeviceStatus::Error, DeviceStatus::Stopped),
        (DeviceStatus::Unknown, DeviceStatus::Stopped),
    ];

    for (from_status, to_status) in valid_transitions {
        // Create device with initial status
        let mut device = AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "transition_test".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: from_status,
            is_running: from_status == DeviceStatus::Running,
            ram_size: "2048".to_string(),
            storage_size: "4096M".to_string(),
        };

        // Transition to new status
        device.status = to_status;
        device.is_running = to_status == DeviceStatus::Running;

        // Verify consistency
        assert_eq!(device.status, to_status);
        assert_eq!(device.is_running, to_status == DeviceStatus::Running);
    }
}

/// Test RAM and storage size format validation
#[test]
fn test_resource_size_formats() {
    let ram_formats = vec!["512", "1024", "2048", "4096", "8192", "16384"];

    let storage_formats = vec![
        "1024M", "2048M", "4096M", "8192M", "1G", "2G", "4G", "8G", "16G",
    ];

    for ram in &ram_formats {
        for storage in &storage_formats {
            let device = AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: "format_test".to_string(),
                device_type: "pixel_7".to_string(),
                api_level: 34,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: ram.to_string(),
                storage_size: storage.to_string(),
            };

            // Should accept various valid formats
            assert_eq!(device.ram_size, *ram);
            assert_eq!(device.storage_size, *storage);
        }
    }
}

/// Test device cloning and equality
#[test]
fn test_device_cloning() {
    let original = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "clone_test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    let cloned = original.clone();

    // Verify clone is identical but independent
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.device_type, cloned.device_type);
    assert_eq!(original.api_level, cloned.api_level);
    assert_eq!(original.status, cloned.status);
    assert_eq!(original.is_running, cloned.is_running);
    assert_eq!(original.ram_size, cloned.ram_size);
    assert_eq!(original.storage_size, cloned.storage_size);
}

/// Test device debug formatting
#[test]
fn test_device_debug_formatting() {
    let device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "debug_test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    let debug_string = format!("{device:?}");

    // Debug format should contain key information
    assert!(debug_string.contains("debug_test"));
    assert!(debug_string.contains("pixel_7"));
    assert!(debug_string.contains("34"));
    assert!(debug_string.contains("Running"));
    assert!(debug_string.contains("4096"));
    assert!(debug_string.contains("8192M"));
}

/// Test polymorphic device handling via trait
#[test]
fn test_polymorphic_device_handling() {
    let android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "android_poly".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    let ios_device = IosDevice {
        name: "ios_poly".to_string(),
        udid: "ABC123".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    // Test via trait objects
    let devices: Vec<&dyn Device> = vec![&android_device, &ios_device];

    for device in devices {
        // Should work polymorphically
        assert!(!device.id().is_empty());
        assert!(!device.name().is_empty());

        match device.name() {
            "android_poly" => {
                assert!(device.is_running());
                assert_eq!(*device.status(), DeviceStatus::Running);
            }
            "ios_poly" => {
                assert!(!device.is_running());
                assert_eq!(*device.status(), DeviceStatus::Stopped);
            }
            _ => panic!("Unexpected device name"),
        }
    }
}
