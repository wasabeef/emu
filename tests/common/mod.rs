//! Common test utilities for integration tests
//!
//! This module provides shared test helpers for creating mock applications,
//! terminals, and test scenarios without requiring any real emulators or simulators.

pub mod assertions;
pub mod helpers;

#[cfg(feature = "test-utils")]
use emu::app::state::AppState;
#[cfg(feature = "test-utils")]
use std::sync::Arc;
#[cfg(feature = "test-utils")]
use tokio::sync::Mutex;

/// Create a test AppState with MockDeviceManager
#[cfg(feature = "test-utils")]
#[allow(dead_code)]
pub async fn setup_test_app(_android_count: usize, _ios_count: usize) -> Arc<Mutex<AppState>> {
    // For now, return a basic AppState until we implement MockDeviceManager integration
    let app_state = AppState::new();
    Arc::new(Mutex::new(app_state))
}

/// Create a test AppState with specific device configuration
#[cfg(feature = "test-utils")]
#[allow(dead_code)]
pub async fn setup_test_app_with_scenario() -> Arc<Mutex<AppState>> {
    // For now, return a basic AppState until we implement MockDeviceManager integration
    let app_state = AppState::new();
    Arc::new(Mutex::new(app_state))
}
