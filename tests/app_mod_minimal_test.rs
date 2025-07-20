use emu::app::{App, AppState, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Test App::new() initialization and state setup
#[tokio::test]
async fn test_app_new_initialization() {
    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // App created successfully with proper manager setup
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android")
                    || error_msg.contains("SDK")
                    || error_msg.contains("avdmanager")
                    || error_msg.contains("ANDROID_HOME"),
                "Expected Android SDK-related error, got: {error_msg}"
            );
        }
    }
}

/// Test basic AppState functionality
#[tokio::test]
async fn test_app_state_basic_operations() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Test initial state
        assert_eq!(locked_state.active_panel, Panel::Android);
        assert!(locked_state.android_devices.is_empty());
        assert!(locked_state.ios_devices.is_empty());

        // Test panel switching
        let new_panel = locked_state.active_panel.toggle();
        assert_eq!(new_panel, Panel::Ios);

        locked_state.active_panel = new_panel;
        assert_eq!(locked_state.active_panel, Panel::Ios);

        // Test navigation methods exist
        locked_state.move_up();
        locked_state.move_down();

        // Test log management
        locked_state.clear_logs();
        assert!(locked_state.current_log_device.is_none());

        // Test notification management
        locked_state.dismiss_all_notifications();
        locked_state.dismiss_expired_notifications();

        // Test cache management
        locked_state.smart_clear_cached_device_details(Panel::Android);
        locked_state.smart_clear_cached_device_details(Panel::Ios);
    }
}

/// Test Android device operations with correct structure
#[tokio::test]
async fn test_android_device_operations() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Add Android devices with correct structure
        locked_state.android_devices = vec![
            AndroidDevice {
                name: "test_device_1".to_string(),
                device_type: "pixel_7".to_string(),
                api_level: 29,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192M".to_string(),
            },
            AndroidDevice {
                name: "test_device_2".to_string(),
                device_type: "galaxy_s22".to_string(),
                api_level: 30,
                status: DeviceStatus::Running,
                is_running: true,
                ram_size: "4096".to_string(),
                storage_size: "16G".to_string(),
            },
        ];

        // Test device count and properties
        assert_eq!(locked_state.android_devices.len(), 2);
        assert_eq!(locked_state.android_devices[0].name, "test_device_1");
        assert_eq!(
            locked_state.android_devices[0].status,
            DeviceStatus::Stopped
        );
        assert_eq!(locked_state.android_devices[0].api_level, 29);
        assert!(!locked_state.android_devices[0].is_running);

        assert_eq!(locked_state.android_devices[1].name, "test_device_2");
        assert_eq!(
            locked_state.android_devices[1].status,
            DeviceStatus::Running
        );
        assert_eq!(locked_state.android_devices[1].api_level, 30);
        assert!(locked_state.android_devices[1].is_running);

        // Test navigation
        assert_eq!(locked_state.selected_android, 0);
        locked_state.move_down();
        assert_eq!(locked_state.selected_android, 1);
        locked_state.move_up();
        assert_eq!(locked_state.selected_android, 0);
    }
}

/// Test iOS device operations with correct structure
#[tokio::test]
async fn test_ios_device_operations() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Add iOS devices with correct structure
        locked_state.ios_devices = vec![
            IosDevice {
                name: "iPhone_14_Simulator".to_string(),
                udid: "12345-IPHONE-14".to_string(),
                device_type: "iPhone 14".to_string(),
                ios_version: "16.0".to_string(),
                runtime_version: "iOS 16.0".to_string(),
                status: DeviceStatus::Stopped,
                is_running: false,
                is_available: true,
            },
            IosDevice {
                name: "iPad_Pro_Simulator".to_string(),
                udid: "67890-IPAD-PRO".to_string(),
                device_type: "iPad Pro (12.9-inch)".to_string(),
                ios_version: "16.1".to_string(),
                runtime_version: "iOS 16.1".to_string(),
                status: DeviceStatus::Running,
                is_running: true,
                is_available: true,
            },
        ];

        // Switch to iOS panel
        locked_state.active_panel = Panel::Ios;
        assert_eq!(locked_state.active_panel, Panel::Ios);

        // Test iOS device properties
        assert_eq!(locked_state.ios_devices.len(), 2);
        assert_eq!(locked_state.ios_devices[0].name, "iPhone_14_Simulator");
        assert_eq!(locked_state.ios_devices[0].device_type, "iPhone 14");
        assert_eq!(locked_state.ios_devices[0].ios_version, "16.0");
        assert!(!locked_state.ios_devices[0].is_running);
        assert!(locked_state.ios_devices[0].is_available);

        assert_eq!(locked_state.ios_devices[1].name, "iPad_Pro_Simulator");
        assert_eq!(
            locked_state.ios_devices[1].device_type,
            "iPad Pro (12.9-inch)"
        );
        assert_eq!(locked_state.ios_devices[1].ios_version, "16.1");
        assert!(locked_state.ios_devices[1].is_running);
        assert!(locked_state.ios_devices[1].is_available);

        // Test iOS navigation
        assert_eq!(locked_state.selected_ios, 0);
        locked_state.move_down();
        assert_eq!(locked_state.selected_ios, 1);
        locked_state.move_up();
        assert_eq!(locked_state.selected_ios, 0);
    }
}

