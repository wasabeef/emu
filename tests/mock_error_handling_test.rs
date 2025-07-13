//! Error handling tests using MockDeviceManager for emulator-independent testing.
//!
//! This demonstrates how to test error scenarios and edge cases
//! without requiring actual emulator environments.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::mock::MockOperation;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_comprehensive_error_scenarios() {
    println!("=== COMPREHENSIVE ERROR SCENARIOS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure multiple operations to fail
    manager.configure_failure("start_device", "Android SDK not available");
    manager.configure_failure("stop_device", "Device is unresponsive");
    manager.configure_failure("create_device", "Insufficient disk space");
    manager.configure_failure("delete_device", "Device is currently running");
    manager.configure_failure("wipe_device", "Device is locked");

    // Test all operations fail as expected
    let result = manager.start_device("emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Android SDK not available");

    let result = manager.stop_device("emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Device is unresponsive");

    let config = DeviceConfig::new(
        "ErrorTest".to_string(),
        "pixel_6".to_string(),
        "33".to_string(),
    );
    let result = manager.create_device(&config).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Insufficient disk space");

    let result = manager.delete_device("emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Device is currently running"
    );

    let result = manager.wipe_device("emulator-5554").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Device is locked");

    // Verify list_devices still works (not configured to fail)
    let devices = manager
        .list_devices()
        .await
        .expect("List devices should work");
    assert_eq!(devices.len(), 2);

    // Verify all operations were recorded despite failures
    let operations = manager.get_operations();
    assert_eq!(operations.len(), 6); // 1 list + 5 failed operations

    println!("✅ Comprehensive error scenarios test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_partial_failure_recovery() {
    println!("=== PARTIAL FAILURE RECOVERY TEST ===");

    let manager = MockDeviceManager::new_ios();

    // Configure only specific operations to fail
    manager.configure_failure("start_device", "Simulator is busy");

    // Create device should work
    let config = DeviceConfig::new(
        "RecoveryTest".to_string(),
        "iPhone14,3".to_string(),
        "16.0".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Create should work");

    // Start device should fail
    let result = manager.start_device("RecoveryTest").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Simulator is busy");

    // Remove the failure configuration
    manager.clear_behavior();

    // Now start device should work
    manager
        .start_device("RecoveryTest")
        .await
        .expect("Start should work after recovery");

    // Verify device is running
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let test_device = devices
        .iter()
        .find(|d| d.name() == "RecoveryTest")
        .expect("Test device not found");
    assert!(test_device.is_running());

    println!("✅ Partial failure recovery test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_operation_recording_with_errors() {
    println!("=== OPERATION RECORDING WITH ERRORS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure some operations to fail
    manager.configure_failure("start_device", "Test failure");

    // Perform a mix of successful and failing operations
    let _devices = manager.list_devices().await.expect("List should work");

    let config = DeviceConfig::new(
        "RecordTest".to_string(),
        "pixel_7".to_string(),
        "34".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Create should work");

    let _result = manager.start_device("RecordTest").await; // This will fail

    manager
        .stop_device("RecordTest")
        .await
        .expect("Stop should work");

    // Verify all operations were recorded, including failed ones
    let operations = manager.get_operations();
    assert_eq!(operations.len(), 4);

    // Check specific operations
    assert!(manager.assert_operation_called(&MockOperation::ListDevices));
    assert!(
        manager.assert_operation_called(&MockOperation::CreateDevice {
            name: "RecordTest".to_string(),
            device_type: "pixel_7".to_string(),
        })
    );
    assert!(manager.assert_operation_called(&MockOperation::StartDevice("RecordTest".to_string())));
    assert!(manager.assert_operation_called(&MockOperation::StopDevice("RecordTest".to_string())));

    println!("✅ Operation recording with errors test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_delay_with_error_combination() {
    println!("=== DELAY WITH ERROR COMBINATION TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure delay and failure for the same operation
    manager.configure_delay("create_device", 25); // 25ms delay (more reliable)
    manager.configure_failure("create_device", "Delayed failure");

    let start = std::time::Instant::now();

    let config = DeviceConfig::new(
        "DelayErrorTest".to_string(),
        "pixel_8".to_string(),
        "34".to_string(),
    );
    let result = manager.create_device(&config).await;

    let duration = start.elapsed();

    // Should fail but after applying the delay
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Delayed failure");
    assert!(duration >= std::time::Duration::from_millis(20)); // Allow some tolerance
    assert!(duration < std::time::Duration::from_millis(100)); // Shouldn't take too long

    println!("✅ Delay with error combination test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_concurrent_operations_with_errors() {
    println!("=== CONCURRENT OPERATIONS WITH ERRORS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure alternating failures
    manager.configure_failure("start_device", "Concurrent failure");

    // Create test devices first
    for i in 0..5 {
        let config = DeviceConfig::new(
            format!("ConcurrentTest{i}"),
            "pixel_6".to_string(),
            "33".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }

    // Run concurrent operations (some will fail)
    let mut tasks = Vec::new();

    for i in 0..5 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("ConcurrentTest{i}");
            manager_clone.start_device(&device_name).await
        });
        tasks.push(task);
    }

    // Collect results
    let mut success_count = 0;
    let mut error_count = 0;

    for task in tasks {
        match task.await.expect("Task failed") {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }

    // All operations should have failed due to the configuration
    assert_eq!(success_count, 0);
    assert_eq!(error_count, 5);

    // Verify all operations were recorded
    let operations = manager.get_operations();
    assert!(operations.len() >= 10); // 5 creates + 5 start attempts + lists

    println!("✅ Concurrent operations with errors test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_empty_device_list_scenarios() {
    println!("=== EMPTY DEVICE LIST SCENARIOS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Delete all default devices
    manager
        .delete_device("emulator-5554")
        .await
        .expect("Failed to delete device");
    manager
        .delete_device("emulator-5556")
        .await
        .expect("Failed to delete device");

    // Verify empty list
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    assert_eq!(devices.len(), 0);

    // Test operations on empty device list
    let result = manager.start_device("NonExistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Device not found"));

    // Create new device in empty list
    let config = DeviceConfig::new(
        "FirstDevice".to_string(),
        "pixel_7".to_string(),
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
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].name(), "FirstDevice");

    println!("✅ Empty device list scenarios test completed successfully");
}
