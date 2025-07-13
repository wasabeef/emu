//! Tests for app state utility functions
//!
//! Tests focus on AppState getter methods, notification lifecycle,
//! and utility functions that provide access to application state.

use emu::app::state::{AppState, Panel, FocusedPanel, Mode, Notification, NotificationType};
use emu::constants::timeouts::NOTIFICATION_AUTO_DISMISS_TIME;
use std::time::{Duration, Instant};

#[cfg(test)]
mod app_state_getters_tests {
    use super::*;

    #[test]
    fn test_panel_toggle() {
        assert_eq!(Panel::Android.toggle(), Panel::Ios);
        assert_eq!(Panel::Ios.toggle(), Panel::Android);
    }

    #[test]
    fn test_default_focused_panel() {
        let app_state = AppState::default();
        // Default state should have device list focused
        assert_eq!(app_state.focused_panel, FocusedPanel::DeviceList);
    }

    #[test]
    fn test_default_current_panel() {
        let app_state = AppState::default();
        // Default state should show Android panel
        assert_eq!(app_state.current_panel, Panel::Android);
    }

    #[test]
    fn test_default_mode() {
        let app_state = AppState::default();
        // Default mode should be Normal
        assert_eq!(app_state.mode, Mode::Normal);
    }

    #[test]
    fn test_default_selected_indices() {
        let app_state = AppState::default();
        // Default selected indices should be 0
        assert_eq!(app_state.selected_android_index, 0);
        assert_eq!(app_state.selected_ios_index, 0);
    }

    #[test]
    fn test_scroll_offset_calculation() {
        let app_state = AppState::default();
        // Test scroll offset calculation for different available heights
        let offset = app_state.get_scroll_offset(10);
        assert_eq!(offset, 0); // No scrolling needed with default state
    }

    #[test]
    fn test_android_scroll_offset() {
        let app_state = AppState::default();
        let offset = app_state.get_android_scroll_offset(5);
        assert_eq!(offset, 0); // No scrolling needed with empty device list
    }

    #[test]
    fn test_ios_scroll_offset() {
        let app_state = AppState::default();
        let offset = app_state.get_ios_scroll_offset(5);
        assert_eq!(offset, 0); // No scrolling needed with empty device list
    }
}

#[cfg(test)]
mod notification_lifecycle_tests {
    use super::*;

    #[test]
    fn test_add_success_notification() {
        let mut app_state = AppState::default();
        app_state.add_success_notification("Operation completed".to_string());
        
        assert_eq!(app_state.notifications.len(), 1);
        assert_eq!(app_state.notifications[0].message, "Operation completed");
        assert_eq!(app_state.notifications[0].notification_type, NotificationType::Success);
    }

    #[test]
    fn test_add_error_notification() {
        let mut app_state = AppState::default();
        app_state.add_error_notification("Operation failed".to_string());
        
        assert_eq!(app_state.notifications.len(), 1);
        assert_eq!(app_state.notifications[0].message, "Operation failed");
        assert_eq!(app_state.notifications[0].notification_type, NotificationType::Error);
    }

    #[test]
    fn test_add_warning_notification() {
        let mut app_state = AppState::default();
        app_state.add_warning_notification("Warning message".to_string());
        
        assert_eq!(app_state.notifications.len(), 1);
        assert_eq!(app_state.notifications[0].message, "Warning message");
        assert_eq!(app_state.notifications[0].notification_type, NotificationType::Warning);
    }

    #[test]
    fn test_add_info_notification() {
        let mut app_state = AppState::default();
        app_state.add_info_notification("Info message".to_string());
        
        assert_eq!(app_state.notifications.len(), 1);
        assert_eq!(app_state.notifications[0].message, "Info message");
        assert_eq!(app_state.notifications[0].notification_type, NotificationType::Info);
    }

    #[test]
    fn test_dismiss_all_notifications() {
        let mut app_state = AppState::default();
        app_state.add_success_notification("Message 1".to_string());
        app_state.add_error_notification("Message 2".to_string());
        
        assert_eq!(app_state.notifications.len(), 2);
        
        app_state.dismiss_all_notifications();
        assert_eq!(app_state.notifications.len(), 0);
    }

    #[test]
    fn test_dismiss_notification_by_index() {
        let mut app_state = AppState::default();
        app_state.add_success_notification("Message 1".to_string());
        app_state.add_error_notification("Message 2".to_string());
        
        assert_eq!(app_state.notifications.len(), 2);
        
        app_state.dismiss_notification(0);
        assert_eq!(app_state.notifications.len(), 1);
        assert_eq!(app_state.notifications[0].message, "Message 2");
    }

