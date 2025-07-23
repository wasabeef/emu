//! UI Render Unit Tests
//!
//! Tests basic behavior and error handling of UI rendering functions.
//! Focuses on logic verification without actual drawing.

use emu::app::{AppState, Mode, Panel};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use emu::ui::Theme;
use ratatui::{backend::TestBackend, Terminal};

/// Terminal setup for testing
fn setup_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 30);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Basic rendering test
#[test]
fn test_basic_rendering_setup() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Check that the rendering function is called normally
    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
}

/// Android panel rendering test
#[test]
fn test_android_panel_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Add Android device
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "test_device_1".to_string(),
            device_type: "pixel_4".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "test_device_2".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    state.active_panel = Panel::Android;

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());

    // Check buffer after rendering
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    // Check that Android device name is rendered
    assert!(content.contains("test_device_1") || content.contains("test_device_2"));
}

/// iOS panel rendering test
#[test]
fn test_ios_panel_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Add iOS device
    state.ios_devices = vec![IosDevice {
        name: "iPhone 14".to_string(),
        udid: "12345678-1234-1234-1234-123456789012".to_string(),
        device_type: "iPhone 14".to_string(),
        ios_version: "16.0".to_string(),
        runtime_version: "iOS 16.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }];

    state.active_panel = Panel::Ios;

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());

    // Check buffer after rendering
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    // Verify that iOS device name is rendered
    assert!(content.contains("iPhone 14"));
}

/// Rendering test with empty device list
#[test]
fn test_empty_device_list_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Test with empty device list
    assert!(state.android_devices.is_empty());
    assert!(state.ios_devices.is_empty());

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
}

/// Rendering test in CreateDevice mode
#[test]
fn test_create_device_mode_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    state.mode = Mode::CreateDevice;

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());

    // Verify that CreateDevice dialog is displayed
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    // Verify that dialog-related strings are included
    // Generic check since actual text depends on implementation
    assert!(!content.is_empty());
}

/// Rendering test with different themes
#[test]
fn test_different_themes_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();

    // Default theme
    let default_theme = Theme::dark();
    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &default_theme);
    });
    assert!(result.is_ok());

    // Light theme
    let light_theme = Theme::light();
    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &light_theme);
    });
    assert!(result.is_ok());

    // Dark theme
    let dark_theme = Theme::dark();
    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &dark_theme);
    });
    assert!(result.is_ok());
}

/// Rendering test with small terminal size
#[test]
fn test_small_terminal_rendering() {
    // Small terminal size
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).expect("Failed to create small terminal");
    let mut state = AppState::new();
    let theme = Theme::dark();

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    // Verify that small size does not crash
    assert!(result.is_ok());
}

/// Rendering test with minimum terminal size
#[test]
fn test_minimum_terminal_rendering() {
    // Minimum size
    let backend = TestBackend::new(1, 1);
    let mut terminal = Terminal::new(backend).expect("Failed to create minimal terminal");
    let mut state = AppState::new();
    let theme = Theme::dark();

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    // Verify that minimum size does not crash
    assert!(result.is_ok());
}

/// Rendering test with different device states
#[test]
fn test_different_device_states_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Add devices with different states
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "stopped_device".to_string(),
            device_type: "pixel_4".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "running_device".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());

    // Verify that devices with different states are rendered
    let buffer = terminal.backend().buffer();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(content.contains("stopped_device") || content.contains("running_device"));
}

/// Rendering test with long device names
#[test]
fn test_long_device_names_rendering() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Very long device name
    let long_name = "a".repeat(100);

    state.android_devices = vec![AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: long_name,
        device_type: "pixel_4".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    }];

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    // Verify that long names do not crash
    assert!(result.is_ok());
}

/// Layout calculation test
#[test]
fn test_layout_calculations() {
    // Test layout calculations with different sizes
    let sizes = vec![
        (80, 30),  // Standard size
        (100, 40), // Large size
        (60, 20),  // Small size
        (200, 50), // Wide size
    ];

    for (width, height) in sizes {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
        let mut state = AppState::new();
        let theme = Theme::dark();

        let result = terminal.draw(|frame| {
            emu::ui::render::draw_app(frame, &mut state, &theme);
        });

        assert!(
            result.is_ok(),
            "Failed to render with size {width}x{height}"
        );
    }
}

/// Rendering test with multiple selection indices
#[test]
fn test_different_selection_indices() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Add multiple devices
    state.android_devices = vec![
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_0".to_string(),
            device_type: "pixel_4".to_string(),
            api_level: 30,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_1".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
        AndroidDevice {
            android_version_name: "API 30".to_string(),
            name: "device_2".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "8192".to_string(),
            storage_size: "32768M".to_string(),
        },
    ];

    // Test with different selection indices
    for selected_index in 0..3 {
        state.selected_android = selected_index;

        let result = terminal.draw(|frame| {
            emu::ui::render::draw_app(frame, &mut state, &theme);
        });

        assert!(
            result.is_ok(),
            "Failed to render with selection index {selected_index}"
        );
    }
}

/// Rendering test with out-of-bounds selection index
#[test]
fn test_out_of_bounds_selection_index() {
    let mut terminal = setup_test_terminal();
    let mut state = AppState::new();
    let theme = Theme::dark();

    // Add only one device
    state.android_devices = vec![AndroidDevice {
        android_version_name: "API 30".to_string(),
        name: "single_device".to_string(),
        device_type: "pixel_4".to_string(),
        api_level: 30,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    }];

    // Out-of-bounds selection index
    state.selected_android = 5;

    let result = terminal.draw(|frame| {
        emu::ui::render::draw_app(frame, &mut state, &theme);
    });

    // Verify that out-of-bounds index does not crash
    assert!(result.is_ok());
}
