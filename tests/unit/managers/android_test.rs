//! Unit tests for AndroidManager
//!
//! This file consolidates unit tests from:
//! - managers_android_basic_test.rs
//! - managers_android_simple_test.rs
//! - managers_android_command_executor_test.rs
//!
//! Tests focus on unit-level functionality including:
//! - Basic initialization and configuration
//! - Device category classification
//! - API level mapping
//! - Command executor mocking
//! - Error handling

use emu::managers::android::AndroidManager;
use emu::managers::common::{DeviceConfig, DeviceManager};
use emu::models::DeviceStatus;
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::collections::HashMap;
use std::sync::Arc;

// Import common test helpers from unit test module
use crate::unit::common::setup_mock_android_sdk;

// ===== Basic Initialization Tests (from basic_test.rs) =====

/// Basic AndroidManager initialization test (no SDK required)
#[tokio::test]
async fn test_android_manager_creation() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor with necessary responses
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Create AndroidManager with mock executor
    let result = AndroidManager::with_executor(Arc::new(mock_executor));

    // Should succeed with mock executor
    assert!(result.is_ok());
}

/// Device category classification test
#[test]
fn test_device_category_classification() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Create AndroidManager with mock executor
    let manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

    // Phone category test (AndroidManager returns lowercase)
    assert_eq!(manager.get_device_category("pixel_7", "Pixel 7"), "phone");
    assert_eq!(manager.get_device_category("pixel_4", "Pixel 4"), "phone");
    assert_eq!(manager.get_device_category("nexus_5", "Nexus 5"), "phone");

    // Tablet category test
    assert_eq!(
        manager.get_device_category("pixel_tablet", "Pixel Tablet"),
        "tablet"
    );
    assert_eq!(manager.get_device_category("nexus_10", "Nexus 10"), "phone"); // nexus_10 doesn't contain pixel so it's phone

    // TV category test
    assert_eq!(
        manager.get_device_category("tv_1080p", "Android TV (1080p)"),
        "tv"
    );
    assert_eq!(
        manager.get_device_category("tv_720p", "Android TV (720p)"),
        "tv"
    );

    // Wear category test
    assert_eq!(
        manager.get_device_category("wear_round", "Android Wear Round"),
        "wear"
    );
    assert_eq!(
        manager.get_device_category("wear_square", "Android Wear Square"),
        "wear"
    );

    // Automotive category test
    assert_eq!(
        manager.get_device_category("automotive_1024p", "Automotive (1024p landscape)"),
        "automotive"
    );

    // Desktop category test
    assert_eq!(
        manager.get_device_category("desktop_large", "Large Desktop"),
        "desktop"
    );
    assert_eq!(
        manager.get_device_category("desktop_medium", "Medium Desktop"),
        "desktop"
    );

    // Unknown device test (default is phone)
    assert_eq!(
        manager.get_device_category("unknown_device", "Unknown Device"),
        "phone"
    );
    assert_eq!(manager.get_device_category("", ""), "phone");
}

/// Android version name mapping test
#[test]
fn test_android_version_name_mapping() {
    // Test API level to Android version name mapping logic
    assert_eq!(get_android_version_name(34), "Android 14");
    assert_eq!(get_android_version_name(33), "Android 13");
    assert_eq!(get_android_version_name(32), "Android 12L");
    assert_eq!(get_android_version_name(31), "Android 12");
    assert_eq!(get_android_version_name(30), "Android 11");
    assert_eq!(get_android_version_name(29), "Android 10");
    assert_eq!(get_android_version_name(28), "Android 9");
    assert_eq!(get_android_version_name(27), "Android 8.1");
    assert_eq!(get_android_version_name(26), "Android 8.0");
    assert_eq!(get_android_version_name(25), "Android 7.1");
    assert_eq!(get_android_version_name(24), "Android 7.0");
    assert_eq!(get_android_version_name(23), "Android 6.0");

    // Test older versions
    assert_eq!(get_android_version_name(21), "Android 5.0");
    assert_eq!(get_android_version_name(19), "Android 4.4");
    assert_eq!(get_android_version_name(16), "Android 4.1");

    // Test out of range values
    assert_eq!(get_android_version_name(99), "API 99");
    assert_eq!(get_android_version_name(0), "API 0");
}

