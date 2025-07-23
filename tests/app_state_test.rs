//! Comprehensive app state management tests
//! Tests all major state management functions in src/app/state.rs

use emu::{
    app::state::{
        ApiLevelManagementState, AppState, ConfirmDeleteDialog, ConfirmWipeDialog,
        CreateDeviceField, CreateDeviceForm, DeviceCache, DeviceDetails, FocusedPanel, LogEntry,
        Mode, Notification, NotificationType, Panel,
    },
    models::{
        api_level::ApiLevel,
        device::{AndroidDevice, DeviceStatus, IosDevice},
    },
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper to create AppState with test devices
fn create_state_with_devices() -> AppState {
    let mut state = AppState::new();

    // Add Android devices
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "Tablet_API_33".to_string(),
            device_type: "tablet".to_string(),
            api_level: 33,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16G".to_string(),
        },
    ];

    // Add iOS devices
    state.ios_devices = vec![
        IosDevice {
            name: "iPhone 15 Pro".to_string(),
            udid: "12345-67890-ABCDEF".to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            is_available: true,
        },
        IosDevice {
            name: "iPad Air".to_string(),
            udid: "09876-54321-FEDCBA".to_string(),
            device_type: "iPad".to_string(),
            ios_version: "16.4".to_string(),
            runtime_version: "iOS 16.4".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
    ];

    state
}

#[test]
fn test_app_state_new() {
    let state = AppState::new();

    // Test initial values
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.mode, Mode::Normal);
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
    assert!(state.is_loading); // AppState starts in loading state
    assert!(state.device_logs.is_empty());
    assert!(state.notifications.is_empty());
    assert!(!state.fullscreen_logs);
    assert!(state.auto_scroll_logs);
}

#[test]
fn test_panel_operations() {
    let state = AppState::new();

    // Test panel toggle
    assert_eq!(Panel::Android.toggle(), Panel::Ios);
    assert_eq!(Panel::Ios.toggle(), Panel::Android);

    // Test focused panel
    assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
}

#[test]
fn test_mode_transitions() {
    let mut state = AppState::new();

    // Test all mode transitions
    state.mode = Mode::CreateDevice;
    assert_eq!(state.mode, Mode::CreateDevice);

    state.mode = Mode::ConfirmDelete;
    assert_eq!(state.mode, Mode::ConfirmDelete);

    state.mode = Mode::ConfirmWipe;
    assert_eq!(state.mode, Mode::ConfirmWipe);

    state.mode = Mode::ManageApiLevels;
    assert_eq!(state.mode, Mode::ManageApiLevels);

    state.mode = Mode::Help;
    assert_eq!(state.mode, Mode::Help);

    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);
}

#[test]
fn test_device_selection() {
    let mut state = create_state_with_devices();

    // Test Android device selection
    state.selected_android = 1;
    assert_eq!(state.selected_android, 1);

    // Test selected device details
    let android_device = state.android_devices.get(state.selected_android);
    assert!(android_device.is_some());
    assert_eq!(android_device.unwrap().name, "Tablet_API_33");

    // Test iOS device selection
    state.active_panel = Panel::Ios;
    state.selected_ios = 1;
    assert_eq!(state.selected_ios, 1);

    let ios_device = state.ios_devices.get(state.selected_ios);
    assert!(ios_device.is_some());
    assert_eq!(ios_device.unwrap().name, "iPad Air");
}

#[test]
fn test_get_selected_device() {
    let mut state = create_state_with_devices();

    // Test Android panel
    state.active_panel = Panel::Android;
    let device = state.android_devices.get(state.selected_android);
    assert!(device.is_some());
    assert_eq!(device.unwrap().name, "Pixel_7_API_34");

    // Test iOS panel
    state.active_panel = Panel::Ios;
    let device = state.ios_devices.get(state.selected_ios);
    assert!(device.is_some());
    assert_eq!(device.unwrap().name, "iPhone 15 Pro");

    // Test empty device list
    state.ios_devices.clear();
    let device = state.ios_devices.get(state.selected_ios);
    assert!(device.is_none());
}

