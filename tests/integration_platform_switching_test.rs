//! Platform switching integration tests
//!
//! Tests platform switching between Android and iOS,
//! application state management, and UI focus control.

use emu::app::state::{AppState, Panel};
use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use emu::models::{AndroidDevice, DeviceStatus};
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use emu::managers::ios::IosManager;
#[cfg(target_os = "macos")]
use emu::models::IosDevice;

/// Test in mixed environment with Android and iOS devices
#[tokio::test]
async fn test_mixed_platform_device_management() {
    let android_output = r#"Available Android Virtual Devices:
    Name: Android_Platform_Test
    Device: pixel_7 (Pixel 7)
    Path: /Users/user/.android/avd/Android_Platform_Test.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a
---------"#;

    let android_mock = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], android_output)
        .with_success("adb", &["devices"], "List of devices attached\n");

    let android_manager = AndroidManager::with_executor(Arc::new(android_mock)).unwrap();

    #[cfg(target_os = "macos")]
    {
        let ios_output = r#"{
  "devices": {
    "iOS 17.0": [
      {
        "lastBootedAt": "2023-01-01T00:00:00Z",
        "dataPath": "/Users/user/Library/Developer/CoreSimulator/Devices/12345678-ABCD-EFGH-IJKL-123456789ABC/data",
        "logPath": "/Users/user/Library/Logs/CoreSimulator/12345678-ABCD-EFGH-IJKL-123456789ABC",
        "udid": "12345678-ABCD-EFGH-IJKL-123456789ABC",
        "isAvailable": true,
        "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
        "state": "Shutdown",
        "name": "iPhone 15"
      },
      {
        "lastBootedAt": "2023-01-01T00:00:00Z",
        "dataPath": "/Users/user/Library/Developer/CoreSimulator/Devices/87654321-DCBA-HGFE-LKJI-987654321CBA/data",
        "logPath": "/Users/user/Library/Logs/CoreSimulator/87654321-DCBA-HGFE-LKJI-987654321CBA",
        "udid": "87654321-DCBA-HGFE-LKJI-987654321CBA",
        "isAvailable": true,
        "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPad-Pro",
        "state": "Shutdown",
        "name": "iPad Pro"
      }
    ]
  }
}"#;

        let ios_mock = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                ios_output,
            )
            .with_success("xcrun", &["simctl", "list", "devices", "-j"], ios_output);

        let ios_manager = IosManager::with_executor(Arc::new(ios_mock)).unwrap();

        // Get device list for both platforms
        let android_devices = android_manager.list_devices().await.unwrap();
        let ios_devices = ios_manager.list_devices().await.unwrap();

        assert_eq!(android_devices.len(), 1);
        assert_eq!(ios_devices.len(), 2);

        // Integrated management with AppState
        let mut app_state = AppState::new();
        app_state.android_devices = android_devices;
        app_state.ios_devices = ios_devices;

        // Focus switching between platforms
        assert_eq!(app_state.active_panel, Panel::Android);

        // Switch to iOS platform
        app_state.active_panel = Panel::Ios;
        assert_eq!(app_state.active_panel, Panel::Ios);

        // Focus index management
        assert!(app_state.selected_ios < app_state.ios_devices.len());

        // Get selected device (judge by active_panel since there's no iOS-specific method)
        if app_state.active_panel == Panel::Ios && !app_state.ios_devices.is_empty() {
            let selected_ios_device = &app_state.ios_devices[app_state.selected_ios];
            assert_eq!(selected_ios_device.name, "iPhone 15 (iOS iOS 17.0)");
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Test only Android on non-macOS platforms
        let android_devices = android_manager.list_devices().await.unwrap();
        assert_eq!(android_devices.len(), 1);

        let mut app_state = AppState::new();
        app_state.android_devices = android_devices;

        // Verify operation on Android platform
        assert_eq!(app_state.active_panel, emu::app::state::Panel::Android);
        if !app_state.android_devices.is_empty() {
            let selected_device = &app_state.android_devices[app_state.selected_android];
            assert_eq!(selected_device.name, "Android_Platform_Test");
        }
    }
}

