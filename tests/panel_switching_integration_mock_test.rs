//! Panel switching integration tests with MockDeviceManager
//!
//! Tests the comprehensive behavior of panel switching combined with
//! actual device operations using mock managers.

use emu::app::state::{AppState, DeviceDetails, Panel};
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::common::DeviceManager;
#[cfg(any(test, feature = "test-utils"))]
use emu::managers::{common::DeviceConfig, mock::MockDeviceManager};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Test panel switching with real device operations
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_panel_switching_with_device_operations() {
    println!("=== PANEL SWITCHING WITH DEVICE OPERATIONS TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();
    let mut state = AppState::new();

    // Phase 1: Create devices on both platforms
    for i in 0..3 {
        let android_config = DeviceConfig::new(
            format!("PanelTestAndroid{i}"),
            "pixel_8".to_string(),
            "34".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("PanelTestiOS{i}"),
            "iPhone15,2".to_string(),
            "17.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Phase 2: Sync state with mock device data
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    state.android_devices = android_devices
        .into_iter()
        .map(|device| AndroidDevice {
            name: device.name().to_string(),
            device_type: "phone".to_string(),
            api_level: 34,
            status: *device.status(),
            is_running: device.is_running(),
            ram_size: "4096".to_string(),
            storage_size: "16384".to_string(),
        })
        .collect();

    state.ios_devices = ios_devices
        .into_iter()
        .map(|device| IosDevice {
            name: device.name().to_string(),
            udid: device.id().to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: *device.status(),
            is_running: device.is_running(),
            is_available: true,
        })
        .collect();

    // Phase 3: Test operations on Android panel
    assert_eq!(state.active_panel, Panel::Android);

    // Find and select a test device
    let test_android_index = state
        .android_devices
        .iter()
        .position(|d| d.name.starts_with("PanelTestAndroid"))
        .expect("Test Android device not found");
    state.selected_android = test_android_index;

    let selected_android_name = state.android_devices[state.selected_android].name.clone();

    // Start the selected Android device
    android_manager
        .start_device(&selected_android_name)
        .await
        .expect("Failed to start Android device");

    // Update state to reflect change
    state.android_devices[state.selected_android].status = DeviceStatus::Running;
    state.android_devices[state.selected_android].is_running = true;

    // Phase 4: Switch to iOS panel and perform operations
    state.active_panel = Panel::Ios;
    assert_eq!(state.active_panel, Panel::Ios);

    // Find and select an iOS test device
    let test_ios_index = state
        .ios_devices
        .iter()
        .position(|d| d.name.starts_with("PanelTestiOS"))
        .expect("Test iOS device not found");
    state.selected_ios = test_ios_index;

    let selected_ios_name = state.ios_devices[state.selected_ios].name.clone();

    // Start the selected iOS device
    ios_manager
        .start_device(&selected_ios_name)
        .await
        .expect("Failed to start iOS device");

    // Update state to reflect change
    state.ios_devices[state.selected_ios].status = DeviceStatus::Running;
    state.ios_devices[state.selected_ios].is_running = true;

    // Phase 5: Switch back to Android and verify state
    state.active_panel = Panel::Android;
    assert_eq!(state.active_panel, Panel::Android);
    assert!(state.android_devices[state.selected_android].is_running);

    // Phase 6: Verify concurrent operations across panels
    let android_running_count = state
        .android_devices
        .iter()
        .filter(|d| d.is_running)
        .count();
    state.active_panel = Panel::Ios;
    let ios_running_count = state.ios_devices.iter().filter(|d| d.is_running).count();

    assert_eq!(android_running_count, 1);
    assert_eq!(ios_running_count, 1);

    println!(
        "Android devices: {android_len}, iOS devices: {ios_len}",
        android_len = state.android_devices.len(),
        ios_len = state.ios_devices.len()
    );
    println!("Running - Android: {android_running_count}, iOS: {ios_running_count}");
    println!("✅ Panel switching with device operations completed successfully!");
}

/// Test rapid panel switching under device load
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_rapid_panel_switching_under_load() {
    println!("=== RAPID PANEL SWITCHING UNDER LOAD TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();
    let state = Arc::new(Mutex::new(AppState::new()));

    // Create many devices to simulate load
    for i in 0..20 {
        let android_config = DeviceConfig::new(
            format!("LoadTestAndroid{i}"),
            "pixel_7".to_string(),
            "33".to_string(),
        );
        android_manager
            .create_device(&android_config)
            .await
            .expect("Failed to create Android device");

        let ios_config = DeviceConfig::new(
            format!("LoadTestiOS{i}"),
            "iPhone14,3".to_string(),
            "16.0".to_string(),
        );
        ios_manager
            .create_device(&ios_config)
            .await
            .expect("Failed to create iOS device");
    }

    // Sync large device lists to state
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    {
        let mut state_lock = state.lock().await;
        state_lock.android_devices = android_devices
            .into_iter()
            .map(|device| AndroidDevice {
                name: device.name().to_string(),
                device_type: "phone".to_string(),
                api_level: 33,
                status: *device.status(),
                is_running: device.is_running(),
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
            })
            .collect();

        state_lock.ios_devices = ios_devices
            .into_iter()
            .map(|device| IosDevice {
                name: device.name().to_string(),
                udid: device.id().to_string(),
                device_type: "iPhone".to_string(),
                ios_version: "16.0".to_string(),
                runtime_version: "iOS 16.0".to_string(),
                status: *device.status(),
                is_running: device.is_running(),
                is_available: true,
            })
            .collect();
    }

    // Test rapid panel switching with device operations
    let start_time = std::time::Instant::now();

    for i in 0..50 {
        {
            let mut state_lock = state.lock().await;
            state_lock.active_panel = if i % 2 == 0 {
                Panel::Android
            } else {
                Panel::Ios
            };

            // Simulate device selection changes during panel switching
            if state_lock.active_panel == Panel::Android && !state_lock.android_devices.is_empty() {
                state_lock.selected_android = i % state_lock.android_devices.len();
            } else if state_lock.active_panel == Panel::Ios && !state_lock.ios_devices.is_empty() {
                state_lock.selected_ios = i % state_lock.ios_devices.len();
            }
        }

        // Simulate concurrent device operation
        if i % 10 == 0 {
            let _device_name = format!("LoadTestAndroid{device_index}", device_index = i % 20);
            let _ = android_manager.list_devices().await; // Simulated background refresh
        }

        // Small delay to prevent overwhelming
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    let elapsed = start_time.elapsed();

    // Verify final state consistency
    {
        let state_lock = state.lock().await;
        assert_eq!(state_lock.android_devices.len(), 22); // 2 default + 20 created
        assert_eq!(state_lock.ios_devices.len(), 22); // 2 default + 20 created
        assert!(state_lock.selected_android < state_lock.android_devices.len());
        assert!(state_lock.selected_ios < state_lock.ios_devices.len());
    }

    // Performance assertion
    assert!(
        elapsed.as_millis() < 1000,
        "Rapid panel switching under load took too long: {elapsed:?}"
    );

    println!(
        "Completed 50 panel switches with {device_count} devices per platform in: {elapsed:?}",
        device_count = 22
    );
    println!("✅ Rapid panel switching under load completed successfully!");
}

/// Test panel switching with log streaming coordination
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_panel_switching_log_streaming_coordination() {
    println!("=== PANEL SWITCHING LOG STREAMING COORDINATION TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();
    let mut state = AppState::new();

    // Create test devices and start them to enable logging
    let android_config = DeviceConfig::new(
        "LogTestAndroid".to_string(),
        "pixel_8".to_string(),
        "34".to_string(),
    );
    android_manager
        .create_device(&android_config)
        .await
        .expect("Failed to create Android device");
    android_manager
        .start_device("LogTestAndroid")
        .await
        .expect("Failed to start Android device");

    let ios_config = DeviceConfig::new(
        "LogTestiOS".to_string(),
        "iPhone15,3".to_string(),
        "17.0".to_string(),
    );
    ios_manager
        .create_device(&ios_config)
        .await
        .expect("Failed to create iOS device");
    ios_manager
        .start_device("LogTestiOS")
        .await
        .expect("Failed to start iOS device");

    // Phase 1: Start with Android panel and simulate log streaming
    assert_eq!(state.active_panel, Panel::Android);
    state.current_log_device = Some((Panel::Android, "LogTestAndroid".to_string()));

    // Simulate log entries from Android device
    state.add_log("INFO".to_string(), "Android device started".to_string());
    state.add_log(
        "DEBUG".to_string(),
        "Android initialization complete".to_string(),
    );
    assert_eq!(state.device_logs.len(), 2);

    // Phase 2: Switch to iOS panel - should clear logs and update log device
    state.active_panel = Panel::Ios;

    // Simulate log clearing behavior that happens in real app during panel switch
    if state
        .current_log_device
        .as_ref()
        .is_some_and(|(panel, _)| *panel != state.active_panel)
    {
        state.clear_logs();
        state.current_log_device = None;
    }

    assert!(state.device_logs.is_empty());
    assert!(state.current_log_device.is_none());

    // Phase 3: Start new log stream for iOS device
    state.current_log_device = Some((Panel::Ios, "LogTestiOS".to_string()));
    state.add_log("INFO".to_string(), "iOS simulator started".to_string());
    state.add_log("WARN".to_string(), "iOS memory warning".to_string());
    assert_eq!(state.device_logs.len(), 2);

    // Phase 4: Rapid panel switching to test log coordination robustness
    for i in 0..10 {
        let target_panel = if i % 2 == 0 {
            Panel::Android
        } else {
            Panel::Ios
        };
        state.active_panel = target_panel;

        // Simulate log clearing on panel mismatch
        if state
            .current_log_device
            .as_ref()
            .is_some_and(|(panel, _)| *panel != state.active_panel)
        {
            state.clear_logs();
            state.current_log_device = None;
        }

        // Restart appropriate log stream
        match state.active_panel {
            Panel::Android => {
                state.current_log_device = Some((Panel::Android, "LogTestAndroid".to_string()));
                state.add_log("INFO".to_string(), format!("Android log entry {i}"));
            }
            Panel::Ios => {
                state.current_log_device = Some((Panel::Ios, "LogTestiOS".to_string()));
                state.add_log("INFO".to_string(), format!("iOS log entry {i}"));
            }
        }
    }

    // Phase 5: Verify final log state consistency
    assert!(!state.device_logs.is_empty());
    if let Some((panel, device)) = &state.current_log_device {
        assert_eq!(*panel, state.active_panel);
        match state.active_panel {
            Panel::Android => assert_eq!(device, "LogTestAndroid"),
            Panel::Ios => assert_eq!(device, "LogTestiOS"),
        }
    }

    println!(
        "Final log entries: {log_count}",
        log_count = state.device_logs.len()
    );
    println!(
        "Final log device: {current_log_device:?}",
        current_log_device = state.current_log_device
    );
    println!("✅ Panel switching log streaming coordination completed successfully!");
}

/// Test panel switching with device details updates
#[cfg(any(test, feature = "test-utils"))]
#[tokio::test]
async fn test_panel_switching_device_details_updates() {
    println!("=== PANEL SWITCHING DEVICE DETAILS UPDATES TEST ===");

    let android_manager = MockDeviceManager::new_android();
    let ios_manager = MockDeviceManager::new_ios();
    let mut state = AppState::new();

    // Create and configure test devices
    let android_config = DeviceConfig::new(
        "DetailTestAndroid".to_string(),
        "pixel_8_pro".to_string(),
        "34".to_string(),
    );
    android_manager
        .create_device(&android_config)
        .await
        .expect("Failed to create Android device");

    let ios_config = DeviceConfig::new(
        "DetailTestiOS".to_string(),
        "iPhone15,3".to_string(),
        "17.0".to_string(),
    );
    ios_manager
        .create_device(&ios_config)
        .await
        .expect("Failed to create iOS device");

    // Sync devices to state
    let android_devices = android_manager
        .list_devices()
        .await
        .expect("Failed to list Android devices");
    let ios_devices = ios_manager
        .list_devices()
        .await
        .expect("Failed to list iOS devices");

    state.android_devices = android_devices
        .into_iter()
        .map(|device| AndroidDevice {
            name: device.name().to_string(),
            device_type: "phone".to_string(),
            api_level: 34,
            status: *device.status(),
            is_running: device.is_running(),
            ram_size: "8192".to_string(),
            storage_size: "32768".to_string(),
        })
        .collect();

    state.ios_devices = ios_devices
        .into_iter()
        .map(|device| IosDevice {
            name: device.name().to_string(),
            udid: device.id().to_string(),
            device_type: "iPhone".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: *device.status(),
            is_running: device.is_running(),
            is_available: true,
        })
        .collect();

    // Phase 1: Android panel with device details
    assert_eq!(state.active_panel, Panel::Android);

    let android_test_index = state
        .android_devices
        .iter()
        .position(|d| d.name == "DetailTestAndroid")
        .expect("Test Android device not found");
    state.selected_android = android_test_index;

    // Simulate device details loading for Android
    let android_details = DeviceDetails {
        name: "DetailTestAndroid".to_string(),
        status: "Stopped".to_string(),
        platform: Panel::Android,
        device_type: "Pixel 8 Pro".to_string(),
        api_level_or_version: "API 34".to_string(),
        ram_size: Some("8192MB".to_string()),
        storage_size: Some("32768MB".to_string()),
        resolution: Some("2992x1344".to_string()),
        dpi: Some("489".to_string()),
        device_path: Some("/path/to/android/DetailTestAndroid".to_string()),
        system_image: Some("system-images;android-34;google_apis;x86_64".to_string()),
        identifier: "DetailTestAndroid".to_string(),
    };
    state.cached_device_details = Some(android_details);

    // Start Android device and update details
    android_manager
        .start_device("DetailTestAndroid")
        .await
        .expect("Failed to start Android device");
    state.android_devices[android_test_index].status = DeviceStatus::Running;
    state.android_devices[android_test_index].is_running = true;

    // Update cached details to reflect running state
    if let Some(details) = &mut state.cached_device_details {
        details.status = "Running".to_string();
    }

    // Phase 2: Switch to iOS panel and load iOS device details
    state.active_panel = Panel::Ios;

    let ios_test_index = state
        .ios_devices
        .iter()
        .position(|d| d.name == "DetailTestiOS")
        .expect("Test iOS device not found");
    state.selected_ios = ios_test_index;

    // Simulate details clearing and loading for iOS
    state.cached_device_details = None; // Clear during panel switch

    let ios_details = DeviceDetails {
        name: "DetailTestiOS".to_string(),
        status: "Stopped".to_string(),
        platform: Panel::Ios,
        device_type: "iPhone 15 Pro".to_string(),
        api_level_or_version: "iOS 17.0".to_string(),
        ram_size: Some("8GB".to_string()),
        storage_size: Some("128GB".to_string()),
        resolution: Some("2556x1179".to_string()),
        dpi: Some("460".to_string()),
        device_path: None,
        system_image: None,
        identifier: state.ios_devices[ios_test_index].udid.clone(),
    };
    state.cached_device_details = Some(ios_details);

    // Start iOS device and update details
    ios_manager
        .start_device("DetailTestiOS")
        .await
        .expect("Failed to start iOS device");
    state.ios_devices[ios_test_index].status = DeviceStatus::Running;
    state.ios_devices[ios_test_index].is_running = true;

    if let Some(details) = &mut state.cached_device_details {
        details.status = "Running".to_string();
    }

    // Phase 3: Rapid panel switching to test details coordination
    for i in 0..5 {
        let target_panel = if i % 2 == 0 {
            Panel::Android
        } else {
            Panel::Ios
        };
        state.active_panel = target_panel;

        // Simulate details update for new panel
        match state.active_panel {
            Panel::Android => {
                let android_details = DeviceDetails {
                    name: state.android_devices[state.selected_android].name.clone(),
                    status: if state.android_devices[state.selected_android].is_running {
                        "Running"
                    } else {
                        "Stopped"
                    }
                    .to_string(),
                    platform: Panel::Android,
                    device_type: "Pixel 8 Pro".to_string(),
                    api_level_or_version: format!(
                        "API {api_level}",
                        api_level = state.android_devices[state.selected_android].api_level
                    ),
                    ram_size: Some(format!(
                        "{ram_size}MB",
                        ram_size = state.android_devices[state.selected_android].ram_size
                    )),
                    storage_size: Some(format!(
                        "{storage_size}MB",
                        storage_size = state.android_devices[state.selected_android].storage_size
                    )),
                    resolution: Some("2992x1344".to_string()),
                    dpi: Some("489".to_string()),
                    device_path: Some(format!(
                        "/path/to/android/{device_name}",
                        device_name = state.android_devices[state.selected_android].name
                    )),
                    system_image: Some("system-images;android-34;google_apis;x86_64".to_string()),
                    identifier: state.android_devices[state.selected_android].name.clone(),
                };
                state.cached_device_details = Some(android_details);
            }
            Panel::Ios => {
                let ios_details = DeviceDetails {
                    name: state.ios_devices[state.selected_ios].name.clone(),
                    status: if state.ios_devices[state.selected_ios].is_running {
                        "Running"
                    } else {
                        "Stopped"
                    }
                    .to_string(),
                    platform: Panel::Ios,
                    device_type: "iPhone 15 Pro".to_string(),
                    api_level_or_version: state.ios_devices[state.selected_ios].ios_version.clone(),
                    ram_size: Some("8GB".to_string()),
                    storage_size: Some("128GB".to_string()),
                    resolution: Some("2556x1179".to_string()),
                    dpi: Some("460".to_string()),
                    device_path: None,
                    system_image: None,
                    identifier: state.ios_devices[state.selected_ios].udid.clone(),
                };
                state.cached_device_details = Some(ios_details);
            }
        }
    }

    // Phase 4: Verify final details consistency
    if let Some(details) = &state.cached_device_details {
        assert_eq!(details.platform, state.active_panel);
        assert_eq!(details.status, "Running"); // Both test devices should be running

        match state.active_panel {
            Panel::Android => {
                assert_eq!(details.name, "DetailTestAndroid");
                assert!(details.device_path.is_some());
                assert!(details.system_image.is_some());
            }
            Panel::Ios => {
                assert_eq!(details.name, "DetailTestiOS");
                assert!(details.device_path.is_none());
                assert!(details.system_image.is_none());
            }
        }
    }

    println!(
        "Final active panel: {active_panel:?}",
        active_panel = state.active_panel
    );
    println!(
        "Final device details: {final_device_details:?}",
        final_device_details = state.cached_device_details.as_ref().map(|d| &d.name)
    );
    println!("✅ Panel switching device details updates completed successfully!");
}
