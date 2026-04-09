//! Custom assertion helpers for integration tests.
//!
//! All assertions use predicate methods (Phase 1) so they survive
//! field renames and enum variant changes.

use emu::app::state::{AppState, Mode, Panel};
use emu::models::DeviceStatus;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Synchronous assertions (for direct AppState access) ──────────

/// Assert the mode via predicate, not direct enum comparison.
pub fn assert_mode(state: &AppState, expected: Mode) {
    let actual_matches = match expected {
        Mode::Normal => state.is_normal_mode(),
        Mode::CreateDevice => state.is_create_mode(),
        Mode::Help => state.is_help_mode(),
        Mode::ConfirmDelete => state.is_confirm_delete_mode(),
        Mode::ConfirmWipe => state.is_confirm_wipe_mode(),
        Mode::ManageApiLevels => state.is_api_level_mode(),
    };
    assert!(
        actual_matches,
        "Expected mode {expected:?}, but predicate returned false"
    );
}

/// Assert active panel via predicate.
pub fn assert_panel(state: &AppState, expected: Panel) {
    let actual_matches = match expected {
        Panel::Android => state.is_android_panel(),
        Panel::Ios => state.is_ios_panel(),
    };
    assert!(
        actual_matches,
        "Expected panel {expected:?}, but predicate returned false"
    );
}

/// Assert device counts via accessor methods.
pub fn assert_device_counts(state: &AppState, android: usize, ios: usize) {
    assert_eq!(
        state.android_device_count(),
        android,
        "Android device count mismatch"
    );
    assert_eq!(state.ios_device_count(), ios, "iOS device count mismatch");
}

/// Assert a device status via predicate method.
pub fn assert_status(status: &DeviceStatus, check: fn(&DeviceStatus) -> bool, label: &str) {
    assert!(check(status), "Expected device status: {label}");
}

// ── Async assertions (for Arc<Mutex<AppState>>) ──────────────────

/// Assert mode on a locked AppState.
pub async fn assert_locked_mode(app: &Arc<Mutex<AppState>>, expected: Mode) {
    let state = app.lock().await;
    assert_mode(&state, expected);
}

/// Assert panel on a locked AppState.
pub async fn assert_locked_panel(app: &Arc<Mutex<AppState>>, expected: Panel) {
    let state = app.lock().await;
    assert_panel(&state, expected);
}

/// Assert device counts on a locked AppState.
pub async fn assert_locked_device_counts(app: &Arc<Mutex<AppState>>, android: usize, ios: usize) {
    let state = app.lock().await;
    assert_device_counts(&state, android, ios);
}