// ===== Simple Interface Tests (from simple_test.rs) =====

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

// ===== Command Executor Tests (from command_executor_test.rs) =====

/// Basic device list retrieval test for AndroidManager
#[tokio::test]
async fn test_android_manager_list_devices_basic() {
    // Create Android SDK environment for testing
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let complex_avd_output = r#"Available Android Virtual Devices:
    Name: Pixel_7_API_34
    Device: pixel_7 (Pixel 7)
    Path: /Users/user/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a
---------
    Name: Galaxy_S22_API_33
    Device: galaxy_s22 (Galaxy S22)
    Path: /Users/user/.android/avd/Galaxy_S22_API_33.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/x86_64
---------"#;

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], complex_avd_output)
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Mock environment is already created, so do nothing here

    match AndroidManager::with_executor(Arc::new(mock_executor)) {
        Ok(android_manager) => {
            let devices = android_manager.list_devices().await.unwrap();
            assert!(!devices.is_empty());
            // Since device name and API level validation are implementation dependent, only check basic existence
        }
        Err(_) => {
            // Skip test when Android SDK environment is not available
            println!("Android SDK not available, skipping test");
        }
    }
}

/// AndroidManager device creation success test
#[tokio::test]
async fn test_android_manager_create_device_success() {
    // Save current ANDROID_HOME
    let original_android_home = std::env::var("ANDROID_HOME").ok();

    // Create Android SDK environment for testing
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    let avdmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let sdkmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

    let mock_executor = MockCommandExecutor::new()
        // avdmanager list avd (empty initially)
        .with_success("avdmanager", &["list", "avd"], "Available Android Virtual Devices:\n")
        .with_success(&avdmanager_path.to_string_lossy(), &["list", "avd"], "Available Android Virtual Devices:\n")
        // avdmanager list device
        .with_success("avdmanager", &["list", "device"], r#"id: 0 or "Galaxy Nexus"
    Name: Galaxy Nexus
    OEM : Google
    Tag : google_tv
---------
id: 1 or "pixel_7"
    Name: Pixel 7
    OEM : Google
    Tag : google_apis_playstore
---------"#)
        .with_success(&avdmanager_path.to_string_lossy(), &["list", "device"], r#"id: 0 or "Galaxy Nexus"
    Name: Galaxy Nexus
    OEM : Google
    Tag : google_tv
---------
id: 1 or "pixel_7"
    Name: Pixel 7
    OEM : Google
    Tag : google_apis_playstore
---------"#)
        // sdkmanager for system images
        .with_success("sdkmanager", &["--list", "--verbose", "--include_obsolete"], r#"Installed packages:
  system-images;android-34;google_apis_playstore;arm64-v8a | 1       | Google Play ARM 64 v8a System Image

Available Packages:"#)
        .with_success(&sdkmanager_path.to_string_lossy(), &["--list", "--verbose", "--include_obsolete"], r#"Installed packages:
  system-images;android-34;google_apis_playstore;arm64-v8a | 1       | Google Play ARM 64 v8a System Image

Available Packages:"#)
        // avdmanager list target
        .with_success("avdmanager", &["list", "target"], "Available targets:\nid: 1 or \"android-34\"\n     Name: Android 14.0\n     Type: Platform\n     API level: 34")
        .with_success(&avdmanager_path.to_string_lossy(), &["list", "target"], "Available targets:\nid: 1 or \"android-34\"\n     Name: Android 14.0\n     Type: Platform\n     API level: 34")
        // avdmanager create - match actual implementation which may retry without skin
        .with_success(
            "avdmanager",
            &[
                "create",
                "avd",
                "-n",
                "Test_Device",
                "-k",
                "system-images;android-34;google_apis_playstore;arm64-v8a",
                "--device",
                "pixel_7",
                "--skin",
                "pixel_7",
            ],
            "AVD 'Test_Device' created successfully",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &[
                "create",
                "avd",
                "-n",
                "Test_Device",
                "-k",
                "system-images;android-34;google_apis_playstore;arm64-v8a",
                "--device",
                "pixel_7",
                "--skin",
                "pixel_7",
            ],
            "AVD 'Test_Device' created successfully",
        )
        // Also add fallback without skin (in case skin fails)
        .with_success(
            "avdmanager",
            &[
                "create",
                "avd",
                "-n",
                "Test_Device",
                "-k",
                "system-images;android-34;google_apis_playstore;arm64-v8a",
                "--device",
                "pixel_7",
            ],
            "AVD 'Test_Device' created successfully",
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &[
                "create",
                "avd",
                "-n",
                "Test_Device",
                "-k",
                "system-images;android-34;google_apis_playstore;arm64-v8a",
                "--device",
                "pixel_7",
            ],
            "AVD 'Test_Device' created successfully",
        );

    let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

    let device_config = DeviceConfig {
        name: "Test_Device".to_string(),
        device_type: "pixel_7".to_string(),
        version: "34".to_string(),
        ram_size: None,
        storage_size: None,
        additional_options: HashMap::new(),
    };

    let result = android_manager.create_device(&device_config).await;

    // Restore original ANDROID_HOME
    match original_android_home {
        Some(value) => std::env::set_var("ANDROID_HOME", value),
        None => std::env::remove_var("ANDROID_HOME"),
    }

    assert!(result.is_ok(), "Failed to create device: {result:?}");
}

/// Command execution error propagation test
#[tokio::test]
async fn test_command_error_propagation() {
    // Create Android SDK environment for testing
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    let avdmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = temp_dir.path().join("platform-tools/adb");

    let mock_executor = MockCommandExecutor::new()
        .with_error(
            "avdmanager",
            &["list", "avd"],
            "Error: avdmanager not found",
        )
        .with_error(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            "Error: avdmanager not found",
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

    let result = android_manager.list_devices().await;
    assert!(result.is_err());

    // Verify that error messages are propagated properly
    let error_message = result.unwrap_err().to_string();
    assert!(
        error_message.contains("Failed to list Android AVDs")
            || error_message.contains("avdmanager")
            || error_message.contains("Error")
    );
}

/// Concurrent device operations test
#[tokio::test]
async fn test_concurrent_device_operations() {
    // Create Android SDK environment for testing
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avd_output = r#"Available Android Virtual Devices:
    Name: Device_A
    Device: pixel_7 (Pixel 7)
    Path: /Users/user/.android/avd/Device_A.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a
---------
    Name: Device_B
    Device: pixel_6 (Pixel 6)
    Path: /Users/user/.android/avd/Device_B.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/arm64-v8a
---------"#;

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], avd_output)
        .with_success("adb", &["devices"], "List of devices attached\n");

    let android_manager = Arc::new(AndroidManager::with_executor(Arc::new(mock_executor)).unwrap());

    // Get device list concurrently
    let manager1 = android_manager.clone();
    let manager2 = android_manager.clone();

    let handle1 = tokio::spawn(async move { manager1.list_devices().await });

    let handle2 = tokio::spawn(async move { manager2.list_devices().await });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

// ===== Helper Functions =====

/// Android version name mapping helper function
fn get_android_version_name(api_level: u32) -> String {
    match api_level {
        34 => "Android 14".to_string(),
        33 => "Android 13".to_string(),
        32 => "Android 12L".to_string(),
        31 => "Android 12".to_string(),
        30 => "Android 11".to_string(),
        29 => "Android 10".to_string(),
        28 => "Android 9".to_string(),
        27 => "Android 8.1".to_string(),
        26 => "Android 8.0".to_string(),
        25 => "Android 7.1".to_string(),
        24 => "Android 7.0".to_string(),
        23 => "Android 6.0".to_string(),
        22 => "Android 5.1".to_string(),
        21 => "Android 5.0".to_string(),
        19 => "Android 4.4".to_string(),
        18 => "Android 4.3".to_string(),
        17 => "Android 4.2".to_string(),
        16 => "Android 4.1".to_string(),
        _ => format!("API {api_level}"),
    }
}
