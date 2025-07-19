//! UI Rendering Debug Tests
//!
//! Check detailed output of failing test cases

use emu::app::AppState;
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use emu::ui::mock_backend::MockBackend;
use emu::ui::render::draw_app;
use emu::ui::theme::Theme;
use ratatui::Terminal;

#[test]
fn debug_android_devices_rendering() {
    let backend = MockBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app_state = AppState::new();
    app_state.android_devices = vec![
        AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            name: "Galaxy_S22_API_33".to_string(),
            device_type: "galaxy_s22".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16G".to_string(),
        },
    ];

    let theme = Theme::dark();
    terminal
        .draw(|frame| {
            draw_app(frame, &mut app_state, &theme);
        })
        .unwrap();

    let backend = terminal.backend();
    let full_text = backend.get_buffer_text();

    println!("=== Android Devices Debug ===");
    println!("Full text: {full_text}");
    println!(
        "Contains 'Pixel_7_API_34': {}",
        backend.assert_contains_text("Pixel_7_API_34")
    );
    println!(
        "Contains 'Galaxy_S22_API_33': {}",
        backend.assert_contains_text("Galaxy_S22_API_33")
    );
    println!(
        "Contains 'Galaxy': {}",
        backend.assert_contains_text("Galaxy")
    );
    println!("Contains 'S22': {}", backend.assert_contains_text("S22"));
    println!("Contains '33': {}", backend.assert_contains_text("33"));
}

#[test]
fn debug_ios_devices_rendering() {
    let backend = MockBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app_state = AppState::new();
    app_state.ios_devices = vec![
        IosDevice {
            name: "iPhone_15_Pro".to_string(),
            udid: "12345-IPHONE-15".to_string(),
            device_type: "iPhone 15 Pro".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
        IosDevice {
            name: "iPad_Pro_12_9".to_string(),
            udid: "67890-IPAD-PRO".to_string(),
            device_type: "iPad Pro (12.9-inch)".to_string(),
            ios_version: "17.1".to_string(),
            runtime_version: "iOS 17.1".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            is_available: true,
        },
    ];

    let theme = Theme::dark();
    terminal
        .draw(|frame| {
            draw_app(frame, &mut app_state, &theme);
        })
        .unwrap();

    let backend = terminal.backend();
    let full_text = backend.get_buffer_text();

    println!("=== iOS Devices Debug ===");
    println!("Full text: {full_text}");
    println!(
        "Contains 'iPhone_15_Pro': {}",
        backend.assert_contains_text("iPhone_15_Pro")
    );
    println!(
        "Contains 'iPad_Pro_12_9': {}",
        backend.assert_contains_text("iPad_Pro_12_9")
    );
    println!("Contains '17.0': {}", backend.assert_contains_text("17.0"));
    println!("Contains '17.1': {}", backend.assert_contains_text("17.1"));
}

#[test]
fn debug_device_details_rendering() {
    let backend = MockBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app_state = AppState::new();
    app_state.android_devices = vec![AndroidDevice {
        name: "Pixel_7_Details_Test".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Running,
        is_running: true,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    }];
    app_state.selected_android = 0;

    let theme = Theme::dark();
    terminal
        .draw(|frame| {
            draw_app(frame, &mut app_state, &theme);
        })
        .unwrap();

    let backend = terminal.backend();
    let full_text = backend.get_buffer_text();

    println!("=== Device Details Debug ===");
    println!("Full text: {full_text}");
    println!(
        "Contains 'Pixel_7_Details_Test': {}",
        backend.assert_contains_text("Pixel_7_Details_Test")
    );
    println!("Contains '2048': {}", backend.assert_contains_text("2048"));
    println!("Contains '8192': {}", backend.assert_contains_text("8192"));
    println!("Contains 'RAM': {}", backend.assert_contains_text("RAM"));
    println!(
        "Contains 'Storage': {}",
        backend.assert_contains_text("Storage")
    );
}
