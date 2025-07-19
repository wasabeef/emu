//! Common assertion helpers for Emu tests
//!
//! This module provides reusable assertion functions to reduce code duplication
//! across test files while maintaining clear test readability.

use anyhow::Result;
use emu::app::state::AppState;
use emu::app::{FocusedPanel, Mode, Panel};
use emu::models::DeviceStatus;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Assert app state properties with clear error messages
#[allow(dead_code)]
pub async fn assert_app_state(
    app: &Arc<Mutex<AppState>>,
    expected_panel: Panel,
    expected_focus: FocusedPanel,
    expected_mode: Mode,
) -> Result<()> {
    let state = app.lock().await;

    if state.active_panel != expected_panel {
        return Err(anyhow::anyhow!(
            "Expected active panel {:?}, found {:?}",
            expected_panel,
            state.active_panel
        ));
    }

    if state.focused_panel != expected_focus {
        return Err(anyhow::anyhow!(
            "Expected focused panel {:?}, found {:?}",
            expected_focus,
            state.focused_panel
        ));
    }

    if state.mode != expected_mode {
        return Err(anyhow::anyhow!(
            "Expected mode {:?}, found {:?}",
            expected_mode,
            state.mode
        ));
    }

    Ok(())
}

/// Assert device counts with descriptive error messages
#[allow(dead_code)]
pub async fn assert_device_counts(
    app: &Arc<Mutex<AppState>>,
    expected_android: usize,
    expected_ios: usize,
) -> Result<()> {
    let state = app.lock().await;
    let android_count = state.android_devices.len();
    let ios_count = state.ios_devices.len();

    if android_count != expected_android {
        return Err(anyhow::anyhow!(
            "Expected {} Android devices, found {}. Devices: {:?}",
            expected_android,
            android_count,
            state
                .android_devices
                .iter()
                .map(|d| &d.name)
                .collect::<Vec<_>>()
        ));
    }

    if ios_count != expected_ios {
        return Err(anyhow::anyhow!(
            "Expected {} iOS devices, found {}. Devices: {:?}",
            expected_ios,
            ios_count,
            state
                .ios_devices
                .iter()
                .map(|d| &d.name)
                .collect::<Vec<_>>()
        ));
    }

    Ok(())
}

/// Assert device status with detailed error information
#[allow(dead_code)]
pub async fn assert_device_status(
    app: &Arc<Mutex<AppState>>,
    device_name: &str,
    expected_status: DeviceStatus,
) -> Result<()> {
    let state = app.lock().await;

    // Check Android devices
    for device in &state.android_devices {
        if device.name == device_name {
            if device.status != expected_status {
                return Err(anyhow::anyhow!(
                    "Android device '{}' expected status {:?}, found {:?}",
                    device_name,
                    expected_status,
                    device.status
                ));
            }
            return Ok(());
        }
    }

    // Check iOS devices
    for device in &state.ios_devices {
        if device.name == device_name {
            if device.status != expected_status {
                return Err(anyhow::anyhow!(
                    "iOS device '{}' expected status {:?}, found {:?}",
                    device_name,
                    expected_status,
                    device.status
                ));
            }
            return Ok(());
        }
    }

    Err(anyhow::anyhow!(
        "Device '{}' not found in either Android or iOS devices",
        device_name
    ))
}

/// Assert modal state with clear error messages
#[allow(dead_code)]
pub async fn assert_modal_state(app: &Arc<Mutex<AppState>>, expected_mode: Mode) -> Result<()> {
    let state = app.lock().await;

    if state.mode != expected_mode {
        return Err(anyhow::anyhow!(
            "Expected mode: {:?}, found: {:?}",
            expected_mode,
            state.mode
        ));
    }

    // Simplified modal checking - just check mode

    Ok(())
}

/// Assert selection state with bounds checking
#[allow(dead_code)]
pub async fn assert_selection_state(
    app: &Arc<Mutex<AppState>>,
    expected_index: Option<usize>,
) -> Result<()> {
    let state = app.lock().await;

    match expected_index {
        Some(index) => {
            let current_index = match state.active_panel {
                Panel::Android => state.selected_android,
                Panel::Ios => state.selected_ios,
            };

            if current_index != index {
                return Err(anyhow::anyhow!(
                    "Expected selected index {}, found {}",
                    index,
                    current_index
                ));
            }

            // Validate index is within bounds for current panel
            match state.active_panel {
                Panel::Android => {
                    if index >= state.android_devices.len() {
                        return Err(anyhow::anyhow!(
                            "Selected index {} out of bounds for {} Android devices",
                            index,
                            state.android_devices.len()
                        ));
                    }
                }
                Panel::Ios => {
                    if index >= state.ios_devices.len() {
                        return Err(anyhow::anyhow!(
                            "Selected index {} out of bounds for {} iOS devices",
                            index,
                            state.ios_devices.len()
                        ));
                    }
                }
            }
        }
        None => {
            // For cases where we expect no valid selection or want to skip validation
        }
    }

    Ok(())
}

/// Macro for quick app state assertions
#[macro_export]
macro_rules! assert_app_state_quick {
    ($app:expr, $panel:expr, $focus:expr, $mode:expr) => {
        $crate::common::assertions::assert_app_state($app, $panel, $focus, $mode)
            .await
            .map_err(|e| panic!("App state assertion failed: {}", e))?;
    };
}

/// Macro for quick device count assertions
#[macro_export]
macro_rules! assert_devices {
    ($app:expr, android: $android:expr, ios: $ios:expr) => {
        $crate::common::assertions::assert_device_counts($app, $android, $ios)
            .await
            .map_err(|e| panic!("Device count assertion failed: {}", e))?;
    };
}

/// Macro for quick modal state assertions
#[macro_export]
macro_rules! assert_modals {
    ($app:expr, create: $create:expr, delete: $delete:expr, wipe: $wipe:expr) => {
        $crate::common::assertions::assert_modal_state($app, $create, $delete, $wipe)
            .await
            .map_err(|e| panic!("Modal state assertion failed: {}", e))?;
    };
}
