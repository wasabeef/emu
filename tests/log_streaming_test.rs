use emu::app::state::{AppState, Panel};
use emu::models::{AndroidDevice, DeviceStatus};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Test log streaming task management and device selection changes
#[tokio::test]
async fn test_log_streaming_device_selection_change() {
    println!("=== LOG STREAMING DEVICE SELECTION TEST ===");

    let mut state = AppState::new();

    // Setup test devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Device_A".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Device_B".to_string(),
            device_type: "phone".to_string(),
            api_level: 32,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    // Test 1: Initial state - no log device
    assert!(state.current_log_device.is_none());
    assert!(state.log_task_handle.is_none());
    assert_eq!(state.selected_android, 0);

    // Test 2: Simulate starting log stream for first device
    state.current_log_device = Some((Panel::Android, "Device_A".to_string()));

    // Verify current log device
    assert!(state.current_log_device.is_some());
    if let Some((panel, device_name)) = &state.current_log_device {
        assert_eq!(*panel, Panel::Android);
        assert_eq!(device_name, "Device_A");
    }

    // Test 3: Simulate device selection change (move down)
    state.move_down();
    assert_eq!(state.selected_android, 1);

    // Test 4: Simulate clearing logs and device when selection changes
    state.clear_logs();
    state.current_log_device = None;

    // Verify logs are cleared and device is reset
    assert!(state.device_logs.is_empty());
    assert!(state.current_log_device.is_none());

    // Test 5: Simulate starting new log stream for second device
    state.current_log_device = Some((Panel::Android, "Device_B".to_string()));

    // Verify new log device
    if let Some((panel, device_name)) = &state.current_log_device {
        assert_eq!(*panel, Panel::Android);
        assert_eq!(device_name, "Device_B");
    }

    println!("✓ Log streaming device selection change test passed");
}

/// Test log entry management and rotation
#[tokio::test]
async fn test_log_entry_management() {
    println!("=== LOG ENTRY MANAGEMENT TEST ===");

    let mut state = AppState::new();

    // Test 1: Add log entries
    state.add_log("INFO".to_string(), "First log message".to_string());
    state.add_log("ERROR".to_string(), "Error message".to_string());
    state.add_log("DEBUG".to_string(), "Debug message".to_string());

    assert_eq!(state.device_logs.len(), 3);

    // Test 2: Verify log content
    let logs: Vec<_> = state.device_logs.iter().collect();
    assert_eq!(logs[0].level, "INFO");
    assert_eq!(logs[0].message, "First log message");
    assert_eq!(logs[1].level, "ERROR");
    assert_eq!(logs[1].message, "Error message");
    assert_eq!(logs[2].level, "DEBUG");
    assert_eq!(logs[2].message, "Debug message");

    // Test 3: Clear logs
    state.clear_logs();
    assert!(state.device_logs.is_empty());

    // Test 4: Log rotation (fill beyond max_log_entries)
    let max_entries = state.max_log_entries;

    // Add more than max entries
    for i in 0..max_entries + 10 {
        state.add_log("INFO".to_string(), format!("Log entry {}", i));
    }

    // Should not exceed max entries
    assert_eq!(state.device_logs.len(), max_entries);

    // Verify oldest entries were removed
    let logs: Vec<_> = state.device_logs.iter().collect();
    assert!(logs[0].message.contains("Log entry 10")); // First entry should be entry 10

    println!("✓ Log entry management test passed");
}

/// Test log filtering functionality
#[tokio::test]
async fn test_log_filtering() {
    println!("=== LOG FILTERING TEST ===");

    let mut state = AppState::new();

    // Add various log levels
    state.add_log("INFO".to_string(), "Info message 1".to_string());
    state.add_log("ERROR".to_string(), "Error message 1".to_string());
    state.add_log("DEBUG".to_string(), "Debug message 1".to_string());
    state.add_log("WARN".to_string(), "Warning message 1".to_string());
    state.add_log("INFO".to_string(), "Info message 2".to_string());
    state.add_log("ERROR".to_string(), "Error message 2".to_string());

    assert_eq!(state.device_logs.len(), 6);

    // Test 1: No filter - should return all logs
    let all_logs = state.get_filtered_logs();
    assert_eq!(all_logs.len(), 6);

    // Test 2: Filter by ERROR level
    state.toggle_log_filter(Some("ERROR".to_string()));
    let error_logs = state.get_filtered_logs();
    assert_eq!(error_logs.len(), 2);
    assert!(error_logs.iter().all(|log| log.level == "ERROR"));

    // Test 3: Filter by INFO level
    state.toggle_log_filter(Some("INFO".to_string()));
    let info_logs = state.get_filtered_logs();
    assert_eq!(info_logs.len(), 2);
    assert!(info_logs.iter().all(|log| log.level == "INFO"));

    // Test 4: Clear filter
    state.toggle_log_filter(None);
    let all_logs_again = state.get_filtered_logs();
    assert_eq!(all_logs_again.len(), 6);

    println!("✓ Log filtering test passed");
}

