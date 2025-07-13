//! Common test utilities for integration tests
//!
//! This module provides shared test helpers for creating mock applications,
//! terminals, and test scenarios without requiring any real emulators or simulators.

#[cfg(feature = "test-utils")]
use emu::managers::mock::{MockDeviceManager, ScenarioBuilder};
use emu::app::state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Create a test AppState with MockDeviceManager
#[cfg(feature = "test-utils")]
pub async fn setup_test_app(android_count: usize, ios_count: usize) -> Arc<Mutex<AppState>> {
    let mock_manager = Arc::new(
        ScenarioBuilder::new()
            .with_android_devices(android_count)
            .with_ios_devices(ios_count)
            .build()
    );
    
    let app_state = AppState::new_with_mock_manager(mock_manager).await
        .expect("Failed to create test app state");
    
    Arc::new(Mutex::new(app_state))
}

/// Create a test AppState with specific device configuration
#[cfg(feature = "test-utils")]
pub async fn setup_test_app_with_scenario(scenario: MockDeviceManager) -> Arc<Mutex<AppState>> {
    let mock_manager = Arc::new(scenario);
    
    let app_state = AppState::new_with_mock_manager(mock_manager).await
        .expect("Failed to create test app state with scenario");
    
    Arc::new(Mutex::new(app_state))
}

/// Create a test AppState with custom device states
#[cfg(feature = "test-utils")]
pub async fn setup_test_app_with_states(
    android_online: usize,
    android_offline: usize,
    ios_online: usize,
    ios_offline: usize,
) -> Arc<Mutex<AppState>> {
    let mut scenario = ScenarioBuilder::new();
    
    // Add Android devices with specified states
    for i in 0..android_online {
        scenario = scenario.with_android_device_in_state(
            &format!("android_online_{i}"), 
            emu::models::DeviceStatus::Online
        );
    }
    
    for i in 0..android_offline {
        scenario = scenario.with_android_device_in_state(
            &format!("android_offline_{i}"), 
            emu::models::DeviceStatus::Offline
        );
    }
    
    // Add iOS devices with specified states
    for i in 0..ios_online {
        scenario = scenario.with_ios_device_in_state(
            &format!("ios_online_{i}"), 
            emu::models::DeviceStatus::Online
        );
    }
    
    for i in 0..ios_offline {
        scenario = scenario.with_ios_device_in_state(
            &format!("ios_offline_{i}"), 
            emu::models::DeviceStatus::Offline
        );
    }
    
    setup_test_app_with_scenario(scenario.build()).await
}

/// Test helper for creating empty app state
pub async fn setup_empty_app() -> Arc<Mutex<AppState>> {
    #[cfg(feature = "test-utils")]
    {
        setup_test_app(0, 0).await
    }
    #[cfg(not(feature = "test-utils"))]
    {
        // Fallback for non-test builds
        let app_state = AppState::new().await
            .expect("Failed to create empty app state");
        Arc::new(Mutex::new(app_state))
    }
}

/// Simulate user navigation and verify state consistency
pub async fn simulate_navigation_test(
    app: Arc<Mutex<AppState>>, 
    actions: Vec<NavigationAction>
) -> Result<(), String> {
    for action in actions {
        let mut app_lock = app.lock().await;
        match action {
            NavigationAction::SwitchToAndroid => {
                app_lock.set_current_panel(emu::models::Panel::Android);
            }
            NavigationAction::SwitchToIos => {
                app_lock.set_current_panel(emu::models::Panel::Ios);
            }
            NavigationAction::SwitchToDetails => {
                app_lock.set_current_panel(emu::models::Panel::Details);
            }
            NavigationAction::NextDevice => {
                app_lock.next_device();
            }
            NavigationAction::PrevDevice => {
                app_lock.previous_device();
            }
            NavigationAction::SelectDevice(index) => {
                app_lock.set_selected_device_index(index);
            }
            NavigationAction::ShowCreateModal => {
                app_lock.set_showing_create_device_modal(true);
            }
            NavigationAction::HideModal => {
                app_lock.close_modal();
            }
        }
    }
    
    // Verify state consistency
    let app_lock = app.lock().await;
    let current_panel = app_lock.get_current_panel();
    let selected_index = app_lock.get_selected_device_index();
    
    // Basic consistency checks
    match current_panel {
        emu::models::Panel::Android => {
            let android_devices = app_lock.get_android_devices();
            if !android_devices.is_empty() && selected_index >= android_devices.len() {
                return Err(format!(
                    "Selected index {selected_index} out of bounds for {len} Android devices",
                    len = android_devices.len()
                ));
            }
        }
        emu::models::Panel::Ios => {
            let ios_devices = app_lock.get_ios_devices();
            if !ios_devices.is_empty() && selected_index >= ios_devices.len() {
                return Err(format!(
                    "Selected index {selected_index} out of bounds for {len} iOS devices",
                    len = ios_devices.len()
                ));
            }
        }
        emu::models::Panel::Details => {
            // Details panel doesn't use device selection
        }
    }
    
    Ok(())
}

