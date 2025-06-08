use emu::app::state::{AppState, Notification, NotificationType};
use std::time::Duration;

#[test]
fn test_notification_creation() {
    // Test different notification types
    let success = Notification::success("Operation completed".to_string());
    assert_eq!(success.notification_type, NotificationType::Success);
    assert_eq!(success.message, "Operation completed");
    assert!(success.auto_dismiss_after.is_some());

    let error = Notification::error("Something went wrong".to_string());
    assert_eq!(error.notification_type, NotificationType::Error);
    assert_eq!(error.message, "Something went wrong");

    let warning = Notification::warning("Be careful".to_string());
    assert_eq!(warning.notification_type, NotificationType::Warning);

    let info = Notification::info("Just so you know".to_string());
    assert_eq!(info.notification_type, NotificationType::Info);

    // Test persistent notification
    let persistent =
        Notification::persistent("Important message".to_string(), NotificationType::Warning);
    assert!(persistent.auto_dismiss_after.is_none());
    assert!(!persistent.should_dismiss());
}

#[test]
fn test_notification_auto_dismiss() {
    let mut notification = Notification::success("Test".to_string());

    // Should not dismiss immediately
    assert!(!notification.should_dismiss());

    // Simulate expired notification
    notification.auto_dismiss_after = Some(Duration::from_millis(1));
    std::thread::sleep(Duration::from_millis(10));
    assert!(notification.should_dismiss());
}

#[test]
fn test_app_state_notifications() {
    let mut state = AppState::new();

    // Test adding notifications
    state.add_success_notification("Success message".to_string());
    state.add_error_notification("Error message".to_string());
    state.add_warning_notification("Warning message".to_string());
    state.add_info_notification("Info message".to_string());

    assert_eq!(state.notifications.len(), 4);

    // Test notification ordering (newest last)
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

    // Test dismissing specific notification
    state.dismiss_notification(1); // Remove error notification
    assert_eq!(state.notifications.len(), 3);
    assert_eq!(
        state.notifications[1].notification_type,
        NotificationType::Warning
    );

    // Test dismissing all notifications
    state.dismiss_all_notifications();
    assert_eq!(state.notifications.len(), 0);
}

#[test]
fn test_notification_limit() {
    let mut state = AppState::new();
    state.max_notifications = 3; // Set lower limit for testing

    // Add more notifications than the limit
    for i in 0..5 {
        state.add_info_notification(format!("Message {}", i));
    }

    // Should only keep the most recent 3
    assert_eq!(state.notifications.len(), 3);
    assert_eq!(state.notifications[0].message, "Message 2");
    assert_eq!(state.notifications[1].message, "Message 3");
    assert_eq!(state.notifications[2].message, "Message 4");
}

#[test]
fn test_dismiss_expired_notifications() {
    let mut state = AppState::new();

    // Add notifications with different expiry times
    state.add_notification(Notification::success("Will expire soon".to_string()));
    state.add_notification(Notification::persistent(
        "Will not expire".to_string(),
        NotificationType::Info,
    ));

    // Manually set one to expire
    if let Some(notification) = state.notifications.get_mut(0) {
        notification.auto_dismiss_after = Some(Duration::from_millis(1));
    }

    std::thread::sleep(Duration::from_millis(10));

    // Before dismissal
    assert_eq!(state.notifications.len(), 2);

    // After dismissal
    state.dismiss_expired_notifications();
    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.notifications[0].message, "Will not expire");
}
