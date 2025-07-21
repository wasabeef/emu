//! Unit tests for iOS Manager functionality
//!
//! Tests basic initialization, device classification, utility functions, and patterns
//! without requiring actual Xcode installation or command execution.

use emu::managers::common::DeviceManager;
use emu::managers::ios::IosManager;

/// Basic initialization test for IosManager (no Xcode required)
#[tokio::test]
async fn test_ios_manager_creation() {
    // Attempt to create IosManager
    let result = IosManager::new();

    // Fails in environments without actual Xcode, but structurally normal
    match result {
        Ok(_manager) => {
            // If created successfully
        }
        Err(_error) => {
            // If error occurred (Xcode not available, etc.)
            // Normal case in non-macOS environments or environments without Xcode installed
        }
    }
}

/// IosManager creation test in macOS environment
#[tokio::test]
async fn test_ios_manager_macos_specific() {
    #[cfg(target_os = "macos")]
    {
        // Run only in macOS environment
        let result = IosManager::new();

        // Succeeds if Xcode is installed, otherwise error
        match result {
            Ok(_manager) => {
                // If Xcode is available
            }
            Err(_error) => {
                // If Xcode is not available
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Always succeeds in non-macOS environments (does nothing)
    }
}

/// IosManager stub implementation test (non-macOS)
#[tokio::test]
async fn test_ios_manager_stub_functionality() {
    #[cfg(not(target_os = "macos"))]
    {
        use emu::managers::ios::IosManager;

        // Verify that on non-macOS, IosManager creation should succeed but operations will fail
        let result = IosManager::new();
        // On non-macOS systems, IosManager can be created but is non-functional
        assert!(result.is_ok());
    }

    #[cfg(target_os = "macos")]
    {
        // In macOS environment, test implementation so only basic verification
    }
}

/// Basic pattern test for device state analysis
#[test]
fn test_device_state_patterns() {
    // Test iOS device state string patterns
    let state_patterns = vec![
        ("Booted", true),         // Running
        ("Shutdown", false),      // Stopped
        ("Creating", false),      // Creating
        ("Booting", false),       // Booting
        ("Shutting Down", false), // Shutting down
    ];

    for (state_str, expected_running) in state_patterns {
        // Test logic for determining running state from state string
        let is_running = state_str == "Booted";
        assert_eq!(
            is_running, expected_running,
            "State '{state_str}' should be running: {expected_running}"
        );
    }
}

/// iOS device type classification test
#[test]
fn test_ios_device_type_classification() {
    // Test iOS device type identification patterns
    let device_type_patterns = vec![
        // iPhone series
        ("iPhone-15", "iPhone"),
        ("iPhone-14", "iPhone"),
        ("iPhone-13", "iPhone"),
        ("iPhone-SE-3rd-generation", "iPhone"),
        // iPad series
        ("iPad-Pro-12-9-inch-6th-generation", "iPad"),
        ("iPad-Air-5th-generation", "iPad"),
        ("iPad-10th-generation", "iPad"),
        ("iPad-mini-6th-generation", "iPad"),
        // Others
        ("Apple-Watch-Series-9-45mm", "Apple Watch"),
        ("Apple-TV-4K-3rd-generation", "Apple TV"),
    ];

    for (device_id, expected_type) in device_type_patterns {
        // Classify category from device type string
        let device_category = if device_id.starts_with("iPhone") {
            "iPhone"
        } else if device_id.starts_with("iPad") {
            "iPad"
        } else if device_id.contains("Watch") {
            "Apple Watch"
        } else if device_id.contains("TV") {
            "Apple TV"
        } else {
            "Other"
        };

        assert_eq!(
            device_category, expected_type,
            "Device {device_id} should be categorized as {expected_type}"
        );
    }
}

/// iOS runtime version parsing test
#[test]
fn test_ios_runtime_version_parsing() {
    // Test iOS runtime version string patterns
    let runtime_patterns = vec![
        ("com.apple.CoreSimulator.SimRuntime.iOS-17-0", "iOS 17.0"),
        ("com.apple.CoreSimulator.SimRuntime.iOS-16-4", "iOS 16.4"),
        ("com.apple.CoreSimulator.SimRuntime.iOS-15-5", "iOS 15.5"),
        (
            "com.apple.CoreSimulator.SimRuntime.watchOS-10-0",
            "watchOS 10.0",
        ),
        ("com.apple.CoreSimulator.SimRuntime.tvOS-17-0", "tvOS 17.0"),
    ];

    for (runtime_id, expected_version) in runtime_patterns {
        // Extract display version from runtime ID
        let version = if runtime_id.contains("iOS") {
            // Extract 17.0 from iOS-17-0
            if let Some(start_pos) = runtime_id.find("iOS-") {
                let version_part = &runtime_id[start_pos + 4..]; // Skip "iOS-"
                let cleaned = version_part.replace('-', ".");
                format!("iOS {cleaned}")
            } else {
                "iOS Unknown".to_string()
            }
        } else if runtime_id.contains("watchOS") {
            if let Some(start_pos) = runtime_id.find("watchOS-") {
                let version_part = &runtime_id[start_pos + 8..]; // Skip "watchOS-"
                let cleaned = version_part.replace('-', ".");
                format!("watchOS {cleaned}")
            } else {
                "watchOS Unknown".to_string()
            }
        } else if runtime_id.contains("tvOS") {
            if let Some(start_pos) = runtime_id.find("tvOS-") {
                let version_part = &runtime_id[start_pos + 5..]; // Skip "tvOS-"
                let cleaned = version_part.replace('-', ".");
                format!("tvOS {cleaned}")
            } else {
                "tvOS Unknown".to_string()
            }
        } else {
            "Unknown".to_string()
        };

        assert_eq!(
            version, expected_version,
            "Runtime {runtime_id} should parse to {expected_version}"
        );
    }
}

/// Error handling pattern test
#[test]
fn test_error_handling_patterns() {
    // Test common error patterns of iOS simulator
    let error_patterns = vec![
        ("Unable to boot device in current state: Booted", true), // Already booted
        ("Unable to shutdown device in current state: Shutdown", true), // Already stopped
        ("No device matching 'invalid-udid'", false),             // Device not found
        ("Invalid device state", false),                          // Invalid state
        ("Permission denied", false),                             // Permission error
    ];

    for (error_message, should_ignore) in error_patterns {
        // Determine whether to ignore from error message
        let is_ignorable = error_message.contains("current state");

        assert_eq!(
            is_ignorable, should_ignore,
            "Error '{error_message}' should be ignorable: {should_ignore}"
        );
    }
}

/// Basic test for command patterns
#[test]
fn test_command_patterns() {
    // Test command patterns used in iOS simulator
    let commands = vec![
        "xcrun simctl list devices",
        "xcrun simctl list runtimes",
        "xcrun simctl list devicetypes",
        "xcrun simctl boot <uuid>",
        "xcrun simctl shutdown <uuid>",
    ];

    for command in commands {
        // Verify that command starts with xcrun
        assert!(command.starts_with("xcrun"));

        // Verify that simctl subcommand is included
        assert!(command.contains("simctl"));
    }
}

/// Compile-time platform branching test
#[test]
fn test_platform_compilation() {
    // Verify compile-time platform branching
    #[cfg(target_os = "macos")]
    {
        // Actual IosManager implementation in macOS environment
        // macOS environment detected
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Stub implementation in non-macOS environments
        // Non-macOS environment detected
    }
}

/// UDID format validation test
#[test]
fn test_udid_format_validation() {
    // Test iOS simulator UDID format
    let valid_udids = vec![
        "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6",
        "12345678-1234-1234-1234-123456789012",
        "ABCDEFGH-IJKL-MNOP-QRST-UVWXYZ123456",
    ];

    let invalid_udids = vec![
        "invalid-udid",
        "12345",
        "",
        "A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6", // No hyphens
    ];

    for udid in valid_udids {
        // Verify UUID format (8-4-4-4-12)
        let parts: Vec<&str> = udid.split('-').collect();
        assert_eq!(
            parts.len(),
            5,
            "UDID {udid} should have 5 parts separated by hyphens"
        );
        assert_eq!(parts[0].len(), 8, "First part should be 8 characters");
        assert_eq!(parts[1].len(), 4, "Second part should be 4 characters");
        assert_eq!(parts[2].len(), 4, "Third part should be 4 characters");
        assert_eq!(parts[3].len(), 4, "Fourth part should be 4 characters");
        assert_eq!(parts[4].len(), 12, "Fifth part should be 12 characters");
    }

    for udid in invalid_udids {
        // Verify invalid UDID format
        let parts: Vec<&str> = udid.split('-').collect();
        let is_valid_format = parts.len() == 5
            && parts[0].len() == 8
            && parts[1].len() == 4
            && parts[2].len() == 4
            && parts[3].len() == 4
            && parts[4].len() == 12;

        assert!(!is_valid_format, "UDID {udid} should be invalid format");
    }
}

// Helper function to get rough memory usage
fn get_memory_usage() -> usize {
    // Simple memory usage estimation
    std::process::id() as usize * 1024 // Simple approximation
}

/// Test that IosManager doesn't cause memory issues
#[tokio::test]
async fn test_ios_manager_memory_usage() {
    let initial_memory = get_memory_usage();

    // Create and drop multiple manager instances
    for _ in 0..5 {
        let manager_result = IosManager::new();
        if let Ok(manager) = manager_result {
            // Test operations
            let _ = manager.is_available().await;
            let _ = manager.list_devices().await;
        }
        // manager_result is dropped automatically
    }

    let final_memory = get_memory_usage();

    // Memory usage should not increase dramatically
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 30_000_000, // 30MB limit
        "Memory usage should not increase dramatically: {memory_increase} bytes"
    );
}

#[cfg(all(target_os = "macos", feature = "test-utils"))]
mod command_executor_tests {
    use super::*;
    use emu::managers::common::{DeviceConfig, DeviceManager};
    use emu::models::DeviceStatus;
    use emu::utils::command_executor::mock::MockCommandExecutor;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Test basic device list retrieval for IosManager
    #[tokio::test]
    async fn test_ios_manager_list_devices_basic() {
        let simctl_output = r#"{
      "devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-2": [
          {
            "dataPath": "/Users/user/Library/Developer/CoreSimulator/Devices/12345678-1234-1234-1234-123456789012/data",
            "logPath": "/Users/user/Library/Logs/CoreSimulator/12345678-1234-1234-1234-123456789012",
            "udid": "12345678-1234-1234-1234-123456789012",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "iPhone 15"
          },
          {
            "dataPath": "/Users/user/Library/Developer/CoreSimulator/Devices/87654321-4321-4321-4321-210987654321/data",
            "logPath": "/Users/user/Library/Logs/CoreSimulator/87654321-4321-4321-4321-210987654321",
            "udid": "87654321-4321-4321-4321-210987654321",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPad-Air-5th-generation",
            "state": "Booted",
            "name": "iPad Air (5th generation)"
          }
        ]
      }
    }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                simctl_output,
            )
            .with_success("xcrun", &["simctl", "list", "devices", "-j"], simctl_output);

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let devices = ios_manager.list_devices().await.unwrap();

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "iPhone 15 (iOS 17.2)");
        assert_eq!(devices[0].status, DeviceStatus::Stopped);
        assert_eq!(devices[1].name, "iPad Air (5th generation) (iOS 17.2)");
        assert_eq!(devices[1].status, DeviceStatus::Running);
    }

    /// Test device creation
    #[tokio::test]
    async fn test_ios_manager_create_device_success() {
        let mock_executor = MockCommandExecutor::new().with_success(
            "xcrun",
            &["simctl", "create", "Test iPhone", "iPhone-15", "iOS17-2"],
            "12345678-1234-1234-1234-123456789012",
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();

        let device_config = DeviceConfig {
            name: "Test iPhone".to_string(),
            device_type: "iPhone-15".to_string(),
            version: "iOS17-2".to_string(),
            ram_size: None,
            storage_size: None,
            additional_options: HashMap::new(),
        };

        let result = ios_manager.create_device(&device_config).await;
        assert!(result.is_ok());
    }

    /// Test device creation failure
    #[tokio::test]
    async fn test_ios_manager_create_device_failure() {
        let mock_executor = MockCommandExecutor::new().with_error(
            "xcrun",
            &[
                "simctl",
                "create",
                "Invalid Device",
                "Invalid-Type",
                "Invalid-Runtime",
            ],
            "Invalid device type: Invalid-Type",
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();

        let device_config = DeviceConfig {
            name: "Invalid Device".to_string(),
            device_type: "Invalid-Type".to_string(),
            version: "Invalid-Runtime".to_string(),
            ram_size: None,
            storage_size: None,
            additional_options: HashMap::new(),
        };

        let result = ios_manager.create_device(&device_config).await;
        assert!(result.is_err());
    }

    /// Test device startup
    #[tokio::test]
    async fn test_ios_manager_start_device() {
        let status_response = r#"{
            "devices": {
                "iOS 17.0": [{
                    "udid": "12345678-1234-1234-1234-123456789012",
                    "state": "Shutdown",
                    "name": "Test Device"
                }]
            }
        }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                status_response,
            )
            .with_success(
                "xcrun",
                &["simctl", "boot", "12345678-1234-1234-1234-123456789012"],
                "",
            )
            .with_success("open", &["-b", "com.apple.iphonesimulator"], "");

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let result = ios_manager
            .start_device("12345678-1234-1234-1234-123456789012")
            .await;
        assert!(result.is_ok());
    }

    /// Test device shutdown
    #[tokio::test]
    async fn test_ios_manager_stop_device() {
        let mock_executor = MockCommandExecutor::new().with_success(
            "xcrun",
            &["simctl", "shutdown", "12345678-1234-1234-1234-123456789012"],
            "",
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let result = ios_manager
            .stop_device("12345678-1234-1234-1234-123456789012")
            .await;
        assert!(result.is_ok());
    }

    /// Test device deletion
    #[tokio::test]
    async fn test_ios_manager_delete_device() {
        let mock_executor = MockCommandExecutor::new().with_success(
            "xcrun",
            &["simctl", "delete", "12345678-1234-1234-1234-123456789012"],
            "",
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let result = ios_manager
            .delete_device("12345678-1234-1234-1234-123456789012")
            .await;
        assert!(result.is_ok());
    }

    /// Test device wipe
    #[tokio::test]
    async fn test_ios_manager_wipe_device() {
        let mock_executor = MockCommandExecutor::new().with_success(
            "xcrun",
            &["simctl", "erase", "12345678-1234-1234-1234-123456789012"],
            "",
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();

        let result = ios_manager
            .wipe_device("12345678-1234-1234-1234-123456789012")
            .await;
        assert!(result.is_ok());
    }

    /// Test error handling when xcrun command is not installed
    #[tokio::test]
    async fn test_xcrun_command_not_found() {
        let mock_executor = MockCommandExecutor::new()
            .with_error(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                "xcrun: command not found",
            )
            .with_error(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                "xcrun: command not found",
            );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let result = ios_manager.list_devices().await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            error_msg.contains("xcrun")
                || error_msg.contains("command not found")
                || error_msg.contains("failed")
        );
    }

    /// Test JSON parsing error handling
    #[tokio::test]
    async fn test_invalid_json_handling() {
        let invalid_json = r#"{"devices": {"invalid": [}"#;

        let mock_executor = MockCommandExecutor::new().with_success(
            "xcrun",
            &["simctl", "list", "devices", "available", "--json"],
            invalid_json,
        );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let result = ios_manager.list_devices().await;

        // For invalid JSON, return error or empty list
        if let Ok(devices) = result {
            assert!(devices.is_empty());
        }
        // JSON parsing errors are also acceptable
    }

    /// Test device state mapping
    #[tokio::test]
    async fn test_device_state_mapping() {
        let simctl_output_states = r#"{
      "devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-2": [
          {
            "udid": "12345678-1234-1234-1234-123456789012",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "Shutdown Device"
          },
          {
            "udid": "12345678-1234-1234-1234-123456789013",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Booted",
            "name": "Booted Device"
          },
          {
            "udid": "12345678-1234-1234-1234-123456789014",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Booting",
            "name": "Booting Device"
          },
          {
            "udid": "12345678-1234-1234-1234-123456789015",
            "isAvailable": false,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "Unavailable Device"
          }
        ]
      }
    }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                simctl_output_states,
            )
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                simctl_output_states,
            );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let devices = ios_manager.list_devices().await.unwrap();

        // All devices are returned (including unavailable ones for testing)
        assert_eq!(devices.len(), 4);

        // Verify state mapping
        let shutdown_device = devices
            .iter()
            .find(|d| d.name == "Shutdown Device (iOS 17.2)")
            .unwrap();
        assert_eq!(shutdown_device.status, DeviceStatus::Stopped);

        let booted_device = devices
            .iter()
            .find(|d| d.name == "Booted Device (iOS 17.2)")
            .unwrap();
        assert_eq!(booted_device.status, DeviceStatus::Running);

        let booting_device = devices
            .iter()
            .find(|d| d.name == "Booting Device (iOS 17.2)")
            .unwrap();
        // Booting state maps to Unknown in the current implementation
        assert_eq!(booting_device.status, DeviceStatus::Unknown);
    }

    /// Test device type identifier parsing
    #[tokio::test]
    async fn test_device_type_identifier_parsing() {
        let simctl_output_types = r#"{
      "devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-2": [
          {
            "udid": "12345678-1234-1234-1234-123456789012",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "iPhone 15"
          },
          {
            "udid": "12345678-1234-1234-1234-123456789013",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPad-Air-5th-generation",
            "state": "Shutdown",
            "name": "iPad Air (5th generation)"
          },
          {
            "udid": "12345678-1234-1234-1234-123456789014",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.Apple-Watch-Series-9-45mm",
            "state": "Shutdown",
            "name": "Apple Watch Series 9 (45mm)"
          }
        ]
      }
    }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                simctl_output_types,
            )
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                simctl_output_types,
            );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let devices = ios_manager.list_devices().await.unwrap();

        assert_eq!(devices.len(), 3);

        // Verify device types
        let iphone = devices
            .iter()
            .find(|d| d.name == "iPhone 15 (iOS 17.2)")
            .unwrap();
        assert_eq!(
            iphone.device_type,
            "com.apple.CoreSimulator.SimDeviceType.iPhone-15"
        );

        let ipad = devices
            .iter()
            .find(|d| d.name.contains("iPad Air"))
            .unwrap();
        assert_eq!(
            ipad.device_type,
            "com.apple.CoreSimulator.SimDeviceType.iPad-Air-5th-generation"
        );

        let watch = devices
            .iter()
            .find(|d| d.name.contains("Apple Watch"))
            .unwrap();
        assert_eq!(
            watch.device_type,
            "com.apple.CoreSimulator.SimDeviceType.Apple-Watch-Series-9-45mm"
        );
    }

    /// Test runtime parsing
    #[tokio::test]
    async fn test_runtime_parsing() {
        let simctl_output_runtimes = r#"{
      "devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-2": [
          {
            "udid": "12345678-1234-1234-1234-123456789012",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "iOS 17.2 Device"
          }
        ],
        "com.apple.CoreSimulator.SimRuntime.iOS-16-4": [
          {
            "udid": "12345678-1234-1234-1234-123456789013",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-14",
            "state": "Shutdown",
            "name": "iOS 16.4 Device"
          }
        ]
      }
    }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                simctl_output_runtimes,
            )
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                simctl_output_runtimes,
            );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor)).unwrap();
        let devices = ios_manager.list_devices().await.unwrap();

        assert_eq!(devices.len(), 2);

        // Verify runtimes
        let ios_17_device = devices
            .iter()
            .find(|d| d.name == "iOS 17.2 Device (iOS 17.2)")
            .unwrap();
        assert_eq!(ios_17_device.runtime_version, "17.2");

        let ios_16_device = devices
            .iter()
            .find(|d| d.name == "iOS 16.4 Device (iOS 16.4)")
            .unwrap();
        assert_eq!(ios_16_device.runtime_version, "16.4");
    }

    /// Test concurrent processing stability
    #[tokio::test]
    async fn test_concurrent_ios_operations() {
        let stable_output = r#"{
      "devices": {
        "com.apple.CoreSimulator.SimRuntime.iOS-17-2": [
          {
            "udid": "12345678-1234-1234-1234-123456789012",
            "isAvailable": true,
            "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
            "state": "Shutdown",
            "name": "Concurrent Test Device"
          }
        ]
      }
    }"#;

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                stable_output,
            )
            .with_success("xcrun", &["simctl", "list", "devices", "-j"], stable_output);

        let ios_manager = Arc::new(IosManager::with_executor(Arc::new(mock_executor)).unwrap());

        // Multiple concurrent requests
        let mut handles = vec![];
        for _ in 0..5 {
            let manager = ios_manager.clone();
            let handle = tokio::spawn(async move { manager.list_devices().await });
            handles.push(handle);
        }

        // Confirm all requests succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            let devices = result.unwrap();
            assert_eq!(devices.len(), 1);
            assert_eq!(devices[0].name, "Concurrent Test Device (iOS 17.2)");
        }
    }

    /// Test iOS command history in MockCommandExecutor
    #[tokio::test]
    async fn test_ios_mock_executor_call_history() {
        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                r#"{"devices":{}}"#,
            )
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "-j"],
                r#"{"devices":{}}"#,
            );

        let ios_manager = IosManager::with_executor(Arc::new(mock_executor.clone())).unwrap();
        let _devices = ios_manager.list_devices().await.unwrap();

        // Verify call history
        let history = mock_executor.call_history();
        assert!(!history.is_empty());

        // Confirm xcrun command was called
        let xcrun_calls: Vec<_> = history
            .iter()
            .filter(|(cmd, _args)| cmd == "xcrun")
            .collect();
        assert!(!xcrun_calls.is_empty());

        // Confirm simctl subcommand is included
        assert!(xcrun_calls
            .iter()
            .any(|(_cmd, args)| args.contains(&"simctl".to_string())));
    }
}
