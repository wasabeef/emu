//! Device lifecycle tests using MockDeviceManager for emulator-independent testing.
//!
//! This demonstrates how to test complete device lifecycle scenarios
//! without requiring actual emulator environments.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
#[cfg(any(test, feature = "test-utils"))]
use emu::models::DeviceStatus;

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_android_device_complete_lifecycle() {
    println!("=== ANDROID DEVICE LIFECYCLE TEST ===");

    let manager = MockDeviceManager::new_android();

    // Test 1: Create device
    let config = DeviceConfig::new(
        "LifecycleTest".to_string(),
        "pixel_7_pro".to_string(),
        "34".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Failed to create device");

    // Verify device was created
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 3); // 2 default + 1 created

    let test_device = devices
        .iter()
        .find(|d| d.name() == "LifecycleTest")
        .expect("Test device not found");

    assert_eq!(test_device.status(), &DeviceStatus::Stopped);
    assert_eq!(test_device.name(), "LifecycleTest");

    // Test 2: Start device
    manager
        .start_device("LifecycleTest")
        .await
        .expect("Failed to start device");

    // Verify device is running
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "LifecycleTest")
        .expect("Test device not found");

    assert_eq!(test_device.status(), &DeviceStatus::Running);
    assert!(test_device.is_running());

    // Test 3: Stop device
    manager
        .stop_device("LifecycleTest")
        .await
        .expect("Failed to stop device");

    // Verify device is stopped
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "LifecycleTest")
        .expect("Test device not found");

    assert_eq!(test_device.status(), &DeviceStatus::Stopped);
    assert!(!test_device.is_running());

    // Test 4: Wipe device
    manager
        .wipe_device("LifecycleTest")
        .await
        .expect("Failed to wipe device");

    // Test 5: Delete device
    manager
        .delete_device("LifecycleTest")
        .await
        .expect("Failed to delete device");

    // Verify device was deleted
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 2); // Back to original 2 devices
    assert!(devices.iter().all(|d| d.name() != "LifecycleTest"));

    // Test 6: Verify operations were recorded
    let operations = manager.get_operations();
    assert_eq!(operations.len(), 9); // list(1) + create(1) + list(1) + start(1) + list(1) + stop(1) + list(1) + wipe(1) + delete(1)

    println!("✅ Android device lifecycle test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_ios_device_complete_lifecycle() {
    println!("=== IOS DEVICE LIFECYCLE TEST ===");

    let manager = MockDeviceManager::new_ios();

    // Test 1: Create iOS device
    let config = DeviceConfig::new(
        "iPhone 15 Pro Test".to_string(),
        "iPhone15,3".to_string(),
        "17.0".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Failed to create device");

    // Verify device was created
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 3); // 2 default + 1 created

    let test_device = devices
        .iter()
        .find(|d| d.name() == "iPhone 15 Pro Test")
        .expect("Test device not found");

    assert_eq!(test_device.status(), &DeviceStatus::Stopped);

    // Test 2: Device operations
    manager
        .start_device("iPhone 15 Pro Test")
        .await
        .expect("Failed to start device");
    manager
        .stop_device("iPhone 15 Pro Test")
        .await
        .expect("Failed to stop device");
    manager
        .wipe_device("iPhone 15 Pro Test")
        .await
        .expect("Failed to wipe device");
    manager
        .delete_device("iPhone 15 Pro Test")
        .await
        .expect("Failed to delete device");

    // Verify device was deleted
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 2);

    println!("✅ iOS device lifecycle test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_device_not_found_scenarios() {
    println!("=== DEVICE NOT FOUND ERROR SCENARIOS ===");

    let manager = MockDeviceManager::new_android();

    // Test operations on non-existent device
    let result = manager.start_device("NonExistentDevice").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));

    let result = manager.stop_device("NonExistentDevice").await;
    assert!(result.is_err());

    let result = manager.delete_device("NonExistentDevice").await;
    assert!(result.is_err());

    let result = manager.wipe_device("NonExistentDevice").await;
    assert!(result.is_err());

    println!("✅ Error scenario tests completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_operation_failure_scenarios() {
    println!("=== OPERATION FAILURE SCENARIOS ===");

    let manager = MockDeviceManager::new_android();

    // Configure operations to fail
    manager.configure_failure("start_device", "Simulated start failure");
    manager.configure_failure("create_device", "Simulated create failure");

    // Test start device failure
    let result = manager.start_device("emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Simulated start failure");

    // Test create device failure
    let config = DeviceConfig::new(
        "FailureTest".to_string(),
        "pixel_6".to_string(),
        "33".to_string(),
    );
    let result = manager.create_device(&config).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Simulated create failure");

    // Verify devices list still works (not configured to fail)
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 2); // Original devices only

    println!("✅ Operation failure scenarios completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mixed_id_and_name_operations() {
    println!("=== MIXED ID AND NAME OPERATIONS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Create a new device
    let config = DeviceConfig::new(
        "MixedTest".to_string(),
        "pixel_8".to_string(),
        "34".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Failed to create device");

    // Get the device to find its ID
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "MixedTest")
        .expect("Test device not found");
    let device_id = test_device.id().to_string();

    // Test operations using device ID
    manager
        .start_device(&device_id)
        .await
        .expect("Failed to start device by ID");

    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "MixedTest")
        .expect("Test device not found");
    assert_eq!(test_device.status(), &DeviceStatus::Running);

    // Test operations using device name
    manager
        .stop_device("MixedTest")
        .await
        .expect("Failed to stop device by name");

    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "MixedTest")
        .expect("Test device not found");
    assert_eq!(test_device.status(), &DeviceStatus::Stopped);

    // Clean up
    manager
        .delete_device("MixedTest")
        .await
        .expect("Failed to delete device");

    println!("✅ Mixed ID and name operations test completed successfully");
}
