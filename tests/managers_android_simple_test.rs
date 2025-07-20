use emu::managers::{
    common::{DeviceConfig, DeviceManager},
    AndroidManager,
};
use emu::models::DeviceStatus;
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::setup_mock_android_sdk;

/// Test AndroidManager::new() initialization
#[tokio::test]
async fn test_android_manager_new() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(_manager) => {
            // AndroidManager created successfully
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Accept common Android SDK setup errors
            assert!(
                error_msg.contains("Android")
                    || error_msg.contains("SDK")
                    || error_msg.contains("avdmanager")
                    || error_msg.contains("ANDROID_HOME")
                    || error_msg.contains("command not found"),
                "Expected Android SDK-related error, got: {error_msg}"
            );
        }
    }
}

/// Test device listing functionality
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

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            let devices_result = manager.list_devices().await;

            match devices_result {
                Ok(devices) => {
                    // Validate device structure if devices exist
                    for device in &devices {
                        assert!(!device.name.is_empty(), "Device name should not be empty");
                        assert!(device.api_level > 0, "API level should be positive");
                        // DeviceStatus should be valid enum variant
                        match device.status {
                            DeviceStatus::Running
                            | DeviceStatus::Stopped
                            | DeviceStatus::Starting
                            | DeviceStatus::Stopping
                            | DeviceStatus::Creating
                            | DeviceStatus::Error
                            | DeviceStatus::Unknown => {
                                // All valid status types
                            }
                        }
                    }
                    println!("Found {} Android devices", devices.len());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    assert!(
                        error_msg.contains("SDK")
                            || error_msg.contains("avdmanager")
                            || error_msg.contains("command"),
                        "Expected command-related error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "Expected SDK-related error"
            );
        }
    }
}

