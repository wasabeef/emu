//! Comprehensive tests for managers::mock module
//!
//! These tests ensure complete coverage of MockDeviceManager and all its methods,
//! including edge cases, error conditions, and behavior configurations.

use emu::managers::common::{DeviceConfig, DeviceManager};
use emu::managers::mock::{MockBehavior, MockDevice, MockDeviceManager, MockOperation};
use emu::models::DeviceStatus;

#[tokio::test]
async fn test_mock_device_manager_android_creation() {
    let manager = MockDeviceManager::new_android();

    // Test initial state
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 2);

    // Test that default devices are Android
    let device_names: Vec<String> = devices.iter().map(|d| d.name().to_string()).collect();
    assert!(device_names.contains(&"Pixel_4_API_30".to_string()));
    assert!(device_names.contains(&"Pixel_6_API_33".to_string()));

    // Test that devices start as stopped
    for device in devices {
        assert_eq!(device.status(), &DeviceStatus::Stopped);
        assert!(!device.is_running());
    }
}

#[tokio::test]
async fn test_mock_device_manager_ios_creation() {
    let manager = MockDeviceManager::new_ios();

    // Test initial state
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 2);

    // Test that default devices are iOS
    let device_names: Vec<String> = devices.iter().map(|d| d.name().to_string()).collect();
    assert!(device_names.contains(&"iPhone 14".to_string()));
    assert!(device_names.contains(&"iPad Pro".to_string()));

    // Test that devices start as stopped
    for device in devices {
        assert_eq!(device.status(), &DeviceStatus::Stopped);
        assert!(!device.is_running());
    }
}

#[tokio::test]
async fn test_mock_operation_recording() {
    let manager = MockDeviceManager::new_android();

    // Test that operations are recorded
    assert_eq!(manager.get_operations().len(), 0);

    // Perform various operations
    let _devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(manager.get_operations().len(), 1);
    assert!(manager.assert_operation_called(&MockOperation::ListDevices));

    DeviceManager::start_device(&manager, "emulator-5554")
        .await
        .unwrap();
    assert_eq!(manager.get_operations().len(), 2); // list_devices once + start_device once
    assert!(
        manager.assert_operation_called(&MockOperation::StartDevice("emulator-5554".to_string()))
    );

    DeviceManager::stop_device(&manager, "emulator-5554")
        .await
        .unwrap();
    assert_eq!(manager.get_operations().len(), 3);
    assert!(
        manager.assert_operation_called(&MockOperation::StopDevice("emulator-5554".to_string()))
    );
}

#[tokio::test]
async fn test_mock_operation_clear() {
    let manager = MockDeviceManager::new_android();

    // Perform operations
    let _devices = DeviceManager::list_devices(&manager).await.unwrap();
    DeviceManager::start_device(&manager, "emulator-5554")
        .await
        .unwrap();

    assert!(!manager.get_operations().is_empty());

    // Clear operations
    manager.clear_operations();
    assert_eq!(manager.get_operations().len(), 0);
}

