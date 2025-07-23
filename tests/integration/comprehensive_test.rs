use emu::app::state::{AppState, CreateDeviceField, CreateDeviceForm, FocusedPanel, Mode, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

/// Integration test that verifies the complete workflow
#[test]
fn test_complete_device_management_workflow() {
    println!("=== COMPLETE DEVICE MANAGEMENT WORKFLOW TEST ===");

    let mut state = AppState::new();

    // Setup initial devices
    state.android_devices = vec![AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "Pixel_7_API_31".to_string(),
        device_type: "phone".to_string(),
        api_level: 31,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    }];

    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "test-udid-1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Step 1: Navigate between panels
    assert_eq!(state.active_panel, Panel::Android);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Ios);
    state.next_panel();
    assert_eq!(state.active_panel, Panel::Android);

    // Step 2: Navigate within device lists
    assert_eq!(state.selected_android, 0);
    state.move_down(); // Should stay at 0 (only one device)
    assert_eq!(state.selected_android, 0);

    // Test circular navigation with multiple devices
    state.android_devices.push(AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "Pixel_8_API_33".to_string(),
        device_type: "phone".to_string(),
        api_level: 33,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "4096".to_string(),
        storage_size: "16384".to_string(),
    });

    state.move_down(); // 0 -> 1
    assert_eq!(state.selected_android, 1);
    state.move_down(); // 1 -> 0 (circular)
    assert_eq!(state.selected_android, 0);
    state.move_up(); // 0 -> 1 (circular)
    assert_eq!(state.selected_android, 1);

    // Step 3: Test device creation workflow
    state.mode = Mode::CreateDevice;
    state.create_device_form = CreateDeviceForm::for_android();

    // Test field navigation
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );
    state.create_device_form.next_field();
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::Category
    );
    state.create_device_form.next_field();
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::DeviceType
    );

    // Test form input
    state.create_device_form.name = "Test_Device_API_32".to_string();
    state.create_device_form.ram_size = "4096".to_string();
    state.create_device_form.storage_size = "16384".to_string();

    // Validate form
    assert!(!state.create_device_form.name.trim().is_empty());
    assert!(state.create_device_form.ram_size.parse::<u32>().is_ok());
    assert!(state.create_device_form.storage_size.parse::<u32>().is_ok());

    // Return to normal mode
    state.mode = Mode::Normal;

    // Step 4: Test operation status tracking
    state.set_device_operation_status("Starting device 'Pixel_7_API_31'...".to_string());
    assert!(state.get_device_operation_status().is_some());

    // Simulate device start completion
    state.android_devices[0].is_running = true;
    state.android_devices[0].status = DeviceStatus::Running;
    state.clear_device_operation_status();
    state.add_success_notification("Device 'Pixel_7_API_31' is now running!".to_string());

    // Step 5: Test log management during operation
    state.add_log(
        "INFO".to_string(),
        "Device started successfully".to_string(),
    );
    state.add_log("DEBUG".to_string(), "Emulator console started".to_string());
    state.add_log("INFO".to_string(), "ADB connected".to_string());

    assert_eq!(state.device_logs.len(), 3);

    // Test log filtering
    state.toggle_log_filter(Some("INFO".to_string()));
    let filtered_logs = state.get_filtered_logs();
    assert_eq!(filtered_logs.len(), 2); // Only INFO logs

    // Step 6: Test device details caching
    let details = state.get_selected_device_details();
    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.name, "Pixel_8_API_33"); // Currently selected (index 1)
    assert_eq!(details.platform, Panel::Android);

    // Test cache clearing on panel switch
    state.smart_clear_cached_device_details(Panel::Ios);
    assert!(state.cached_device_details.is_none());

    println!("✅ Complete device management workflow test passed");
}

#[test]
fn test_error_handling_and_recovery() {
    println!("=== ERROR HANDLING AND RECOVERY TEST ===");

    let mut state = AppState::new();

    // Test error notification handling
    state.add_error_notification("Failed to start device: Device not found".to_string());
    assert_eq!(state.notifications.len(), 1);

    // Test operation status clearing on error
    state.set_device_operation_status("Starting device...".to_string());
    assert!(state.get_device_operation_status().is_some());

    // Simulate error during operation
    state.clear_device_operation_status();
    state.add_error_notification("Operation failed".to_string());
    assert!(state.get_device_operation_status().is_none());

    // Test form error handling
    state.mode = Mode::CreateDevice;
    state.create_device_form = CreateDeviceForm::for_android();
    state.create_device_form.error_message = Some("Invalid device configuration".to_string());

    assert!(state.create_device_form.error_message.is_some());

    // Test error clearing
    state.create_device_form.error_message = None;
    assert!(state.create_device_form.error_message.is_none());

    // Test graceful handling of empty device lists
    state.android_devices.clear();
    state.ios_devices.clear();

    // Should not crash on navigation
    state.move_up();
    state.move_down();
    state.next_panel();

    // Details should handle empty lists gracefully
    let details = state.get_selected_device_details();
    assert!(details.is_none()); // No devices available

    println!("✅ Error handling and recovery test passed");
}

