//! iOS Device Display Debug Tests

#![cfg(feature = "test-utils")]

use emu::app::AppState;
use emu::models::{DeviceStatus, IosDevice};
use emu::ui::mock_backend::MockBackend;
use emu::ui::render::draw_app;
use emu::ui::theme::Theme;
use ratatui::Terminal;

#[test]
fn test_ios_device_rendering_debug() {
    let backend = MockBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app_state = AppState::new();
    app_state.ios_devices = vec![IosDevice {
        name: "iPhone_15_Pro".to_string(),
        udid: "12345-IPHONE-15".to_string(),
        device_type: "iPhone 15 Pro".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    let theme = Theme::dark();
    terminal
        .draw(|frame| {
            draw_app(frame, &mut app_state, &theme);
        })
        .unwrap();

    let backend = terminal.backend();
    let full_text = backend.get_buffer_text();
    println!("iOS Device Full Text:");
    println!("{full_text}");

    println!("Contains '17.0': {}", backend.assert_contains_text("17.0"));
    println!(
        "Contains 'iOS 17.0': {}",
        backend.assert_contains_text("iOS 17.0")
    );
    println!(
        "Contains 'iPhone': {}",
        backend.assert_contains_text("iPhone")
    );
}