/// UI state persistence test during platform switching
#[tokio::test]
async fn test_platform_switch_ui_state_persistence() {
    let mut app_state = AppState::new();

    // Add Android devices
    app_state.android_devices = vec![
        AndroidDevice {
            name: "Android_Device_1".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Android_Device_2".to_string(),
            device_type: "galaxy_s22".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    #[cfg(target_os = "macos")]
    {
        app_state.ios_devices = vec![IosDevice {
            name: "iPhone 15".to_string(),
            udid: "12345678-ABCD-EFGH-IJKL-123456789ABC".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        }];
    }

    // Set focus on Android
    app_state.active_panel = Panel::Android;
    app_state.selected_android = 1; // Focus on second device

    // Verify selected device
    assert_eq!(
        app_state.android_devices[app_state.selected_android].name,
        "Android_Device_2"
    );

    #[cfg(target_os = "macos")]
    {
        // Switch to iOS
        app_state.active_panel = Panel::Ios;
        app_state.selected_ios = 0;

        let selected_ios = &app_state.ios_devices[app_state.selected_ios];
        assert_eq!(selected_ios.name, "iPhone 15");

        // Return to Android
        app_state.active_panel = Panel::Android;

        // Verify focus state is preserved
        assert_eq!(app_state.selected_android, 1);
        let selected_android_again = &app_state.android_devices[app_state.selected_android];
        assert_eq!(selected_android_again.name, "Android_Device_2");
    }
}

/// Concurrent device operations test on multiple platforms
#[tokio::test]
async fn test_conactive_panel_operations() {
    let android_output = r#"Available Android Virtual Devices:
    Name: Concurrent_Android_Device
    Device: pixel_7 (Pixel 7)
    Path: /Users/user/.android/avd/Concurrent_Android_Device.avd
    Target: Google APIs (Google Inc.)
            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a
---------"#;

    let android_mock = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], android_output)
        .with_success("adb", &["devices"], "List of devices attached\n");

    let android_manager = Arc::new(AndroidManager::with_executor(Arc::new(android_mock)).unwrap());

    #[cfg(target_os = "macos")]
    {
        let ios_output = r#"{
  "devices": {
    "iOS 17.0": [
      {
        "lastBootedAt": "2023-01-01T00:00:00Z",
        "dataPath": "/Users/user/Library/Developer/CoreSimulator/Devices/12345678-ABCD-EFGH-IJKL-123456789ABC/data",
        "logPath": "/Users/user/Library/Logs/CoreSimulator/12345678-ABCD-EFGH-IJKL-123456789ABC",
        "udid": "12345678-ABCD-EFGH-IJKL-123456789ABC",
        "isAvailable": true,
        "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
        "state": "Shutdown",
        "name": "iPhone 15"
      }
    ]
  }
}"#;

        let ios_mock = MockCommandExecutor::new()
            .with_success(
                "xcrun",
                &["simctl", "list", "devices", "--json"],
                ios_output,
            )
            .with_success("xcrun", &["simctl", "list", "devices", "-j"], ios_output);

        let ios_manager = Arc::new(IosManager::with_executor(Arc::new(ios_mock)).unwrap());

        // Get device list concurrently
        let android_handle = {
            let manager = android_manager.clone();
            tokio::spawn(async move { manager.list_devices().await })
        };

        let ios_handle = {
            let manager = ios_manager.clone();
            tokio::spawn(async move { manager.list_devices().await })
        };

        // Verify both operations succeed
        let android_result = android_handle.await.unwrap();
        let ios_result = ios_handle.await.unwrap();

        assert!(android_result.is_ok());
        assert!(ios_result.is_ok());

        let android_devices = android_result.unwrap();
        let ios_devices = ios_result.unwrap();

        assert_eq!(android_devices.len(), 1);
        assert_eq!(ios_devices.len(), 1);
        assert_eq!(android_devices[0].name, "Concurrent_Android_Device");
        assert_eq!(ios_devices[0].name, "iPhone 15 (iOS iOS 17.0)");
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Test Android only on non-macOS platforms
        let android_result = android_manager.list_devices().await;
        assert!(android_result.is_ok());
        let devices = android_result.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "Concurrent_Android_Device");
    }
}

/// Platform-specific error handling test
#[tokio::test]
async fn test_platform_specific_error_handling() {
    // Android error case
    let android_error_mock = MockCommandExecutor::new().with_error(
        "avdmanager",
        &["list", "avd"],
        "Android SDK not found",
    );

    let android_manager = AndroidManager::with_executor(Arc::new(android_error_mock)).unwrap();
    let android_result = android_manager.list_devices().await;
    assert!(android_result.is_err());

    #[cfg(target_os = "macos")]
    {
        // iOS error case
        let ios_error_mock = MockCommandExecutor::new().with_error(
            "xcrun",
            &["simctl", "list", "devices"],
            "Xcode not installed",
        );

        let ios_manager = IosManager::with_executor(Arc::new(ios_error_mock)).unwrap();
        let ios_result = ios_manager.list_devices().await;
        assert!(ios_result.is_err());

        // Error handling in AppState
        let mut app_state = AppState::new();

        // Verify errors don't affect other platforms
        app_state.android_devices = vec![]; // Empty state due to error
        app_state.ios_devices = vec![]; // Empty state due to error

        // Verify safe access even in empty state
        assert!(app_state.get_selected_device_details().is_none());
    }
}