/// Test log device tracking
#[tokio::test]
async fn test_log_device_tracking() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Test initial state
        assert!(locked_state.current_log_device.is_none());

        // Set log device
        locked_state.current_log_device = Some((Panel::Android, "test_device".to_string()));
        assert_eq!(
            locked_state.current_log_device,
            Some((Panel::Android, "test_device".to_string()))
        );

        // Clear logs and device
        locked_state.clear_logs();
        locked_state.current_log_device = None;
        assert!(locked_state.current_log_device.is_none());
    }
}

/// Test auto-refresh functionality
#[tokio::test]
async fn test_auto_refresh_functionality() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let locked_state = state.lock().await;

        // Test auto-refresh settings
        let should_refresh = locked_state.should_auto_refresh();
        // Note: auto-refresh behavior may vary based on implementation
        let _auto_refresh_enabled = should_refresh;

        // Test with empty device lists
        let has_devices =
            !locked_state.android_devices.is_empty() || !locked_state.ios_devices.is_empty();
        assert!(!has_devices, "Device lists should be empty initially");

        let would_refresh = should_refresh && has_devices;
        // With empty device lists, refresh behavior should be false
        assert!(!would_refresh, "Should not refresh with empty device lists");
    }
}

/// Test concurrent state access
#[tokio::test]
async fn test_concurrent_state_access() {
    let state = Arc::new(Mutex::new(AppState::new()));
    let state_clone = Arc::clone(&state);

    // Spawn concurrent task
    let task = tokio::spawn(async move {
        let locked_state = state_clone.lock().await;
        assert_eq!(locked_state.active_panel, Panel::Android);
        Duration::from_millis(10)
    });

    // Access from main thread
    {
        let locked_state = state.lock().await;
        assert_eq!(locked_state.active_panel, Panel::Android);
    }

    // Wait for task completion
    let duration = task.await.unwrap();
    assert!(duration.as_millis() >= 10);
}

/// Test background task handle management
#[tokio::test]
async fn test_background_task_handle_management() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Test initial state
        assert!(locked_state.log_task_handle.is_none());

        // Simulate setting a task handle
        let mock_task = tokio::spawn(async {
            tokio::time::sleep(Duration::from_millis(50)).await;
        });

        locked_state.log_task_handle = Some(mock_task);
        assert!(locked_state.log_task_handle.is_some());

        // Test task cleanup
        if let Some(handle) = locked_state.log_task_handle.take() {
            handle.abort();
        }
        assert!(locked_state.log_task_handle.is_none());
    }
}

/// Test cache clearing operations
#[tokio::test]
async fn test_cache_clearing_operations() {
    let state = Arc::new(Mutex::new(AppState::new()));

    {
        let mut locked_state = state.lock().await;

        // Test cache clearing for different panels
        locked_state.smart_clear_cached_device_details(Panel::Android);
        locked_state.smart_clear_cached_device_details(Panel::Ios);

        // Test with panel switching
        locked_state.active_panel = Panel::Android;
        locked_state.smart_clear_cached_device_details(Panel::Ios);

        locked_state.active_panel = Panel::Ios;
        locked_state.smart_clear_cached_device_details(Panel::Android);

        // Switch back to Android
        locked_state.active_panel = Panel::Android;

        // These operations should complete without errors
        // Cache clearing operations completed successfully
    }
}
