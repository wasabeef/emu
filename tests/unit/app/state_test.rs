//! Basic tests for app/state.rs
//!
//! Tests the basic functionality of application state management, enums, and struct operations.

use emu::app::state::{
    ConfirmDeleteDialog, ConfirmWipeDialog, CreateDeviceField, DeviceCache, FocusedPanel, Mode,
    Notification, NotificationType, Panel,
};
use std::time::Duration;

#[test]
fn test_panel_toggle() {
    assert_eq!(Panel::Android.toggle(), Panel::Ios);
    assert_eq!(Panel::Ios.toggle(), Panel::Android);
}

#[test]
fn test_panel_equality() {
    assert_eq!(Panel::Android, Panel::Android);
    assert_eq!(Panel::Ios, Panel::Ios);
    assert_ne!(Panel::Android, Panel::Ios);
}

#[test]
fn test_panel_debug() {
    let android_debug = format!("{:?}", Panel::Android);
    let ios_debug = format!("{:?}", Panel::Ios);

    assert!(android_debug.contains("Android"));
    assert!(ios_debug.contains("Ios"));
}

#[test]
fn test_focused_panel_variants() {
    let device_focus = FocusedPanel::DeviceList;
    let log_focus = FocusedPanel::LogArea;

    assert_eq!(device_focus, FocusedPanel::DeviceList);
    assert_eq!(log_focus, FocusedPanel::LogArea);
    assert_ne!(device_focus, log_focus);
}

#[test]
fn test_mode_variants() {
    let modes = vec![
        Mode::Normal,
        Mode::CreateDevice,
        Mode::ConfirmDelete,
        Mode::ConfirmWipe,
        Mode::ManageApiLevels,
        Mode::Help,
    ];

    // Verify that each mode can be compared for equality
    for mode in &modes {
        assert_eq!(*mode, *mode);
    }

    // Verify that different modes are not equal
    assert_ne!(Mode::Normal, Mode::CreateDevice);
    assert_ne!(Mode::ConfirmDelete, Mode::ConfirmWipe);
}

#[test]
fn test_notification_type_variants() {
    let types = vec![
        NotificationType::Success,
        NotificationType::Error,
        NotificationType::Warning,
        NotificationType::Info,
    ];

    for notification_type in &types {
        assert_eq!(*notification_type, *notification_type);
    }

    assert_ne!(NotificationType::Success, NotificationType::Error);
    assert_ne!(NotificationType::Warning, NotificationType::Info);
}

#[test]
fn test_notification_creation() {
    let notification = Notification {
        message: "Test message".to_string(),
        notification_type: NotificationType::Success,
        timestamp: chrono::Local::now(),
        auto_dismiss_after: Some(Duration::from_secs(5)),
    };

    assert_eq!(notification.message, "Test message");
    assert_eq!(notification.notification_type, NotificationType::Success);
    assert!(notification.auto_dismiss_after.is_some());
    assert_eq!(
        notification.auto_dismiss_after.unwrap(),
        Duration::from_secs(5)
    );
}

#[test]
fn test_notification_persistent() {
    let notification = Notification {
        message: "Persistent message".to_string(),
        notification_type: NotificationType::Error,
        timestamp: chrono::Local::now(),
        auto_dismiss_after: None,
    };

    assert_eq!(notification.message, "Persistent message");
    assert_eq!(notification.notification_type, NotificationType::Error);
    assert!(notification.auto_dismiss_after.is_none());
}

#[test]
fn test_create_device_field_variants() {
    let fields = vec![
        CreateDeviceField::ApiLevel,
        CreateDeviceField::Category,
        CreateDeviceField::DeviceType,
        CreateDeviceField::RamSize,
        CreateDeviceField::StorageSize,
        CreateDeviceField::Name,
    ];

    for field in &fields {
        assert_eq!(*field, *field);
    }

    // Verify that fields represent navigation order
    assert_ne!(CreateDeviceField::ApiLevel, CreateDeviceField::Name);
    assert_ne!(CreateDeviceField::Category, CreateDeviceField::DeviceType);
}

#[test]
fn test_device_cache_default() {
    let cache = DeviceCache::default();

    assert!(cache.android_device_types.is_empty());
    assert!(cache.android_api_levels.is_empty());
    assert!(cache.android_device_cache.is_none());
    assert!(cache.ios_device_types.is_empty());
    assert!(cache.ios_runtimes.is_empty());
    assert!(!cache.is_loading);

    // last_updated should be close to current time
    assert!(cache.last_updated.elapsed().as_secs() < 1);
}

#[test]
fn test_device_cache_is_stale() {
    let mut cache = DeviceCache::default();

    // New cache is not stale
    assert!(!cache.is_stale());

    // Simulate timestamp from more than 5 minutes ago
    cache.last_updated = std::time::Instant::now() - Duration::from_secs(301);
    assert!(cache.is_stale());
}

