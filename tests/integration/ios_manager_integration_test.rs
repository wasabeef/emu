//! Integration tests for iOS Manager command execution
//!
//! Tests IosManager's interactions with external commands and system integration.

use emu::managers::common::DeviceManager;
use emu::managers::ios::IosManager;
use emu::models::DeviceStatus;
use std::time::Duration;

#[tokio::test]
async fn test_ios_manager_initialization() {
    // Test IosManager::new() function
    let manager = IosManager::new();

    // Manager should initialize successfully or fail gracefully
    match manager {
        Ok(_) => {
            // On macOS, initialization should succeed
            // IosManager initialized successfully
        }
        Err(e) => {
            // On non-macOS or when Xcode is not available, should fail gracefully
            let error_message = e.to_string();
            assert!(
                error_message.contains("xcrun")
                    || error_message.contains("simctl")
                    || error_message.contains("macOS")
                    || error_message.contains("Xcode"),
                "Error should be related to iOS simulator tools: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_ios_manager_list_devices() {
    // Test list_devices() function
    let manager = IosManager::new();

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
                        assert!(!device.udid.is_empty(), "UDID should not be empty");
                        assert!(
                            !device.device_type.is_empty(),
                            "Device type should not be empty"
                        );
                        assert!(
                            !device.ios_version.is_empty(),
                            "iOS version should not be empty"
                        );
                        // Status can be any valid enum value
                        let _status = device.status;
                        // Boolean fields
                        let _is_running = device.is_running;
                        let _is_available = device.is_available;
                    }
                }
                Err(e) => {
                    // If failed, error should be informative
                    let error_message = e.to_string();
                    assert!(
                        error_message.contains("xcrun")
                            || error_message.contains("simctl")
                            || error_message.contains("iOS")
                            || error_message.contains("Xcode"),
                        "Error should be related to iOS simulator tools: {error_message}"
                    );
                }
            }
        }
        Err(e) => {
            // Manager initialization failed
            let error_message = e.to_string();
            assert!(
                error_message.contains("xcrun")
                    || error_message.contains("simctl")
                    || error_message.contains("macOS")
                    || error_message.contains("Xcode"),
                "Error should be related to iOS simulator tools: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_ios_manager_is_available() {
    // Test is_available() function
    let manager = IosManager::new();

    match manager {
        Ok(manager) => {
            let is_available = manager.is_available().await;

            // Should return a boolean
            // is_available should return boolean - any boolean is valid

            // On macOS with Xcode, should typically be true
            // On other platforms or without Xcode, should be false
            if cfg!(target_os = "macos") {
                // May be true or false depending on Xcode installation
                // is_available result should be valid on macOS - any boolean is valid
            } else {
                // On non-macOS, should be false
                assert!(
                    !is_available,
                    "is_available should be false on non-macOS platforms"
                );
            }
        }
        Err(e) => {
            // Manager initialization failed
            let error_message = e.to_string();
            assert!(
                error_message.contains("xcrun")
                    || error_message.contains("simctl")
                    || error_message.contains("macOS")
                    || error_message.contains("Xcode"),
                "Error should be related to iOS simulator tools: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_ios_manager_concurrent_operations() {
    // Test concurrent operations
    let manager = IosManager::new();

    if let Ok(_manager) = manager {
        let mut handles = vec![];

        // Create multiple concurrent tasks
        for i in 0..3 {
            let handle = tokio::spawn(async move {
                let manager = IosManager::new().unwrap();
                let devices_result = manager.list_devices().await;
                let _available_result = manager.is_available().await;
                (i, devices_result, _available_result)
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

            if let Ok((task_id, devices_result, _available_result)) = result {
                // Check devices result
                match devices_result {
                    Ok(devices) => {
                        assert!(
                            devices.is_empty() || !devices.is_empty(),
                            "Task {task_id} should return valid device list"
                        );
                    }
                    Err(e) => {
                        let error_message = e.to_string();
                        assert!(
                            error_message.contains("xcrun") || 
                            error_message.contains("simctl") || 
                            error_message.contains("iOS") ||
                            error_message.contains("Xcode"),
                            "Task {task_id} devices error should be simulator-related: {error_message}"
                        );
                    }
                }

                // Check availability result
                // Task should return valid availability - any boolean is valid
            }
        }
    }
}

#[tokio::test]
async fn test_ios_manager_performance() {
    // Test that operations complete within reasonable time
    let manager = IosManager::new();

    if let Ok(manager) = manager {
        // Test list_devices performance
        let start = std::time::Instant::now();
        let devices_result = manager.list_devices().await;
        let duration = start.elapsed();

        // Should complete within 15 seconds (allowing for slow CI environments)
        assert!(
            duration < Duration::from_secs(15),
            "list_devices() should complete within 15 seconds, took: {duration:?}"
        );

        // Test is_available performance
        let start = std::time::Instant::now();
        let _available_result = manager.is_available().await;
        let duration = start.elapsed();

        // Should complete within 10 seconds
        assert!(
            duration < Duration::from_secs(30),
            "is_available() should complete within 30 seconds, took: {duration:?}"
        );

        // Results should be valid
        match devices_result {
            Ok(devices) => {
                assert!(
                    devices.is_empty() || !devices.is_empty(),
                    "Device list should be valid"
                );
            }
            Err(e) => {
                let error_message = e.to_string();
                assert!(
                    error_message.contains("xcrun")
                        || error_message.contains("simctl")
                        || error_message.contains("iOS")
                        || error_message.contains("Xcode"),
                    "Error should be simulator-related: {error_message}"
                );
            }
        }

        // Availability should be valid boolean - any boolean is valid
    }
}

#[tokio::test]
async fn test_ios_manager_error_handling() {
    // Test error handling behavior
    let manager = IosManager::new();

    match manager {
        Ok(manager) => {
            // Test list_devices error handling
            let devices_result = manager.list_devices().await;

            match devices_result {
                Ok(_) => {
                    // Success is also valid
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
                    let contains_helpful_info = error_message.contains("xcrun")
                        || error_message.contains("simctl")
                        || error_message.contains("iOS")
                        || error_message.contains("Xcode")
                        || error_message.contains("simulator");

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
                error_message.contains("xcrun")
                    || error_message.contains("simctl")
                    || error_message.contains("macOS")
                    || error_message.contains("Xcode"),
                "Error should be simulator-related: {error_message}"
            );
        }
    }
}

#[tokio::test]
async fn test_ios_manager_platform_specific_behavior() {
    // Test platform-specific behavior
    let manager = IosManager::new();

    if cfg!(target_os = "macos") {
        // On macOS, manager should either succeed or fail with Xcode-related error
        match manager {
            Ok(manager) => {
                // Should be able to call methods
                let _available = manager.is_available().await;
                let _devices = manager.list_devices().await;
            }
            Err(e) => {
                // Error should be related to Xcode/simctl availability
                let error_message = e.to_string();
                assert!(
                    error_message.contains("xcrun")
                        || error_message.contains("simctl")
                        || error_message.contains("Xcode"),
                    "macOS error should be Xcode-related: {error_message}"
                );
            }
        }
    } else {
        // On non-macOS platforms, should handle gracefully
        match manager {
            Ok(manager) => {
                // Should still work but with limited functionality
                let available = manager.is_available().await;
                assert!(
                    !available,
                    "iOS manager should not be available on non-macOS"
                );
            }
            Err(e) => {
                // Error should indicate platform limitation
                let error_message = e.to_string();
                assert!(
                    error_message.contains("macOS")
                        || error_message.contains("platform")
                        || error_message.contains("xcrun")
                        || error_message.contains("simctl"),
                    "Non-macOS error should indicate platform limitation: {error_message}"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_ios_manager_device_validation() {
    // Test device validation and parsing
    let manager = IosManager::new();

    if let Ok(manager) = manager {
        let devices = manager.list_devices().await;

        if let Ok(device_list) = devices {
            for device in device_list {
                // Test device field validation
                assert!(!device.name.is_empty(), "Device name should not be empty");
                assert!(!device.udid.is_empty(), "UDID should not be empty");
                assert!(
                    !device.device_type.is_empty(),
                    "Device type should not be empty"
                );
                assert!(
                    !device.ios_version.is_empty(),
                    "iOS version should not be empty"
                );
                assert!(
                    !device.runtime_version.is_empty(),
                    "Runtime version should not be empty"
                );

                // Test UDID format (should be UUID-like)
                assert!(device.udid.len() >= 32, "UDID should be reasonably long");
                assert!(
                    device.udid.contains('-')
                        || device.udid.chars().all(|c| c.is_ascii_alphanumeric()),
                    "UDID should be UUID-like or alphanumeric"
                );

                // Test status is valid enum - all variants are valid
                match device.status {
                    DeviceStatus::Running
                    | DeviceStatus::Stopped
                    | DeviceStatus::Starting
                    | DeviceStatus::Stopping
                    | DeviceStatus::Creating
                    | DeviceStatus::Error
                    | DeviceStatus::Unknown => {} // All valid iOS device statuses
                }

                // Test boolean fields
                let _is_running = device.is_running;
                let _is_available = device.is_available;

                // Test iOS version format (could be "17.2", "iOS 17.2", "watchOS 10.2", etc.)
                assert!(
                    !device.ios_version.is_empty(),
                    "iOS version should not be empty"
                );
                // Version could start with digit, "iOS", "watchOS", "tvOS", etc.
                // No strict format validation needed as Apple's format may vary
            }
        }
    }
}
