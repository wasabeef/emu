use super::*;

#[test]
fn test_panel_toggle() {
    assert_eq!(Panel::Android.toggle(), Panel::Ios);
    assert_eq!(Panel::Ios.toggle(), Panel::Android);
}

#[test]
fn test_api_level_management_state_new() {
    let state = ApiLevelManagementState::new();
    assert_eq!(state.selected_index, 0);
    assert!(state.api_levels.is_empty());
    assert!(state.is_loading);
    assert!(state.install_progress.is_none());
    assert!(state.installing_package.is_none());
    assert!(state.error_message.is_none());
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_notification_creation() {
    let notification = Notification {
        message: "Test message".to_string(),
        notification_type: NotificationType::Success,
        timestamp: chrono::Local::now(),
        auto_dismiss_after: Some(std::time::Duration::from_secs(5)),
    };
    assert_eq!(notification.message, "Test message");
    assert!(notification.auto_dismiss_after.is_some());

    let persistent = Notification {
        message: "Error".to_string(),
        notification_type: NotificationType::Error,
        timestamp: chrono::Local::now(),
        auto_dismiss_after: None,
    };
    assert!(persistent.auto_dismiss_after.is_none());
}

#[test]
fn test_device_cache_default_and_staleness() {
    let cache = DeviceCache::default();
    assert!(cache.android_device_types.is_empty());
    assert!(cache.ios_device_types.is_empty());
    assert!(!cache.is_loading);
    assert!(!cache.is_stale());

    let stale = DeviceCache {
        last_updated: std::time::Instant::now() - std::time::Duration::from_secs(301),
        ..Default::default()
    };
    assert!(stale.is_stale());
}

#[test]
fn test_device_cache_update_android() {
    let mut cache = DeviceCache {
        is_loading: true,
        ..Default::default()
    };
    let types = vec![("pixel_7".to_string(), "Pixel 7".to_string())];
    let levels = vec![("34".to_string(), "Android 14".to_string())];
    cache.update_android_cache(types.clone(), levels.clone());

    assert_eq!(cache.android_device_types, types);
    assert_eq!(cache.android_api_levels, levels);
    assert!(!cache.is_loading);
}

#[test]
fn test_device_cache_ios_data() {
    let cache = DeviceCache {
        ios_device_types: vec![("iPhone15,2".to_string(), "iPhone 15".to_string())],
        ios_runtimes: vec![("iOS-17-0".to_string(), "iOS 17.0".to_string())],
        ..Default::default()
    };
    assert_eq!(cache.ios_device_types.len(), 1);
    assert_eq!(cache.ios_runtimes.len(), 1);
}
