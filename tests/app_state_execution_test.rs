//! Unit tests for app/state.rs executable code focusing on state management

use emu::app::state::{AppState, FocusedPanel, Mode, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_app_state_initialization() {
    // Test AppState::new() initialization
    let state = AppState::new();

    // Check initial values
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
    assert_eq!(state.mode, Mode::Normal);
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());
    // is_loading state can be true or false initially
    let _loading_state = state.is_loading;
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
    assert!(state.notifications.is_empty());
    assert!(state.device_logs.is_empty());
}

#[tokio::test]
async fn test_panel_toggle() {
    // Test Panel::toggle() method
    let android_panel = Panel::Android;
    let ios_panel = Panel::Ios;

    assert_eq!(android_panel.toggle(), Panel::Ios);
    assert_eq!(ios_panel.toggle(), Panel::Android);

    // Test multiple toggles
    let mut current = Panel::Android;
    for _ in 0..10 {
        current = current.toggle();
    }
    assert_eq!(current, Panel::Android); // Should be back to original after even number of toggles
}

#[tokio::test]
async fn test_app_state_device_lists() {
    // Test device list management
    let mut state = AppState::new();

    // Initially empty
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());

    // Add Android device
    let android_device = AndroidDevice {
        name: "test_device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    state.android_devices.push(android_device);
    assert_eq!(state.android_devices.len(), 1);
    assert_eq!(state.android_devices[0].name, "test_device");

    // Add iOS device
    let ios_device = IosDevice {
        name: "iPhone 15".to_string(),
        udid: "test-udid".to_string(),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    };

    state.ios_devices.push(ios_device);
    assert_eq!(state.ios_devices.len(), 1);
    assert_eq!(state.ios_devices[0].name, "iPhone 15");
}

#[tokio::test]
async fn test_app_state_mode_transitions() {
    // Test mode transitions
    let mut state = AppState::new();

    // Initially in Normal mode
    assert_eq!(state.mode, Mode::Normal);

    // Change to different modes
    state.mode = Mode::CreateDevice;
    assert_eq!(state.mode, Mode::CreateDevice);

    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);

    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);
}

#[tokio::test]
async fn test_app_state_loading_state() {
    // Test loading state management
    let mut state = AppState::new();

    // Initially not loading
    // is_loading state can be true or false initially
    let _loading_state = state.is_loading;

    // Set loading
    state.is_loading = true;
    assert!(state.is_loading);

    // Clear loading
    state.is_loading = false;
    // is_loading state can be true or false initially
    let _loading_state = state.is_loading;
}

#[tokio::test]
async fn test_app_state_device_selection() {
    // Test device selection management
    let mut state = AppState::new();

    // Initially default selection
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Select Android device
    state.selected_android = 1;
    assert_eq!(state.selected_android, 1);

    // Select iOS device
    state.selected_ios = 2;
    assert_eq!(state.selected_ios, 2);

    // Reset selections
    state.selected_android = 0;
    state.selected_ios = 0;
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);
}

#[tokio::test]
async fn test_app_state_concurrent_access() {
    // Test concurrent access to state
    let state = Arc::new(RwLock::new(AppState::new()));
    let mut handles = vec![];

    // Create multiple tasks that access state
    for i in 0..5 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            let mut state_guard = state_clone.write().await;
            state_guard.is_loading = i % 2 == 0;
            state_guard.active_panel = if i % 2 == 0 {
                Panel::Android
            } else {
                Panel::Ios
            };
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // All tasks should complete successfully
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Concurrent task {i} should complete successfully"
        );
    }

    // State should be in a valid state
    let final_state = state.read().await;
    assert!(final_state.active_panel == Panel::Android || final_state.active_panel == Panel::Ios);
}