/// Navigation actions for testing
#[derive(Debug, Clone)]
pub enum NavigationAction {
    SwitchToAndroid,
    SwitchToIos, 
    SwitchToDetails,
    NextDevice,
    PrevDevice,
    SelectDevice(usize),
    ShowCreateModal,
    HideModal,
}

/// Create test terminal dimensions for UI testing
pub fn get_test_terminal_size() -> (u16, u16) {
    (80, 30) // Standard test terminal size
}

/// Calculate expected panel widths for test terminal
pub fn calculate_test_panel_widths() -> (u16, u16, u16) {
    let (width, _) = get_test_terminal_size();
    let android_width = width * 30 / 100; // 30%
    let ios_width = width * 30 / 100;     // 30%
    let details_width = width - android_width - ios_width; // Remaining 40%
    
    (android_width, ios_width, details_width)
}

/// Test helper for verifying device counts
pub async fn verify_device_counts(
    app: Arc<Mutex<AppState>>,
    expected_android: usize,
    expected_ios: usize,
) -> Result<(), String> {
    let app_lock = app.lock().await;
    let android_count = app_lock.get_android_devices().len();
    let ios_count = app_lock.get_ios_devices().len();
    
    if android_count != expected_android {
        return Err(format!(
            "Expected {expected_android} Android devices, found {android_count}"
        ));
    }
    
    if ios_count != expected_ios {
        return Err(format!(
            "Expected {expected_ios} iOS devices, found {ios_count}"
        ));
    }
    
    Ok(())
}

/// Test helper for verifying panel layout calculations
pub fn verify_panel_layout(terminal_width: u16) -> Result<(), String> {
    let (android_width, ios_width, details_width) = if terminal_width > 0 {
        let android = terminal_width * 30 / 100;
        let ios = terminal_width * 30 / 100;
        let details = terminal_width - android - ios;
        (android, ios, details)
    } else {
        return Err("Terminal width must be greater than 0".to_string());
    };
    
    let total_width = android_width + ios_width + details_width;
    
    if total_width != terminal_width {
        return Err(format!(
            "Panel widths don't add up: {android_width} + {ios_width} + {details_width} = {total_width}, expected {terminal_width}"
        ));
    }
    
    // Verify minimum panel widths
    if android_width < 5 || ios_width < 5 || details_width < 5 {
        return Err(format!(
            "Panel widths too small: Android={android_width}, iOS={ios_width}, Details={details_width}"
        ));
    }
    
    Ok(())
}

/// Test configuration for various scenarios
pub struct TestConfig {
    pub android_devices: usize,
    pub ios_devices: usize,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub enable_delays: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            android_devices: 3,
            ios_devices: 2,
            terminal_width: 80,
            terminal_height: 30,
            enable_delays: false,
        }
    }
}

impl TestConfig {
    pub fn with_devices(android: usize, ios: usize) -> Self {
        Self {
            android_devices: android,
            ios_devices: ios,
            ..Default::default()
        }
    }
    
    pub fn with_terminal_size(width: u16, height: u16) -> Self {
        Self {
            terminal_width: width,
            terminal_height: height,
            ..Default::default()
        }
    }
    
    pub fn with_delays(enabled: bool) -> Self {
        Self {
            enable_delays: enabled,
            ..Default::default()
        }
    }
}

/// Create app with test configuration
#[cfg(feature = "test-utils")]
pub async fn setup_app_with_config(config: TestConfig) -> Arc<Mutex<AppState>> {
    let mut scenario_builder = ScenarioBuilder::new()
        .with_android_devices(config.android_devices)
        .with_ios_devices(config.ios_devices);
    
    if config.enable_delays {
        scenario_builder = scenario_builder.with_operation_delay(std::time::Duration::from_millis(10));
    }
    
    setup_test_app_with_scenario(scenario_builder.build()).await
}