/// Focus management and navigation integration test
#[tokio::test]
async fn test_focus_management_integration() {
    let mut app_state = AppState::new();

    // Set up multiple devices
    app_state.android_devices = vec![
        AndroidDevice {
            name: "Android_Focus_1".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Android_Focus_2".to_string(),
            device_type: "galaxy_s22".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
        AndroidDevice {
            name: "Android_Focus_3".to_string(),
            device_type: "pixel_tablet".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "6144".to_string(),
            storage_size: "32768".to_string(),
        },
    ];

    // Verify initial focus
    app_state.active_panel = Panel::Android;
    assert_eq!(app_state.selected_android, 0);

    let initial_selected = &app_state.android_devices[app_state.selected_android];
    assert_eq!(initial_selected.name, "Android_Focus_1");

    // Simulate focus movement (move down)
    app_state.selected_android = 1;
    let second_selected = &app_state.android_devices[app_state.selected_android];
    assert_eq!(second_selected.name, "Android_Focus_2");

    // Focus movement (move down further)
    app_state.selected_android = 2;
    let third_selected = &app_state.android_devices[app_state.selected_android];
    assert_eq!(third_selected.name, "Android_Focus_3");

    // Boundary check: verify index doesn't go out of range
    assert!(app_state.selected_android < app_state.android_devices.len());

    #[cfg(target_os = "macos")]
    {
        app_state.ios_devices = vec![
            IosDevice {
                name: "iOS_Focus_1".to_string(),
                udid: "12345678-ABCD-EFGH-IJKL-123456789ABC".to_string(),
                device_type: "iPhone 15".to_string(),
                ios_version: "17.0".to_string(),
                runtime_version: "iOS 17.0".to_string(),
                status: DeviceStatus::Stopped,
                is_running: false,
                is_available: true,
            },
            IosDevice {
                name: "iOS_Focus_2".to_string(),
                udid: "87654321-DCBA-HGFE-LKJI-987654321CBA".to_string(),
                device_type: "iPad Pro".to_string(),
                ios_version: "17.0".to_string(),
                runtime_version: "iOS 17.0".to_string(),
                status: DeviceStatus::Running,
                is_running: true,
                is_available: true,
            },
        ];

        // Switch to iOS platform
        app_state.active_panel = Panel::Ios;
        app_state.selected_ios = 0;

        let ios_selected = &app_state.ios_devices[app_state.selected_ios];
        assert_eq!(ios_selected.name, "iOS_Focus_1");

        // Focus movement in iOS
        app_state.selected_ios = 1;
        let ios_second_selected = &app_state.ios_devices[app_state.selected_ios];
        assert_eq!(ios_second_selected.name, "iOS_Focus_2");

        // Verify focus is preserved when returning to Android
        app_state.active_panel = Panel::Android;
        assert_eq!(app_state.selected_android, 2); // Previous position is preserved

        let android_restored = &app_state.android_devices[app_state.selected_android];
        assert_eq!(android_restored.name, "Android_Focus_3");
    }
}

/// Device state synchronization test between platforms
#[tokio::test]
async fn test_platform_state_synchronization() {
    let mut app_state = AppState::new();

    // Set device states
    let running_android = AndroidDevice {
        name: "Running_Android".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    };

    let stopped_android = AndroidDevice {
        name: "Stopped_Android".to_string(),
        device_type: "galaxy_s22".to_string(),
        api_level: 33,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "4096".to_string(),
        storage_size: "16384".to_string(),
    };

    app_state.android_devices = vec![running_android, stopped_android];

    // Identify running and stopped devices
    let running_devices: Vec<_> = app_state
        .android_devices
        .iter()
        .filter(|d| d.status == DeviceStatus::Running)
        .collect();
    let stopped_devices: Vec<_> = app_state
        .android_devices
        .iter()
        .filter(|d| d.status == DeviceStatus::Stopped)
        .collect();

    assert_eq!(running_devices.len(), 1);
    assert_eq!(stopped_devices.len(), 1);
    assert_eq!(running_devices[0].name, "Running_Android");
    assert_eq!(stopped_devices[0].name, "Stopped_Android");

    // Verify state consistency
    for device in &app_state.android_devices {
        assert_eq!(device.status == DeviceStatus::Running, device.is_running);
    }

    #[cfg(target_os = "macos")]
    {
        let running_ios = IosDevice {
            name: "Running_iOS".to_string(),
            udid: "12345678-ABCD-EFGH-IJKL-123456789ABC".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            is_available: true,
        };

        app_state.ios_devices = vec![running_ios];

        // Verify running device count across all platforms
        let android_running = app_state
            .android_devices
            .iter()
            .filter(|d| d.status == DeviceStatus::Running)
            .count();
        let ios_running = app_state
            .ios_devices
            .iter()
            .filter(|d| d.status == DeviceStatus::Running)
            .count();

        assert_eq!(android_running + ios_running, 2); // 1 Android + 1 iOS
    }
}
