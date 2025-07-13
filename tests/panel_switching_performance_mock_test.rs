//! Panel switching performance tests using MockDeviceManager for emulator-independent testing.
//!
//! This file replaces the emulator-dependent panel_switching_performance_test.rs with
//! mock-based equivalents that test the core state management and UI responsiveness logic.

#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use std::time::Instant;

#[test]
fn test_panel_switching_algorithm_performance() {
    println!("=== PANEL SWITCHING ALGORITHM PERFORMANCE TEST ===");

    use emu::app::state::{AppState, Panel};

    let mut state = AppState::new();

    // Test the panel switching logic performance
    let iterations = 10000;
    let start = Instant::now();

    for _ in 0..iterations {
        // Simulate rapid panel switching
        state.next_panel();
        state.active_panel = match state.active_panel {
            Panel::Android => Panel::Ios,
            Panel::Ios => Panel::Android,
        };
    }

    let duration = start.elapsed();
    let avg_switch_time = duration / iterations;

    println!("10,000 panel switches completed in: {duration:?}");
    println!(
        "Average switch time: {avg_switch_time:?} ({nanos} ns)",
        nanos = avg_switch_time.as_nanos()
    );

    // Should be extremely fast - just memory operations
    assert!(
        avg_switch_time < std::time::Duration::from_nanos(1000),
        "Panel switching algorithm too slow: {nanos} ns",
        nanos = avg_switch_time.as_nanos()
    );

    println!("✅ Panel switching algorithm is highly optimized!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_panel_data_switching_performance() {
    println!("=== MOCK PANEL DATA SWITCHING PERFORMANCE TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Create some devices on each platform
    for i in 0..5 {
        let android_config = DeviceConfig::new(
            format!("AndroidDevice{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("iOSDevice{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Simulate rapid panel switching with data loading
    let switch_count = 100;
    let mut switch_times = Vec::new();

    for i in 0..switch_count {
        let start = Instant::now();

        if i % 2 == 0 {
            // Switch to Android panel - load Android devices
            let _android_devices = android_manager
                .list_devices()
                .await
                .expect("Failed to list Android devices");
        } else {
            // Switch to iOS panel - load iOS devices
            let _ios_devices = ios_manager
                .list_devices()
                .await
                .expect("Failed to list iOS devices");
        }

        let duration = start.elapsed();
        switch_times.push(duration);
    }

    // Calculate statistics
    let avg_time = switch_times.iter().sum::<std::time::Duration>() / switch_times.len() as u32;
    let max_time = switch_times.iter().max().unwrap();
    let min_time = switch_times.iter().min().unwrap();

    println!("\n📊 MOCK PANEL DATA SWITCHING PERFORMANCE:");
    println!(
        "  Average switch time: {avg_time:?} ({avg_micros} μs)",
        avg_micros = avg_time.as_micros()
    );
    println!(
        "  Maximum switch time: {max_time:?} ({max_micros} μs)",
        max_micros = max_time.as_micros()
    );
    println!(
        "  Minimum switch time: {min_time:?} ({min_micros} μs)",
        min_micros = min_time.as_micros()
    );

    // Performance assertions for mock data loading
    assert!(
        *max_time < std::time::Duration::from_millis(10),
        "Mock panel data switching too slow: {max_micros} μs > 10ms threshold",
        max_micros = max_time.as_micros()
    );

    assert!(
        avg_time < std::time::Duration::from_millis(1),
        "Average mock panel data switching too slow: {avg_micros} μs > 1ms threshold",
        avg_micros = avg_time.as_micros()
    );

    println!("✅ Mock panel data switching meets high responsiveness standards!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_concurrent_panel_operations() {
    println!("=== MOCK CONCURRENT PANEL OPERATIONS TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Test concurrent operations that might happen during panel switching
    let start = Instant::now();

    let mut tasks = Vec::new();

    // Simulate Android panel operations
    for i in 0..5 {
        let manager_clone = android_manager.clone();
        let task = tokio::spawn(async move {
            let config = DeviceConfig::new(
                format!("ConcurrentAndroid{i}"),
                "pixel_8".to_string(),
                "34".to_string(),
            );
            manager_clone.create_device(&config).await?;
            manager_clone.list_devices().await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // Simulate iOS panel operations
    for i in 0..5 {
        let manager_clone = ios_manager.clone();
        let task = tokio::spawn(async move {
            let config = DeviceConfig::new(
                format!("ConcurrentiOS{i}"),
                "iPhone15,3".to_string(),
                "17.0".to_string(),
            );
            manager_clone.create_device(&config).await?;
            manager_clone.list_devices().await?;
            anyhow::Ok(())
        });
        tasks.push(task);
    }

    // Wait for all concurrent operations
    for task in tasks {
        task.await.expect("Task failed").expect("Operation failed");
    }

    let concurrent_duration = start.elapsed();
    println!("10 concurrent panel operations: {concurrent_duration:?}");

    // Verify results
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    println!(
        "Android devices: {android_len}",
        android_len = android_devices.len()
    );
    println!("iOS devices: {ios_len}", ios_len = ios_devices.len());

    // Performance assertion
    assert!(
        concurrent_duration < std::time::Duration::from_millis(100),
        "Concurrent panel operations too slow: {concurrent_duration:?}"
    );

    println!("✅ Mock concurrent panel operations completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_state_consistency_during_switching() {
    println!("=== MOCK STATE CONSISTENCY DURING SWITCHING TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Create initial state
    for i in 0..3 {
        let android_config = DeviceConfig::new(
            format!("StateTestAndroid{i}"),
            "pixel_7".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("StateTestiOS{i}"),
            "iPhone14,3".to_string(),
            "16.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Perform rapid switching with state modifications
    for cycle in 0..10 {
        // Android panel operations
        let android_devices = android_manager
            .list_devices()
            .await
            .expect("Failed to list Android devices");
        assert_eq!(android_devices.len(), 5); // 2 default + 3 created

        // Modify Android state
        if cycle < 5 {
            android_manager
                .start_device(&format!(
                    "StateTestAndroid{device_index}",
                    device_index = cycle % 3
                ))
                .await
                .expect("Failed to start Android device");
        } else {
            android_manager
                .stop_device(&format!(
                    "StateTestAndroid{device_index}",
                    device_index = cycle % 3
                ))
                .await
                .expect("Failed to stop Android device");
        }

        // Switch to iOS panel
        let ios_devices = ios_manager
            .list_devices()
            .await
            .expect("Failed to list iOS devices");
        assert_eq!(ios_devices.len(), 5); // 2 default + 3 created

        // Modify iOS state
        if cycle < 5 {
            ios_manager
                .start_device(&format!(
                    "StateTestiOS{device_index}",
                    device_index = cycle % 3
                ))
                .await
                .expect("Failed to start iOS device");
        } else {
            ios_manager
                .stop_device(&format!(
                    "StateTestiOS{device_index}",
                    device_index = cycle % 3
                ))
                .await
                .expect("Failed to stop iOS device");
        }
    }

    // Verify final state consistency
    let final_android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let final_ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    println!(
        "Final Android devices: {android_len}",
        android_len = final_android_devices.len()
    );
    println!(
        "Final iOS devices: {ios_len}",
        ios_len = final_ios_devices.len()
    );

    // Check operation counts
    let android_ops = android_manager.get_operations().len();
    let ios_ops = ios_manager.get_operations().len();

    println!("Android operations recorded: {android_ops}");
    println!("iOS operations recorded: {ios_ops}");

    // State should remain consistent
    assert_eq!(final_android_devices.len(), 5);
    assert_eq!(final_ios_devices.len(), 5);
    assert!(android_ops > 0);
    assert!(ios_ops > 0);

    println!("✅ State consistency maintained during rapid switching!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_panel_switching_with_delays() {
    println!("=== MOCK PANEL SWITCHING WITH DELAYS TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Configure delays to simulate realistic panel switching delays
    android_manager.configure_delay("list_devices", 25); // Fast detail update debounce
    ios_manager.configure_delay("list_devices", 25);

    // Test panel switching with realistic delays
    let switch_count = 20;
    let start = Instant::now();

    for i in 0..switch_count {
        if i % 2 == 0 {
            let _android_devices = android_manager
                .list_devices()
                .await
                .expect("Failed to list Android devices");
        } else {
            let _ios_devices = ios_manager
                .list_devices()
                .await
                .expect("Failed to list iOS devices");
        }
    }

    let total_duration = start.elapsed();
    let avg_switch_time = total_duration / switch_count;

    println!("Panel switching with delays:");
    println!("  Total time for {switch_count} switches: {total_duration:?}");
    println!("  Average time per switch: {avg_switch_time:?}");

    // Should respect configured delays but still be reasonable
    let expected_min_time = std::time::Duration::from_millis(25 * switch_count as u64);
    assert!(
        total_duration >= expected_min_time,
        "Delays not being applied correctly"
    );
    assert!(
        total_duration < std::time::Duration::from_secs(2),
        "Panel switching with delays too slow: {total_duration:?}"
    );

    println!("✅ Mock panel switching with delays completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_memory_efficiency_simulation() {
    println!("=== MOCK MEMORY EFFICIENCY SIMULATION TEST ===");

    // Test that mock managers can be created and destroyed efficiently
    let iterations = 100;
    let start = Instant::now();

    for _i in 0..iterations {
        let android_manager = MockDeviceManager::new_android();
        let ios_manager = MockDeviceManager::new_ios();

        // Simulate some operations
        let _android_devices = android_manager
            .list_devices()
            .await
            .expect("Failed to list Android devices");
        let _ios_devices = ios_manager
            .list_devices()
            .await
            .expect("Failed to list iOS devices");

        // Managers are automatically dropped here
    }

    let total_duration = start.elapsed();
    let avg_creation_time = total_duration / iterations;

    println!("Memory efficiency simulation:");
    println!("  {iterations} manager pairs created/destroyed: {total_duration:?}");
    println!("  Average creation time: {avg_creation_time:?}");

    // Should be very fast for mock objects
    assert!(
        avg_creation_time < std::time::Duration::from_millis(1),
        "Mock manager creation too slow: {avg_creation_time:?}"
    );

    println!("✅ Mock memory efficiency simulation completed successfully!");
}

#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_mock_panel_responsiveness_under_load() {
    println!("=== MOCK PANEL RESPONSIVENESS UNDER LOAD TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();

    // Create a large number of devices to simulate load
    for i in 0..50 {
        let android_config = DeviceConfig::new(
            format!("LoadTestAndroid{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("LoadTestiOS{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Test panel switching responsiveness under load
    let mut switch_times = Vec::new();

    for i in 0..20 {
        let start = Instant::now();

        if i % 2 == 0 {
            let android_devices = android_manager
                .list_devices()
                .await
                .expect("Failed to list Android devices");
            assert_eq!(android_devices.len(), 52); // 2 default + 50 created
        } else {
            let ios_devices = ios_manager
                .list_devices()
                .await
                .expect("Failed to list iOS devices");
            assert_eq!(ios_devices.len(), 52); // 2 default + 50 created
        }

        let duration = start.elapsed();
        switch_times.push(duration);
    }

    let avg_time = switch_times.iter().sum::<std::time::Duration>() / switch_times.len() as u32;
    let max_time = switch_times.iter().max().unwrap();

    println!("Panel responsiveness under load (52 devices per platform):");
    println!("  Average switch time: {avg_time:?}");
    println!("  Maximum switch time: {max_time:?}");

    // Should remain responsive even with many devices
    assert!(
        *max_time < std::time::Duration::from_millis(50),
        "Panel switching under load too slow: {max_time:?}"
    );
    assert!(
        avg_time < std::time::Duration::from_millis(10),
        "Average panel switching under load too slow: {avg_time:?}"
    );

    println!("✅ Mock panel responsiveness under load test completed successfully!");
}
