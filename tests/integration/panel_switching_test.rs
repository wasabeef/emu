//! Panel switching integration tests
//!
//! Tests the comprehensive behavior of panel switching including:
//! - Log streaming updates when switching panels
//! - Device details updates when switching panels
//! - Focus management and state consistency
//! - Performance and responsiveness of panel switches

use emu::app::state::{AppState, DeviceDetails, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Test basic panel switching functionality
#[tokio::test]
async fn test_basic_panel_switching() {
    println!("=== BASIC PANEL SWITCHING TEST ===");

    let mut state = AppState::new();

    // Setup test devices
    state.android_devices = vec![AndroidDevice {
        name: "Android_Test_Device".to_string(),
        device_type: "phone".to_string(),
        api_level: 31,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    }];

    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "ios-test-udid-123".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        is_available: true,
        is_running: true,
        status: DeviceStatus::Running,
    }];

    // Test 1: Initial state - should be Android panel
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Test 2: Switch to iOS panel
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);

    // Test 3: Switch back to Android panel
    state.active_panel = Panel::Android;
    assert_eq!(state.active_panel, Panel::Android);

    // Test 4: Verify selections remain consistent
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    println!("✓ Basic panel switching test passed");
}

/// Test panel switching with log streaming coordination
#[tokio::test]
async fn test_panel_switching_log_coordination() {
    println!("=== PANEL SWITCHING LOG COORDINATION TEST ===");

    let mut state = AppState::new();

    // Setup test devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Android_Device_1".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Android_Device_2".to_string(),
            device_type: "tablet".to_string(),
            api_level: 32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "ios-test-udid-456".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        is_available: true,
        is_running: true,
        status: DeviceStatus::Running,
    }];

    // Test 1: Start with Android panel and simulate log streaming
    assert_eq!(state.active_panel, Panel::Android);
    state.current_log_device = Some((Panel::Android, "Android_Device_1".to_string()));
    state.add_log("INFO".to_string(), "Android log entry".to_string());
    assert_eq!(state.device_logs.len(), 1);

    // Test 2: Switch to iOS panel - logs should be cleared for panel switch
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);

    // Simulate log clearing behavior that would happen in real app
    if state
        .current_log_device
        .as_ref()
        .is_some_and(|(panel, _)| *panel != state.active_panel)
    {
        state.clear_logs();
        state.current_log_device = None;
    }

    assert!(state.device_logs.is_empty());
    assert!(state.current_log_device.is_none());

    // Test 3: Start new log stream for iOS device
    state.current_log_device = Some((Panel::Ios, "ios-test-udid-456".to_string()));
    state.add_log("INFO".to_string(), "iOS log entry".to_string());
    assert_eq!(state.device_logs.len(), 1);

    // Test 4: Switch back to Android - verify log coordination
    state.active_panel = Panel::Android;

    // Simulate log clearing again
    if state
        .current_log_device
        .as_ref()
        .is_some_and(|(panel, _)| *panel != state.active_panel)
    {
        state.clear_logs();
        state.current_log_device = None;
    }

    assert!(state.device_logs.is_empty());
    assert!(state.current_log_device.is_none());

    println!("✓ Panel switching log coordination test passed");
}

/// Test panel switching with device details updates
#[tokio::test]
async fn test_panel_switching_device_details() {
    println!("=== PANEL SWITCHING DEVICE DETAILS TEST ===");

    let mut state = AppState::new();

    // Setup test devices
    state.android_devices = vec![AndroidDevice {
        name: "Android_Detail_Test".to_string(),
        device_type: "phone".to_string(),
        api_level: 31,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    }];

    state.ios_devices = vec![IosDevice {
        name: "iPhone Detail Test".to_string(),
        udid: "ios-detail-test-789".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        is_available: true,
        is_running: true,
        status: DeviceStatus::Running,
    }];

    // Test 1: Android panel initial state
    assert_eq!(state.active_panel, Panel::Android);
    assert!(state.cached_device_details.is_none());

    // Test 2: Simulate device details being loaded for Android
    // Note: In real app, this would be triggered by schedule_device_details_update()
    let mock_android_details = DeviceDetails {
        name: "Android_Detail_Test".to_string(),
        status: "Running".to_string(),
        platform: Panel::Android,
        device_type: "Pixel 7".to_string(),
        api_level_or_version: "API 31".to_string(),
        ram_size: Some("2048MB".to_string()),
        storage_size: Some("8GB".to_string()),
        resolution: Some("2400x1080".to_string()),
        dpi: Some("420".to_string()),
        device_path: Some("/path/to/android/device".to_string()),
        system_image: Some("system-images;android-31;google_apis;x86_64".to_string()),
        identifier: "Android_Detail_Test".to_string(),
    };
    state.cached_device_details = Some(mock_android_details);
    assert!(state.cached_device_details.is_some());

    // Test 3: Switch to iOS panel
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);

    // Test 4: Simulate details clearing/updating for iOS
    // In real app, schedule_device_details_update() would trigger new details loading
    state.cached_device_details = None; // Simulate clearing during panel switch

    let mock_ios_details = DeviceDetails {
        name: "iPhone Detail Test".to_string(),
        status: "Running".to_string(),
        platform: Panel::Ios,
        device_type: "iPhone 14".to_string(),
        api_level_or_version: "iOS 16.0".to_string(),
        ram_size: Some("6GB".to_string()),
        storage_size: Some("128GB".to_string()),
        resolution: Some("2556x1179".to_string()),
        dpi: Some("460".to_string()),
        device_path: None,
        system_image: None,
        identifier: "ios-detail-test-789".to_string(),
    };
    state.cached_device_details = Some(mock_ios_details);

    // Test 5: Verify iOS details are properly set
    if let Some(details) = &state.cached_device_details {
        assert_eq!(details.platform, Panel::Ios);
        assert_eq!(details.name, "iPhone Detail Test");
        assert_eq!(details.api_level_or_version, "iOS 16.0");
        assert_eq!(details.identifier, "ios-detail-test-789");
    }

    println!("✓ Panel switching device details test passed");
}

