//! Smoke tests verifying the tests/support/ infrastructure works correctly.
//!
//! This is a standalone test binary that exercises the shared test
//! utilities (device factories, TestStateBuilder, assertions).

mod support;

use emu::app::state::{Mode, Panel};
use emu::models::DeviceStatus;
use support::*;

#[test]
fn test_device_factories() {
    let android = android_device("Pixel");
    assert_eq!(android.name, "Pixel");
    assert!(android.status.is_stopped());

    let running = android_device_with_status("Running", DeviceStatus::Running);
    assert!(running.status.is_running());
    assert!(running.is_running);

    let ios = ios_device("iPhone");
    assert_eq!(ios.name, "iPhone");
    assert!(ios.status.is_stopped());

    let ios_err = ios_device_with_status("Error", DeviceStatus::Error);
    assert!(ios_err.status.is_error());
}

#[test]
fn test_state_builder() {
    let state = TestStateBuilder::new()
        .with_android_devices(vec![android_device("Pixel_7")])
        .with_ios_devices(vec![ios_device("iPhone_15")])
        .in_mode(Mode::Normal)
        .on_panel(Panel::Android)
        .build();

    assert!(state.is_normal_mode());
    assert!(state.is_android_panel());
    assert_eq!(state.android_device_count(), 1);
    assert_eq!(state.ios_device_count(), 1);
}

#[test]
fn test_state_builder_escape_hatch() {
    let state = TestStateBuilder::new()
        .with_raw(|s| {
            s.fullscreen_logs = true;
        })
        .build();

    assert!(state.fullscreen_logs);
}

#[test]
fn test_predicate_assertions() {
    let state = TestStateBuilder::new()
        .in_mode(Mode::Help)
        .on_panel(Panel::Ios)
        .build();

    support::assertions::assert_mode(&state, Mode::Help);
    support::assertions::assert_panel(&state, Panel::Ios);
    support::assertions::assert_device_counts(&state, 0, 0);
}

#[test]
fn test_device_status_predicates() {
    let device = android_device_with_status("test", DeviceStatus::Starting);
    support::assertions::assert_status(
        &device.status,
        DeviceStatus::is_transitioning,
        "transitioning",
    );
}
