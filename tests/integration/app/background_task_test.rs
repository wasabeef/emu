//! Background task integration tests
//!
//! Tests background task coordination, async operations, and task lifecycle
//! management using MockDeviceManager for emulator-independent testing.

use emu::app::state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio::time::{sleep, Instant};

#[cfg(feature = "test-utils")]
use emu::managers::mock::{MockDeviceManager, ScenarioBuilder};

mod common;
use common::{setup_test_app, setup_test_app_with_scenario};

#[tokio::test]
async fn test_background_device_loading() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(5)
                .with_ios_devices(3)
                .with_operation_delay(Duration::from_millis(10))
                .build()
        );
        
        let start_time = Instant::now();
        
        // Create app state (should trigger background device loading)
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        let loading_time = start_time.elapsed();
        
        // Verify devices were loaded
        let app_lock = app.lock().await;
        assert_eq!(app_lock.get_android_devices().len(), 5);
        assert_eq!(app_lock.get_ios_devices().len(), 3);
        
        // Background loading should be reasonably fast
        assert!(loading_time < Duration::from_secs(1), 
               "Background loading took too long: {loading_time:?}");
    }
}

#[tokio::test]
async fn test_concurrent_background_operations() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(3)
                .with_ios_devices(2)
                .with_operation_delay(Duration::from_millis(5))
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Get device IDs for operations
        let (android_id, ios_id) = {
            let app_lock = app.lock().await;
            let android_devices = app_lock.get_android_devices();
            let ios_devices = app_lock.get_ios_devices();
            
            let android_id = android_devices.first().map(|d| d.id.clone());
            let ios_id = ios_devices.first().map(|d| d.id.clone());
            
            (android_id, ios_id)
        };
        
        // Spawn concurrent background operations
        let manager1 = mock_manager.clone();
        let manager2 = mock_manager.clone();
        
        let handle1 = if let Some(id) = android_id {
            Some(tokio::spawn(async move {
                let result = manager1.start_device(&id, emu::models::Platform::Android).await;
                result.is_ok()
            }))
        } else {
            None
        };
        
        let handle2 = if let Some(id) = ios_id {
            Some(tokio::spawn(async move {
                let result = manager2.start_device(&id, emu::models::Platform::Ios).await;
                result.is_ok()
            }))
        } else {
            None
        };
        
        // Wait for operations to complete
        if let Some(h1) = handle1 {
            let result1 = h1.await.expect("Task should complete");
            assert!(result1, "Android device start should succeed");
        }
        
        if let Some(h2) = handle2 {
            let result2 = h2.await.expect("Task should complete");
            assert!(result2, "iOS device start should succeed");
        }
    }
}

#[tokio::test]
async fn test_background_task_cancellation() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(2)
                .with_operation_delay(Duration::from_millis(100)) // Longer delay
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        let device_id = {
            let app_lock = app.lock().await;
            app_lock.get_android_devices().first().map(|d| d.id.clone())
        };
        
        if let Some(id) = device_id {
            let manager_clone = mock_manager.clone();
            
            // Start a long-running operation
            let handle = tokio::spawn(async move {
                manager_clone.start_device(&id, emu::models::Platform::Android).await
            });
            
            // Cancel the task quickly
            sleep(Duration::from_millis(10)).await;
            handle.abort();
            
            // Verify the task was cancelled
            let result = handle.await;
            assert!(result.is_err() && result.unwrap_err().is_cancelled());
        }
    }
}

#[tokio::test]
async fn test_background_cache_updates() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(1)
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Simulate cache updates in background
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            for i in 0..5 {
                sleep(Duration::from_millis(10)).await;
                
                let mut app_lock = app_clone.lock().await;
                app_lock.set_notification(format!("Background update {i}"));
                
                // Simulate periodic cache refresh
                if i % 2 == 0 {
                    app_lock.clear_notification();
                }
            }
        });
        
        // Wait for background updates
        handle.await.expect("Background task should complete");
        
        // Verify final state
        let app_lock = app.lock().await;
        assert_eq!(app_lock.get_notification(), None); // Should be cleared
    }
}