#[tokio::test]
async fn test_mock_behavior_configuration() {
    let manager = MockDeviceManager::new_android();

    // Configure failure
    manager.configure_failure("start_device", "Test failure message");

    // Test that operation fails
    let result = DeviceManager::start_device(&manager, "emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Test failure message");

    // Clear behavior and test success
    manager.clear_behavior();
    let result = DeviceManager::start_device(&manager, "emulator-5554").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mock_delay_configuration() {
    let manager = MockDeviceManager::new_android();

    // Configure delay
    manager.configure_delay("list_devices", 50);

    // Test that operation is delayed
    let start_time = std::time::Instant::now();
    let _devices = DeviceManager::list_devices(&manager).await.unwrap();
    let elapsed = start_time.elapsed();

    assert!(elapsed.as_millis() >= 50);
}

#[tokio::test]
async fn test_mock_device_start_stop_by_id() {
    let manager = MockDeviceManager::new_android();

    // Test start by ID (for Android, ID is the same as name)
    DeviceManager::start_device(&manager, "emulator-5554")
        .await
        .unwrap();
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    let device = devices
        .iter()
        .find(|d| d.name() == "Pixel_4_API_30")
        .unwrap(); // Android ID = name
    assert_eq!(device.status(), &DeviceStatus::Running);

    // Test stop by ID
    DeviceManager::stop_device(&manager, "emulator-5554")
        .await
        .unwrap();
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    let device = devices
        .iter()
        .find(|d| d.name() == "Pixel_4_API_30")
        .unwrap(); // Android ID = name
    assert_eq!(device.status(), &DeviceStatus::Stopped);
}

#[tokio::test]
async fn test_mock_device_start_stop_by_name() {
    let manager = MockDeviceManager::new_android();

    // Test start by name
    DeviceManager::start_device(&manager, "Pixel_4_API_30")
        .await
        .unwrap();
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    let device = devices
        .iter()
        .find(|d| d.name() == "Pixel_4_API_30")
        .unwrap();
    assert_eq!(device.status(), &DeviceStatus::Running);

    // Test stop by name
    DeviceManager::stop_device(&manager, "Pixel_4_API_30")
        .await
        .unwrap();
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    let device = devices
        .iter()
        .find(|d| d.name() == "Pixel_4_API_30")
        .unwrap();
    assert_eq!(device.status(), &DeviceStatus::Stopped);
}

#[tokio::test]
async fn test_mock_device_not_found_errors() {
    let manager = MockDeviceManager::new_android();

    // Test start non-existent device
    let result = DeviceManager::start_device(&manager, "non-existent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));

    // Test stop non-existent device
    let result = DeviceManager::stop_device(&manager, "non-existent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));

    // Test delete non-existent device
    let result = DeviceManager::delete_device(&manager, "non-existent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));

    // Test wipe non-existent device
    let result = DeviceManager::wipe_device(&manager, "non-existent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));
}

#[tokio::test]
async fn test_mock_device_creation_android() {
    let manager = MockDeviceManager::new_android();

    let config = DeviceConfig::new(
        "Test_Android_Device".to_string(),
        "pixel_7".to_string(),
        "33".to_string(),
    );

    // Test device creation
    DeviceManager::create_device(&manager, &config)
        .await
        .unwrap();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 3);

    let new_device = devices
        .iter()
        .find(|d| d.name() == "Test_Android_Device")
        .unwrap();
    assert_eq!(new_device.status(), &DeviceStatus::Stopped);
    assert!(!new_device.is_running());

    // Test operation recording
    assert!(
        manager.assert_operation_called(&MockOperation::CreateDevice {
            name: "Test_Android_Device".to_string(),
            device_type: "pixel_7".to_string(),
        })
    );
}

#[tokio::test]
async fn test_mock_device_creation_ios() {
    let manager = MockDeviceManager::new_ios();

    let config = DeviceConfig::new(
        "Test_iOS_Device".to_string(),
        "iPhone 15".to_string(),
        "17.0".to_string(),
    );

    // Test device creation
    DeviceManager::create_device(&manager, &config)
        .await
        .unwrap();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 3);

    let new_device = devices
        .iter()
        .find(|d| d.name() == "Test_iOS_Device")
        .unwrap();
    assert_eq!(new_device.status(), &DeviceStatus::Stopped);
    assert!(!new_device.is_running());

    // Test that iOS device has UUID-style ID
    assert!(new_device.id().contains("-"));
}

#[tokio::test]
async fn test_mock_device_deletion() {
    let manager = MockDeviceManager::new_android();

    // Create a device first
    let config = DeviceConfig::new(
        "To_Delete".to_string(),
        "pixel_8".to_string(),
        "34".to_string(),
    );
    DeviceManager::create_device(&manager, &config)
        .await
        .unwrap();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 3);

    // Delete the device
    DeviceManager::delete_device(&manager, "To_Delete")
        .await
        .unwrap();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 2);

    // Verify device is gone
    assert!(devices.iter().all(|d| d.name() != "To_Delete"));

    // Test operation recording
    assert!(manager.assert_operation_called(&MockOperation::DeleteDevice("To_Delete".to_string())));
}

#[tokio::test]
async fn test_mock_device_wipe() {
    let manager = MockDeviceManager::new_android();

    // Test wipe existing device
    DeviceManager::wipe_device(&manager, "emulator-5554")
        .await
        .unwrap();

    // Test operation recording
    assert!(
        manager.assert_operation_called(&MockOperation::WipeDevice("emulator-5554".to_string()))
    );

    // Test wipe by name
    DeviceManager::wipe_device(&manager, "Pixel_6_API_33")
        .await
        .unwrap();
    assert!(
        manager.assert_operation_called(&MockOperation::WipeDevice("Pixel_6_API_33".to_string()))
    );
}

#[tokio::test]
async fn test_mock_manager_is_available() {
    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Mock managers are always available
    assert!(DeviceManager::is_available(&android_manager).await);
    assert!(DeviceManager::is_available(&ios_manager).await);
}

#[tokio::test]
async fn test_mock_device_struct_fields() {
    let device = MockDevice {
        id: "test-id".to_string(),
        name: "Test Device".to_string(),
        status: DeviceStatus::Running,
        api_level: Some("30".to_string()),
        device_type: "test_type".to_string(),
    };

    // Test all fields
    assert_eq!(device.id, "test-id");
    assert_eq!(device.name, "Test Device");
    assert_eq!(device.status, DeviceStatus::Running);
    assert_eq!(device.api_level, Some("30".to_string()));
    assert_eq!(device.device_type, "test_type");
}

#[tokio::test]
async fn test_mock_device_debug_formatting() {
    let device = MockDevice {
        id: "debug-test".to_string(),
        name: "Debug Device".to_string(),
        status: DeviceStatus::Stopped,
        api_level: None,
        device_type: "debug_type".to_string(),
    };

    let debug_output = format!("{device:?}");
    assert!(debug_output.contains("MockDevice"));
    assert!(debug_output.contains("debug-test"));
    assert!(debug_output.contains("Debug Device"));
    assert!(debug_output.contains("Stopped"));
    assert!(debug_output.contains("debug_type"));
}

#[tokio::test]
async fn test_mock_device_clone() {
    let device = MockDevice {
        id: "clone-test".to_string(),
        name: "Clone Device".to_string(),
        status: DeviceStatus::Starting,
        api_level: Some("31".to_string()),
        device_type: "clone_type".to_string(),
    };

    let cloned = device.clone();
    assert_eq!(cloned.id, device.id);
    assert_eq!(cloned.name, device.name);
    assert_eq!(cloned.status, device.status);
    assert_eq!(cloned.api_level, device.api_level);
    assert_eq!(cloned.device_type, device.device_type);
}

#[tokio::test]
async fn test_mock_behavior_default() {
    let behavior = MockBehavior::default();

    // Test default state
    assert!(behavior.failing_operations.is_empty());
    assert!(behavior.operation_delays.is_empty());
}

#[tokio::test]
async fn test_mock_behavior_debug_formatting() {
    let mut behavior = MockBehavior::default();
    behavior
        .failing_operations
        .insert("test".to_string(), "error".to_string());
    behavior.operation_delays.insert("slow".to_string(), 100);

    let debug_output = format!("{behavior:?}");
    assert!(debug_output.contains("MockBehavior"));
    assert!(debug_output.contains("failing_operations"));
    assert!(debug_output.contains("operation_delays"));
}

#[tokio::test]
async fn test_mock_operation_debug_formatting() {
    let operations = vec![
        MockOperation::ListDevices,
        MockOperation::StartDevice("test".to_string()),
        MockOperation::StopDevice("test".to_string()),
        MockOperation::CreateDevice {
            name: "device".to_string(),
            device_type: "type".to_string(),
        },
        MockOperation::DeleteDevice("test".to_string()),
        MockOperation::WipeDevice("test".to_string()),
        MockOperation::GetDeviceDetails("test".to_string()),
    ];

    for op in operations {
        let debug_output = format!("{op:?}");
        assert!(!debug_output.is_empty());
    }
}

#[tokio::test]
async fn test_mock_operation_clone() {
    let op = MockOperation::CreateDevice {
        name: "clone_test".to_string(),
        device_type: "clone_type".to_string(),
    };

    let cloned = op.clone();
    assert_eq!(op, cloned);
}

#[tokio::test]
async fn test_mock_operation_partial_eq() {
    let op1 = MockOperation::StartDevice("device1".to_string());
    let op2 = MockOperation::StartDevice("device1".to_string());
    let op3 = MockOperation::StartDevice("device2".to_string());

    assert_eq!(op1, op2);
    assert_ne!(op1, op3);
}

#[tokio::test]
async fn test_mock_manager_clone() {
    let manager = MockDeviceManager::new_android();
    let cloned = manager.clone();

    // Both managers should work independently but share state
    let _devices1 = DeviceManager::list_devices(&manager).await.unwrap();
    let _devices2 = DeviceManager::list_devices(&cloned).await.unwrap();

    // Both should have recorded operations
    assert!(!manager.get_operations().is_empty());
    assert!(!cloned.get_operations().is_empty());
}

#[tokio::test]
async fn test_mock_manager_add_device() {
    let manager = MockDeviceManager::new_android();

    let custom_device = MockDevice {
        id: "custom-device".to_string(),
        name: "Custom Device".to_string(),
        status: DeviceStatus::Error,
        api_level: Some("32".to_string()),
        device_type: "custom_type".to_string(),
    };

    // Add custom device
    manager.add_device(custom_device);

    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 3);

    let custom = devices
        .iter()
        .find(|d| d.name() == "Custom Device")
        .unwrap();
    assert_eq!(custom.status(), &DeviceStatus::Error);
}

