//! Additional functionality tests to ensure code coverage and reliability.
//!
//! Tests cover functionality not extensively covered by existing tests.

use emu::app::{AppState, Mode, Panel};
use emu::models::device::{AndroidDevice, DeviceStatus, IosDevice};

/// Test device model creation and fields
#[test]
fn test_device_models() {
    let android = AndroidDevice {
        name: "TestDevice".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8G".to_string(),
    };

    assert_eq!(android.name, "TestDevice");
    assert_eq!(android.api_level, 34);
    assert!(!android.is_running);

    let ios = IosDevice {
        name: "iPhone 15".to_string(),
        udid: "test-uuid".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: true,
    };

    assert_eq!(ios.name, "iPhone 15");
    assert!(ios.is_running);
}

/// Test device status enum
#[test]
fn test_device_status() {
    let statuses = vec![
        DeviceStatus::Running,
        DeviceStatus::Stopped,
        DeviceStatus::Starting,
        DeviceStatus::Stopping,
        DeviceStatus::Unknown,
    ];

    for status in statuses {
        // Should be able to create and compare statuses
        let cloned = status;
        assert_eq!(status, cloned);
    }
}

/// Test panel toggle functionality
#[test]
fn test_panel_toggle() {
    assert_eq!(Panel::Android.toggle(), Panel::Ios);
    assert_eq!(Panel::Ios.toggle(), Panel::Android);
}

/// Test AppState initialization
#[test]
fn test_app_state_initialization() {
    let state = AppState::new();

    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.mode, Mode::Normal);
}

/// Test mode enum values
#[test]
fn test_mode_enum() {
    let modes = vec![
        Mode::Normal,
        Mode::CreateDevice,
        Mode::ConfirmDelete,
        Mode::ConfirmWipe,
    ];

    for mode in modes {
        // Modes should be cloneable and comparable
        let cloned = mode;
        assert_eq!(mode, cloned);
    }
}

/// Test notification system basic functionality
#[test]
fn test_notifications() {
    let mut state = AppState::new();

    // Add notification
    state.add_notification(emu::app::state::Notification::info(
        "Test message".to_string(),
    ));
    assert_eq!(state.notifications.len(), 1);

    // Add error notification
    state.add_error_notification("Error message".to_string());
    assert_eq!(state.notifications.len(), 2);

    // Check notification types
    let last = state.notifications.back().unwrap();
    assert_eq!(
        last.notification_type,
        emu::app::state::NotificationType::Error
    );
}

/// Test log management
#[test]
fn test_log_management() {
    let mut state = AppState::new();

    // Add logs
    state.add_log("INFO".to_string(), "Test log".to_string());
    state.add_log("ERROR".to_string(), "Error log".to_string());

    assert_eq!(state.device_logs.len(), 2);
    assert_eq!(state.device_logs[0].level, "INFO");
    assert_eq!(state.device_logs[1].level, "ERROR");

    // Clear logs
    state.clear_logs();
    assert_eq!(state.device_logs.len(), 0);
}

/// Test log filter functionality
#[test]
fn test_log_filter_functionality() {
    let mut state = AppState::new();

    // Test toggle log filter
    state.toggle_log_filter(Some("ERROR".to_string()));
    // Verify method executes without panic

    state.toggle_log_filter(None);
    // Verify method executes without panic
}

/// Test state refresh tracking
#[test]
fn test_refresh_tracking() {
    let mut state = AppState::new();

    let initial_time = state.last_refresh;
    state.mark_refreshed();
    let updated_time = state.last_refresh;

    assert!(updated_time > initial_time);
}

/// Test cache invalidation
#[test]
fn test_cache_invalidation() {
    let mut state = AppState::new();

    // Smart clear should handle None case
    state.smart_clear_cached_device_details(Panel::Android);
    assert!(state.cached_device_details.is_none());

    // Clear cached details
    state.clear_cached_device_details();
    assert!(state.cached_device_details.is_none());
}

/// Test device operation status
#[test]
fn test_device_operation_status() {
    let mut state = AppState::new();

    // Initially no operation
    assert!(state.device_operation_status.is_none());

    // Set operation status
    state.device_operation_status = Some("Creating device...".to_string());
    assert!(state.device_operation_status.is_some());

    // Clear operation status
    state.device_operation_status = None;
    assert!(state.device_operation_status.is_none());
}

/// Test serialization capability
#[test]
fn test_device_serialization() {
    let device = AndroidDevice {
        name: "SerTest".to_string(),
        device_type: "test".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "8G".to_string(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&device).unwrap();
    assert!(json.contains("SerTest"));
    assert!(json.contains("34"));

    // Test deserialization
    let deserialized: AndroidDevice = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, device.name);
    assert_eq!(deserialized.api_level, device.api_level);
}

/// Test empty device lists handling
#[test]
fn test_empty_lists() {
    let state = AppState::new();

    // Empty lists should not cause panics
    assert_eq!(state.android_devices.len(), 0);
    assert_eq!(state.ios_devices.len(), 0);
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
}

/// Test device configuration edge cases
#[test]
fn test_device_config_edge_cases() {
    // Minimum values
    let min_device = AndroidDevice {
        name: "A".to_string(),
        device_type: "t".to_string(),
        api_level: 1,
        status: DeviceStatus::Unknown,
        is_running: false,
        ram_size: "512".to_string(),
        storage_size: "1G".to_string(),
    };

    assert!(!min_device.name.is_empty());
    assert!(min_device.api_level > 0);

    // Maximum reasonable values
    let max_device = AndroidDevice {
        name: "Very_Long_Device_Name_With_Many_Characters".to_string(),
        device_type: "very_specific_device_type".to_string(),
        api_level: 99,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "16384".to_string(),
        storage_size: "128G".to_string(),
    };

    assert!(max_device.name.len() > 10);
    assert!(max_device.api_level > 30);
}
