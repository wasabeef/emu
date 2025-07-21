//! App layer integration tests using fixture data
//!
//! These tests validate application state management, event processing,
//! and UI coordination using controlled test data.

use anyhow::Result;
use emu::app::state::{
    AppState, ConfirmDeleteDialog, ConfirmWipeDialog, FocusedPanel, Mode, Panel,
};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Test app state management and device list synchronization
#[tokio::test]
async fn test_app_state_device_management() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test initial state
    {
        let state = app_state.read().await;
        assert_eq!(state.active_panel, Panel::Android);
        assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
        assert_eq!(state.mode, Mode::Normal);
        assert_eq!(state.android_devices.len(), 0);
        assert_eq!(state.ios_devices.len(), 0);
    }

    // Add Android devices
    let android_devices = vec![
        AndroidDevice {
            name: "Test_Pixel_7".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            name: "Test_Galaxy_S24".to_string(),
            device_type: "galaxy_s24".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    {
        let mut state = app_state.write().await;
        state.android_devices = android_devices.clone();
    }

    // Test device list updates
    {
        let state = app_state.read().await;
        let devices = &state.android_devices;
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].name, "Test_Pixel_7");
        assert_eq!(devices[1].name, "Test_Galaxy_S24");
        assert!(!devices[0].is_running);
        assert!(devices[1].is_running);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_device_details_caching() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test device operation status
    {
        let mut state = app_state.write().await;
        state.device_operation_status = Some("Starting device...".to_string());
    }

    {
        let state = app_state.read().await;
        assert!(state.device_operation_status.is_some());
        assert_eq!(
            state.device_operation_status.as_ref().unwrap(),
            "Starting device..."
        );
    }

    // Clear operation status
    {
        let mut state = app_state.write().await;
        state.device_operation_status = None;
    }

    {
        let state = app_state.read().await;
        assert!(state.device_operation_status.is_none());
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_modal_management() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test normal mode
    {
        let state = app_state.read().await;
        assert_eq!(state.mode, Mode::Normal);
    }

    // Test device creation mode
    {
        let mut state = app_state.write().await;
        state.mode = Mode::CreateDevice;
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.mode, Mode::CreateDevice);
    }

    // Test delete confirmation dialog
    let delete_dialog = ConfirmDeleteDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_device_id".to_string(),
        platform: Panel::Android,
    };

    {
        let mut state = app_state.write().await;
        state.mode = Mode::ConfirmDelete;
        state.confirm_delete_dialog = Some(delete_dialog.clone());
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.mode, Mode::ConfirmDelete);
        let dialog = &state.confirm_delete_dialog;
        assert!(dialog.is_some());

        let dialog = dialog.as_ref().unwrap();
        assert_eq!(dialog.device_name, "Test Device");
        assert_eq!(dialog.device_identifier, "test_device_id");
        assert_eq!(dialog.platform, Panel::Android);
    }

    // Test wipe confirmation dialog
    let wipe_dialog = ConfirmWipeDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_device_id".to_string(),
        platform: Panel::Android,
    };

    {
        let mut state = app_state.write().await;
        state.mode = Mode::ConfirmWipe;
        state.confirm_wipe_dialog = Some(wipe_dialog.clone());
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.mode, Mode::ConfirmWipe);
        let dialog = &state.confirm_wipe_dialog;
        assert!(dialog.is_some());

        let dialog = dialog.as_ref().unwrap();
        assert_eq!(dialog.device_name, "Test Device");
        assert_eq!(dialog.device_identifier, "test_device_id");
        assert_eq!(dialog.platform, Panel::Android);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_panel_navigation() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test initial panel
    {
        let state = app_state.read().await;
        assert_eq!(state.active_panel, Panel::Android);
    }

    // Test panel toggle
    {
        let mut state = app_state.write().await;
        state.next_panel();
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.active_panel, Panel::Ios);
    }

    // Test panel toggle back
    {
        let mut state = app_state.write().await;
        state.next_panel();
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.active_panel, Panel::Android);
    }

    // Test focus panel switching
    {
        let state = app_state.read().await;
        assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
    }

    {
        let mut state = app_state.write().await;
        state.focused_panel = FocusedPanel::LogArea;
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.focused_panel, FocusedPanel::LogArea);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_device_selection() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Add test devices first
    let android_devices = vec![
        AndroidDevice {
            name: "Device_1".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            name: "Device_2".to_string(),
            device_type: "galaxy_s24".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    {
        let mut state = app_state.write().await;
        state.android_devices = android_devices;
    }

    // Test initial selection
    {
        let state = app_state.read().await;
        assert_eq!(state.selected_android, 0);
    }

    // Test selection change
    {
        let mut state = app_state.write().await;
        state.selected_android = 1;
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.selected_android, 1);
    }

    // Test iOS selection (with devices)
    let ios_devices = vec![IosDevice {
        name: "iPhone 15".to_string(),
        udid: "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6".to_string(),
        device_type: "com.apple.CoreSimulator.SimDeviceType.iPhone-15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    {
        let mut state = app_state.write().await;
        state.ios_devices = ios_devices;
    }

    {
        let state = app_state.read().await;
        assert_eq!(state.selected_ios, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_notification_system() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test adding notifications
    {
        let mut state = app_state.write().await;
        let notification1 = emu::app::state::Notification {
            message: "Test notification 1".to_string(),
            notification_type: emu::app::state::NotificationType::Info,
            timestamp: std::time::SystemTime::now().into(),
            auto_dismiss_after: None,
        };
        let notification2 = emu::app::state::Notification {
            message: "Test notification 2".to_string(),
            notification_type: emu::app::state::NotificationType::Success,
            timestamp: std::time::SystemTime::now().into(),
            auto_dismiss_after: None,
        };
        state.add_notification(notification1);
        state.add_notification(notification2);
    }

    {
        let state = app_state.read().await;
        let notifications = &state.notifications;
        assert_eq!(notifications.len(), 2);
        assert_eq!(notifications[0].message, "Test notification 1");
        assert_eq!(notifications[1].message, "Test notification 2");
    }

    // Test notification clearing
    {
        let mut state = app_state.write().await;
        state.notifications.clear();
    }

    {
        let state = app_state.read().await;
        let notifications = &state.notifications;
        assert_eq!(notifications.len(), 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_log_management() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test initial log state
    {
        let state = app_state.read().await;
        let logs = &state.device_logs;
        assert_eq!(logs.len(), 0);
    }

    // Test adding log entries
    {
        let mut state = app_state.write().await;
        let log_entry1 = emu::app::state::LogEntry {
            timestamp: "10:30:15".to_string(),
            level: "INFO".to_string(),
            message: "Test log entry 1".to_string(),
        };
        let log_entry2 = emu::app::state::LogEntry {
            timestamp: "10:30:16".to_string(),
            level: "DEBUG".to_string(),
            message: "Test log entry 2".to_string(),
        };
        let log_entry3 = emu::app::state::LogEntry {
            timestamp: "10:30:17".to_string(),
            level: "ERROR".to_string(),
            message: "Test log entry 3".to_string(),
        };
        state.device_logs.push_back(log_entry1);
        state.device_logs.push_back(log_entry2);
        state.device_logs.push_back(log_entry3);
    }

    {
        let state = app_state.read().await;
        let logs = &state.device_logs;
        assert_eq!(logs.len(), 3);

        // Check that logs were added
        assert!(logs
            .iter()
            .any(|log| log.message.contains("Test log entry 1")));
        assert!(logs
            .iter()
            .any(|log| log.message.contains("Test log entry 2")));
        assert!(logs
            .iter()
            .any(|log| log.message.contains("Test log entry 3")));
    }

    // Test log clearing
    {
        let mut state = app_state.write().await;
        state.device_logs.clear();
    }

    {
        let state = app_state.read().await;
        let logs = &state.device_logs;
        assert_eq!(logs.len(), 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_background_loading() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test initial loading state
    {
        let state = app_state.read().await;
        assert!(state.is_loading); // AppState starts in loading state by default
    }

    // Test setting loading state
    {
        let mut state = app_state.write().await;
        state.is_loading = true;
    }

    {
        let state = app_state.read().await;
        assert!(state.is_loading);
    }

    // Simulate background loading completion
    {
        let mut state = app_state.write().await;
        state.is_loading = false;
    }

    {
        let state = app_state.read().await;
        assert!(!state.is_loading);
    }

    Ok(())
}

#[tokio::test]
async fn test_app_state_concurrent_access() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Test concurrent read access
    let app_state_clone1 = Arc::clone(&app_state);
    let app_state_clone2 = Arc::clone(&app_state);

    let task1 = tokio::spawn(async move {
        let state = app_state_clone1.read().await;
        let panel = state.active_panel;
        tokio::time::sleep(Duration::from_millis(10)).await;
        panel
    });

    let task2 = tokio::spawn(async move {
        let state = app_state_clone2.read().await;
        let panel = state.active_panel;
        tokio::time::sleep(Duration::from_millis(10)).await;
        panel
    });

    let (panel1, panel2): (Panel, Panel) = tokio::try_join!(task1, task2)?;
    assert_eq!(panel1, Panel::Android);
    assert_eq!(panel2, Panel::Android);

    // Test write access coordination
    let app_state_clone3 = Arc::clone(&app_state);
    let app_state_clone4 = Arc::clone(&app_state);

    let write_task = tokio::spawn(async move {
        let mut state = app_state_clone3.write().await;
        state.next_panel();
        tokio::time::sleep(Duration::from_millis(10)).await;
    });

    let read_task = tokio::spawn(async move {
        // Wait for write task to start
        tokio::time::sleep(Duration::from_millis(5)).await;
        let state = app_state_clone4.read().await;
        state.active_panel
    });

    write_task.await?;
    let final_panel = read_task.await?;

    // Should see the updated state after write completes
    assert_eq!(final_panel, Panel::Ios);

    Ok(())
}

#[tokio::test]
async fn test_app_state_comprehensive_workflow() -> Result<()> {
    let app_state = Arc::new(RwLock::new(AppState::new()));

    // Simulate a complete app workflow

    // 1. Initial device loading
    {
        let mut state = app_state.write().await;
        state.is_loading = true;
        let notification = emu::app::state::Notification {
            message: "Loading devices...".to_string(),
            notification_type: emu::app::state::NotificationType::Info,
            timestamp: std::time::SystemTime::now().into(),
            auto_dismiss_after: None,
        };
        state.add_notification(notification);
    }

    // 2. Add discovered devices
    let android_devices = vec![AndroidDevice {
        name: "Pixel_7_API_34".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    }];

    {
        let mut state = app_state.write().await;
        state.android_devices = android_devices;
        state.is_loading = false;
        let notification = emu::app::state::Notification {
            message: "Devices loaded successfully".to_string(),
            notification_type: emu::app::state::NotificationType::Success,
            timestamp: std::time::SystemTime::now().into(),
            auto_dismiss_after: None,
        };
        state.add_notification(notification);
    }

    // 3. User navigates and selects device
    {
        let mut state = app_state.write().await;
        state.selected_android = 0;
    }

    // 4. User initiates device creation
    {
        let mut state = app_state.write().await;
        state.mode = Mode::CreateDevice;
    }

    // 5. User cancels and returns to normal mode
    {
        let mut state = app_state.write().await;
        state.mode = Mode::Normal;
    }

    // 6. User switches to iOS panel
    {
        let mut state = app_state.write().await;
        state.next_panel();
    }

    // 7. Verify final state
    {
        let state = app_state.read().await;
        assert_eq!(state.active_panel, Panel::Ios);
        assert_eq!(state.mode, Mode::Normal);
        assert!(!state.is_loading);
        assert_eq!(state.android_devices.len(), 1);
        assert_eq!(state.notifications.len(), 2);
    }

    Ok(())
}