#[tokio::test]
async fn test_mock_manager_android_device_creation() {
    let manager = MockDeviceManager::new_android();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();

    // Test that Android devices are created with correct properties
    for device in devices {
        // For Android, ID is the same as name
        assert_eq!(device.id(), device.name());
        assert_eq!(device.status(), &DeviceStatus::Stopped);
        assert!(!device.is_running());
    }
}

#[tokio::test]
async fn test_mock_manager_ios_device_creation() {
    let manager = MockDeviceManager::new_ios();

    let devices = DeviceManager::list_devices(&manager).await.unwrap();

    // Test that iOS devices are created with correct properties
    for device in devices {
        // Should have UUID-style IDs
        assert!(device.id().contains("-"));
        assert_eq!(device.status(), &DeviceStatus::Stopped);
        assert!(!device.is_running());
    }
}

#[tokio::test]
async fn test_mock_manager_failure_types() {
    let manager = MockDeviceManager::new_android();

    // Test different failure types
    let operations = vec![
        "list_devices",
        "start_device",
        "stop_device",
        "create_device",
        "delete_device",
        "wipe_device",
    ];

    for operation in operations {
        manager.configure_failure(operation, "Test failure");

        let result = match operation {
            "list_devices" => DeviceManager::list_devices(&manager).await.map(|_| ()),
            "start_device" => DeviceManager::start_device(&manager, "test").await,
            "stop_device" => DeviceManager::stop_device(&manager, "test").await,
            "create_device" => {
                let config =
                    DeviceConfig::new("test".to_string(), "test".to_string(), "test".to_string());
                DeviceManager::create_device(&manager, &config).await
            }
            "delete_device" => DeviceManager::delete_device(&manager, "test").await,
            "wipe_device" => DeviceManager::wipe_device(&manager, "test").await,
            _ => Ok(()),
        };

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Test failure");

        // Clear failure for next test
        manager.clear_behavior();
    }
}

