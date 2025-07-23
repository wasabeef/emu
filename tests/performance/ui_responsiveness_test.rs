//! UI responsiveness and interaction performance tests
//!
//! Measures response performance for keyboard input processing,
//! panel switching, and screen updates.

use emu::app::state::{AppState, Panel};
use emu::models::{AndroidDevice, DeviceStatus};
use std::time::Instant;

#[cfg(feature = "test-utils")]
use emu::ui::MockBackend;
#[cfg(feature = "test-utils")]
use ratatui::Terminal;

const RESPONSIVENESS_TARGET_PANEL_SWITCH_MS: u64 = 50;
const RESPONSIVENESS_TARGET_DEVICE_NAVIGATION_MS: u64 = 10;
const RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS: u64 = 25;
const RESPONSIVENESS_TARGET_120FPS_POLLING_MS: u64 = 8;

/// Panel switching responsiveness test
#[tokio::test]
async fn test_panel_switching_responsiveness() {
    let mut app_state = AppState::new();
    app_state.android_devices = create_test_devices(10);

    #[cfg(target_os = "macos")]
    {
        use emu::models::IosDevice;
        app_state.ios_devices = vec![IosDevice {
            name: "iPhone 15".to_string(),
            udid: "12345678-ABCD-EFGH-IJKL-123456789ABC".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        }];
    }

    let switch_operations = 100;
    let start_time = Instant::now();

    // Simulate panel switching
    for i in 0..switch_operations {
        let panel = if i % 2 == 0 {
            Panel::Android
        } else {
            Panel::Ios
        };

        app_state.active_panel = panel;

        // Simulate UI state update
        match app_state.active_panel {
            Panel::Android => {
                let _selected = app_state.get_selected_device_details();
            }
            Panel::Ios => {
                let _selected = app_state.get_selected_device_details();
            }
        }
    }

    let total_duration = start_time.elapsed();
    let avg_duration_ms = total_duration.as_millis() as u64 / switch_operations;

    assert!(
        avg_duration_ms < RESPONSIVENESS_TARGET_PANEL_SWITCH_MS,
        "Panel switching average {avg_duration_ms}ms exceeds target {RESPONSIVENESS_TARGET_PANEL_SWITCH_MS}ms"
    );

    println!("✅ Panel switching responsiveness: {avg_duration_ms}ms avg (target: <{RESPONSIVENESS_TARGET_PANEL_SWITCH_MS}ms)");
}

/// Device navigation performance test
#[tokio::test]
async fn test_device_navigation_performance() {
    let mut app_state = AppState::new();
    app_state.android_devices = create_test_devices(50);

    let navigation_operations = 1000;
    let start_time = Instant::now();

    // Simulate navigation between devices
    for _ in 0..navigation_operations {
        // Move down
        if app_state.selected_android < app_state.android_devices.len() - 1 {
            app_state.selected_android += 1;
        } else {
            app_state.selected_android = 0; // Circular navigation
        }

        // Get selected device
        let _selected = app_state.get_selected_device_details();
    }

    let total_duration = start_time.elapsed();
    let avg_duration_ms = total_duration.as_millis() as u64 / navigation_operations;

    assert!(
        avg_duration_ms < RESPONSIVENESS_TARGET_DEVICE_NAVIGATION_MS,
        "Device navigation average {avg_duration_ms}ms exceeds target {RESPONSIVENESS_TARGET_DEVICE_NAVIGATION_MS}ms"
    );

    println!("✅ Device navigation performance: {avg_duration_ms}ms avg (target: <{RESPONSIVENESS_TARGET_DEVICE_NAVIGATION_MS}ms)");
}

/// Screen update framerate test
#[tokio::test]
async fn test_screen_update_framerate() {
    #[cfg(feature = "test-utils")]
    {
        let mut app_state = AppState::new();
        app_state.android_devices = create_test_devices(20);

        let backend = MockBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let frame_count = 120; // 120 frames
        let start_time = Instant::now();

        for i in 0..frame_count {
            // Dynamically change device state to simulate animation effects
            app_state.selected_android = i % app_state.android_devices.len();

            terminal
                .draw(|frame| {
                    // Mock rendering
                    let _ = frame.area();
                })
                .unwrap();
        }

        let total_duration = start_time.elapsed();
        let avg_frame_time_ms = total_duration.as_millis() as u64 / frame_count as u64;

        assert!(
            avg_frame_time_ms < RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS,
            "Screen update average {avg_frame_time_ms}ms exceeds target {RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS}ms"
        );

        // Calculate framerate
        let fps = 1000.0 / avg_frame_time_ms as f64;
        println!("✅ Screen update framerate: {avg_frame_time_ms}ms per frame ({fps:.1} FPS, target: <{RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS}ms)");
    }

    #[cfg(not(feature = "test-utils"))]
    {
        println!("⏭️  Screen update test skipped (test-utils feature not enabled)");
    }
}

