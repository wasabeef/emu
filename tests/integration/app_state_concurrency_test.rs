//! App::State concurrency complete test
//!
//! This test suite verifies concurrent access patterns and
//! state management integrity of AppState.

use emu::app::state::AppState;
use emu::app::Panel;
use emu::models::device::{AndroidDevice, DeviceStatus, IosDevice};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_concurrent_panel_switching() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Switch panels concurrently with 10 tasks
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.active_panel = if i % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state
    let final_state = state.lock().unwrap();
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
}

#[tokio::test]
async fn test_concurrent_device_selection() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Execute device selection concurrently
    for i in 0..5 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..3 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.selected_android = i;
                    state.selected_ios = j;
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    assert!(final_state.selected_android < 10);
    assert!(final_state.selected_ios < 10);
}

#[tokio::test]
async fn test_concurrent_device_list_updates() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Update device lists concurrently
    for i in 0..3 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                {
                    let mut state = state_clone.lock().unwrap();
                    let device = AndroidDevice {
                        android_version_name: "API 30".to_string(),
                        name: format!("device_{i}_{j}"),
                        device_type: "pixel_7".to_string(),
                        api_level: 34,
                        status: DeviceStatus::Stopped,
                        is_running: false,
                        ram_size: "4096".to_string(),
                        storage_size: "8192".to_string(),
                    };
                    state.android_devices.push(device);
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    assert_eq!(final_state.android_devices.len(), 15); // 3 * 5
}

#[tokio::test]
async fn test_concurrent_loading_states() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Toggle loading state concurrently
    for i in 0..8 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.is_loading = i % 2 == 0;
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    // Loading state is a boolean value, verify no deadlock occurs
    // Actual value is result of concurrent processing, so only verify it's a boolean
    let _is_loading = final_state.is_loading; // Verify type is bool
}

#[tokio::test]
async fn test_concurrent_error_handling() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Set error state concurrently
    for i in 0..5 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..3 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state
                        .notifications
                        .push_back(emu::app::state::Notification::error(format!(
                            "Error from task {i} iteration {j}"
                        )));
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    assert!(!final_state.notifications.is_empty());
    assert!(final_state
        .notifications
        .back()
        .unwrap()
        .message
        .contains("Error from task"));
}

#[tokio::test]
async fn test_state_consistency_under_load() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Execute multiple operations concurrently
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..20 {
                {
                    let mut state = state_clone.lock().unwrap();

                    // Change multiple states simultaneously
                    state.active_panel = if (i + j) % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                    state.selected_android = i;
                    state.selected_ios = j;
                    state.is_loading = (i + j) % 3 == 0;

                    // Add device
                    let device = AndroidDevice {
                        android_version_name: "API 30".to_string(),
                        name: format!("consistency_test_{i}_{j}"),
                        device_type: "pixel_7".to_string(),
                        api_level: 34,
                        status: DeviceStatus::Stopped,
                        is_running: false,
                        ram_size: "4096".to_string(),
                        storage_size: "8192".to_string(),
                    };
                    state.android_devices.push(device);
                }

                // Short wait
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state consistency
    let final_state = state.lock().unwrap();
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
    assert_eq!(final_state.android_devices.len(), 200); // 10 * 20
    assert!(final_state.selected_android < 10);
    assert!(final_state.selected_ios < 20);
}

#[tokio::test]
async fn test_concurrent_mixed_operations() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Read-only operations
    for _i in 0..5 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..50 {
                {
                    let state = state_clone.lock().unwrap();
                    let _ = state.active_panel;
                    let _ = state.android_devices.len();
                    let _ = state.ios_devices.len();
                }
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Write operations
    for i in 0..3 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.active_panel = if i % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };

                    let android_device = AndroidDevice {
                        android_version_name: "API 30".to_string(),
                        name: format!("mixed_android_{i}_{j}"),
                        device_type: "pixel_7".to_string(),
                        api_level: 34,
                        status: DeviceStatus::Stopped,
                        is_running: false,
                        ram_size: "4096".to_string(),
                        storage_size: "8192".to_string(),
                    };
                    state.android_devices.push(android_device);

                    let ios_device = IosDevice {
                        name: format!("mixed_ios_{i}_{j}"),
                        udid: format!("UUID-{i}_{j}"),
                        device_type: "iPhone 15".to_string(),
                        ios_version: "17.0".to_string(),
                        runtime_version: "iOS 17.0".to_string(),
                        status: DeviceStatus::Stopped,
                        is_running: false,
                        is_available: true,
                    };
                    state.ios_devices.push(ios_device);
                }
                sleep(Duration::from_millis(2)).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    assert_eq!(final_state.android_devices.len(), 30); // 3 * 10
    assert_eq!(final_state.ios_devices.len(), 30); // 3 * 10
}

#[tokio::test]
async fn test_high_contention_scenario() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let mut handles = vec![];

    // Create high contention state
    for i in 0..20 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                {
                    let mut state = state_clone.lock().unwrap();

                    // Execute many operations in short time
                    state.active_panel = if (i + j) % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                    state.selected_android = (i + j) % 10;
                    state.selected_ios = (i * j) % 10;
                    state.is_loading = (i + j) % 4 == 0;

                    if j % 10 == 0 {
                        let device = AndroidDevice {
                            android_version_name: "API 30".to_string(),
                            name: format!("high_contention_{i}_{j}"),
                            device_type: "pixel_7".to_string(),
                            api_level: 34,
                            status: DeviceStatus::Stopped,
                            is_running: false,
                            ram_size: "4096".to_string(),
                            storage_size: "8192".to_string(),
                        };
                        state.android_devices.push(device);
                    }
                }

                // Very short wait
                if j % 50 == 0 {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let final_state = state.lock().unwrap();
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
    assert_eq!(final_state.android_devices.len(), 200); // 20 * 10
    assert!(final_state.selected_android < 10);
    assert!(final_state.selected_ios < 10);
}

#[tokio::test]
async fn test_state_performance_under_concurrency() {
    let state = Arc::new(Mutex::new(AppState::new()));

    let start = std::time::Instant::now();

    let mut handles = vec![];

    // Performance test: verify concurrent operations complete within 100ms
    for i in 0..10 {
        let state_clone = state.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                {
                    let mut state = state_clone.lock().unwrap();
                    state.active_panel = if (i + j) % 2 == 0 {
                        Panel::Android
                    } else {
                        Panel::Ios
                    };
                    state.selected_android = i;
                    state.selected_ios = j;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    assert!(
        duration.as_millis() < 100,
        "Concurrent operations took too long: {duration:?}"
    );

    let final_state = state.lock().unwrap();
    assert!(matches!(
        final_state.active_panel,
        Panel::Android | Panel::Ios
    ));
}