#[test]
fn test_device_cache_update_android() {
    let mut cache = DeviceCache {
        is_loading: true,
        ..Default::default()
    };

    let device_types = vec![
        ("pixel_7".to_string(), "Pixel 7".to_string()),
        ("pixel_8".to_string(), "Pixel 8".to_string()),
    ];

    let api_levels = vec![
        ("33".to_string(), "Android 13".to_string()),
        ("34".to_string(), "Android 14".to_string()),
    ];

    cache.update_android_cache(device_types.clone(), api_levels.clone());

    assert_eq!(cache.android_device_types, device_types);
    assert_eq!(cache.android_api_levels, api_levels);
    assert!(!cache.is_loading);
    assert!(cache.last_updated.elapsed().as_secs() < 1);
}

#[test]
fn test_confirm_delete_dialog() {
    let dialog = ConfirmDeleteDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_device".to_string(),
        platform: Panel::Android,
    };

    assert_eq!(dialog.device_name, "Test Device");
    assert_eq!(dialog.device_identifier, "test_device");
    assert_eq!(dialog.platform, Panel::Android);
}

#[test]
fn test_confirm_wipe_dialog() {
    let dialog = ConfirmWipeDialog {
        device_name: "Test iPhone".to_string(),
        device_identifier: "ABCD-1234".to_string(),
        platform: Panel::Ios,
    };

    assert_eq!(dialog.device_name, "Test iPhone");
    assert_eq!(dialog.device_identifier, "ABCD-1234");
    assert_eq!(dialog.platform, Panel::Ios);
}

#[test]
fn test_dialog_clone() {
    let delete_dialog = ConfirmDeleteDialog {
        device_name: "Original".to_string(),
        device_identifier: "original_id".to_string(),
        platform: Panel::Android,
    };

    let cloned_dialog = delete_dialog.clone();

    assert_eq!(delete_dialog.device_name, cloned_dialog.device_name);
    assert_eq!(
        delete_dialog.device_identifier,
        cloned_dialog.device_identifier
    );
    assert_eq!(delete_dialog.platform, cloned_dialog.platform);
}

#[test]
fn test_notification_clone() {
    let notification = Notification {
        message: "Clone test".to_string(),
        notification_type: NotificationType::Info,
        timestamp: chrono::Local::now(),
        auto_dismiss_after: Some(Duration::from_secs(3)),
    };

    let cloned = notification.clone();

    assert_eq!(notification.message, cloned.message);
    assert_eq!(notification.notification_type, cloned.notification_type);
    assert_eq!(notification.auto_dismiss_after, cloned.auto_dismiss_after);
}

#[test]
fn test_device_cache_clone() {
    let mut cache = DeviceCache::default();
    cache
        .android_device_types
        .push(("test".to_string(), "Test".to_string()));
    cache.is_loading = true;

    let cloned = cache.clone();

    assert_eq!(
        cache.android_device_types.len(),
        cloned.android_device_types.len()
    );
    assert_eq!(cache.is_loading, cloned.is_loading);
}

#[test]
fn test_enum_debug_formatting() {
    // Verify that debug formatting works correctly
    let panel_debug = format!("{:?}", Panel::Android);
    let mode_debug = format!("{:?}", Mode::CreateDevice);
    let field_debug = format!("{:?}", CreateDeviceField::Name);
    let type_debug = format!("{:?}", NotificationType::Success);

    assert!(panel_debug.contains("Android"));
    assert!(mode_debug.contains("CreateDevice"));
    assert!(field_debug.contains("Name"));
    assert!(type_debug.contains("Success"));
}

#[test]
fn test_struct_debug_formatting() {
    let dialog = ConfirmDeleteDialog {
        device_name: "Debug Test".to_string(),
        device_identifier: "debug_test".to_string(),
        platform: Panel::Ios,
    };

    let debug_str = format!("{dialog:?}");

    assert!(debug_str.contains("Debug Test"));
    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("Ios"));
}

#[test]
fn test_notification_timestamp_ordering() {
    let timestamp1 = chrono::Local::now();
    std::thread::sleep(Duration::from_millis(1));
    let timestamp2 = chrono::Local::now();

    let notification1 = Notification {
        message: "First".to_string(),
        notification_type: NotificationType::Info,
        timestamp: timestamp1,
        auto_dismiss_after: None,
    };

    let notification2 = Notification {
        message: "Second".to_string(),
        notification_type: NotificationType::Info,
        timestamp: timestamp2,
        auto_dismiss_after: None,
    };

    assert!(notification1.timestamp < notification2.timestamp);
}

#[test]
fn test_device_cache_ios_update() {
    let cache = DeviceCache {
        ios_device_types: vec![
            ("iPhone15,2".to_string(), "iPhone 15".to_string()),
            ("iPhone15,3".to_string(), "iPhone 15 Pro".to_string()),
        ],
        ios_runtimes: vec![
            ("iOS-17-0".to_string(), "iOS 17.0".to_string()),
            ("iOS-17-1".to_string(), "iOS 17.1".to_string()),
        ],
        ..Default::default()
    };

    assert_eq!(cache.ios_device_types.len(), 2);
    assert_eq!(cache.ios_runtimes.len(), 2);
    assert_eq!(cache.ios_device_types[0].1, "iPhone 15");
    assert_eq!(cache.ios_runtimes[0].1, "iOS 17.0");
}