#[test]
fn test_performance_critical_operations() {
    println!("=== PERFORMANCE CRITICAL OPERATIONS TEST ===");

    let mut state = AppState::new();

    // Test rapid panel switching
    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        state.next_panel();
    }

    let panel_switch_time = start.elapsed();
    println!("1000 panel switches took: {panel_switch_time:?}");

    // Panel switching should be very fast (memory operations only)
    assert!(panel_switch_time < std::time::Duration::from_millis(10));

    // Test rapid device navigation
    state.android_devices = (0..100)
        .map(|i| AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: format!("Device{i}"),
            device_type: "phone".to_string(),
            api_level: 30 + (i % 5) as u32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        })
        .collect();

    let start = std::time::Instant::now();

    for _ in 0..1000 {
        state.move_down();
        state.move_up();
    }

    let navigation_time = start.elapsed();
    println!("1000 navigation operations took: {navigation_time:?}");

    // Device navigation should be fast (index operations)
    assert!(navigation_time < std::time::Duration::from_millis(50));

    // Test cache operation performance
    let start = std::time::Instant::now();

    for i in 0..100 {
        state.smart_clear_cached_device_details(if i % 2 == 0 {
            Panel::Android
        } else {
            Panel::Ios
        });
    }

    let cache_time = start.elapsed();
    println!("100 cache operations took: {cache_time:?}");

    // Cache operations should be fast
    assert!(cache_time < std::time::Duration::from_millis(10));

    println!("✅ Performance critical operations test passed");
}

#[test]
fn test_state_consistency_during_complex_operations() {
    println!("=== STATE CONSISTENCY DURING COMPLEX OPERATIONS TEST ===");

    let mut state = AppState::new();

    // Set up complex state
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "Device1".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "Device2".to_string(),
            device_type: "tablet".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    state.ios_devices = vec![IosDevice {
        name: "iPhone".to_string(),
        udid: "udid1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Running,
        is_running: true,
        is_available: true,
    }];

    // Test multiple simultaneous state changes
    state.active_panel = Panel::Android;
    state.selected_android = 1;
    state.set_device_operation_status("Testing operation".to_string());
    state.add_info_notification("Test notification".to_string());
    state.add_log("INFO".to_string(), "Test log".to_string());
    state.set_pending_device_start("Device2".to_string());

    // Verify all state is consistent
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.selected_android, 1);
    assert!(state.get_device_operation_status().is_some());
    assert_eq!(state.notifications.len(), 1);
    assert_eq!(state.device_logs.len(), 1);
    assert!(state.get_pending_device_start().is_some());

    // Test state transitions
    state.next_panel(); // Switch to iOS
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.selected_android, 1); // Should remain unchanged
    assert_eq!(state.selected_ios, 0);

    // Test clearing operations
    state.clear_device_operation_status();
    state.clear_pending_device_start();
    state.dismiss_all_notifications();
    state.clear_logs();

    // Verify clean state
    assert!(state.get_device_operation_status().is_none());
    assert!(state.get_pending_device_start().is_none());
    assert!(state.notifications.is_empty());
    assert!(state.device_logs.is_empty());

    // But panel state should remain
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.selected_ios, 0);

    println!("✅ State consistency during complex operations test passed");
}

#[test]
fn test_ui_state_synchronization() {
    println!("=== UI STATE SYNCHRONIZATION TEST ===");

    let mut state = AppState::new();

    // Test focus and panel state coordination
    state.focused_panel = FocusedPanel::DeviceList;
    state.active_panel = Panel::Android;

    // Device list should be focused and Android should be active
    let is_device_list_active =
        state.focused_panel == FocusedPanel::DeviceList && state.active_panel == Panel::Android;
    assert!(is_device_list_active);

    // Test fullscreen logs coordination
    state.toggle_fullscreen_logs();
    assert!(state.fullscreen_logs);

    // Test log scrolling state - need logs first for scrolling to work
    state.add_log("INFO".to_string(), "Test log 1".to_string());
    state.add_log("INFO".to_string(), "Test log 2".to_string());
    state.add_log("INFO".to_string(), "Test log 3".to_string());
    state.add_log("INFO".to_string(), "Test log 4".to_string());
    state.add_log("INFO".to_string(), "Test log 5".to_string());

    // Move to top first to enable scrolling down
    state.log_scroll_offset = 0;

    assert!(!state.manually_scrolled);
    state.scroll_logs_down();
    assert!(state.manually_scrolled);

    // Test auto-scroll coordination
    assert!(state.auto_scroll_logs);
    state.toggle_auto_scroll();
    assert!(!state.auto_scroll_logs);
    assert!(state.manually_scrolled); // Should remain true when auto-scroll disabled

    // When toggling back to auto-scroll, manually_scrolled should reset
    state.toggle_auto_scroll();
    assert!(state.auto_scroll_logs);
    assert!(!state.manually_scrolled); // Should reset when auto-scroll enabled

    // Test mode transitions
    assert_eq!(state.mode, Mode::Normal);
    state.mode = Mode::CreateDevice;
    assert_eq!(state.mode, Mode::CreateDevice);

    // Create device form should be initialized properly for the mode
    state.create_device_form = CreateDeviceForm::for_android();
    assert_eq!(
        state.create_device_form.active_field,
        CreateDeviceField::ApiLevel
    );

    // Test dialog state coordination
    state.mode = Mode::ConfirmDelete;
    // Dialog should be set when mode is ConfirmDelete (in real app)
    // Here we just verify the mode change worked
    assert_eq!(state.mode, Mode::ConfirmDelete);

    state.mode = Mode::Normal;
    assert_eq!(state.mode, Mode::Normal);

    println!("✅ UI state synchronization test passed");
}
