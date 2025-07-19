//! CommandExecutor integration tests for IosManager
//!
//! This test uses MockCommandExecutor to test IosManager's
//! xcrun simctl command execution and response handling.

use emu::managers::common::{DeviceConfig, DeviceManager};
use emu::managers::ios::IosManager;
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