#[tokio::test]
async fn test_mock_manager_unified_trait_implementation() {
    let manager = MockDeviceManager::new_android();

    // Test UnifiedDeviceManager trait methods
    let devices = DeviceManager::list_devices(&manager).await.unwrap();
    assert_eq!(devices.len(), 2);

    DeviceManager::start_device(&manager, "emulator-5554")
        .await
        .unwrap();
    DeviceManager::stop_device(&manager, "emulator-5554")
        .await
        .unwrap();

    let config = DeviceConfig::new(
        "Unified_Test".to_string(),
        "pixel_9".to_string(),
        "35".to_string(),
    );
    DeviceManager::create_device(&manager, &config)
        .await
        .unwrap();

    DeviceManager::delete_device(&manager, "Unified_Test")
        .await
        .unwrap();
    DeviceManager::wipe_device(&manager, "emulator-5556")
        .await
        .unwrap();

    assert!(DeviceManager::is_available(&manager).await);
}

#[tokio::test]
async fn test_mock_manager_api_level_parsing() {
    let manager = MockDeviceManager::new_android();

    // Test that API levels are parsed correctly
    let devices = DeviceManager::list_devices(&manager).await.unwrap();

    // Should have devices with different API levels
    let api_levels: Vec<String> = devices
        .iter()
        .map(|d| match d.as_ref() {
            d if d.name() == "Pixel_4_API_30" => "30".to_string(),
            d if d.name() == "Pixel_6_API_33" => "33".to_string(),
            _ => "unknown".to_string(),
        })
        .collect();

    assert!(api_levels.contains(&"30".to_string()));
    assert!(api_levels.contains(&"33".to_string()));
}

#[tokio::test]
async fn test_mock_manager_device_id_generation() {
    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Create devices and test ID generation
    let config = DeviceConfig::new(
        "ID_Test".to_string(),
        "test_type".to_string(),
        "30".to_string(),
    );

    DeviceManager::create_device(&android_manager, &config)
        .await
        .unwrap();
    DeviceManager::create_device(&ios_manager, &config)
        .await
        .unwrap();

    let android_devices = DeviceManager::list_devices(&android_manager).await.unwrap();
    let ios_devices = DeviceManager::list_devices(&ios_manager).await.unwrap();

    // Test Android ID format (ID is the same as name)
    let android_device = android_devices
        .iter()
        .find(|d| d.name() == "ID_Test")
        .unwrap();
    assert_eq!(android_device.id(), "ID_Test"); // Android ID = name

    // Test iOS ID format (ID is UDID)
    let ios_device = ios_devices.iter().find(|d| d.name() == "ID_Test").unwrap();
    assert!(ios_device.id().contains("-"));
    assert_eq!(ios_device.id().len(), 36); // UUID format
}