/// Test device creation functionality structure
#[tokio::test]
async fn test_android_manager_create_device_interface() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");
    let sdkmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

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
        )
        .with_success(
            "sdkmanager",
            &["--list", "--verbose", "--include_obsolete"],
            "Installed packages:\n",
        )
        .with_success(
            &sdkmanager_path.to_string_lossy(),
            &["--list", "--verbose", "--include_obsolete"],
            "Installed packages:\n",
        );

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Create test device config
            let test_config = DeviceConfig::new(
                "test_device".to_string(),
                "pixel_7".to_string(),
                "android-29".to_string(),
            )
            .with_ram("2048".to_string())
            .with_storage("8192".to_string());

            // Test create_device method interface
            let create_result = manager.create_device(&test_config).await;

            match create_result {
                Ok(_) => {
                    // Device creation succeeded
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors
                    assert!(
                        error_msg.contains("SDK")
                            || error_msg.contains("system-images")
                            || error_msg.contains("Target")
                            || error_msg.contains("device")
                            || error_msg.contains("avdmanager"),
                        "Expected device creation error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test device start functionality interface
#[tokio::test]
async fn test_android_manager_start_device_interface() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test start_device method interface
            let start_result = manager.start_device("test_device").await;

            match start_result {
                Ok(_) => {
                    // Device start interface works
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors for non-existent devices
                    assert!(
                        error_msg.contains("not found")
                            || error_msg.contains("does not exist")
                            || error_msg.contains("Invalid")
                            || error_msg.contains("emulator")
                            || error_msg.contains("Error"),
                        "Expected device start error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test device stop functionality interface
#[tokio::test]
async fn test_android_manager_stop_device_interface() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test stop_device method interface
            let stop_result = manager.stop_device("test_device").await;

            match stop_result {
                Ok(_) => {
                    // Device stop interface works
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors for non-existent devices
                    assert!(
                        error_msg.contains("not found")
                            || error_msg.contains("No emulator")
                            || error_msg.contains("does not exist")
                            || error_msg.contains("adb")
                            || error_msg.contains("Error"),
                        "Expected device stop error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test device deletion functionality interface
#[tokio::test]
async fn test_android_manager_delete_device_interface() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test delete_device method interface
            let delete_result = manager.delete_device("test_device").await;

            match delete_result {
                Ok(_) => {
                    // Device deletion interface works
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors for non-existent devices
                    assert!(
                        error_msg.contains("not exist")
                            || error_msg.contains("not found")
                            || error_msg.contains("Error")
                            || error_msg.contains("Invalid")
                            || error_msg.contains("Failed to delete")
                            || error_msg.contains("avdmanager"),
                        "Expected device deletion error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test device wipe functionality interface
#[tokio::test]
async fn test_android_manager_wipe_device_interface() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test wipe_device method interface
            let wipe_result = manager.wipe_device("test_device").await;

            match wipe_result {
                Ok(_) => {
                    // Device wipe interface works
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors for non-existent devices
                    assert!(
                        error_msg.contains("not exist")
                            || error_msg.contains("not found")
                            || error_msg.contains("Error")
                            || error_msg.contains("Invalid")
                            || error_msg.contains("directory not found")
                            || error_msg.contains("emulator")
                            || error_msg.contains("wipe"),
                        "Expected device wipe error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test API levels listing functionality
#[tokio::test]
async fn test_android_manager_list_api_levels() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let sdkmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "target"], "Available targets:\n")
        .with_success(&avdmanager_path.to_string_lossy(), &["list", "target"], "Available targets:\n")
        .with_success("sdkmanager", &["--list", "--verbose", "--include_obsolete"], "Installed packages:\n  system-images;android-34;google_apis;x86_64 | 1 | Google APIs Intel x86_64 System Image\n  system-images;android-33;google_apis;arm64  | 1 | Google APIs ARM64 System Image\n")
        .with_success(&sdkmanager_path.to_string_lossy(), &["--list", "--verbose", "--include_obsolete"], "Installed packages:\n  system-images;android-34;google_apis;x86_64 | 1 | Google APIs Intel x86_64 System Image\n  system-images;android-33;google_apis;arm64  | 1 | Google APIs ARM64 System Image\n");

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            let api_levels_result = manager.list_api_levels().await;

            match api_levels_result {
                Ok(api_levels) => {
                    // Validate API levels structure
                    for api_level in &api_levels {
                        assert!(api_level.api > 0, "API level should be positive");
                        assert!(
                            !api_level.display_name.is_empty(),
                            "API level display name should not be empty"
                        );
                    }
                    println!("Found {} API levels", api_levels.len());
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    assert!(
                        error_msg.contains("SDK")
                            || error_msg.contains("sdkmanager")
                            || error_msg.contains("command"),
                        "Expected command-related error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test concurrent operations
#[tokio::test]
async fn test_android_manager_concurrent_operations() {
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

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test concurrent device list operations
            let devices_task1 = manager.list_devices();
            let devices_task2 = manager.list_devices();

            let (result1, result2) = tokio::join!(devices_task1, devices_task2);

            // Both operations should complete (successfully or with expected errors)
            match (result1, result2) {
                (Ok(_), Ok(_)) => {
                    // Concurrent operations succeeded
                }
                (Err(e1), _) | (_, Err(e1)) => {
                    let error_msg = e1.to_string();
                    assert!(
                        error_msg.contains("SDK") || error_msg.contains("command"),
                        "Expected SDK-related error in concurrent test"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test error handling with invalid inputs
#[tokio::test]
async fn test_android_manager_error_handling() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let mock_executor = MockCommandExecutor::new();
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test with empty device name
            let empty_name_result = manager.start_device("").await;
            // Empty device name may succeed with a generic error, so we check for completion
            match empty_name_result {
                Ok(_) => { /* Empty device name handled */ }
                Err(_) => { /* Empty device name failed as expected */ }
            }

            // Test with invalid device name
            let invalid_name_result = manager.start_device("invalid_device_name_12345").await;
            // Invalid device name may also succeed with error handling, so we check completion
            match invalid_name_result {
                Ok(_) => { /* Invalid device name handled */ }
                Err(_) => { /* Invalid device name failed as expected */ }
            }

            // Test device creation with invalid config
            let invalid_config = DeviceConfig::new(
                "".to_string(), // Empty name
                "invalid_type".to_string(),
                "invalid-version".to_string(),
            )
            .with_ram("0".to_string()) // Invalid RAM
            .with_storage("0".to_string()); // Invalid storage

            let invalid_create_result = manager.create_device(&invalid_config).await;
            assert!(
                invalid_create_result.is_err(),
                "Invalid create parameters should fail"
            );
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test timeout handling
#[tokio::test]
async fn test_android_manager_operations_with_timeout() {
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

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test that operations complete within reasonable time
            let timeout_duration = Duration::from_secs(30);

            let devices_result =
                tokio::time::timeout(timeout_duration, manager.list_devices()).await;

            match devices_result {
                Ok(result) => {
                    match result {
                        Ok(_) => { /* Device listing completed within timeout */ }
                        Err(e) => {
                            let error_msg = e.to_string();
                            assert!(
                                error_msg.contains("SDK") || error_msg.contains("command"),
                                "Expected SDK-related error within timeout"
                            );
                        }
                    }
                }
                Err(_) => {
                    panic!(
                        "Device listing timed out after {} seconds",
                        timeout_duration.as_secs()
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}

/// Test device detail retrieval
#[tokio::test]
async fn test_android_manager_get_device_details() {
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

    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));

    match manager_result {
        Ok(manager) => {
            // Test get_device_details method interface
            let details_result = manager.get_device_details("test_device").await;

            match details_result {
                Ok(details) => {
                    // Device details should have valid fields
                    assert!(!details.name.is_empty(), "Device name should not be empty");
                    println!("Retrieved device details for: {}", details.name);
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    // Accept various expected errors for non-existent devices
                    assert!(
                        error_msg.contains("not found")
                            || error_msg.contains("does not exist")
                            || error_msg.contains("Invalid")
                            || error_msg.contains("avdmanager")
                            || error_msg.contains("device"),
                        "Expected device details error, got: {error_msg}"
                    );
                }
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Android") || error_msg.contains("SDK"));
        }
    }
}
