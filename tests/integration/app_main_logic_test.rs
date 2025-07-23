//! Main logic tests for app/mod.rs
//!
//! Tests basic functionality of application initialization, device management, and state updates.

use emu::app::{AppState, Mode, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

/// Basic initialization test for App
#[tokio::test]
async fn test_app_state_creation_basic() {
    // Since App's new() method creates actual Android/iOS managers,
    // first test basic initialization of AppState
    let state = AppState::new();

    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.mode, Mode::Normal);
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
}

/// State transition test for AppState
#[tokio::test]
async fn test_app_state_transitions() {
    let mut state = AppState::new();

    // Panel switching test
    assert_eq!(state.active_panel, Panel::Android);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Android);
}

/// Device selection behavior test
#[tokio::test]
async fn test_device_selection() {
    let mut state = AppState::new();

    // Add Android devices
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "test_device_1".to_string(),
            device_type: "pixel_4".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "test_device_2".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    // Verify initial state
    assert_eq!(state.selected_android, 0);

    // Test downward movement
    state.move_down();
    assert_eq!(state.selected_android, 1);

    // Test circular navigation (from last to first)
    state.move_down();
    assert_eq!(state.selected_android, 0);

    // Test upward movement
    state.move_up();
    assert_eq!(state.selected_android, 1);

    // Test circular navigation (from first to last)
    state.selected_android = 0;
    state.move_up();
    assert_eq!(state.selected_android, 1);
}

/// iOS device selection behavior test
#[tokio::test]
async fn test_ios_device_selection() {
    let mut state = AppState::new();
    state.active_panel = Panel::Ios;

    // Add iOS devices
    state.ios_devices = vec![
        IosDevice {
            name: "iPhone 14".to_string(),
            udid: "12345678-1234-1234-1234-123456789012".to_string(),
            device_type: "iPhone 14".to_string(),
            ios_version: "16.0".to_string(),
            runtime_version: "iOS 16.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
        IosDevice {
            name: "iPad Pro".to_string(),
            udid: "87654321-4321-4321-4321-210987654321".to_string(),
            device_type: "iPad Pro (12.9-inch)".to_string(),
            ios_version: "16.0".to_string(),
            runtime_version: "iOS 16.0".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            is_available: true,
        },
    ];

    // Verify initial state
    assert_eq!(state.selected_ios, 0);

    // Test downward movement
    state.move_down();
    assert_eq!(state.selected_ios, 1);

    // Test circular navigation (from last to first)
    state.move_down();
    assert_eq!(state.selected_ios, 0);
}

/// Selection test with empty device list
#[tokio::test]
async fn test_empty_device_list_selection() {
    let mut state = AppState::new();

    // No selection change with empty list
    state.move_down();
    assert_eq!(state.selected_android, 0);

    state.move_up();
    assert_eq!(state.selected_android, 0);

    // Same for iOS
    state.active_panel = Panel::Ios;
    state.move_down();
    assert_eq!(state.selected_ios, 0);

    state.move_up();
    assert_eq!(state.selected_ios, 0);
}

/// Mode transition test
#[tokio::test]
async fn test_mode_transitions() {
    let mut state = AppState::new();

    // Initial state
    assert_eq!(state.mode, Mode::Normal);

    // To CreateDevice mode
    state.mode = Mode::CreateDevice;
    assert_eq!(state.mode, Mode::CreateDevice);

    // Return to Normal mode
    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);

    // Other modes
    state.mode = Mode::ConfirmDelete;
    assert_eq!(state.mode, Mode::ConfirmDelete);

    state.mode = Mode::ConfirmWipe;
    assert_eq!(state.mode, Mode::ConfirmWipe);

    state.mode = Mode::ManageApiLevels;
    assert_eq!(state.mode, Mode::ManageApiLevels);

    state.mode = Mode::Help;
    assert_eq!(state.mode, Mode::Help);
}

/// Device list update test
#[tokio::test]
async fn test_device_list_updates() {
    let mut state = AppState::new();

    // Add Android device
    let android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "new_device".to_string(),
        device_type: "pixel_5".to_string(),
        api_level: 31,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "3072".to_string(),
        storage_size: "12288M".to_string(),
    };

    state.android_devices.push(android_device);
    assert_eq!(state.android_devices.len(), 1);
    assert_eq!(state.android_devices[0].name, "new_device");

    // Add iOS device
    let ios_device = IosDevice {
        name: "iPhone 15".to_string(),
        udid: "11111111-2222-3333-4444-555555555555".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    state.ios_devices.push(ios_device);
    assert_eq!(state.ios_devices.len(), 1);
    assert_eq!(state.ios_devices[0].name, "iPhone 15");
}

/// Test selection state persistence during panel switching
#[tokio::test]
async fn test_panel_switch_selection_persistence() {
    let mut state = AppState::new();

    // Add Android devices
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "android_1".to_string(),
            device_type: "pixel_4".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "android_2".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    // Add iOS devices
    state.ios_devices = vec![IosDevice {
        name: "ios_1".to_string(),
        udid: "12345678-1234-1234-1234-123456789012".to_string(),
        device_type: "iPhone 14".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Change selection in Android panel
    assert_eq!(state.active_panel, Panel::Android);
    state.move_down();
    assert_eq!(state.selected_android, 1);

    // Switch to iOS panel
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.selected_ios, 0); // iOS selection remains unchanged

    // Return to Android panel
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.selected_android, 1); // Android selection is maintained
}

/// Device status update test
#[tokio::test]
async fn test_device_status_updates() {
    let mut android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "test_device".to_string(),
        device_type: "pixel_4".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    // Initial state
    assert_eq!(android_device.status, DeviceStatus::Stopped);
    assert!(!android_device.is_running);

    // Update to device started state
    android_device.status = DeviceStatus::Running;
    android_device.is_running = true;

    assert_eq!(android_device.status, DeviceStatus::Running);
    assert!(android_device.is_running);

    // Update to device stopped state
    android_device.status = DeviceStatus::Stopped;
    android_device.is_running = false;

    assert_eq!(android_device.status, DeviceStatus::Stopped);
    assert!(!android_device.is_running);
}

/// Device details test
#[tokio::test]
async fn test_device_details() {
    let android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "test_device_details".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 33,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "8192".to_string(),
        storage_size: "32768M".to_string(),
    };

    // Verify device details
    assert_eq!(android_device.name, "test_device_details");
    assert_eq!(android_device.device_type, "pixel_7");
    assert_eq!(android_device.api_level, 33);
    assert_eq!(android_device.ram_size, "8192");
    assert_eq!(android_device.storage_size, "32768M");

    let ios_device = IosDevice {
        name: "test_ios_details".to_string(),
        udid: "ABCDEF12-3456-7890-ABCD-EF1234567890".to_string(),
        device_type: "iPhone 15 Pro".to_string(),
        ios_version: "17.1".to_string(),
        runtime_version: "iOS 17.1".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: true,
    };

    // Verify iOS device details
    assert_eq!(ios_device.name, "test_ios_details");
    assert_eq!(ios_device.device_type, "iPhone 15 Pro");
    assert_eq!(ios_device.ios_version, "17.1");
    assert_eq!(ios_device.runtime_version, "iOS 17.1");
    assert!(ios_device.is_available);
}