#[test]
fn test_notification_system() {
    let mut state = AppState::new();

    // Test adding notifications
    state.add_notification(Notification::success("Success message".to_string()));
    state.add_notification(Notification::error("Error message".to_string()));
    state.add_notification(Notification::warning("Warning message".to_string()));
    state.add_notification(Notification::info("Info message".to_string()));

    assert_eq!(state.notifications.len(), 4);

    // Test notification types
    assert_eq!(
        state.notifications[0].notification_type,
        NotificationType::Success
    );
    assert_eq!(
        state.notifications[1].notification_type,
        NotificationType::Error
    );
    assert_eq!(
        state.notifications[2].notification_type,
        NotificationType::Warning
    );
    assert_eq!(
        state.notifications[3].notification_type,
        NotificationType::Info
    );

    // Test convenience methods
    state.add_success_notification("Another success".to_string());
    state.add_error_notification("Another error".to_string());

    assert_eq!(state.notifications.len(), 6);
}

#[test]
fn test_notification_limit() {
    let mut state = AppState::new();
    state.max_notifications = 3;

    // Add more notifications than limit
    for i in 0..5 {
        state.add_notification(Notification::info(format!("Message {i}")));
    }

    // Should only keep last 3
    assert_eq!(state.notifications.len(), 3);
    assert_eq!(state.notifications[0].message, "Message 2");
    assert_eq!(state.notifications[2].message, "Message 4");
}

#[test]
fn test_log_management() {
    let mut state = AppState::new();

    // Test adding logs
    state.add_log("INFO".to_string(), "Test info log".to_string());
    state.add_log("ERROR".to_string(), "Test error log".to_string());
    state.add_log("DEBUG".to_string(), "Test debug log".to_string());

    assert_eq!(state.device_logs.len(), 3);

    // Test log filtering
    state.log_filter_level = Some("ERROR".to_string());
    let filtered = state.get_filtered_logs();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].level, "ERROR");

    // Test clearing logs
    state.clear_logs();
    assert!(state.device_logs.is_empty());
}

#[test]
fn test_log_limit() {
    let mut state = AppState::new();
    state.max_log_entries = 5;

    // Add more logs than limit
    for i in 0..10 {
        state.add_log("INFO".to_string(), format!("Log entry {i}"));
    }

    // Should only keep last 5
    assert_eq!(state.device_logs.len(), 5);
    assert_eq!(state.device_logs[0].message, "Log entry 5");
    assert_eq!(state.device_logs[4].message, "Log entry 9");
}

#[test]
fn test_log_filtering() {
    let mut state = AppState::new();

    // Add various log levels
    state.add_log("DEBUG".to_string(), "Debug message".to_string());
    state.add_log("INFO".to_string(), "Info message".to_string());
    state.add_log("WARN".to_string(), "Warning message".to_string());
    state.add_log("ERROR".to_string(), "Error message".to_string());
    state.add_log("VERBOSE".to_string(), "Verbose message".to_string());

    // Test no filter
    state.log_filter_level = None;
    assert_eq!(state.get_filtered_logs().len(), 5);

    // Test ERROR filter
    state.log_filter_level = Some("ERROR".to_string());
    let filtered = state.get_filtered_logs();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].level, "ERROR");

    // Test WARN filter (currently implemented as exact match)
    state.log_filter_level = Some("WARN".to_string());
    let filtered = state.get_filtered_logs();
    assert_eq!(filtered.len(), 1); // Only WARN logs, not ERROR

    // Test INFO filter (currently implemented as exact match)
    state.log_filter_level = Some("INFO".to_string());
    let filtered = state.get_filtered_logs();
    assert_eq!(filtered.len(), 1); // Only INFO logs
}

#[test]
fn test_create_device_form() {
    let mut state = AppState::new();

    // Test initial form state
    assert_eq!(state.create_device_form.name, "");
    assert_eq!(state.create_device_form.ram_size, "2048");
    assert_eq!(state.create_device_form.storage_size, "8192");

    // Test form field navigation
    state.create_device_form.active_field = CreateDeviceField::Name;
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Name
    );

    state.create_device_form.active_field = CreateDeviceField::DeviceType;
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );
}

