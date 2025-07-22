//! Basic tests for models/device.rs
//!
//! Tests basic functionality, validation, and Trait implementations of device models.

use crate::unit::common::helpers::*;
use emu::models::device::Device;
use emu::models::{AndroidDevice, DeviceStatus};

#[test]
fn test_android_device_creation() {
    let device = create_test_android_device("TestDevice");

    assert_eq!(device.name, "TestDevice");
    assert_eq!(device.device_type, "pixel_7");
    assert_eq!(device.api_level, 33);
    assert_eq!(device.status, DeviceStatus::Stopped);
    assert!(!device.is_running);
    assert_eq!(device.ram_size, "2048");
    assert_eq!(device.storage_size, "8192M");
}

#[test]
fn test_ios_device_creation() {
    let device = create_test_ios_device("TestiPhone");

    assert_eq!(device.name, "TestiPhone");
    assert_eq!(device.udid, "test-udid-TestiPhone");
    assert_eq!(device.device_type, "iPhone 15");
    assert_eq!(device.ios_version, "17.0");
    assert_eq!(device.status, DeviceStatus::Stopped);
    assert!(!device.is_running);
}

#[test]
fn test_android_device_trait_implementation() {
    let device = create_test_android_device("TraitTest");

    // Test Device trait methods
    assert_eq!(device.id(), "TraitTest");
    assert_eq!(device.name(), "TraitTest");
    assert_eq!(*device.status(), DeviceStatus::Stopped);
    assert!(!device.is_running());
}

#[test]
fn test_ios_device_trait_implementation() {
    let device = create_test_ios_device("iOSTraitTest");

    // Test Device trait methods
    assert_eq!(device.id(), "test-udid-iOSTraitTest");
    assert_eq!(device.name(), "iOSTraitTest");
    assert_eq!(*device.status(), DeviceStatus::Stopped);
    assert!(!device.is_running());
}

#[test]
fn test_device_status_variants() {
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
        let device = create_android_device_with_status("StatusTest", status);
        assert_device_status(device.status, status);
        assert_eq!(device.is_running, status == DeviceStatus::Running);
    }
}

#[test]
fn test_android_device_with_different_api_levels() {
    let api_levels = vec![28, 29, 30, 31, 32, 33, 34];

    for api_level in api_levels {
        let device = create_android_device_with_api("APITest", api_level);
        assert_eq!(device.api_level, api_level);
        assert_api_level_in_range(device.api_level, 28, 34);
    }
}

#[test]
fn test_android_device_ram_validation() {
    let device = create_test_android_device("RAMTest");

    assert_valid_ram_size(&device.ram_size);
}

#[test]
fn test_ios_device_with_different_statuses() {
    let statuses = vec![
        DeviceStatus::Running,
        DeviceStatus::Stopped,
        DeviceStatus::Starting,
        DeviceStatus::Error,
    ];

    for status in statuses {
        let device = create_ios_device_with_status("iOSStatusTest", status);
        assert_eq!(device.status, status);
        assert_eq!(device.is_running, status == DeviceStatus::Running);
    }
}

#[test]
fn test_device_default_values() {
    let device = AndroidDevice::default();

    assert_eq!(device.name, "");
    assert_eq!(device.device_type, "");
    assert_eq!(device.api_level, 0);
    assert_eq!(device.status, DeviceStatus::Stopped);
    assert!(!device.is_running);
    assert!(!device.ram_size.is_empty());
    assert!(!device.storage_size.is_empty());
}

#[test]
fn test_device_status_equality() {
    assert_eq!(DeviceStatus::Running, DeviceStatus::Running);
    assert_ne!(DeviceStatus::Running, DeviceStatus::Stopped);
    assert_eq!(DeviceStatus::Unknown, DeviceStatus::Unknown);
}

#[test]
fn test_device_clone() {
    let device = create_test_android_device("CloneTest");
    let cloned = device.clone();

    assert_eq!(device.name, cloned.name);
    assert_eq!(device.api_level, cloned.api_level);
    assert_eq!(device.status, cloned.status);
    assert_eq!(device.is_running, cloned.is_running);
}

#[test]
fn test_device_serialization() {
    let device = create_test_android_device("SerializationTest");

    // Actual serialization/deserialization
    let serialized = serde_json::to_string(&device).unwrap();
    let deserialized: AndroidDevice = serde_json::from_str(&serialized).unwrap();

    assert_eq!(device.name, deserialized.name);
    assert_eq!(device.api_level, deserialized.api_level);
    assert_eq!(device.status, deserialized.status);
}

#[test]
fn test_device_debug_format() {
    let device = create_test_android_device("DebugTest");
    let debug_str = format!("{device:?}");

    assert!(debug_str.contains("AndroidDevice"));
    assert!(debug_str.contains("DebugTest"));
    assert!(debug_str.contains("api_level"));
}

#[test]
fn test_device_list_operations() {
    let devices = create_android_device_list(3);

    assert_eq!(devices.len(), 3);
    assert_eq!(devices[0].name, "Device0");
    assert_eq!(devices[1].name, "Device1");
    assert_eq!(devices[2].name, "Device2");

    // Verify each device is created correctly
    for device in &devices {
        assert_eq!(device.device_type, "pixel_7");
        assert_eq!(device.api_level, 33);
        assert_eq!(device.status, DeviceStatus::Stopped);
    }
}

#[test]
fn test_ios_device_list_operations() {
    let devices = create_ios_device_list(2);

    assert_eq!(devices.len(), 2);
    assert_eq!(devices[0].name, "iPhone0");
    assert_eq!(devices[1].name, "iPhone1");

    // Verify each device is created correctly
    for device in &devices {
        assert_eq!(device.device_type, "iPhone 15");
        assert_eq!(device.ios_version, "17.0");
        assert_eq!(device.status, DeviceStatus::Stopped);
    }
}
