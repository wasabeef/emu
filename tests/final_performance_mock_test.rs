//! Final performance validation tests using MockDeviceManager for emulator-independent testing.
//!
//! This file replaces the emulator-dependent final_performance_test.rs with
//! comprehensive mock-based performance validation that can run in any environment.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use std::time::Instant;

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_startup_performance_validation() {
    println!("=== MOCK STARTUP PERFORMANCE VALIDATION ===");

    let iterations = 10;
    let mut android_startup_times = Vec::new();
    let mut ios_startup_times = Vec::new();
    let mut combined_startup_times = Vec::new();

    for i in 1..=iterations {
        println!("Iteration {i}/{iterations}");

        // Test Android manager startup
        let start = Instant::now();
        let android_manager = MockDeviceManager::new_android();
        let android_devices = android_manager
            .list_devices()
            .await
            .expect("Failed to list Android devices");
        let android_duration = start.elapsed();
        android_startup_times.push(android_duration);
        println!(
            "  ✅ Android startup: {android_duration:?} ({} devices)",
            android_devices.len()
        );

        // Test iOS manager startup
        let start = Instant::now();
        let ios_manager = MockDeviceManager::new_ios();
        let ios_devices = ios_manager
            .list_devices()
            .await
            .expect("Failed to list iOS devices");
        let ios_duration = start.elapsed();
        ios_startup_times.push(ios_duration);
        println!(
            "  ✅ iOS startup: {ios_duration:?} ({} devices)",
            ios_devices.len()
        );

        // Test combined startup
        let start = Instant::now();
        let _android_manager = MockDeviceManager::new_android();
        let _ios_manager = MockDeviceManager::new_ios();
        let _android_devices = _android_manager
            .list_devices()
            .await
            .expect("Failed to list Android devices");
        let _ios_devices = _ios_manager
            .list_devices()
            .await
            .expect("Failed to list iOS devices");
        let combined_duration = start.elapsed();
        combined_startup_times.push(combined_duration);
        println!("  ✅ Combined startup: {combined_duration:?}");
    }

    // Calculate statistics
    let android_avg = android_startup_times.iter().sum::<std::time::Duration>() / iterations;
    let android_max = *android_startup_times.iter().max().unwrap();
    let ios_avg = ios_startup_times.iter().sum::<std::time::Duration>() / iterations;
    let ios_max = *ios_startup_times.iter().max().unwrap();
    let combined_avg = combined_startup_times.iter().sum::<std::time::Duration>() / iterations;
    let combined_max = *combined_startup_times.iter().max().unwrap();

    println!("\n📊 MOCK STARTUP PERFORMANCE SUMMARY:");
    println!("Android - Average: {android_avg:?}, Maximum: {android_max:?}");
    println!("iOS     - Average: {ios_avg:?}, Maximum: {ios_max:?}");
    println!("Combined- Average: {combined_avg:?}, Maximum: {combined_max:?}");

    // Performance assertions - mocks should be very fast
    assert!(
        android_avg < std::time::Duration::from_millis(10),
        "Android mock startup too slow: {android_avg:?}"
    );
    assert!(
        ios_avg < std::time::Duration::from_millis(10),
        "iOS mock startup too slow: {ios_avg:?}"
    );
    assert!(
        combined_avg < std::time::Duration::from_millis(20),
        "Combined mock startup too slow: {combined_avg:?}"
    );

    assert!(
        android_max < std::time::Duration::from_millis(50),
        "Android mock startup max too slow: {android_max:?}"
    );
    assert!(
        ios_max < std::time::Duration::from_millis(50),
        "iOS mock startup max too slow: {ios_max:?}"
    );
    assert!(
        combined_max < std::time::Duration::from_millis(100),
        "Combined mock startup max too slow: {combined_max:?}"
    );

    println!("✅ ALL MOCK STARTUP PERFORMANCE THRESHOLDS MET!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_comprehensive_device_operations_validation() {
    println!("=== MOCK COMPREHENSIVE DEVICE OPERATIONS VALIDATION ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Test comprehensive device lifecycle operations
    let device_count = 20;
    let start = Instant::now();

    // Create devices
    for i in 0..device_count {
        let android_config = DeviceConfig::new(
            format!("ValidationAndroid{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("ValidationiOS{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    let creation_duration = start.elapsed();
    println!("Created {device_count} devices per platform in: {creation_duration:?}");

    // Test device operations
    let start = Instant::now();
    for i in 0..device_count {
        let android_device = format!("ValidationAndroid{i}");
        let ios_device = format!("ValidationiOS{i}");

        android_manager
            .start_device(&android_device)
            .await
            .expect("Failed to start Android device");
        ios_manager
            .start_device(&ios_device)
            .await
            .expect("Failed to start iOS device");

        if i % 2 == 0 {
            android_manager
                .stop_device(&android_device)
                .await
                .expect("Failed to stop Android device");
            ios_manager
                .stop_device(&ios_device)
                .await
                .expect("Failed to stop iOS device");
        }
    }

    let operations_duration = start.elapsed();
    println!(
        "Performed {} operations per platform in: {operations_duration:?}",
        device_count * 2
    );

    // Test cleanup
    let start = Instant::now();
    for i in 0..device_count {
        let android_device = format!("ValidationAndroid{i}");
        let ios_device = format!("ValidationiOS{i}");

        android_manager
            .wipe_device(&android_device)
            .await
            .expect("Failed to wipe Android device");
        ios_manager
            .wipe_device(&ios_device)
            .await
            .expect("Failed to wipe iOS device");

        android_manager
            .delete_device(&android_device)
            .await
            .expect("Failed to delete Android device");
        ios_manager
            .delete_device(&ios_device)
            .await
            .expect("Failed to delete iOS device");
    }

    let cleanup_duration = start.elapsed();
    println!("Cleaned up {device_count} devices per platform in: {cleanup_duration:?}");

    // Verify final state
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    assert_eq!(
        android_devices.len(),
        2,
        "Should return to original 2 Android devices"
    );
    assert_eq!(
        ios_devices.len(),
        2,
        "Should return to original 2 iOS devices"
    );

    // Performance assertions
    assert!(
        creation_duration < std::time::Duration::from_secs(1),
        "Device creation too slow: {creation_duration:?}"
    );
    assert!(
        operations_duration < std::time::Duration::from_secs(1),
        "Device operations too slow: {operations_duration:?}"
    );
    assert!(
        cleanup_duration < std::time::Duration::from_secs(1),
        "Device cleanup too slow: {cleanup_duration:?}"
    );

    println!("✅ Comprehensive device operations validation completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_concurrent_stress_test() {
    println!("=== MOCK CONCURRENT STRESS TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Stress test with concurrent operations
    let concurrent_tasks = 50;
    let start = Instant::now();

    let mut tasks = Vec::new();

    // Android concurrent operations
    for i in 0..concurrent_tasks {
        let manager_clone = android_manager.clone();
        let task = tokio::spawn(async move {
            let config = DeviceConfig::new(
                format!("StressAndroid{i}"),
                "pixel_7".to_string(),
                "34".to_string(),
            );
            manager_clone.create_device(&config).await?;
            manager_clone
                .start_device(&format!("StressAndroid{i}"))
                .await?;
            manager_clone.list_devices().await?;
            manager_clone
                .stop_device(&format!("StressAndroid{i}"))
                .await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // iOS concurrent operations
    for i in 0..concurrent_tasks {
        let manager_clone = ios_manager.clone();
        let task = tokio::spawn(async move {
            let config = DeviceConfig::new(
                format!("StressiOS{i}"),
                "iPhone15,3".to_string(),
                "17.0".to_string(),
            );
            manager_clone.create_device(&config).await?;
            manager_clone.start_device(&format!("StressiOS{i}")).await?;
            manager_clone.list_devices().await?;
            manager_clone.stop_device(&format!("StressiOS{i}")).await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // Wait for all tasks
    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    let stress_duration = start.elapsed();
    println!(
        "Completed {concurrent_tasks} concurrent operations per platform in: {stress_duration:?}"
    );

    // Verify results
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    println!("Final Android devices: {}", android_devices.len());
    println!("Final iOS devices: {}", ios_devices.len());

    // Check operation counts
    let android_ops = android_manager.get_operations().len();
    let ios_ops = ios_manager.get_operations().len();

    println!("Android operations recorded: {android_ops}");
    println!("iOS operations recorded: {ios_ops}");

    // Performance assertions
    assert!(
        stress_duration < std::time::Duration::from_secs(5),
        "Concurrent stress test too slow: {stress_duration:?}"
    );
    assert!(
        android_devices.len() == 2 + concurrent_tasks as usize,
        "Unexpected Android device count: {}",
        android_devices.len()
    );
    assert!(
        ios_devices.len() == 2 + concurrent_tasks as usize,
        "Unexpected iOS device count: {}",
        ios_devices.len()
    );

    println!("✅ Concurrent stress test completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_memory_efficiency_validation() {
    println!("=== MOCK MEMORY EFFICIENCY VALIDATION ===");

    // Test memory efficiency with repeated creation/destruction
    let iterations = 100;
    let start = Instant::now();

    for _i in 0..iterations {
        let android_manager = MockDeviceManager::new_android();
        let ios_manager = MockDeviceManager::new_ios();

        // Perform some operations
        let config = DeviceConfig::new(
            "MemoryTest".to_string(),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&config)
            .await
            .expect("Failed to create device");
        ios_manager
            .create_device(&config)
            .await
            .expect("Failed to create device");

        let _android_devices = android_manager
            .list_devices()
            .await
            .expect("Failed to list Android devices");
        let _ios_devices = ios_manager
            .list_devices()
            .await
            .expect("Failed to list iOS devices");

        android_manager
            .delete_device("MemoryTest")
            .await
            .expect("Failed to delete Android device");
        ios_manager
            .delete_device("MemoryTest")
            .await
            .expect("Failed to delete iOS device");

        // Managers are automatically dropped here
    }

    let total_duration = start.elapsed();
    let avg_iteration_time = total_duration / iterations;

    println!("Memory efficiency test:");
    println!("  {iterations} iterations: {total_duration:?}");
    println!("  Average iteration time: {avg_iteration_time:?}");

    // Should be very efficient
    assert!(
        avg_iteration_time < std::time::Duration::from_millis(10),
        "Memory efficiency iteration too slow: {avg_iteration_time:?}"
    );

    println!("✅ Memory efficiency validation completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_error_recovery_validation() {
    println!("=== MOCK ERROR RECOVERY VALIDATION ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Configure some operations to fail
    android_manager.configure_failure("start_device", "Simulated Android failure");
    ios_manager.configure_failure("start_device", "Simulated iOS failure");

    // Test error handling performance
    let start = Instant::now();

    for i in 0..10 {
        let android_config = DeviceConfig::new(
            format!("ErrorTestAndroid{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        let ios_config = DeviceConfig::new(
            format!("ErrorTestiOS{i}"),
            "iPhone14,3".to_string(),
            "16.0".to_string(),
        );

        android_manager
            .create_device(&android_config)
            .await
            .expect("Create should work");
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Create should work");

        // These should fail
        let android_result = android_manager
            .start_device(&format!("ErrorTestAndroid{i}"))
            .await;
        let ios_result = ios_manager.start_device(&format!("ErrorTestiOS{i}")).await;

        assert!(android_result.is_err(), "Android start should fail");
        assert!(ios_result.is_err(), "iOS start should fail");
    }

    let error_handling_duration = start.elapsed();
    println!("Error handling for 20 operations: {error_handling_duration:?}");

    // Clear failures and test recovery
    android_manager.clear_behavior();
    ios_manager.clear_behavior();

    let start = Instant::now();

    // These should now work
    for i in 0..10 {
        android_manager
            .start_device(&format!("ErrorTestAndroid{i}"))
            .await
            .expect("Start should work after recovery");
        ios_manager
            .start_device(&format!("ErrorTestiOS{i}"))
            .await
            .expect("Start should work after recovery");
    }

    let recovery_duration = start.elapsed();
    println!("Recovery operations for 20 devices: {recovery_duration:?}");

    // Performance assertions
    assert!(
        error_handling_duration < std::time::Duration::from_secs(1),
        "Error handling too slow: {error_handling_duration:?}"
    );
    assert!(
        recovery_duration < std::time::Duration::from_millis(500),
        "Recovery too slow: {recovery_duration:?}"
    );

    println!("✅ Error recovery validation completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn mock_final_integration_validation() {
    println!("=== MOCK FINAL INTEGRATION VALIDATION ===");

    // Ultimate integration test combining all aspects
    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    let start = Instant::now();

    // Phase 1: Setup
    for i in 0..5 {
        let android_config = DeviceConfig::new(
            format!("IntegrationAndroid{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        let ios_config = DeviceConfig::new(
            format!("IntegrationiOS{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );

        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Phase 2: Operations with delays
    android_manager.configure_delay("list_devices", 2);
    ios_manager.configure_delay("list_devices", 2);

    let mut concurrent_tasks = Vec::new();

    for i in 0..5 {
        let android_clone = android_manager.clone();
        let ios_clone = ios_manager.clone();

        let task = tokio::spawn(async move {
            let android_device = format!("IntegrationAndroid{i}");
            let ios_device = format!("IntegrationiOS{i}");

            android_clone.start_device(&android_device).await?;
            ios_clone.start_device(&ios_device).await?;

            let _android_devices = android_clone.list_devices().await?;
            let _ios_devices = ios_clone.list_devices().await?;

            android_clone.stop_device(&android_device).await?;
            ios_clone.stop_device(&ios_device).await?;

            anyhow::Ok(())
        });
        concurrent_tasks.push(task);
    }

    for task in concurrent_tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    let integration_duration = start.elapsed();
    println!("Final integration test duration: {integration_duration:?}");

    // Verify final state
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");
    let android_ops = android_manager.get_operations().len();
    let ios_ops = ios_manager.get_operations().len();

    println!("Final state:");
    println!(
        "  Android devices: {android_len}",
        android_len = android_devices.len()
    );
    println!("  iOS devices: {ios_len}", ios_len = ios_devices.len());
    println!("  Android operations: {android_ops}");
    println!("  iOS operations: {ios_ops}");

    // Final assertions
    assert_eq!(android_devices.len(), 7); // 2 original + 5 created
    assert_eq!(ios_devices.len(), 7); // 2 original + 5 created
    assert!(android_ops > 20); // Multiple operations per device
    assert!(ios_ops > 20); // Multiple operations per device
    assert!(
        integration_duration < std::time::Duration::from_secs(2),
        "Final integration test too slow: {integration_duration:?}"
    );

    println!("✅ FINAL INTEGRATION VALIDATION COMPLETED SUCCESSFULLY!");
    println!("🎉 ALL MOCK PERFORMANCE TESTS PASSED!");
}