#[test]
fn test_device_operations_status() {
    let mut state = AppState::new();

    // Test setting operation status
    state.device_operation_status = Some("Starting device...".to_string());
    assert!(state.device_operation_status.is_some());
    assert_eq!(
        state.device_operation_status.as_ref().unwrap(),
        "Starting device..."
    );

    // Test clearing status
    state.device_operation_status = None;
    assert!(state.device_operation_status.is_none());
}

#[test]
fn test_refresh_timing() {
    let mut state = AppState::new();

    // The state uses Instant::now() at creation, so we need to reset it
    // to test the refresh timing properly
    state.mark_refreshed();

    // Should not refresh immediately after marking refreshed
    assert!(!state.should_auto_refresh());

    // Test pending device start (faster refresh)
    state.pending_device_start = Some("test_device".to_string());
    // With pending device start, should always refresh
    assert!(state.should_auto_refresh());
}

#[test]
fn test_scroll_offsets() {
    let mut state = create_state_with_devices();

    // Test Android scroll offset
    state.selected_android = 1;
    state.android_scroll_offset = 0;
    let offset = state.get_android_scroll_offset(10);
    assert_eq!(offset, 0); // With only 2 devices, no scrolling needed

    // Test iOS scroll offset
    state.selected_ios = 1;
    state.ios_scroll_offset = 0;
    let offset = state.get_ios_scroll_offset(10);
    assert_eq!(offset, 0);
}

#[test]
fn test_confirm_dialogs() {
    let mut state = AppState::new();

    // Test delete confirmation dialog
    state.confirm_delete_dialog = Some(ConfirmDeleteDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_id".to_string(),
        platform: Panel::Android,
    });

    assert!(state.confirm_delete_dialog.is_some());
    let dialog = state.confirm_delete_dialog.as_ref().unwrap();
    assert_eq!(dialog.device_name, "Test Device");
    assert_eq!(dialog.device_identifier, "test_id");
    assert_eq!(dialog.platform, Panel::Android);

    // Test wipe confirmation dialog
    state.confirm_wipe_dialog = Some(ConfirmWipeDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_id".to_string(),
        platform: Panel::Ios,
    });

    assert!(state.confirm_wipe_dialog.is_some());
    let dialog = state.confirm_wipe_dialog.as_ref().unwrap();
    assert_eq!(dialog.device_name, "Test Device");
    assert_eq!(dialog.platform, Panel::Ios);
}

#[test]
fn test_fullscreen_logs() {
    let mut state = AppState::new();

    // Test fullscreen toggle
    assert!(!state.fullscreen_logs);
    state.fullscreen_logs = true;
    assert!(state.fullscreen_logs);

    // Test auto scroll
    assert!(state.auto_scroll_logs);
    state.auto_scroll_logs = false;
    assert!(!state.auto_scroll_logs);

    // Test manual scroll flag
    assert!(!state.manually_scrolled);
    state.manually_scrolled = true;
    assert!(state.manually_scrolled);
}

#[test]
fn test_cached_device_details() {
    let mut state = AppState::new();

    // Test cache operations
    let details = DeviceDetails {
        platform: Panel::Android,
        name: "Test Device".to_string(),
        identifier: "test_id".to_string(),
        api_level_or_version: "API 34".to_string(),
        device_type: "Phone".to_string(),
        status: "Running".to_string(),
        resolution: Some("1080x1920".to_string()),
        dpi: Some("420dpi".to_string()),
        ram_size: Some("2048 MB".to_string()),
        storage_size: Some("8192 MB".to_string()),
        system_image: Some("android-34".to_string()),
        device_path: Some("/path/to/device".to_string()),
    };

    // Cache should initially be empty
    assert!(state.cached_device_details.is_none());

    // Add to cache
    state.cached_device_details = Some(details.clone());
    assert!(state.cached_device_details.is_some());

    // Retrieve from cache
    let cached = state.cached_device_details.as_ref();
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().name, "Test Device");
}

