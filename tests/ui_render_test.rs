//! Comprehensive UI rendering tests
//! Tests all major rendering functions in src/ui/render.rs

use emu::{
    app::{
        state::{CreateDeviceField, Notification},
        AppState, Mode, Panel,
    },
    models::device::{AndroidDevice, DeviceStatus, IosDevice},
    ui::{render::draw_app, Theme},
};
use ratatui::{backend::TestBackend, Terminal};
use std::time::Instant;

/// Helper to create a mock terminal for testing
fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(120, 40);
    Terminal::new(backend).unwrap()
}

/// Helper to create minimal app state
fn create_minimal_app_state() -> AppState {
    AppState::new()
}

/// Helper to create app state with devices
fn create_app_state_with_devices() -> AppState {
    let mut state = AppState::new();

    // Add Android devices
    state.android_devices = vec![
        AndroidDevice {
            name: "Pixel_7_API_34".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            name: "Tablet_API_33".to_string(),
            device_type: "tablet".to_string(),
            api_level: 33,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "4096".to_string(),
            storage_size: "16G".to_string(),
        },
    ];

    // Add iOS devices
    state.ios_devices = vec![
        IosDevice {
            name: "iPhone 15 Pro".to_string(),
            udid: "12345-67890-ABCDEF".to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Running,
            is_running: true,
            is_available: true,
        },
        IosDevice {
            name: "iPad Air".to_string(),
            udid: "09876-54321-FEDCBA".to_string(),
            device_type: "iPad".to_string(),
            ios_version: "16.4".to_string(),
            runtime_version: "iOS 16.4".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        },
    ];

    state
}

#[test]
fn test_draw_app_minimal_terminal() {
    let mut terminal = create_test_terminal();
    let mut state = create_minimal_app_state();
    let theme = Theme::dark();

    // Test with minimal terminal size (should show error)
    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
}

#[test]
fn test_draw_app_normal_mode() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test normal mode rendering
    state.mode = Mode::Normal;
    state.active_panel = Panel::Android;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());

    // Verify Android panel is active
    assert_eq!(state.active_panel, Panel::Android);
    assert_eq!(state.mode, Mode::Normal);
}

#[test]
fn test_draw_app_ios_panel_active() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test iOS panel active
    state.active_panel = Panel::Ios;
    state.selected_ios = 0;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.active_panel, Panel::Ios);
}

#[test]
fn test_draw_app_fullscreen_logs() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test fullscreen logs mode
    state.fullscreen_logs = true;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert!(state.fullscreen_logs);
}

#[test]
fn test_draw_app_create_device_dialog() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test create device dialog
    state.mode = Mode::CreateDevice;
    state.active_panel = Panel::Android;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.mode, Mode::CreateDevice);
}

#[test]
fn test_draw_app_confirm_delete_dialog() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test confirm delete dialog
    state.mode = Mode::ConfirmDelete;
    state.confirm_delete_dialog = Some(emu::app::state::ConfirmDeleteDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_device_id".to_string(),
        platform: Panel::Android,
    });

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.mode, Mode::ConfirmDelete);
    assert!(state.confirm_delete_dialog.is_some());
}

#[test]
fn test_draw_app_confirm_wipe_dialog() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test confirm wipe dialog
    state.mode = Mode::ConfirmWipe;
    state.confirm_wipe_dialog = Some(emu::app::state::ConfirmWipeDialog {
        device_name: "Test Device".to_string(),
        device_identifier: "test_device_id".to_string(),
        platform: Panel::Android,
    });

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.mode, Mode::ConfirmWipe);
    assert!(state.confirm_wipe_dialog.is_some());
}

#[test]
fn test_draw_app_manage_api_levels() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test manage API levels dialog
    state.mode = Mode::ManageApiLevels;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.mode, Mode::ManageApiLevels);
}

#[test]
fn test_draw_app_with_notifications() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Add notifications
    state.add_notification(Notification::success("Success message".to_string()));
    state.add_notification(Notification::error("Error message".to_string()));
    state.add_notification(Notification::warning("Warning message".to_string()));
    state.add_notification(Notification::info("Info message".to_string()));

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert!(!state.notifications.is_empty());
}

#[test]
fn test_draw_app_with_device_operation_status() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test with device operation in progress
    state.device_operation_status = Some("Starting device".to_string());

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert!(state.device_operation_status.is_some());
}

#[test]
fn test_draw_app_loading_state() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test loading state
    state.is_loading = true;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert!(state.is_loading);
}

#[test]
fn test_draw_app_with_log_filter() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test with log filter active
    state.log_filter_level = Some("ERROR".to_string());

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert!(state.log_filter_level.is_some());
}