#[tokio::test]
async fn test_background_device_status_monitoring() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(3)
                .with_operation_delay(Duration::from_millis(1))
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Simulate background status monitoring
        let device_ids: Vec<String> = {
            let app_lock = app.lock().await;
            app_lock.get_android_devices().iter().map(|d| d.id.clone()).collect()
        };
        
        // Spawn monitoring tasks for each device
        let mut handles = Vec::new();
        
        for device_id in device_ids {
            let manager_clone = mock_manager.clone();
            let app_clone = app.clone();
            
            let handle = tokio::spawn(async move {
                // Simulate periodic status checks
                for _ in 0..3 {
                    sleep(Duration::from_millis(5)).await;
                    
                    // Check device status
                    let _ = manager_clone.get_device_status(&device_id, emu::models::Platform::Android).await;
                    
                    // Update pending device tracking
                    {
                        let mut app_lock = app_clone.lock().await;
                        app_lock.add_pending_device_start(device_id.clone());
                        sleep(Duration::from_millis(1)).await;
                        app_lock.remove_pending_device_start(&device_id);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all monitoring tasks
        for handle in handles {
            handle.await.expect("Monitoring task should complete");
        }
        
        // Verify no pending operations remain
        let app_lock = app.lock().await;
        let android_devices = app_lock.get_android_devices();
        for device in &android_devices {
            assert!(!app_lock.is_device_pending_start(&device.id));
        }
    }
}

#[tokio::test]
async fn test_background_error_recovery() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(2)
                .with_failure_rate(0.5) // 50% failure rate
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        let device_id = {
            let app_lock = app.lock().await;
            app_lock.get_android_devices().first().map(|d| d.id.clone())
        };
        
        if let Some(id) = device_id {
            // Attempt operations with potential failures
            let mut success_count = 0;
            let mut failure_count = 0;
            
            for _ in 0..10 {
                let result = mock_manager.start_device(&id, emu::models::Platform::Android).await;
                
                if result.is_ok() {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                
                // Short delay between attempts
                sleep(Duration::from_millis(1)).await;
            }
            
            // With 50% failure rate, we should have both successes and failures
            assert!(success_count > 0, "Should have some successful operations");
            assert!(failure_count > 0, "Should have some failed operations");
            assert_eq!(success_count + failure_count, 10);
        }
    }
}

#[tokio::test]
async fn test_background_task_coordination() {
    let app = setup_test_app(2, 1).await;
    let app_clone1 = app.clone();
    let app_clone2 = app.clone();
    
    // Task 1: UI state updates
    let ui_task = tokio::spawn(async move {
        for i in 0..10 {
            let mut app_lock = app_clone1.lock().await;
            
            app_lock.set_current_panel(if i % 2 == 0 { 
                emu::models::Panel::Android 
            } else { 
                emu::models::Panel::Ios 
            });
            
            app_lock.set_selected_device_index(i % 2);
            
            sleep(Duration::from_millis(1)).await;
        }
    });
    
    // Task 2: Notification updates
    let notification_task = tokio::spawn(async move {
        for i in 0..10 {
            let mut app_lock = app_clone2.lock().await;
            
            if i % 3 == 0 {
                app_lock.set_notification(format!("Task notification {i}"));
            } else {
                app_lock.clear_notification();
            }
            
            sleep(Duration::from_millis(1)).await;
        }
    });
    
    // Wait for both tasks to coordinate properly
    let (ui_result, notification_result) = tokio::join!(ui_task, notification_task);
    
    assert!(ui_result.is_ok());
    assert!(notification_result.is_ok());
    
    // Verify final state is consistent
    let app_lock = app.lock().await;
    let current_panel = app_lock.get_current_panel();
    
    // Should be in a valid state
    match current_panel {
        emu::models::Panel::Android => {
            let android_count = app_lock.get_android_devices().len();
            let selected = app_lock.get_selected_device_index();
            if android_count > 0 {
                assert!(selected < android_count);
            }
        }
        emu::models::Panel::Ios => {
            let ios_count = app_lock.get_ios_devices().len();
            let selected = app_lock.get_selected_device_index();
            if ios_count > 0 {
                assert!(selected < ios_count);
            }
        }
        emu::models::Panel::Details => {
            // Details panel is always valid
        }
    }
}

#[tokio::test]
async fn test_background_task_memory_cleanup() {
    let app = setup_test_app(1, 1).await;
    
    // Create many short-lived background tasks
    let mut handles = Vec::new();
    
    for i in 0..50 {
        let app_clone = app.clone();
        
        let handle = tokio::spawn(async move {
            let mut app_lock = app_clone.lock().await;
            app_lock.set_notification(format!("Temp notification {i}"));
            
            // Short operation
            sleep(Duration::from_millis(1)).await;
            
            app_lock.clear_notification();
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Background task should complete");
    }
    
    // Verify memory is cleaned up (no lingering notifications)
    let app_lock = app.lock().await;
    assert_eq!(app_lock.get_notification(), None);
}

#[tokio::test]
async fn test_background_performance_monitoring() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(5)
                .with_operation_delay(Duration::from_millis(1))
                .build()
        );
        
        let start_time = Instant::now();
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Perform background operations and measure performance
        let device_ids: Vec<String> = {
            let app_lock = app.lock().await;
            app_lock.get_android_devices().iter().map(|d| d.id.clone()).collect()
        };
        
        // Parallel background operations
        let mut handles = Vec::new();
        
        for device_id in device_ids {
            let manager_clone = mock_manager.clone();
            
            let handle = tokio::spawn(async move {
                manager_clone.start_device(&device_id, emu::models::Platform::Android).await
            });
            
            handles.push(handle);
        }
        
        // Wait for all operations
        for handle in handles {
            let result = handle.await.expect("Operation should complete");
            assert!(result.is_ok());
        }
        
        let total_time = start_time.elapsed();
        
        // Background operations should complete efficiently
        assert!(total_time < Duration::from_secs(1), 
               "Background operations took too long: {total_time:?}");
    }
}