/// Test log streaming state consistency
#[tokio::test]
async fn test_log_streaming_state_consistency() {
    println!("=== LOG STREAMING STATE CONSISTENCY TEST ===");

    let mut state = AppState::new();

    // Setup devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Test_Device_1".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Test_Device_2".to_string(),
            device_type: "tablet".to_string(),
            api_level: 32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    // Test 1: Log streaming for running device
    state.selected_android = 0;
    let selected_device = &state.android_devices[state.selected_android];
    assert!(selected_device.is_running);

    // Simulate log streaming start
    state.current_log_device = Some((Panel::Android, selected_device.name.clone()));
    state.add_log("INFO".to_string(), "Test log from device 1".to_string());

    assert!(state.current_log_device.is_some());
    assert_eq!(state.device_logs.len(), 1);

    // Test 2: Switch to stopped device - should clear logs
    state.selected_android = 1;
    let new_selected_device = &state.android_devices[state.selected_android];
    assert!(!new_selected_device.is_running);

    // Simulate clearing when switching to stopped device
    state.clear_logs();
    state.current_log_device = None;

    assert!(state.current_log_device.is_none());
    assert!(state.device_logs.is_empty());

    // Test 3: Auto-scroll and manual scroll behavior
    state.selected_android = 0; // Back to running device
    state.current_log_device = Some((Panel::Android, "Test_Device_1".to_string()));

    // Add logs with auto-scroll enabled
    assert!(state.auto_scroll_logs);
    assert!(!state.manually_scrolled);

    state.add_log("INFO".to_string(), "Auto scroll test 1".to_string());
    state.add_log("INFO".to_string(), "Auto scroll test 2".to_string());

    // Should auto-scroll to bottom
    let expected_offset = state.device_logs.len().saturating_sub(1);
    assert_eq!(state.log_scroll_offset, expected_offset);

    // Test manual scrolling
    state.scroll_logs_to_top();
    assert_eq!(state.log_scroll_offset, 0);
    assert!(state.manually_scrolled);

    // Adding new log should not auto-scroll when manually scrolled
    let old_offset = state.log_scroll_offset;
    state.add_log("INFO".to_string(), "Manual scroll test".to_string());
    assert_eq!(state.log_scroll_offset, old_offset); // Should not change

    println!("✓ Log streaming state consistency test passed");
}

/// Mock test for log streaming task lifecycle
#[tokio::test]
async fn test_mock_log_streaming_task_lifecycle() {
    println!("=== MOCK LOG STREAMING TASK LIFECYCLE TEST ===");

    let state = Arc::new(Mutex::new(AppState::new()));

    // Setup device
    {
        let mut state_lock = state.lock().await;
        state_lock.android_devices = vec![AndroidDevice {
            name: "Mock_Device".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        }];
    }

    // Test 1: Simulate starting a log streaming task
    let state_clone = Arc::clone(&state);
    let mock_task = tokio::spawn(async move {
        // Simulate log streaming
        let mut counter = 0;
        loop {
            tokio::time::sleep(Duration::from_millis(10)).await;

            // Check if task should continue
            let should_continue = {
                let state_lock = state_clone.lock().await;
                state_lock.current_log_device.is_some() && counter < 5 // Limit iterations for test
            };

            if !should_continue {
                break;
            }

            // Add mock log entry
            {
                let mut state_lock = state_clone.lock().await;
                state_lock.add_log("INFO".to_string(), format!("Mock log entry {}", counter));
            }

            counter += 1;
        }
    });

    // Set up log streaming state
    {
        let mut state_lock = state.lock().await;
        state_lock.current_log_device = Some((Panel::Android, "Mock_Device".to_string()));
        state_lock.log_task_handle = Some(mock_task);
    }

    // Wait a bit for task to run
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 2: Verify logs were added
    {
        let state_lock = state.lock().await;
        assert!(!state_lock.device_logs.is_empty());
        assert!(state_lock.device_logs.len() <= 5); // Should have added some logs
    }

    // Test 3: Simulate stopping the task
    let handle = {
        let mut state_lock = state.lock().await;
        state_lock.current_log_device = None; // This should cause task to stop
        state_lock.log_task_handle.take()
    };

    // Wait for task to finish
    if let Some(handle) = handle {
        let _ = handle.await; // Task should complete naturally
    }

    // Test 4: Verify final state
    {
        let state_lock = state.lock().await;
        assert!(state_lock.current_log_device.is_none());
        assert!(state_lock.log_task_handle.is_none());
    }

    println!("✓ Mock log streaming task lifecycle test passed");
}

/// Test edge cases in log streaming
#[tokio::test]
async fn test_log_streaming_edge_cases() {
    println!("=== LOG STREAMING EDGE CASES TEST ===");

    let mut state = AppState::new();

    // Test 1: Empty device list
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());

    // Should handle empty selection gracefully
    state.move_up();
    state.move_down();
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Test 2: Device list with stopped devices only
    state.android_devices = vec![
        AndroidDevice {
            name: "Stopped_Device_1".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Stopped_Device_2".to_string(),
            device_type: "tablet".to_string(),
            api_level: 32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    // Should not start log streaming for stopped devices
    state.selected_android = 0;
    let device = &state.android_devices[state.selected_android];
    assert!(!device.is_running);

    // Verify no log streaming state is set
    assert!(state.current_log_device.is_none());

    // Test 3: Very long log messages
    state.add_log("INFO".to_string(), "A".repeat(10000)); // Very long message
    assert_eq!(state.device_logs.len(), 1);

    let log_entry = &state.device_logs[0];
    assert_eq!(log_entry.message.len(), 10000);

    // Test 4: Special characters in log messages
    state.add_log(
        "DEBUG".to_string(),
        "Log with 🚀 emoji and special chars: äöü ñ 中文".to_string(),
    );
    assert_eq!(state.device_logs.len(), 2);

    let special_log = &state.device_logs[1];
    assert!(special_log.message.contains("🚀"));
    assert!(special_log.message.contains("中文"));

    println!("✓ Log streaming edge cases test passed");
}