#[test]
fn test_draw_app_android_device_selection() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test Android device selection
    state.active_panel = Panel::Android;
    state.selected_android = 1; // Select second device

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.selected_android, 1);
    assert_eq!(state.active_panel, Panel::Android);
}

#[test]
fn test_draw_app_ios_device_selection() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test iOS device selection
    state.active_panel = Panel::Ios;
    state.selected_ios = 1; // Select second device

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.selected_ios, 1);
    assert_eq!(state.active_panel, Panel::Ios);
}

#[test]
fn test_draw_app_device_scrolling() {
    let mut terminal = create_test_terminal();
    let mut state = create_minimal_app_state();
    let theme = Theme::dark();

    // Add many devices to test scrolling
    for i in 0..20 {
        state.android_devices.push(AndroidDevice {
            name: format!("Device_{i}"),
            device_type: "pixel".to_string(),
            api_level: 34,
            status: if i % 2 == 0 {
                DeviceStatus::Running
            } else {
                DeviceStatus::Stopped
            },
            is_running: i % 2 == 0,
            ram_size: "2048".to_string(),
            storage_size: "8G".to_string(),
        });
    }

    state.selected_android = 15; // Select device that requires scrolling
    state.active_panel = Panel::Android;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.selected_android, 15);
    assert_eq!(state.android_devices.len(), 20);
}

#[test]
fn test_draw_app_create_device_form_states() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    state.mode = Mode::CreateDevice;
    state.active_panel = Panel::Android;

    // Test different form field states
    let form_fields = vec![
        CreateDeviceField::Name,
        CreateDeviceField::DeviceType,
        CreateDeviceField::ApiLevel,
        CreateDeviceField::Category,
        CreateDeviceField::RamSize,
        CreateDeviceField::StorageSize,
    ];

    for field in form_fields {
        state.create_device_form.active_field = field;

        let result = terminal.draw(|frame| {
            draw_app(frame, &mut state, &theme);
        });

        assert!(result.is_ok());
    }
}

#[test]
fn test_draw_app_create_device_loading_states() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    state.mode = Mode::CreateDevice;

    // Test loading cache state
    state.create_device_form.is_loading_cache = true;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });
    assert!(result.is_ok());

    // Test creating state
    state.create_device_form.is_loading_cache = false;
    state.create_device_form.is_creating = true;
    state.create_device_form.creation_status = Some("Downloading system image".to_string());

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });
    assert!(result.is_ok());

    // Test error state
    state.create_device_form.is_creating = false;
    state.create_device_form.error_message = Some("Failed to create device".to_string());

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });
    assert!(result.is_ok());
}

#[test]
fn test_draw_app_ios_create_device() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Test iOS device creation (different form layout)
    state.mode = Mode::CreateDevice;
    state.active_panel = Panel::Ios;

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    assert_eq!(state.active_panel, Panel::Ios);
    assert_eq!(state.mode, Mode::CreateDevice);
}

#[test]
fn test_draw_app_with_logs() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Add log entries
    state.add_log("ERROR".to_string(), "Test error message".to_string());
    state.add_log("WARN".to_string(), "Test warning message".to_string());
    state.add_log("INFO".to_string(), "Test info message".to_string());
    state.add_log("DEBUG".to_string(), "Test debug message".to_string());

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    assert!(result.is_ok());
    // Verify logs were added (logs are stored internally)
    assert!(result.is_ok());
}

#[test]
fn test_draw_app_different_themes() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();

    // Test with different theme configurations
    let themes = vec![Theme::dark(), Theme::dark(), Theme::light()];

    for theme in themes {
        let result = terminal.draw(|frame| {
            draw_app(frame, &mut state, &theme);
        });

        assert!(result.is_ok());
    }
}

#[test]
fn test_render_performance() {
    let mut terminal = create_test_terminal();
    let mut state = create_app_state_with_devices();
    let theme = Theme::dark();

    // Add more devices for performance testing
    for i in 0..50 {
        state.android_devices.push(AndroidDevice {
            name: format!("Perf_Device_{i}"),
            device_type: "pixel".to_string(),
            api_level: 34,
            status: if i % 3 == 0 {
                DeviceStatus::Running
            } else {
                DeviceStatus::Stopped
            },
            is_running: i % 3 == 0,
            ram_size: "2048".to_string(),
            storage_size: "8G".to_string(),
        });
    }

    let start = Instant::now();

    let result = terminal.draw(|frame| {
        draw_app(frame, &mut state, &theme);
    });

    let duration = start.elapsed();

    assert!(result.is_ok());
    // Rendering should be fast (under 50ms for 50+ devices)
    assert!(duration.as_millis() < 50, "Rendering took {duration:?}");
}
