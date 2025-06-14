use emu::app::state::{AppState, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

#[test]
fn test_device_operation_status_management() {
    println!("=== DEVICE OPERATION STATUS MANAGEMENT TEST ===");

    let mut state = AppState::new();

    // Test initial state
    assert!(state.get_device_operation_status().is_none());

    // Test setting operation status
    state.set_device_operation_status("Starting device 'Test Device'...".to_string());
    assert!(state.get_device_operation_status().is_some());
    assert_eq!(
        state.get_device_operation_status().unwrap(),
        "Starting device 'Test Device'..."
    );

    // Test updating operation status
    state.set_device_operation_status("Stopping device 'Test Device'...".to_string());
    assert_eq!(
        state.get_device_operation_status().unwrap(),
        "Stopping device 'Test Device'..."
    );

    // Test clearing operation status
    state.clear_device_operation_status();
    assert!(state.get_device_operation_status().is_none());

    println!("✅ Device operation status management works correctly");
}

#[test]
fn test_device_cache_management() {
    println!("=== DEVICE CACHE MANAGEMENT TEST ===");

    let mut state = AppState::new();

    // Test initial cache state
    assert!(state.cached_device_details.is_none());

    // Create mock device details
    let mock_details = emu::app::state::DeviceDetails {
        name: "Test Device".to_string(),
        status: "Running".to_string(),
        platform: Panel::Android,
        device_type: "Phone".to_string(),
        api_level_or_version: "API 31".to_string(),
        ram_size: Some("2048 MB".to_string()),
        storage_size: Some("8192 MB".to_string()),
        resolution: Some("1080x1920".to_string()),
        dpi: Some("420".to_string()),
        device_path: Some("/path/to/device".to_string()),
        system_image: Some("system.img".to_string()),
        identifier: "test_device".to_string(),
        udid: None,    // Android devices don't have UDID
        runtime: None, // Android devices don't have runtime version
    };

    // Test updating cache
    state.update_cached_device_details(mock_details.clone());
    assert!(state.cached_device_details.is_some());
    assert_eq!(
        state.cached_device_details.as_ref().unwrap().name,
        "Test Device"
    );

    // Test clearing cache
    state.clear_cached_device_details();
    assert!(state.cached_device_details.is_none());

    // Test smart cache clearing
    state.update_cached_device_details(mock_details.clone());

    // Smart clear with same panel should not clear
    state.smart_clear_cached_device_details(Panel::Android);
    assert!(state.cached_device_details.is_some());

    // Smart clear with different panel should clear
    state.smart_clear_cached_device_details(Panel::Ios);
    assert!(state.cached_device_details.is_none());

    println!("✅ Device cache management works correctly");
}

#[test]
fn test_pending_device_start_tracking() {
    println!("=== PENDING DEVICE START TRACKING TEST ===");

    let mut state = AppState::new();

    // Test initial state
    assert!(state.get_pending_device_start().is_none());

    // Test setting pending device
    state.set_pending_device_start("Test Device".to_string());
    assert!(state.get_pending_device_start().is_some());
    assert_eq!(state.get_pending_device_start().unwrap(), "Test Device");

    // Verify refresh interval changed
    assert_eq!(state.auto_refresh_interval.as_secs(), 1); // Should be faster when pending

    // Test clearing pending device
    state.clear_pending_device_start();
    assert!(state.get_pending_device_start().is_none());
    assert_eq!(state.auto_refresh_interval.as_secs(), 3); // Should return to normal

    println!("✅ Pending device start tracking works correctly");
}

#[test]
fn test_device_selection_and_details_sync() {
    println!("=== DEVICE SELECTION AND DETAILS SYNC TEST ===");

    let mut state = AppState::new();

    // Set up mock Android devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Device1".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Device2".to_string(),
            device_type: "tablet".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    // Set up mock iOS devices
    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "udid1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Test Android panel selection
    state.active_panel = Panel::Android;
    state.selected_android = 0;

    let details = state.get_selected_device_details();
    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.name, "Device1");
    assert_eq!(details.platform, Panel::Android);
    assert_eq!(details.identifier, "Device1");

    // Test device selection change
    state.selected_android = 1;
    let details = state.get_selected_device_details();
    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.name, "Device2");
    assert_eq!(details.status, "Running");

    // Test iOS panel selection
    state.active_panel = Panel::Ios;
    state.selected_ios = 0;

    let details = state.get_selected_device_details();
    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.name, "iPhone 14");
    assert_eq!(details.platform, Panel::Ios);
    assert_eq!(details.identifier, "udid1");

    println!("✅ Device selection and details sync works correctly");
}