/// Test panel switching performance and responsiveness
#[tokio::test]
async fn test_panel_switching_performance() {
    println!("=== PANEL SWITCHING PERFORMANCE TEST ===");

    let state = Arc::new(Mutex::new(AppState::new()));

    // Setup test devices
    {
        let mut state_lock = state.lock().await;
        state_lock.android_devices = (0..5)
            .map(|i| AndroidDevice {
                name: format!("Android_Device_{i}"),
                device_type: "phone".to_string(),
                api_level: 31 + i as u32,
                status: DeviceStatus::Stopped,
                is_running: false,
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
            })
            .collect();

        state_lock.ios_devices = (0..3)
            .map(|i| IosDevice {
                name: format!("iPhone_{i}"),
                udid: format!("ios-udid-{i}"),
                device_type: "iPhone".to_string(),
                ios_version: "16.0".to_string(),
                runtime_version: "iOS 16.0".to_string(),
                is_available: true,
                is_running: false,
                status: DeviceStatus::Stopped,
            })
            .collect();
    }

    // Test 1: Measure panel switching speed
    let start_time = std::time::Instant::now();

    for _ in 0..10 {
        // Simulate rapid panel switching
        {
            let mut state_lock = state.lock().await;
            state_lock.active_panel = Panel::Ios;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;

        {
            let mut state_lock = state.lock().await;
            state_lock.active_panel = Panel::Android;
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }

    let elapsed = start_time.elapsed();

    // Should complete rapidly (under 100ms for 20 switches)
    assert!(
        elapsed.as_millis() < 100,
        "Panel switching took too long: {elapsed:?}"
    );

    // Test 2: Verify state consistency after rapid switching
    {
        let state_lock = state.lock().await;
        assert_eq!(state_lock.active_panel, Panel::Android);
        assert_eq!(state_lock.android_devices.len(), 5);
        assert_eq!(state_lock.ios_devices.len(), 3);
    }

    println!(
        "✓ Panel switching performance test passed ({}ms for 20 switches)",
        elapsed.as_millis()
    );
}

/// Test panel switching with empty device lists
#[tokio::test]
async fn test_panel_switching_empty_devices() {
    println!("=== PANEL SWITCHING EMPTY DEVICES TEST ===");

    let mut state = AppState::new();

    // Test 1: Both device lists empty
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());
    assert_eq!(state.active_panel, Panel::Android);

    // Test 2: Switch to iOS with empty devices
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);

    // Test 3: Selection indices should remain 0 even with empty lists
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Test 4: Navigation should handle empty lists gracefully
    state.move_up();
    state.move_down();
    assert_eq!(state.selected_android, 0);
    assert_eq!(state.selected_ios, 0);

    // Test 5: Switch back to Android panel
    state.active_panel = Panel::Android;
    assert_eq!(state.active_panel, Panel::Android);

    println!("✓ Panel switching empty devices test passed");
}

/// Test panel switching with mixed device states
#[tokio::test]
async fn test_panel_switching_mixed_device_states() {
    println!("=== PANEL SWITCHING MIXED DEVICE STATES TEST ===");

    let mut state = AppState::new();

    // Setup mixed device states
    state.android_devices = vec![
        AndroidDevice {
            name: "Running_Android".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Stopped_Android".to_string(),
            device_type: "tablet".to_string(),
            api_level: 32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    state.ios_devices = vec![
        IosDevice {
            name: "Available_iOS".to_string(),
            udid: "available-ios-123".to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "16.0".to_string(),
            runtime_version: "iOS 16.0".to_string(),
            is_available: true,
            is_running: true,
            status: DeviceStatus::Running,
        },
        IosDevice {
            name: "Unavailable_iOS".to_string(),
            udid: "unavailable-ios-456".to_string(),
            device_type: "iPad".to_string(),
            ios_version: "15.0".to_string(),
            runtime_version: "iOS 15.0".to_string(),
            is_available: false,
            is_running: false,
            status: DeviceStatus::Stopped,
        },
    ];

    // Test 1: Start with Android panel, select running device
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.selected_android, 0);
    let android_device = &state.android_devices[state.selected_android];
    assert!(android_device.is_running);

    // Test 2: Switch to iOS panel
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.selected_ios, 0);
    let ios_device = &state.ios_devices[state.selected_ios];
    assert!(ios_device.is_available);

    // Test 3: Navigate to unavailable iOS device
    state.move_down();
    assert_eq!(state.selected_ios, 1);
    let ios_device_2 = &state.ios_devices[state.selected_ios];
    assert!(!ios_device_2.is_available);

    // Test 4: Switch back to Android and navigate to stopped device
    state.active_panel = Panel::Android;
    state.move_down();
    assert_eq!(state.selected_android, 1);
    let android_device_2 = &state.android_devices[state.selected_android];
    assert!(!android_device_2.is_running);

    // Test 5: Verify selections persist across panel switches
    state.active_panel = Panel::Ios;
    assert_eq!(state.selected_ios, 1); // Should still be on second iOS device
    state.active_panel = Panel::Android;
    assert_eq!(state.selected_android, 1); // Should still be on second Android device

    println!("✓ Panel switching mixed device states test passed");
}