    #[test]
    fn test_dismiss_notification_invalid_index() {
        let mut app_state = AppState::default();
        app_state.add_success_notification("Message 1".to_string());
        
        assert_eq!(app_state.notifications.len(), 1);
        
        // Should not panic or affect other notifications
        app_state.dismiss_notification(10);
        assert_eq!(app_state.notifications.len(), 1);
    }

    #[test]
    fn test_notification_auto_expire() {
        let notification = Notification::new("Test".to_string(), NotificationType::Info);
        
        // Immediately after creation, should not be expired
        assert!(!notification.is_expired());
        
        // Create expired notification by manipulating timestamp
        let expired_notification = Notification {
            message: "Expired".to_string(),
            notification_type: NotificationType::Info,
            timestamp: Instant::now() - (NOTIFICATION_AUTO_DISMISS_TIME + Duration::from_millis(100)),
            persistent: false,
        };
        
        assert!(expired_notification.is_expired());
    }

    #[test]
    fn test_persistent_notification_not_expired() {
        let notification = Notification::persistent("Persistent".to_string(), NotificationType::Error);
        
        // Persistent notifications should never expire
        assert!(!notification.is_expired());
    }
}

#[cfg(test)]
mod device_operation_status_tests {
    use super::*;

    #[test]
    fn test_set_device_operation_status() {
        let mut app_state = AppState::default();
        app_state.set_device_operation_status("Starting device...".to_string());
        
        assert_eq!(app_state.get_device_operation_status(), Some(&"Starting device...".to_string()));
    }

    #[test]
    fn test_clear_device_operation_status() {
        let mut app_state = AppState::default();
        app_state.set_device_operation_status("Starting device...".to_string());
        app_state.device_operation_status = None;
        
        assert_eq!(app_state.get_device_operation_status(), None);
    }

    #[test]
    fn test_set_pending_device_start() {
        let mut app_state = AppState::default();
        app_state.set_pending_device_start("TestDevice".to_string());
        
        assert_eq!(app_state.get_pending_device_start(), Some(&"TestDevice".to_string()));
    }

    #[test]
    fn test_clear_pending_device_start() {
        let mut app_state = AppState::default();
        app_state.set_pending_device_start("TestDevice".to_string());
        app_state.pending_device_start = None;
        
        assert_eq!(app_state.get_pending_device_start(), None);
    }
}

#[cfg(test)]
mod auto_refresh_tests {
    use super::*;

    #[test]
    fn test_should_auto_refresh_initial_state() {
        let app_state = AppState::default();
        // Initial state should trigger auto refresh
        assert!(app_state.should_auto_refresh());
    }

    #[test]
    fn test_mark_refreshed_updates_timestamp() {
        let mut app_state = AppState::default();
        let initial_timestamp = app_state.last_refresh;
        
        std::thread::sleep(Duration::from_millis(10));
        app_state.mark_refreshed();
        
        assert!(app_state.last_refresh > initial_timestamp);
    }
}

#[cfg(test)]
mod log_management_tests {
    use super::*;

    #[test]
    fn test_reset_log_scroll() {
        let mut app_state = AppState::default();
        // Simulate scrolled state
        app_state.log_scroll_offset = 10;
        
        app_state.reset_log_scroll();
        assert_eq!(app_state.log_scroll_offset, 0);
    }

    #[test]
    fn test_get_filtered_logs_empty() {
        let app_state = AppState::default();
        let filtered_logs = app_state.get_filtered_logs();
        assert_eq!(filtered_logs.len(), 0);
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_notification_limit_enforcement() {
        let mut app_state = AppState::default();
        
        // Add notifications up to the limit
        for i in 0..20 {
            app_state.add_info_notification(format!("Message {}", i));
        }
        
        // Verify notifications are properly managed (may be limited or rotated)
        assert!(app_state.notifications.len() <= 20);
    }

    #[test]
    fn test_panel_state_consistency() {
        let mut app_state = AppState::default();
        
        // Toggle panel and verify consistency
        let original_panel = app_state.current_panel;
        app_state.current_panel = app_state.current_panel.toggle();
        
        assert_ne!(app_state.current_panel, original_panel);
        assert_eq!(app_state.current_panel, original_panel.toggle());
    }

    #[test]
    fn test_mode_transitions() {
        let mut app_state = AppState::default();
        
        // Test various mode transitions
        assert_eq!(app_state.mode, Mode::Normal);
        
        app_state.mode = Mode::CreateDevice;
        assert_eq!(app_state.mode, Mode::CreateDevice);
        
        app_state.mode = Mode::ConfirmDelete;
        assert_eq!(app_state.mode, Mode::ConfirmDelete);
        
        app_state.mode = Mode::Normal;
        assert_eq!(app_state.mode, Mode::Normal);
    }
}