#[test]
fn test_notification_system() {
    println!("=== NOTIFICATION SYSTEM TEST ===");

    let mut state = AppState::new();

    // Test initial state
    assert!(state.notifications.is_empty());

    // Test adding different types of notifications
    state.add_success_notification("Device started successfully".to_string());
    assert_eq!(state.notifications.len(), 1);

    state.add_error_notification("Failed to start device".to_string());
    assert_eq!(state.notifications.len(), 2);

    state.add_warning_notification("Device may be slow".to_string());
    assert_eq!(state.notifications.len(), 3);

    state.add_info_notification("Starting device...".to_string());
    assert_eq!(state.notifications.len(), 4);

    // Test notification content
    let first_notification = &state.notifications[0];
    assert_eq!(first_notification.message, "Device started successfully");

    // Test clearing all notifications
    state.dismiss_all_notifications();
    assert!(state.notifications.is_empty());

    // Test maximum notifications limit
    state.max_notifications = 2;
    state.add_info_notification("Notification 1".to_string());
    state.add_info_notification("Notification 2".to_string());
    state.add_info_notification("Notification 3".to_string()); // Should push out first

    assert_eq!(state.notifications.len(), 2);
    assert_eq!(state.notifications[0].message, "Notification 2");
    assert_eq!(state.notifications[1].message, "Notification 3");

    println!("✅ Notification system works correctly");
}

#[test]
fn test_log_management() {
    println!("=== LOG MANAGEMENT TEST ===");

    let mut state = AppState::new();

    // Test initial state
    assert!(state.device_logs.is_empty());
    assert_eq!(state.log_scroll_offset, 0);
    assert!(state.auto_scroll_logs);
    assert!(!state.manually_scrolled);

    // Test adding logs
    state.add_log("INFO".to_string(), "Device started".to_string());
    state.add_log("ERROR".to_string(), "Connection failed".to_string());
    state.add_log("DEBUG".to_string(), "Debug information".to_string());

    assert_eq!(state.device_logs.len(), 3);

    // Test log filtering
    state.toggle_log_filter(Some("ERROR".to_string()));
    let filtered_logs = state.get_filtered_logs();
    assert_eq!(filtered_logs.len(), 1);
    assert_eq!(filtered_logs[0].level, "ERROR");

    // Test clearing filter
    state.toggle_log_filter(None);
    let all_logs = state.get_filtered_logs();
    assert_eq!(all_logs.len(), 3);

    // Test log scrolling
    state.scroll_logs_down();
    assert!(state.manually_scrolled);

    state.scroll_logs_up();
    assert!(state.manually_scrolled);

    // Test auto-scroll toggle
    state.toggle_auto_scroll();
    assert!(!state.auto_scroll_logs);

    // Test clearing logs
    state.clear_logs();
    assert!(state.device_logs.is_empty());

    println!("✅ Log management works correctly");
}

#[test]
fn test_panel_switching_state_consistency() {
    println!("=== PANEL SWITCHING STATE CONSISTENCY TEST ===");

    let mut state = AppState::new();

    // Set up devices for both panels
    state.android_devices = vec![AndroidDevice {
        name: "Android1".to_string(),
        device_type: "phone".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    }];

    state.ios_devices = vec![IosDevice {
        name: "iOS1".to_string(),
        udid: "udid1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Test initial state
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Test panel switching
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);

    state.next_panel();
    assert_eq!(state.active_panel, Panel::Android);

    // Test selection persistence during panel switching
    state.selected_android = 0;
    state.selected_ios = 0;

    state.active_panel = Panel::Ios;
    assert_eq!(state.selected_android, 0); // Should remain unchanged
    assert_eq!(state.selected_ios, 0);

    state.active_panel = Panel::Android;
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0); // Should remain unchanged

    println!("✅ Panel switching state consistency works correctly");
}
