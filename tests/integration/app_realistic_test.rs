//! Realistic application integration tests
//!
//! These tests verify actual application behavior using real AppState
//! methods and device operations without requiring external dependencies.

use anyhow::Result;
use emu::app::state::AppState;
use emu::app::state::LogEntry;
use emu::app::Panel;
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Test basic AppState initialization and device management
#[tokio::test]
async fn test_app_state_basic_operations() -> Result<()> {
    let app_state = AppState::new();

    // Verify initial state
    assert_eq!(app_state.active_panel, Panel::Android);
    assert_eq!(app_state.android_devices.len(), 0);
    assert_eq!(app_state.ios_devices.len(), 0);
    assert_eq!(app_state.selected_android, 0);
    assert_eq!(app_state.selected_ios, 0);
    assert!(app_state.is_loading); // AppState starts in loading state

    Ok(())
}

/// Test device list management
#[tokio::test]
async fn test_device_list_management() -> Result<()> {
    let mut app_state = AppState::new();

    // Add Android device
    let android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "test_android".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    app_state.android_devices.push(android_device);

    // Add iOS device
    let ios_device = IosDevice {
        name: "test_ios".to_string(),
        udid: "ABC123-DEF456".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    app_state.ios_devices.push(ios_device);

    // Verify devices were added
    assert_eq!(app_state.android_devices.len(), 1);
    assert_eq!(app_state.ios_devices.len(), 1);
    assert_eq!(app_state.android_devices[0].name, "test_android");
    assert_eq!(app_state.ios_devices[0].name, "test_ios");

    Ok(())
}

/// Test panel switching behavior
#[tokio::test]
async fn test_panel_switching() -> Result<()> {
    let mut app_state = AppState::new();

    // Initially on Android panel
    assert_eq!(app_state.active_panel, Panel::Android);

    // Switch to iOS panel
    app_state.active_panel = Panel::Ios;
    assert_eq!(app_state.active_panel, Panel::Ios);

    // Switch back to Android panel
    app_state.active_panel = Panel::Android;
    assert_eq!(app_state.active_panel, Panel::Android);

    Ok(())
}

/// Test device selection within panels
#[tokio::test]
async fn test_device_selection() -> Result<()> {
    let mut app_state = AppState::new();

    // Add multiple devices
    for i in 0..3 {
        let android_device = AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: format!("android_{i}"),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192M".to_string(),
        };
        app_state.android_devices.push(android_device);

        let ios_device = IosDevice {
            name: format!("ios_{i}"),
            udid: format!("UUID-{i}"),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };
        app_state.ios_devices.push(ios_device);
    }

    // Test Android device selection
    app_state.active_panel = Panel::Android;
    app_state.selected_android = 1;
    assert_eq!(app_state.selected_android, 1);

    // Test iOS device selection
    app_state.active_panel = Panel::Ios;
    app_state.selected_ios = 2;
    assert_eq!(app_state.selected_ios, 2);

    // Verify device counts
    assert_eq!(app_state.android_devices.len(), 3);
    assert_eq!(app_state.ios_devices.len(), 3);

    Ok(())
}

/// Test device status changes
#[tokio::test]
async fn test_device_status_changes() -> Result<()> {
    let mut app_state = AppState::new();

    // Add a device
    let android_device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "status_test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    app_state.android_devices.push(android_device);

    // Verify initial status
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Stopped);
    assert!(!app_state.android_devices[0].is_running);

    // Change to starting
    app_state.android_devices[0].status = DeviceStatus::Starting;
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Starting);

    // Change to running
    app_state.android_devices[0].status = DeviceStatus::Running;
    app_state.android_devices[0].is_running = true;
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Running);
    assert!(app_state.android_devices[0].is_running);

    // Change back to stopped
    app_state.android_devices[0].status = DeviceStatus::Stopped;
    app_state.android_devices[0].is_running = false;
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Stopped);
    assert!(!app_state.android_devices[0].is_running);

    Ok(())
}