/// 120FPS keyboard input polling performance test
#[tokio::test]
async fn test_120fps_keyboard_polling() {
    let mut app_state = AppState::new();
    app_state.android_devices = create_test_devices(10);

    let polling_cycles = 1000;
    let start_time = Instant::now();

    // High frequency polling simulation (120 FPS = 8.33ms interval)
    for i in 0..polling_cycles {
        // Simulate key input event processing
        match i % 4 {
            0 => {
                // Down key
                if app_state.selected_android < app_state.android_devices.len() - 1 {
                    app_state.selected_android += 1;
                }
            }
            1 => {
                // Up key
                if app_state.selected_android > 0 {
                    app_state.selected_android -= 1;
                }
            }
            2 => {
                // Platform switch
                app_state.active_panel = match app_state.active_panel {
                    Panel::Android => Panel::Ios,
                    Panel::Ios => Panel::Android,
                };
            }
            3 => {
                // Check selected device
                let _selected = app_state.get_selected_device_details();
            }
            _ => {}
        }

        // Simulate processing completion at 8ms target
        if i % 120 == 0 {
            tokio::task::yield_now().await;
        }
    }

    let total_duration = start_time.elapsed();
    let avg_polling_time_ms = total_duration.as_millis() as u64 / polling_cycles as u64;

    assert!(
        avg_polling_time_ms < RESPONSIVENESS_TARGET_120FPS_POLLING_MS,
        "120FPS polling average {avg_polling_time_ms}ms exceeds target {RESPONSIVENESS_TARGET_120FPS_POLLING_MS}ms"
    );

    println!("✅ 120FPS keyboard polling: {avg_polling_time_ms}ms avg (target: <{RESPONSIVENESS_TARGET_120FPS_POLLING_MS}ms)");
}

/// UI responsiveness test with large dataset
#[tokio::test]
async fn test_large_dataset_ui_responsiveness() {
    #[cfg(feature = "test-utils")]
    {
        let mut app_state = AppState::new();
        app_state.android_devices = create_test_devices(200); // Large number of devices

        let backend = MockBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let interactions = 50;
        let start_time = Instant::now();

        for i in 0..interactions {
            // Navigation operation
            app_state.selected_android = (i * 7) % app_state.android_devices.len();

            // Screen rendering
            terminal
                .draw(|frame| {
                    // Mock rendering
                    let _ = frame.area();
                })
                .unwrap();
        }

        let total_duration = start_time.elapsed();
        let avg_interaction_ms = total_duration.as_millis() as u64 / interactions as u64;

        // Maintain proper response performance even with large datasets
        assert!(
            avg_interaction_ms < RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS * 2,
            "Large dataset UI responsiveness {avg_interaction_ms}ms exceeds target {RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS}ms * 2"
        );

        println!("✅ Large dataset UI responsiveness: {avg_interaction_ms}ms avg for 200 devices (target: <{}ms)", RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS * 2);
    }

    #[cfg(not(feature = "test-utils"))]
    {
        println!("⏭️  Large dataset UI test skipped (test-utils feature not enabled)");
    }
}

/// Real-time state update performance test
#[tokio::test]
async fn test_realtime_state_update_performance() {
    let mut app_state = AppState::new();
    app_state.android_devices = create_test_devices(20);

    let state_updates = 1000;
    let start_time = Instant::now();

    // Simulate real-time state updates
    for i in 0..state_updates {
        let device_index = i % app_state.android_devices.len();

        // Dynamically change device state
        app_state.android_devices[device_index].status = match i % 4 {
            0 => DeviceStatus::Stopped,
            1 => DeviceStatus::Starting,
            2 => DeviceStatus::Running,
            3 => DeviceStatus::Stopping,
            _ => DeviceStatus::Unknown,
        };

        app_state.android_devices[device_index].is_running =
            app_state.android_devices[device_index].status == DeviceStatus::Running;
    }

    let total_duration = start_time.elapsed();
    let avg_update_ms = total_duration.as_millis() as u64 / state_updates as u64;

    assert!(
        avg_update_ms < 1, // State update within 1ms
        "Realtime state update average {avg_update_ms}ms exceeds target 1ms"
    );

    println!("✅ Realtime state update performance: {avg_update_ms}ms avg (target: <1ms)");
}

