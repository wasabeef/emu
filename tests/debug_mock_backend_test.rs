//! MockBackend Debug Tests
//!
//! Verifies MockBackend behavior and tests text extraction functionality.

#![cfg(feature = "test-utils")]

use emu::app::AppState;
use emu::models::{AndroidDevice, DeviceStatus};
use emu::ui::mock_backend::MockBackend;
use emu::ui::render::draw_app;
use emu::ui::theme::Theme;
use ratatui::Terminal;

#[test]
fn test_mock_backend_basic_functionality() {
    let backend = MockBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app_state = AppState::new();
    app_state.android_devices = vec![AndroidDevice {
        name: "TestDevice".to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8G".to_string(),
    }];

    let theme = Theme::dark();
    terminal
        .draw(|frame| {
            draw_app(frame, &mut app_state, &theme);
        })
        .unwrap();

    let backend = terminal.backend();

    // Debug: Check buffer contents
    if let Some(buffer) = backend.get_last_buffer() {
        println!("Buffer area: {:?}", buffer.area());
        println!("Buffer content:");
        for y in 0..buffer.area().height {
            for x in 0..buffer.area().width {
                if let Some(cell) = buffer.cell((x, y)) {
                    if !cell.symbol().trim().is_empty() {
                        println!("  [{},{}]: '{}'", x, y, cell.symbol());
                    }
                }
            }
        }
    }

    // Actually perform the search
    let full_text = backend.get_buffer_text();
    println!("Full buffer text: '{full_text}'");

    // Execute text search
    println!(
        "Contains 'TestDevice': {}",
        backend.assert_contains_text("TestDevice")
    );
    println!(
        "Contains 'Android': {}",
        backend.assert_contains_text("Android")
    );
    println!(
        "Contains 'Devices': {}",
        backend.assert_contains_text("Devices")
    );
}