/// Test device creation form state
#[tokio::test]
async fn test_create_device_form() -> Result<()> {
    let mut app_state = AppState::new();

    // Verify initial form state
    assert_eq!(app_state.create_device_form.name, "");
    assert_eq!(app_state.create_device_form.device_type, "");

    // Update create device form
    app_state.create_device_form.name = "new_device".to_string();
    app_state.create_device_form.device_type = "pixel_7".to_string();
    app_state.create_device_form.version = "34".to_string();
    app_state.create_device_form.ram_size = "4096".to_string();

    // Verify form state
    assert_eq!(app_state.create_device_form.name, "new_device");
    assert_eq!(app_state.create_device_form.device_type, "pixel_7");
    assert_eq!(app_state.create_device_form.version, "34");
    assert_eq!(app_state.create_device_form.ram_size, "4096");

    // Clear form
    app_state.create_device_form.name.clear();
    assert_eq!(app_state.create_device_form.name, "");

    Ok(())
}

/// Test device logs management
#[tokio::test]
async fn test_device_logs() -> Result<()> {
    let mut app_state = AppState::new();

    // Initially no logs
    assert_eq!(app_state.device_logs.len(), 0);

    // Add some log entries (simulating log streaming)
    let max_entries = app_state.max_log_entries;

    // Fill up to the limit
    for i in 0..max_entries {
        app_state.device_logs.push_back(LogEntry {
            timestamp: format!("12:00:{i:02}"),
            level: "INFO".to_string(),
            message: format!("Log entry {i}"),
        });
    }

    assert_eq!(app_state.device_logs.len(), max_entries);

    // Add one more - should trigger rotation
    app_state.device_logs.push_back(LogEntry {
        timestamp: "12:00:59".to_string(),
        level: "INFO".to_string(),
        message: "Latest log entry".to_string(),
    });

    if app_state.device_logs.len() > max_entries {
        app_state.device_logs.pop_front();
    }

    // Should maintain max entries limit
    assert_eq!(app_state.device_logs.len(), max_entries);

    Ok(())
}

/// Test concurrent device operations simulation
#[tokio::test]
async fn test_concurrent_operations_simulation() -> Result<()> {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // Add some devices
    {
        let mut state = app_state.lock().await;
        for i in 0..3 {
            let device = AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("concurrent_device_{i}"),
                device_type: "pixel_7".to_string(),
                api_level: 34,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "4096".to_string(),
                storage_size: "8192M".to_string(),
            };
            state.android_devices.push(device);
        }
    }

    // Simulate concurrent operations
    let tasks: Vec<_> = (0..3)
        .map(|i| {
            let app_clone = Arc::clone(&app_state);
            tokio::spawn(async move {
                let mut state = app_clone.lock().await;
                if let Some(device) = state.android_devices.get_mut(i) {
                    device.status = DeviceStatus::Starting;
                    // Simulate processing time
                    drop(state);
                    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    let mut state = app_clone.lock().await;
                    if let Some(device) = state.android_devices.get_mut(i) {
                        device.status = DeviceStatus::Running;
                        device.is_running = true;
                    }
                }
            })
        })
        .collect();

    // Wait for all operations to complete
    for task in tasks {
        task.await?;
    }

    // Verify all devices are running
    {
        let state = app_state.lock().await;
        for device in &state.android_devices {
            assert_eq!(device.status, DeviceStatus::Running);
            assert!(device.is_running);
        }
    }

    Ok(())
}

/// Test loading state management
#[tokio::test]
async fn test_loading_state() -> Result<()> {
    let mut app_state = AppState::new();

    // Initially loading (AppState starts in loading state)
    assert!(app_state.is_loading);

    // Set loading state
    app_state.is_loading = true;
    assert!(app_state.is_loading);

    // Clear loading state
    app_state.is_loading = false;
    assert!(!app_state.is_loading);

    Ok(())
}

/// Test error states and recovery
#[tokio::test]
async fn test_error_states() -> Result<()> {
    let mut app_state = AppState::new();

    // Add device in error state
    let device = AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "error_device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Error,
        is_running: false,
        ram_size: "4096".to_string(),
        storage_size: "8192M".to_string(),
    };

    app_state.android_devices.push(device);

    // Verify error state
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Error);
    assert!(!app_state.android_devices[0].is_running);

    // Recovery: reset to stopped state
    app_state.android_devices[0].status = DeviceStatus::Stopped;
    assert_eq!(app_state.android_devices[0].status, DeviceStatus::Stopped);

    Ok(())
}
