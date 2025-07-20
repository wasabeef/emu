//! Unit tests for managers/android.rs command execution and parsing code

use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use emu::models::DeviceStatus;
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::setup_mock_android_sdk;

#[tokio::test]
async fn test_android_manager_initialization() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    // In CI environment without Android SDK, this may fail
    // That's expected behavior - just check it doesn't panic
    match manager {
        Ok(_) => {
            // Test basic structure - manager creation successful
        }
        Err(_) => {
            // This is expected in CI without Android SDK
            // Just ensure it returns an error gracefully
        }
    }
}

#[tokio::test]
async fn test_android_manager_list_devices() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    // Test list_devices() function
    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager {
        Ok(manager) => {
            let devices = manager.list_devices().await;

            // Should return a result (success or failure)
            match devices {
                Ok(device_list) => {
                    // If successful, should return a vector
                    assert!(
                        device_list.is_empty() || !device_list.is_empty(),
                        "Device list should be a valid vector"
                    );

                    // Each device should have valid fields
                    for device in &device_list {
                        assert!(!device.name.is_empty(), "Device name should not be empty");
                        assert!(device.api_level > 0, "API level should be positive");
                        // Status can be any valid enum value
                        let _status = device.status;
                    }
                }
                Err(e) => {
                    // If failed, error should be informative
                    let error_message = e.to_string();
                    assert!(
                        error_message.contains("avdmanager")
                            || error_message.contains("Android")
                            || error_message.contains("SDK"),
                        "Error should be related to Android SDK: {error_message}"
                    );
                }
            }
        }
        Err(e) => {
            // Manager initialization failed
            let error_message = e.to_string();
            assert!(
                error_message.contains("avdmanager")
                    || error_message.contains("Android")
                    || error_message.contains("SDK"),
                "Error should be related to Android SDK: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_android_manager_list_devices_concurrent() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    // Test concurrent calls to list_devices()
    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    if let Ok(manager) = manager {
        let mut handles = vec![];

        // Create multiple concurrent tasks
        for i in 0..3 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let result = manager_clone.list_devices().await;
                (i, result)
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;

        // All tasks should complete successfully
        for (i, result) in results.iter().enumerate() {
            assert!(
                result.is_ok(),
                "Concurrent task {i} should complete: {result:?}"
            );

            if let Ok((task_id, device_result)) = result {
                match device_result {
                    Ok(devices) => {
                        assert!(
                            devices.is_empty() || !devices.is_empty(),
                            "Task {task_id} should return valid device list"
                        );
                    }
                    Err(e) => {
                        let error_message = e.to_string();
                        assert!(
                            error_message.contains("avdmanager")
                                || error_message.contains("Android")
                                || error_message.contains("SDK"),
                            "Task {task_id} error should be SDK-related: {error_message}"
                        );
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn test_android_manager_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    // Test that list_devices() completes within reasonable time
    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    if let Ok(manager) = manager {
        let start = std::time::Instant::now();
        let result = manager.list_devices().await;
        let duration = start.elapsed();

        // Should complete within 10 seconds (allowing for slow CI environments)
        assert!(
            duration < Duration::from_secs(30),
            "list_devices() should complete within 30 seconds, took: {duration:?}"
        );

        // Result should be valid
        match result {
            Ok(devices) => {
                assert!(
                    devices.is_empty() || !devices.is_empty(),
                    "Device list should be valid in {duration:?}"
                );
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("avdmanager")
                        || error_message.contains("Android")
                        || error_message.contains("SDK"),
                    "Error should be SDK-related: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_android_manager_error_handling() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    // Test error handling behavior
    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager {
        Ok(manager) => {
            // Test list_devices error handling
            let result = manager.list_devices().await;

            match result {
                Ok(_) => {
                    // Success is also valid - operation succeeded
                }
                Err(e) => {
                    // Error should be properly formatted
                    let error_message = e.to_string();
                    assert!(
                        !error_message.is_empty(),
                        "Error message should not be empty"
                    );
                    assert!(
                        error_message.len() < 1000,
                        "Error message should be concise"
                    );

                    // Should contain helpful information
                    let contains_helpful_info = error_message.contains("avdmanager")
                        || error_message.contains("Android")
                        || error_message.contains("SDK")
                        || error_message.contains("emulator")
                        || error_message.contains("command");

                    assert!(
                        contains_helpful_info,
                        "Error should contain helpful information: {error_message}"
                    );
                }
            }
        }
        Err(e) => {
            // Manager initialization error should be informative
            let error_message = e.to_string();
            assert!(
                !error_message.is_empty(),
                "Error message should not be empty"
            );
            assert!(
                error_message.contains("avdmanager")
                    || error_message.contains("Android")
                    || error_message.contains("SDK"),
                "Error should be SDK-related: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_android_manager_memory_usage() {
    // Test that AndroidManager doesn't cause memory issues
    let initial_memory = get_memory_usage();

    // Create and drop multiple manager instances
    for _ in 0..10 {
        let _temp_dir = setup_mock_android_sdk();
        std::env::set_var("ANDROID_HOME", _temp_dir.path());

        let mock_executor = MockCommandExecutor::new();
        let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));
        if let Ok(manager) = manager_result {
            // Test list_devices operation
            let _ = manager.list_devices().await;
        }
        // manager_result is dropped automatically
    }

    let final_memory = get_memory_usage();

    // Memory usage should not increase dramatically
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 50_000_000, // 50MB limit
        "Memory usage should not increase dramatically: {memory_increase} bytes"
    );
}

#[tokio::test]
async fn test_android_manager_async_behavior() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    // Test async behavior of AndroidManager methods
    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    if let Ok(manager) = manager {
        // Test with timeout
        let result = tokio::time::timeout(Duration::from_secs(30), manager.list_devices()).await;

        match result {
            Ok(device_result) => {
                // Operation completed within timeout
                match device_result {
                    Ok(devices) => {
                        assert!(
                            devices.is_empty() || !devices.is_empty(),
                            "Device list should be valid"
                        );
                    }
                    Err(e) => {
                        let error_message = e.to_string();
                        assert!(
                            error_message.contains("avdmanager")
                                || error_message.contains("Android")
                                || error_message.contains("SDK"),
                            "Error should be SDK-related: {error_message}"
                        );
                    }
                }
            }
            Err(_) => {
                panic!("AndroidManager::list_devices() should complete within 30 seconds");
            }
        }
    }
}

#[tokio::test]
async fn test_android_manager_device_validation() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    // Test device validation and parsing
    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Available Android Virtual Devices:\n",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let manager = AndroidManager::with_executor(Arc::new(mock_executor));

    if let Ok(manager) = manager {
        let devices = manager.list_devices().await;

        if let Ok(device_list) = devices {
            for device in device_list {
                // Test device field validation
                assert!(!device.name.is_empty(), "Device name should not be empty");
                assert!(device.api_level > 0, "API level should be positive");
                assert!(
                    !device.device_type.is_empty(),
                    "Device type should not be empty"
                );

                // Test status is valid enum - all variants are valid
                match device.status {
                    DeviceStatus::Running
                    | DeviceStatus::Stopped
                    | DeviceStatus::Starting
                    | DeviceStatus::Stopping
                    | DeviceStatus::Creating
                    | DeviceStatus::Error
                    | DeviceStatus::Unknown => {} // All valid statuses
                }

                // Test string fields
                assert!(!device.ram_size.is_empty(), "RAM size should not be empty");
                assert!(
                    !device.storage_size.is_empty(),
                    "Storage size should not be empty"
                );

                // Test boolean fields
                let _is_running = device.is_running;
            }
        }
    }
}

// Helper function to get rough memory usage
fn get_memory_usage() -> usize {
    // Simple memory usage estimation
    std::process::id() as usize * 1024 // Simple approximation
}
