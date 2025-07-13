//! Startup performance tests using MockDeviceManager for emulator-independent testing.
//!
//! This file replaces the emulator-dependent startup_performance_test.rs with
//! mock-based equivalents that can run in any environment including CI.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use std::time::Instant;

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_startup_performance() {
    println!("=== MOCK STARTUP PERFORMANCE TEST ===");

    // Test MockDeviceManager initialization time
    let start = Instant::now();
    let android_manager = MockDeviceManager::new_android();
    let android_init_duration = start.elapsed();
    println!("Android MockDeviceManager creation: {android_init_duration:?}");

    let start = Instant::now();
    let ios_manager = MockDeviceManager::new_ios();
    let ios_init_duration = start.elapsed();
    println!("iOS MockDeviceManager creation: {ios_init_duration:?}");

    // Test device listing performance
    let start = Instant::now();
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let android_list_duration = start.elapsed();
    println!("Android list_devices(): {android_list_duration:?}");
    println!("Found {} Android devices", android_devices.len());

    let start = Instant::now();
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");
    let ios_list_duration = start.elapsed();
    println!("iOS list_devices(): {ios_list_duration:?}");
    println!("Found {} iOS devices", ios_devices.len());

    // Calculate total startup time
    let total_startup =
        android_init_duration + ios_init_duration + android_list_duration + ios_list_duration;
    println!("📊 Total mock startup time: {total_startup:?}");

    // Performance assertions - mocks should be very fast
    assert!(
        android_init_duration < std::time::Duration::from_millis(10),
        "Mock Android init too slow: {android_init_duration:?}"
    );
    assert!(
        ios_init_duration < std::time::Duration::from_millis(10),
        "Mock iOS init too slow: {ios_init_duration:?}"
    );
    assert!(
        android_list_duration < std::time::Duration::from_millis(10),
        "Mock Android list too slow: {android_list_duration:?}"
    );
    assert!(
        ios_list_duration < std::time::Duration::from_millis(10),
        "Mock iOS list too slow: {ios_list_duration:?}"
    );
    assert!(
        total_startup < std::time::Duration::from_millis(50),
        "Total mock startup too slow: {total_startup:?}"
    );

    println!("✅ Mock startup performance test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_device_operations_performance() {
    println!("=== MOCK DEVICE OPERATIONS PERFORMANCE TEST ===");

    let manager = MockDeviceManager::new_android();

    // Test device creation performance
    let start = Instant::now();
    for i in 0..10 {
        let config = DeviceConfig::new(
            format!("PerfTest{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }
    let create_duration = start.elapsed();
    println!("10 device creations: {create_duration:?}");

    // Test device operations performance
    let start = Instant::now();
    for i in 0..10 {
        let device_name = format!("PerfTest{i}");
        manager
            .start_device(&device_name)
            .await
            .expect("Failed to start device");
        manager
            .stop_device(&device_name)
            .await
            .expect("Failed to stop device");
    }
    let operations_duration = start.elapsed();
    println!("10 start+stop operations: {operations_duration:?}");

    // Test concurrent operations
    let start = Instant::now();
    let mut tasks = Vec::new();
    for i in 0..10 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("PerfTest{i}");
            manager_clone.start_device(&device_name).await
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }
    let concurrent_duration = start.elapsed();
    println!("10 concurrent start operations: {concurrent_duration:?}");

    // Performance assertions
    assert!(
        create_duration < std::time::Duration::from_millis(100),
        "Device creation too slow: {create_duration:?}"
    );
    assert!(
        operations_duration < std::time::Duration::from_millis(100),
        "Device operations too slow: {operations_duration:?}"
    );
    assert!(
        concurrent_duration < std::time::Duration::from_millis(50),
        "Concurrent operations too slow: {concurrent_duration:?}"
    );

    println!("✅ Mock device operations performance test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_parallel_operations_simulation() {
    println!("=== MOCK PARALLEL OPERATIONS SIMULATION TEST ===");

    let manager = MockDeviceManager::new_android();

    // Create devices for testing
    for i in 0..5 {
        let config = DeviceConfig::new(
            format!("ParallelTest{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }

    // Test sequential operations
    let start = Instant::now();
    for i in 0..5 {
        let device_name = format!("ParallelTest{i}");
        let _devices = manager
            .list_devices()
            .await
            .expect("Failed to list devices");
        manager
            .start_device(&device_name)
            .await
            .expect("Failed to start device");
    }
    let sequential_duration = start.elapsed();
    println!("Sequential operations: {sequential_duration:?}");

    // Reset devices
    for i in 0..5 {
        let device_name = format!("ParallelTest{i}");
        manager
            .stop_device(&device_name)
            .await
            .expect("Failed to stop device");
    }

    // Test parallel operations
    let start = Instant::now();
    let mut tasks = Vec::new();

    for i in 0..5 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("ParallelTest{i}");
            let _devices = manager_clone
                .list_devices()
                .await
                .expect("Failed to list devices");
            manager_clone
                .start_device(&device_name)
                .await
                .expect("Failed to start device");
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Task failed");
    }
    let parallel_duration = start.elapsed();
    println!("Parallel operations: {parallel_duration:?}");

    // Calculate improvement
    if sequential_duration > parallel_duration {
        let improvement = ((sequential_duration.as_micros() - parallel_duration.as_micros())
            as f64
            / sequential_duration.as_micros() as f64)
            * 100.0;
        println!("✅ Parallel execution improved by {improvement:.1}%");
    } else {
        println!("⚠️  Parallel execution not faster (expected for lightweight mocks)");
    }

    println!("✅ Mock parallel operations simulation completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_large_device_set_performance() {
    println!("=== MOCK LARGE DEVICE SET PERFORMANCE TEST ===");

    let manager = MockDeviceManager::new_android();

    // Create a large number of devices
    let device_count = 100;
    let start = Instant::now();
    for i in 0..device_count {
        let config = DeviceConfig::new(
            format!("LargeTest{i}"),
            "pixel_6".to_string(),
            "33".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }
    let creation_duration = start.elapsed();
    println!("{device_count} device creations: {creation_duration:?}");

    // Test listing performance with many devices
    let start = Instant::now();
    let devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let list_duration = start.elapsed();
    println!(
        "list_devices() with {} devices: {list_duration:?}",
        devices.len()
    );

    // Test concurrent operations on subset
    let start = Instant::now();
    let mut tasks = Vec::new();
    for i in 0..20 {
        let manager_clone = manager.clone();
        let task = tokio::spawn(async move {
            let device_name = format!("LargeTest{i}");
            manager_clone.start_device(&device_name).await
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }
    let concurrent_subset_duration = start.elapsed();
    println!("20 concurrent operations on large set: {concurrent_subset_duration:?}");

    // Performance assertions for scalability
    assert!(
        creation_duration < std::time::Duration::from_secs(1),
        "Large device creation too slow: {creation_duration:?}"
    );
    assert!(
        list_duration < std::time::Duration::from_millis(100),
        "Large device list too slow: {list_duration:?}"
    );
    assert!(
        concurrent_subset_duration < std::time::Duration::from_millis(100),
        "Concurrent operations on large set too slow: {concurrent_subset_duration:?}"
    );

    println!("✅ Mock large device set performance test completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_incremental_refresh_simulation() {
    println!("=== MOCK INCREMENTAL REFRESH SIMULATION TEST ===");

    let manager = MockDeviceManager::new_android();

    // Initial device set
    for i in 0..10 {
        let config = DeviceConfig::new(
            format!("RefreshTest{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
    }

    // Simulate full refresh
    let start = Instant::now();
    let initial_devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let full_refresh_duration = start.elapsed();
    println!(
        "Full refresh ({}): {full_refresh_duration:?}",
        initial_devices.len()
    );

    // Simulate incremental changes
    manager
        .create_device(&DeviceConfig::new(
            "NewDevice".to_string(),
            "pixel_8".to_string(),
            "34".to_string(),
        ))
        .await
        .expect("Failed to create new device");

    // Simulate incremental refresh (just the difference)
    let start = Instant::now();
    let updated_devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let incremental_refresh_duration = start.elapsed();
    println!(
        "Incremental refresh ({}): {incremental_refresh_duration:?}",
        updated_devices.len()
    );

    // Compare operation counts
    let initial_ops = manager.get_operations().len();

    // Additional operations
    manager
        .start_device("RefreshTest0")
        .await
        .expect("Failed to start device");
    manager
        .stop_device("RefreshTest1")
        .await
        .expect("Failed to stop device");

    let final_ops = manager.get_operations().len();
    println!("Operation count increased by: {}", final_ops - initial_ops);

    // Performance assertions
    assert!(
        full_refresh_duration < std::time::Duration::from_millis(50),
        "Full refresh too slow: {full_refresh_duration:?}"
    );
    assert!(
        incremental_refresh_duration < std::time::Duration::from_millis(10),
        "Incremental refresh too slow: {incremental_refresh_duration:?}"
    );

    println!("✅ Mock incremental refresh simulation completed successfully");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_operations_with_delays() {
    println!("=== MOCK OPERATIONS WITH DELAYS TEST ===");

    let manager = MockDeviceManager::new_android();

    // Configure realistic delays to simulate real manager behavior
    manager.configure_delay("create_device", 5); // 5ms
    manager.configure_delay("start_device", 3); // 3ms
    manager.configure_delay("list_devices", 1); // 1ms

    // Test with delays
    let start = Instant::now();

    let config = DeviceConfig::new(
        "DelayTest".to_string(),
        "pixel_7".to_string(),
        "34".to_string(),
    );
    manager
        .create_device(&config)
        .await
        .expect("Failed to create device");
    manager
        .start_device("DelayTest")
        .await
        .expect("Failed to start device");
    let _devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");

    let total_with_delays = start.elapsed();
    println!("Operations with realistic delays: {total_with_delays:?}");

    // Clear delays and test again
    manager.clear_behavior();

    let start = Instant::now();
    manager
        .stop_device("DelayTest")
        .await
        .expect("Failed to stop device");
    manager
        .delete_device("DelayTest")
        .await
        .expect("Failed to delete device");
    let _devices = manager
        .list_devices()
        .await
        .expect("Failed to list devices");
    let total_without_delays = start.elapsed();
    println!("Operations without delays: {total_without_delays:?}");

    // Should see the difference in timing
    assert!(
        total_with_delays >= std::time::Duration::from_millis(9), // 5+3+1
        "Delays not applied correctly: {total_with_delays:?}"
    );
    assert!(
        total_without_delays < std::time::Duration::from_millis(5),
        "Operations without delays too slow: {total_without_delays:?}"
    );

    println!("✅ Mock operations with delays test completed successfully");
}
