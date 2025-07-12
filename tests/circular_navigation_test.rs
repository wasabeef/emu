use emu::app::state::{AppState, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

#[test]
fn test_android_circular_navigation() {
    println!("=== ANDROID CIRCULAR NAVIGATION TEST ===");

    let mut state = AppState::new();
    state.active_panel = Panel::Android;

    // Create test Android devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Device1".to_string(),
            device_type: "phone".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Device2".to_string(),
            device_type: "phone".to_string(),
            api_level: 31,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        },
        AndroidDevice {
            name: "Device3".to_string(),
            device_type: "tablet".to_string(),
            api_level: 32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        },
    ];

    // Initial position should be 0
    assert_eq!(state.selected_android, 0);
    println!("Initial position: {}", state.selected_android);

    // Move down through all devices
    state.move_down(); // 0 -> 1
    assert_eq!(state.selected_android, 1);
    println!("After move_down: {}", state.selected_android);

    state.move_down(); // 1 -> 2
    assert_eq!(state.selected_android, 2);
    println!("After move_down: {}", state.selected_android);

    // Move down from last position should loop to first
    state.move_down(); // 2 -> 0 (loop)
    assert_eq!(state.selected_android, 0);
    println!("After move_down (loop to top): {}", state.selected_android);

    // Move up from first position should loop to last
    state.move_up(); // 0 -> 2 (loop)
    assert_eq!(state.selected_android, 2);
    println!("After move_up (loop to bottom): {}", state.selected_android);

    // Move up normally
    state.move_up(); // 2 -> 1
    assert_eq!(state.selected_android, 1);
    println!("After move_up: {}", state.selected_android);

    state.move_up(); // 1 -> 0
    assert_eq!(state.selected_android, 0);
    println!("After move_up: {}", state.selected_android);

    println!("✅ Android circular navigation works correctly");
}

#[test]
fn test_ios_circular_navigation() {
    println!("=== iOS CIRCULAR NAVIGATION TEST ===");

    let mut state = AppState::new();
    state.active_panel = Panel::Ios;

    // Create test iOS devices
    state.ios_devices = vec![
        IosDevice {
            name: "iPhone 14".to_string(),
            udid: "udid1".to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "16.0".to_string(),
            runtime_version: "iOS 16.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
        IosDevice {
            name: "iPad Pro".to_string(),
            udid: "udid2".to_string(),
            device_type: "iPad".to_string(),
            ios_version: "16.1".to_string(),
            runtime_version: "iOS 16.1".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
    ];

    // Initial position should be 0
    assert_eq!(state.selected_ios, 0);
    println!("Initial position: {}", state.selected_ios);

    // Move down to last device
    state.move_down(); // 0 -> 1
    assert_eq!(state.selected_ios, 1);
    println!("After move_down: {}", state.selected_ios);

    // Move down from last position should loop to first
    state.move_down(); // 1 -> 0 (loop)
    assert_eq!(state.selected_ios, 0);
    println!("After move_down (loop to top): {}", state.selected_ios);

    // Move up from first position should loop to last
    state.move_up(); // 0 -> 1 (loop)
    assert_eq!(state.selected_ios, 1);
    println!("After move_up (loop to bottom): {}", state.selected_ios);

    // Move up normally
    state.move_up(); // 1 -> 0
    assert_eq!(state.selected_ios, 0);
    println!("After move_up: {}", state.selected_ios);

    println!("✅ iOS circular navigation works correctly");
}

#[test]
fn test_empty_device_lists() {
    println!("=== EMPTY DEVICE LISTS TEST ===");

    let mut state = AppState::new();

    // Test Android with empty list
    state.active_panel = Panel::Android;
    state.android_devices = vec![];

    let initial_android = state.selected_android;
    state.move_up();
    assert_eq!(state.selected_android, initial_android);
    state.move_down();
    assert_eq!(state.selected_android, initial_android);
    println!("✅ Android empty list handled correctly");

    // Test iOS with empty list
    state.active_panel = Panel::Ios;
    state.ios_devices = vec![];

    let initial_ios = state.selected_ios;
    state.move_up();
    assert_eq!(state.selected_ios, initial_ios);
    state.move_down();
    assert_eq!(state.selected_ios, initial_ios);
    println!("✅ iOS empty list handled correctly");

    println!("✅ Empty device lists test passed");
}

#[test]
fn test_single_device_navigation() {
    println!("=== SINGLE DEVICE NAVIGATION TEST ===");

    let mut state = AppState::new();

    // Test Android with single device
    state.active_panel = Panel::Android;
    state.android_devices = vec![AndroidDevice {
        name: "OnlyDevice".to_string(),
        device_type: "phone".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    }];

    // Should stay at position 0 for single device
    assert_eq!(state.selected_android, 0);
    state.move_down();
    assert_eq!(state.selected_android, 0);
    state.move_up();
    assert_eq!(state.selected_android, 0);
    println!("✅ Android single device navigation works correctly");

    // Test iOS with single device
    state.active_panel = Panel::Ios;
    state.ios_devices = vec![IosDevice {
        name: "OnlyiPhone".to_string(),
        udid: "udid1".to_string(),
        device_type: "iPhone".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    // Should stay at position 0 for single device
    assert_eq!(state.selected_ios, 0);
    state.move_down();
    assert_eq!(state.selected_ios, 0);
    state.move_up();
    assert_eq!(state.selected_ios, 0);
    println!("✅ iOS single device navigation works correctly");

    println!("✅ Single device navigation test passed");
}

#[test]
fn test_rapid_circular_navigation() {
    println!("=== RAPID CIRCULAR NAVIGATION TEST ===");

    let mut state = AppState::new();
    state.active_panel = Panel::Android;

    // Create 5 Android devices
    state.android_devices = (0..5)
        .map(|i| AndroidDevice {
            name: format!("Device{i}"),
            device_type: "phone".to_string(),
            api_level: 30 + i as u32,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192".to_string(),
        })
        .collect();

    // Test rapid down navigation
    let start_pos = state.selected_android;
    for i in 1..=10 {
        state.move_down();
        let expected_pos = (start_pos + i) % 5;
        assert_eq!(state.selected_android, expected_pos);
    }
    println!("✅ Rapid down navigation works correctly");

    // Test rapid up navigation
    let start_pos = state.selected_android;
    for i in 1..=10 {
        state.move_up();
        let expected_pos = (start_pos + 5 - (i % 5)) % 5;
        assert_eq!(state.selected_android, expected_pos);
    }
    println!("✅ Rapid up navigation works correctly");

    println!("✅ Rapid circular navigation test passed");
}
