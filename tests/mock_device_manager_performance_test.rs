//! Performance tests using MockDeviceManager for emulator-independent testing.
//!
//! This demonstrates how to use MockDeviceManager for performance testing
//! without requiring actual emulator environments.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use std::time::Instant;

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_device_operations_performance() {
    println!("=== MOCK DEVICE OPERATIONS PERFORMANCE TEST ===");

    // Create a mock manager with multiple devices
    let manager = MockDeviceManager::new_android();

    // Add more devices for performance testing
    for i in 0..50 {
        let config = DeviceConfig::new(
            format!("TestDevice{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }

    // Test list_devices performance
    let start = Instant::now();
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let list_duration = start.elapsed();
    println!(
        "list_devices() with {devices_len} devices: {list_duration:?}",
        devices_len = devices.len()
    );

    // Should be very fast with mocks
    assert!(
        list_duration < std::time::Duration::from_millis(50),
        "Mock list_devices should be fast: {list_duration:?}"
    );
    assert_eq!(devices.len(), 52); // 2 default + 50 created

    // Test concurrent operations using device names that exist
    let start = Instant::now();
    let mut tasks = Vec::new();

    // Use the first 10 created devices
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("TestDevice{i}");
            manager_clone.start_device(&device_name).await
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    let concurrent_duration = start.elapsed();
    println!("10 concurrent start_device operations: {concurrent_duration:?}");

    // Should be fast even with concurrent operations
    assert!(
        concurrent_duration < std::time::Duration::from_millis(100),
        "Concurrent mock operations should be fast: {concurrent_duration:?}"
    );

    println!("✅ Mock performance test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_operation_recording_performance() {
    println!("=== MOCK OPERATION RECORDING PERFORMANCE TEST ===");

    let manager = MockDeviceManager::new_ios();

    // Perform many operations
    let start = Instant::now();
    for i in 0..1000 {
        let config = DeviceConfig::new(
            format!("PerfTest{i}"),
            "iPhone15".to_string(),
            "17.0".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }
    let operations_duration = start.elapsed();

    println!("1000 create_device operations: {operations_duration:?}");

    // Check operation recording
    let operations = manager.get_operations();
    println!(
        "Recorded {operations_len} operations",
        operations_len = operations.len()
    );
    assert_eq!(operations.len(), 1000);

    // Should be very fast
    assert!(
        operations_duration < std::time::Duration::from_millis(500),
        "Mock operations should be fast: {operations_duration:?}"
    );

    println!("✅ Operation recording performance test completed");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_with_delays_performance() {
    println!("=== MOCK WITH CONFIGURED DELAYS PERFORMANCE TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure a small delay for operations
    manager.configure_delay("start_device", 10); // 10ms delay

    // Test operation with delay
    let start = Instant::now();
    manager
        .start_device("emulator-5554")
        .await
        .expect("Failed to start device");
    let delayed_duration = start.elapsed();

    println!("start_device with 10ms delay: {delayed_duration:?}");

    // Should respect the configured delay
    assert!(
        delayed_duration >= std::time::Duration::from_millis(10),
        "Should respect configured delay"
    );
    assert!(
        delayed_duration < std::time::Duration::from_millis(50),
        "Should not take much longer than configured delay"
    );

    println!("✅ Delay configuration performance test completed");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_memory_usage() {
    println!("=== MOCK MEMORY USAGE TEST ===");

    let manager = MockDeviceManager::new_android();

    // Create many devices to test memory usage
    for i in 0..10000 {
        let config = DeviceConfig::new(
            format!("MemTest{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }

    // List devices should still be fast
    let start = Instant::now();
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let list_duration = start.elapsed();

    println!(
        "list_devices() with {devices_len} devices: {list_duration:?}",
        devices_len = devices.len()
    );
    assert_eq!(devices.len(), 10002); // 2 default + 10000 created

    // Should still be reasonably fast even with many devices
    assert!(
        list_duration < std::time::Duration::from_millis(1000),
        "Should handle many devices efficiently: {list_duration:?}"
    );

    println!("✅ Memory usage test completed");
}
