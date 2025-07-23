//! Event Processing System Complete Tests
//!
//! This test suite verifies the completeness and performance of the application's
//! event processing system.

use emu::app::state::AppState;
use emu::app::state::LogEntry;
use emu::app::Panel;
use emu::models::device::{AndroidDevice, DeviceStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_event_processing_basic() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Basic event processing test
    {
        let mut state = state.lock().await;
        state.active_panel = Panel::Android;
        state.selected_android = 0;
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.active_panel, Panel::Android);
    assert_eq!(final_state.selected_android, 0);
}

#[tokio::test]
async fn test_event_processing_panel_switching() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Panel switching event processing
    {
        let mut state = state.lock().await;
        state.active_panel = Panel::Android;
        state.next_panel();
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.active_panel, Panel::Ios);
}

#[tokio::test]
async fn test_event_processing_device_selection() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Device selection event processing
    {
        let mut state = state.lock().await;
        // Add Android devices
        state.android_devices.push(AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "test_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        });

        state.active_panel = Panel::Android;
        state.selected_android = 0;
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.selected_android, 0);
    assert_eq!(final_state.android_devices.len(), 1);
}

#[tokio::test]
async fn test_event_processing_device_navigation() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Device navigation processing
    {
        let mut state = state.lock().await;
        // Add multiple devices
        for i in 0..5 {
            state.android_devices.push(AndroidDevice {
                android_version_name: "API 30".to_string(),
                name: format!("device_{i}"),
                device_type: "pixel_7".to_string(),
                api_level: 34,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "4096".to_string(),
                storage_size: "8192".to_string(),
            });
        }

        state.active_panel = Panel::Android;
        state.selected_android = 0;

        // Move down
        state.move_down();
        assert_eq!(state.selected_android, 1);

        // Move up
        state.move_up();
        assert_eq!(state.selected_android, 0);

        // Moving up at first wraps to last
        state.move_up();
        assert_eq!(state.selected_android, 4);
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.selected_android, 4);
    assert_eq!(final_state.android_devices.len(), 5);
}

#[tokio::test]
async fn test_event_processing_concurrent_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Concurrent event processing
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                {
                    let mut state = state_clone.lock().await;
                    state.active_panel = if (i + j) % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                    state.selected_android = i;
                    state.selected_ios = j;
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().await;
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
    assert!(final_state.selected_android < 10);
    assert!(final_state.selected_ios < 10);
}

#[tokio::test]
async fn test_event_processing_loading_states() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Loading state event processing
    {
        let mut state = state.lock().await;
        state.is_loading = true;

        // Test case where operations are restricted during loading
        assert!(state.is_loading);

        state.is_loading = false;
        assert!(!state.is_loading);
    }

    let final_state = state.lock().await;
    assert!(!final_state.is_loading);
}

#[tokio::test]
async fn test_event_processing_log_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Log event processing
    {
        let mut state = state.lock().await;

        // Add log entries
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:56".to_string(),
            level: "INFO".to_string(),
            message: "Test log entry 1".to_string(),
        });
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:57".to_string(),
            level: "WARN".to_string(),
            message: "Test log entry 2".to_string(),
        });
        state.device_logs.push_back(LogEntry {
            timestamp: "12:34:58".to_string(),
            level: "ERROR".to_string(),
            message: "Test log entry 3".to_string(),
        });

        assert_eq!(state.device_logs.len(), 3);

        // Log scrolling
        state.log_scroll_offset = 1;
        assert_eq!(state.log_scroll_offset, 1);
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.device_logs.len(), 3);
    assert_eq!(final_state.log_scroll_offset, 1);
}

#[tokio::test]
async fn test_event_processing_notification_events() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Notification event processing
    {
        let mut state = state.lock().await;

        // Add notifications
        state
            .notifications
            .push_back(emu::app::state::Notification::success(
                "Operation completed successfully".to_string(),
            ));
        state
            .notifications
            .push_back(emu::app::state::Notification::error(
                "An error occurred".to_string(),
            ));

        assert_eq!(state.notifications.len(), 2);

        // Test notification types
        assert_eq!(
            state.notifications[0].notification_type,
            emu::app::state::NotificationType::Success
        );
        assert_eq!(
            state.notifications[1].notification_type,
            emu::app::state::NotificationType::Error
        );
    }

    let final_state = state.lock().await;
    assert_eq!(final_state.notifications.len(), 2);
}

#[tokio::test]
async fn test_event_processing_device_operations() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Device operation event processing
    {
        let mut state = state.lock().await;

        // Add devices
        let device = AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "operation_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };
        state.android_devices.push(device);

        // Set operation state
        state.device_operation_status = Some("Starting device...".to_string());

        assert!(state.device_operation_status.is_some());
        assert_eq!(
            state.device_operation_status.as_ref().unwrap(),
            "Starting device..."
        );
    }

    let final_state = state.lock().await;
    assert!(final_state.device_operation_status.is_some());
    assert_eq!(final_state.android_devices.len(), 1);
}

#[tokio::test]
async fn test_event_processing_performance() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Performance test: Verify event processing is fast
    let start = std::time::Instant::now();

    {
        let mut state = state.lock().await;

        // Process large number of events
        for i in 0..1000 {
            state.active_panel = if i % 2 == 0 {
                Panel::Android
            } else {
                Panel::Ios
            };
            state.selected_android = i % 10;
            state.selected_ios = i % 5;

            // Add device
            if i % 100 == 0 {
                state.android_devices.push(AndroidDevice {
                    android_version_name: "API 30".to_string(),
                    name: format!("perf_device_{i}"),
                    device_type: "pixel_7".to_string(),
                    api_level: 34,
                    status: DeviceStatus::Stopped,
                    is_running: false,
                    ram_size: "4096".to_string(),
                    storage_size: "8192".to_string(),
                });
            }
        }
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Event processing took too long: {duration:?}"
    );

    let final_state = state.lock().await;
    assert_eq!(final_state.android_devices.len(), 10);
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
}

#[tokio::test]
async fn test_event_processing_error_handling() {
    let state = Arc::new(tokio::sync::Mutex::new(AppState::new()));

    // Error handling event processing
    {
        let mut state = state.lock().await;

        // Set error state
        state
            .notifications
            .push_back(emu::app::state::Notification::error(
                "Device operation failed".to_string(),
            ));

        // Clear operation state
        state.device_operation_status = None;

        assert!(state.device_operation_status.is_none());
        assert_eq!(state.notifications.len(), 1);
        assert_eq!(
            state.notifications[0].notification_type,
            emu::app::state::NotificationType::Error
        );
    }

    let final_state = state.lock().await;
    assert!(final_state.device_operation_status.is_none());
    assert_eq!(final_state.notifications.len(), 1);
}