#[tokio::test]
async fn test_device_cache() {
    let cache = Arc::new(RwLock::new(DeviceCache::default()));

    // Test initial state
    {
        let cache_guard = cache.read().await;
        assert!(cache_guard.android_device_types.is_empty());
        assert!(cache_guard.ios_device_types.is_empty());
        assert!(!cache_guard.is_loading);
    }

    // Test staleness check
    {
        let cache_guard = cache.read().await;
        // New cache is not immediately stale (uses Instant::now())
        assert!(!cache_guard.is_stale());
    }

    // Test updating cache
    {
        let mut cache_guard = cache.write().await;
        cache_guard.update_android_cache(
            vec![
                ("pixel_7".to_string(), "Pixel 7".to_string()),
                ("tablet".to_string(), "Tablet".to_string()),
            ],
            vec![], // Empty API levels for this test
        );
        assert_eq!(cache_guard.android_device_types.len(), 2);
    }
}

#[test]
fn test_api_level_management() {
    let mut api_mgmt = ApiLevelManagementState::new();

    // Test initial state
    assert_eq!(api_mgmt.selected_index, 0);
    assert!(api_mgmt.api_levels.is_empty());
    assert!(api_mgmt.error_message.is_none());
    assert!(api_mgmt.is_loading); // ApiLevelManagementState starts in loading state

    // Add test API levels
    api_mgmt.api_levels = vec![
        ApiLevel {
            api: 34,
            version: "14".to_string(),
            display_name: "API 34 (Android 14)".to_string(),
            system_image_id: "system-images;android-34;google_apis;x86_64".to_string(),
            is_installed: true,
            variants: vec![],
        },
        ApiLevel {
            api: 33,
            version: "13".to_string(),
            display_name: "API 33 (Android 13)".to_string(),
            system_image_id: "system-images;android-33;google_apis;x86_64".to_string(),
            is_installed: false,
            variants: vec![],
        },
    ];

    // Test selection
    assert_eq!(api_mgmt.get_selected_api_level().unwrap().api, 34);

    // Test scroll offset
    let offset = api_mgmt.get_scroll_offset(10);
    assert_eq!(offset, 0); // With only 2 items, no scrolling needed
}

#[test]
fn test_notification_types() {
    // Test notification type creation
    let success = Notification::success("Success".to_string());
    assert_eq!(success.notification_type, NotificationType::Success);

    let error = Notification::error("Error".to_string());
    assert_eq!(error.notification_type, NotificationType::Error);

    let warning = Notification::warning("Warning".to_string());
    assert_eq!(warning.notification_type, NotificationType::Warning);

    let info = Notification::info("Info".to_string());
    assert_eq!(info.notification_type, NotificationType::Info);
}

#[test]
fn test_log_entry_creation() {
    use chrono::Local;
    let entry = LogEntry {
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        level: "ERROR".to_string(),
        message: "Test error message".to_string(),
    };

    assert_eq!(entry.level, "ERROR");
    assert_eq!(entry.message, "Test error message");
    assert!(!entry.timestamp.is_empty());
}

#[test]
fn test_create_device_form_default() {
    let form = CreateDeviceForm::default();

    // Check default state
    assert_eq!(form.name, "");
    assert_eq!(form.device_type, "");
    assert_eq!(form.ram_size, "2048");
    assert_eq!(form.storage_size, "8192");
    assert!(!form.is_creating);
    assert!(form.error_message.is_none());
}

#[test]
fn test_state_thread_safety() {
    use std::thread;

    let state = Arc::new(std::sync::Mutex::new(AppState::new()));
    let state_clone = Arc::clone(&state);

    // Spawn thread to modify state
    let handle = thread::spawn(move || {
        let mut state = state_clone.lock().unwrap();
        state.add_log("INFO".to_string(), "Thread log".to_string());
        state.selected_android = 5;
    });

    // Modify state in main thread
    {
        let mut state = state.lock().unwrap();
        state.add_log("ERROR".to_string(), "Main log".to_string());
        state.selected_ios = 3;
    }

    handle.join().unwrap();

    // Verify both modifications
    let state = state.lock().unwrap();
    assert_eq!(state.device_logs.len(), 2);
    assert_eq!(state.selected_android, 5);
    assert_eq!(state.selected_ios, 3);
}