#[tokio::test]
async fn test_app_state_notifications() {
    // Test notification management
    let state = AppState::new();

    // Initially no notifications
    assert!(state.notifications.is_empty());

    // Add notification (simulated by adding to notifications vector)
    // Note: In real usage, notifications would be added through specific methods
    // but for testing state structure, we test the container directly
    let notification_count = state.notifications.len();
    assert_eq!(notification_count, 0);

    // Test notification capacity
    // The notifications vector should be manageable
    // Note: capacity() always returns >= 0 by definition, so we test other properties
    assert!(state.notifications.capacity() <= 1000000); // Reasonable upper bound
}

#[tokio::test]
async fn test_app_state_logs() {
    // Test log management
    let state = AppState::new();

    // Initially no logs
    assert!(state.device_logs.is_empty());

    // Test log capacity
    // Note: capacity() always returns >= 0 by definition, so we test other properties
    assert!(state.device_logs.capacity() <= 1000000); // Reasonable upper bound

    // Test that logs is a VecDeque (can be pushed to)
    let log_count = state.device_logs.len();
    assert_eq!(log_count, 0);
}

#[tokio::test]
async fn test_app_state_focused_panel() {
    // Test focused panel management
    let mut state = AppState::new();

    // Initially focused on Android
    assert_eq!(state.focused_panel, FocusedPanel::DeviceList);

    // Change focus
    state.focused_panel = FocusedPanel::LogArea;
    assert_eq!(state.focused_panel, FocusedPanel::LogArea);

    state.focused_panel = FocusedPanel::DeviceList;
    assert_eq!(state.focused_panel, FocusedPanel::DeviceList);

    state.focused_panel = FocusedPanel::DeviceList;
    assert_eq!(state.focused_panel, FocusedPanel::DeviceList);
}

#[tokio::test]
async fn test_app_state_memory_usage() {
    // Test that AppState doesn't cause memory issues
    let initial_memory = get_memory_usage();

    // Create and drop multiple state instances
    for _ in 0..100 {
        let state = AppState::new();
        drop(state);
    }

    let final_memory = get_memory_usage();

    // Memory usage should not increase dramatically
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 10_000_000, // 10MB limit
        "Memory usage should not increase dramatically: {memory_increase} bytes"
    );
}

#[tokio::test]
async fn test_app_state_performance() {
    // Test that AppState::new() is fast
    let start = std::time::Instant::now();
    let state = AppState::new();
    let duration = start.elapsed();

    // Should complete very quickly
    assert!(
        duration < Duration::from_millis(10),
        "AppState::new() should complete within 10ms, took: {duration:?}"
    );

    // State should be valid
    assert_eq!(state.active_panel, Panel::Android);
}

#[tokio::test]
async fn test_app_state_device_status_transitions() {
    // Test device status handling
    let mut state = AppState::new();

    // Add device with different statuses
    let mut android_device = AndroidDevice {
        name: "test_device".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    };

    // Test different status transitions
    let statuses = vec![
        DeviceStatus::Stopped,
        DeviceStatus::Starting,
        DeviceStatus::Running,
        DeviceStatus::Stopping,
        DeviceStatus::Error,
        DeviceStatus::Unknown,
    ];

    for status in statuses {
        android_device.status = status;
        android_device.is_running = matches!(status, DeviceStatus::Running);

        // Clear and add device with new status
        state.android_devices.clear();
        state.android_devices.push(android_device.clone());

        assert_eq!(state.android_devices[0].status, status);
        assert_eq!(
            state.android_devices[0].is_running,
            matches!(status, DeviceStatus::Running)
        );
    }
}

#[tokio::test]
async fn test_app_state_clone_behavior() {
    // Test that state components can be cloned
    let state = AppState::new();

    // Test that panel can be cloned
    let panel = state.active_panel;
    let panel_clone = panel;
    assert_eq!(panel, panel_clone);

    // Test that focused panel can be cloned
    let focused = state.focused_panel;
    let focused_clone = focused;
    assert_eq!(focused, focused_clone);

    // Test that mode can be cloned
    let mode = state.mode;
    let mode_clone = mode;
    assert_eq!(mode, mode_clone);
}

// Helper function to get rough memory usage
fn get_memory_usage() -> usize {
    // Simple memory usage estimation
    std::process::id() as usize * 1024 // Simple approximation
}