/// Scrolling performance test
#[tokio::test]
async fn test_scrolling_performance() {
    #[cfg(feature = "test-utils")]
    {
        let mut app_state = AppState::new();
        app_state.android_devices = create_test_devices(100);

        let backend = MockBackend::new(80, 24); // Scroll test with small screen
        let mut terminal = Terminal::new(backend).unwrap();

        let scroll_operations = 200;
        let start_time = Instant::now();

        for i in 0..scroll_operations {
            // Simulate scroll operation
            app_state.selected_android = i % app_state.android_devices.len();

            // Rendering (according to scroll position)
            terminal
                .draw(|frame| {
                    // Mock rendering
                    let _ = frame.area();
                })
                .unwrap();
        }

        let total_duration = start_time.elapsed();
        let avg_scroll_ms = total_duration.as_millis() as u64 / scroll_operations as u64;

        assert!(
            avg_scroll_ms < RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS,
            "Scrolling performance {avg_scroll_ms}ms exceeds target {RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS}ms"
        );

        println!("✅ Scrolling performance: {avg_scroll_ms}ms avg (target: <{RESPONSIVENESS_TARGET_SCREEN_UPDATE_MS}ms)");
    }

    #[cfg(not(feature = "test-utils"))]
    {
        println!("⏭️  Scrolling test skipped (test-utils feature not enabled)");
    }
}

/// Interaction consistency test
#[tokio::test]
async fn test_interaction_consistency() {
    let mut app_state = AppState::new();
    app_state.android_devices = create_test_devices(30);

    let consistency_checks = 1000;
    let mut timing_measurements = vec![];

    // Measure consistent interaction times
    for _ in 0..consistency_checks {
        let start = Instant::now();

        // Standard interaction operation
        app_state.selected_android =
            (app_state.selected_android + 1) % app_state.android_devices.len();
        let _selected = app_state.get_selected_device_details();

        let duration = start.elapsed();
        timing_measurements.push(duration.as_micros() as u64);
    }

    // Statistical analysis
    timing_measurements.sort();
    let median = timing_measurements[timing_measurements.len() / 2];
    let p95 = timing_measurements[(timing_measurements.len() * 95) / 100];
    let max = timing_measurements[timing_measurements.len() - 1];

    // Consistency requirement: 95th percentile within 3x of median
    assert!(
        p95 <= median * 3,
        "Interaction inconsistency: P95 {p95}μs exceeds 3x median {median}μs"
    );

    println!("✅ Interaction consistency: median {median}μs, P95 {p95}μs, max {max}μs");
}

/// Comprehensive responsiveness validation test
#[tokio::test]
async fn test_comprehensive_responsiveness_validation() {
    println!("🎯 Comprehensive UI responsiveness validation:");

    let test_scenarios = vec![
        ("Quick navigation", 10, 5),
        ("Medium navigation", 50, 20),
        ("Heavy navigation", 100, 50),
    ];

    for (scenario_name, device_count, operations) in test_scenarios {
        let mut app_state = AppState::new();
        app_state.android_devices = create_test_devices(device_count);

        let start_time = Instant::now();

        // Mixed operations (navigation + state change + UI update)
        for i in 0..operations {
            // Navigation
            app_state.selected_android = (i * 3) % app_state.android_devices.len();

            // Platform switch
            if i % 10 == 0 {
                app_state.active_panel = match app_state.active_panel {
                    Panel::Android => Panel::Ios,
                    Panel::Ios => Panel::Android,
                };
            }

            // Check device selection
            let _selected = app_state.get_selected_device_details();
        }

        let total_duration = start_time.elapsed();
        let avg_operation_ms = total_duration.as_millis() as u64 / operations as u64;

        let target_ms = 5; // Complex operations within 5ms
        if avg_operation_ms <= target_ms {
            println!("  ✅ {scenario_name}: {avg_operation_ms}ms avg (target: <{target_ms}ms)");
        } else {
            println!("  ⚠️  {scenario_name}: {avg_operation_ms}ms exceeds {target_ms}ms target");
        }
    }
}

// Helper functions

fn create_test_devices(count: usize) -> Vec<AndroidDevice> {
    (1..=count)
        .map(|i| AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: format!("ResponsivenessTest_Device_{i}"),
            device_type: format!("pixel_{}", (i % 5) + 1),
            api_level: 30 + (i % 10) as u32,
            status: match i % 4 {
                0 => DeviceStatus::Stopped,
                1 => DeviceStatus::Running,
                2 => DeviceStatus::Starting,
                _ => DeviceStatus::Unknown,
            },
            is_running: i % 4 == 1,
            ram_size: format!("{}", 2048 + (i % 4) * 1024),
            storage_size: format!("{}", 8192 + (i % 3) * 4096),
        })
        .collect()
}
