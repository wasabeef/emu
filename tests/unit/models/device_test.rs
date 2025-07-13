use emu::models::{AndroidDevice, IosDevice, DeviceStatus};

#[cfg(feature = "test-utils")]
use emu::managers::mock::MockDevice;

#[test]
fn test_android_device_creation() {
    let device = AndroidDevice {
        name: "test_device".to_string(),
        device_id: "test-id".to_string(),
        avd_name: "test_device".to_string(),
        status: DeviceStatus::Offline,
        api_level: "34".to_string(),
        device_type: "Pixel 8".to_string(),
    };
    
    assert_eq!(device.name(), "test_device");
    assert_eq!(device.id(), "test-id");
    assert_eq!(device.status(), DeviceStatus::Offline);
    assert_eq!(device.api_level(), "34");
    assert_eq!(device.device_type(), "Pixel 8");
}

#[test]
fn test_ios_device_creation() {
    let device = IosDevice {
        name: "iPhone 15 Pro".to_string(),
        device_id: "test-ios-id".to_string(),
        udid: "test-udid".to_string(),
        status: DeviceStatus::Offline,
        runtime: "iOS-17.0".to_string(),
        device_type: "iPhone 15 Pro".to_string(),
    };
    
    assert_eq!(device.name(), "iPhone 15 Pro");
    assert_eq!(device.id(), "test-ios-id");
    assert_eq!(device.status(), DeviceStatus::Offline);
    assert_eq!(device.runtime(), "iOS-17.0");
    assert_eq!(device.device_type(), "iPhone 15 Pro");
}

#[test]
fn test_device_status_display() {
    let statuses = vec![
        (DeviceStatus::Online, "Online"),
        (DeviceStatus::Offline, "Offline"),
        (DeviceStatus::Starting, "Starting"),
        (DeviceStatus::Stopping, "Stopping"),
    ];
    
    for (status, expected) in statuses {
        assert_eq!(format!("{status}"), expected);
    }
}

#[test]
fn test_device_status_equality() {
    assert_eq!(DeviceStatus::Online, DeviceStatus::Online);
    assert_ne!(DeviceStatus::Online, DeviceStatus::Offline);
    
    let device1 = AndroidDevice {
        name: "device1".to_string(),
        device_id: "id1".to_string(),
        avd_name: "device1".to_string(),
        status: DeviceStatus::Online,
        api_level: "34".to_string(),
        device_type: "Pixel 8".to_string(),
    };
    
    let mut device2 = device1.clone();
    device2.status = DeviceStatus::Offline;
    
    assert_ne!(device1.status(), device2.status());
}

#[cfg(feature = "test-utils")]
#[test]
fn test_mock_device_trait_implementation() {
    let mock = MockDevice {
        name: "mock_device".to_string(),
        id: "mock-123".to_string(),
        status: DeviceStatus::Online,
        device_type: "MockPhone".to_string(),
        details: Default::default(),
    };
    
    // Device trait メソッドのテスト
    assert_eq!(mock.name(), "mock_device");
    assert_eq!(mock.id(), "mock-123");
    assert_eq!(mock.status(), DeviceStatus::Online);
    assert_eq!(mock.device_type(), "MockPhone");
}

#[cfg(feature = "test-utils")]
#[test]
fn test_mock_device_status_transitions() {
    let mut mock = MockDevice {
        name: "test_device".to_string(),
        id: "test-id".to_string(),
        status: DeviceStatus::Offline,
        device_type: "MockPhone".to_string(),
        details: Default::default(),
    };
    
    // ステータス遷移のテスト
    assert_eq!(mock.status(), DeviceStatus::Offline);
    
    mock.status = DeviceStatus::Starting;
    assert_eq!(mock.status(), DeviceStatus::Starting);
    
    mock.status = DeviceStatus::Online;
    assert_eq!(mock.status(), DeviceStatus::Online);
    
    mock.status = DeviceStatus::Stopping;
    assert_eq!(mock.status(), DeviceStatus::Stopping);
    
    mock.status = DeviceStatus::Offline;
    assert_eq!(mock.status(), DeviceStatus::Offline);
}

#[test]
fn test_android_device_clone() {
    let device = AndroidDevice {
        name: "original".to_string(),
        device_id: "id".to_string(),
        avd_name: "original".to_string(),
        status: DeviceStatus::Online,
        api_level: "34".to_string(),
        device_type: "Pixel 8".to_string(),
    };
    
    let cloned = device.clone();
    assert_eq!(device.name(), cloned.name());
    assert_eq!(device.id(), cloned.id());
    assert_eq!(device.status(), cloned.status());
}

#[test]
fn test_ios_device_clone() {
    let device = IosDevice {
        name: "iPhone Test".to_string(),
        device_id: "ios-id".to_string(),
        udid: "test-udid".to_string(),
        status: DeviceStatus::Online,
        runtime: "iOS-17.0".to_string(),
        device_type: "iPhone 15 Pro".to_string(),
    };
    
    let cloned = device.clone();
    assert_eq!(device.name(), cloned.name());
    assert_eq!(device.id(), cloned.id());
    assert_eq!(device.status(), cloned.status());
}

#[test]
fn test_device_status_debug() {
    let status = DeviceStatus::Online;
    let debug_str = format!("{status:?}");
    assert_eq!(debug_str, "Online");
}

#[test]
fn test_android_device_debug() {
    let device = AndroidDevice {
        name: "debug_device".to_string(),
        device_id: "debug-id".to_string(),
        avd_name: "debug_device".to_string(),
        status: DeviceStatus::Online,
        api_level: "34".to_string(),
        device_type: "Pixel 8".to_string(),
    };
    
    let debug_str = format!("{device:?}");
    assert!(debug_str.contains("debug_device"));
    assert!(debug_str.contains("debug-id"));
    assert!(debug_str.contains("Online"));
}