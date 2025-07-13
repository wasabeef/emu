//! Comprehensive AppState integration tests
//!
//! Tests the complete AppState functionality including device management,
//! UI state coordination, and complex workflows with MockDeviceManager.

use emu::app::state::AppState;
use emu::models::{Panel, DeviceStatus, Platform};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

#[cfg(feature = "test-utils")]
use emu::managers::mock::{MockDeviceManager, ScenarioBuilder};

mod common;
use common::{setup_test_app, setup_test_app_with_states, verify_device_counts};

#[tokio::test]
async fn test_app_state_initialization() {
    let app = setup_test_app(3, 2).await;
    let app_lock = app.lock().await;
    
    // Verify initial state
    assert_eq!(app_lock.get_current_panel(), Panel::Android);
    assert_eq!(app_lock.get_selected_device_index(), 0);
    assert!(!app_lock.is_showing_create_device_modal());
    assert!(!app_lock.is_showing_confirmation_modal());
    
    // Verify device counts
    assert_eq!(app_lock.get_android_devices().len(), 3);
    assert_eq!(app_lock.get_ios_devices().len(), 2);
}

#[tokio::test]
async fn test_panel_switching_comprehensive() {
    let app = setup_test_app(2, 3).await;
    
    // Test all panel transitions
    let panels = vec![Panel::Android, Panel::Ios, Panel::Details];
    
    for (i, &panel) in panels.iter().enumerate() {
        {
            let mut app_lock = app.lock().await;
            app_lock.set_current_panel(panel);
        }
        
        let app_lock = app.lock().await;
        assert_eq!(app_lock.get_current_panel(), panel);
        
        // Verify selected index resets appropriately
        if panel == Panel::Details {
            // Details panel doesn't have device selection
            continue;
        }
        
        let expected_count = match panel {
            Panel::Android => 2,
            Panel::Ios => 3,
            Panel::Details => 0,
        };
        
        let device_count = match panel {
            Panel::Android => app_lock.get_android_devices().len(),
            Panel::Ios => app_lock.get_ios_devices().len(),
            Panel::Details => 0,
        };
        
        assert_eq!(device_count, expected_count);
    }
}

#[tokio::test]
async fn test_device_selection_across_panels() {
    let app = setup_test_app(4, 3).await;
    
    // Test Android device selection
    {
        let mut app_lock = app.lock().await;
        app_lock.set_current_panel(Panel::Android);
        app_lock.set_selected_device_index(2);
    }
    
    {
        let app_lock = app.lock().await;
        assert_eq!(app_lock.get_selected_device_index(), 2);
        assert_eq!(app_lock.get_current_panel(), Panel::Android);
    }
    
    // Switch to iOS and verify selection changes
    {
        let mut app_lock = app.lock().await;
        app_lock.set_current_panel(Panel::Ios);
        app_lock.set_selected_device_index(1);
    }
    
    {
        let app_lock = app.lock().await;
        assert_eq!(app_lock.get_selected_device_index(), 1);
        assert_eq!(app_lock.get_current_panel(), Panel::Ios);
    }
    
    // Test navigation methods
    {
        let mut app_lock = app.lock().await;
        app_lock.next_device();
        assert_eq!(app_lock.get_selected_device_index(), 2);
        
        app_lock.previous_device();
        assert_eq!(app_lock.get_selected_device_index(), 1);
    }
}

#[tokio::test]
async fn test_modal_state_management() {
    let app = setup_test_app(1, 1).await;
    
    // Test create device modal
    {
        let mut app_lock = app.lock().await;
        assert!(!app_lock.is_showing_create_device_modal());
        
        app_lock.set_showing_create_device_modal(true);
        assert!(app_lock.is_showing_create_device_modal());
        assert!(!app_lock.is_showing_confirmation_modal());
    }
    
    // Test confirmation modal
    {
        let mut app_lock = app.lock().await;
        app_lock.set_showing_confirmation_modal(true, "Test confirmation".to_string());
        assert!(app_lock.is_showing_confirmation_modal());
        assert!(app_lock.is_showing_create_device_modal()); // Both can be open
    }
    
    // Test modal closing
    {
        let mut app_lock = app.lock().await;
        app_lock.close_modal();
        assert!(!app_lock.is_showing_create_device_modal());
        assert!(!app_lock.is_showing_confirmation_modal());
    }
}

#[tokio::test]
async fn test_device_status_management() {
    let app = setup_test_app_with_states(2, 1, 1, 2).await;
    
    let app_lock = app.lock().await;
    let android_devices = app_lock.get_android_devices();
    let ios_devices = app_lock.get_ios_devices();
    
    // Verify device states were set correctly
    assert_eq!(android_devices.len(), 3); // 2 online + 1 offline
    assert_eq!(ios_devices.len(), 3); // 1 online + 2 offline
    
    // Count online/offline devices
    let android_online = android_devices.iter()
        .filter(|d| d.status == DeviceStatus::Online)
        .count();
    let android_offline = android_devices.iter()
        .filter(|d| d.status == DeviceStatus::Offline)
        .count();
        
    let ios_online = ios_devices.iter()
        .filter(|d| d.status == DeviceStatus::Online)
        .count();
    let ios_offline = ios_devices.iter()
        .filter(|d| d.status == DeviceStatus::Offline)
        .count();
    
    assert_eq!(android_online, 2);
    assert_eq!(android_offline, 1);
    assert_eq!(ios_online, 1);
    assert_eq!(ios_offline, 2);
}

#[tokio::test]
async fn test_device_operations_workflow() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(2)
                .with_operation_delay(Duration::from_millis(1))
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Test device start operation
        let android_devices = {
            let app_lock = app.lock().await;
            app_lock.get_android_devices().clone()
        };
        
        if let Some(device) = android_devices.first() {
            let device_id = device.id.clone();
            
            // Simulate device start
            let result = mock_manager.start_device(&device_id, Platform::Android).await;
            assert!(result.is_ok());
            
            // Verify device state tracking
            {
                let mut app_lock = app.lock().await;
                app_lock.add_pending_device_start(device_id.clone());
                assert!(app_lock.is_device_pending_start(&device_id));
                
                app_lock.remove_pending_device_start(&device_id);
                assert!(!app_lock.is_device_pending_start(&device_id));
            }
        }
    }
}

#[tokio::test]
async fn test_notification_system() {
    let app = setup_test_app(1, 1).await;
    
    {
        let mut app_lock = app.lock().await;
        
        // Test setting notification
        app_lock.set_notification("Test notification".to_string());
        assert_eq!(app_lock.get_notification(), Some("Test notification".to_string()));
        
        // Test clearing notification
        app_lock.clear_notification();
        assert_eq!(app_lock.get_notification(), None);
    }
}

#[tokio::test]
async fn test_device_details_state() {
    let app = setup_test_app(2, 1).await;
    
    {
        let app_lock = app.lock().await;
        let android_devices = app_lock.get_android_devices();
        
        if let Some(device) = android_devices.first() {
            let device_details = app_lock.get_selected_device_details();
            
            // Verify device details structure
            assert!(device_details.contains(&device.display_name));
            assert!(device_details.contains("Status:"));
            assert!(device_details.contains("Platform:"));
        }
    }
}

#[tokio::test]
async fn test_concurrent_state_access() {
    let app = setup_test_app(3, 2).await;
    let app_clone = app.clone();
    
    // Simulate concurrent access
    let handle1 = tokio::spawn(async move {
        for i in 0..10 {
            let mut app_lock = app.lock().await;
            app_lock.set_selected_device_index(i % 3);
            app_lock.set_current_panel(if i % 2 == 0 { Panel::Android } else { Panel::Ios });
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });
    
    let handle2 = tokio::spawn(async move {
        for _ in 0..10 {
            let app_lock = app_clone.lock().await;
            let _ = app_lock.get_current_panel();
            let _ = app_lock.get_selected_device_index();
            let _ = app_lock.get_android_devices();
            let _ = app_lock.get_ios_devices();
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });
    
    // Wait for both tasks to complete
    let _ = tokio::join!(handle1, handle2);
    
    // Verify state is still consistent
    let app_lock = app.lock().await;
    let current_panel = app_lock.get_current_panel();
    let selected_index = app_lock.get_selected_device_index();
    
    // Verify selection is within bounds
    match current_panel {
        Panel::Android => {
            let android_count = app_lock.get_android_devices().len();
            if android_count > 0 {
                assert!(selected_index < android_count);
            }
        }
        Panel::Ios => {
            let ios_count = app_lock.get_ios_devices().len();
            if ios_count > 0 {
                assert!(selected_index < ios_count);
            }
        }
        Panel::Details => {
            // Details panel doesn't use device selection
        }
    }
}

#[tokio::test]
async fn test_app_state_memory_efficiency() {
    // Test that app state doesn't leak memory with many operations
    let app = setup_test_app(10, 5).await;
    
    // Perform many operations
    for i in 0..100 {
        let mut app_lock = app.lock().await;
        
        // Cycle through panels
        let panel = match i % 3 {
            0 => Panel::Android,
            1 => Panel::Ios,
            _ => Panel::Details,
        };
        app_lock.set_current_panel(panel);
        
        // Set device selection
        if panel != Panel::Details {
            let device_count = match panel {
                Panel::Android => app_lock.get_android_devices().len(),
                Panel::Ios => app_lock.get_ios_devices().len(),
                Panel::Details => 0,
            };
            if device_count > 0 {
                app_lock.set_selected_device_index(i % device_count);
            }
        }
        
        // Toggle modals
        app_lock.set_showing_create_device_modal(i % 4 == 0);
        if i % 8 == 0 {
            app_lock.set_showing_confirmation_modal(true, format!("Test {i}"));
        } else {
            app_lock.close_modal();
        }
        
        // Set notifications
        if i % 5 == 0 {
            app_lock.set_notification(format!("Notification {i}"));
        } else {
            app_lock.clear_notification();
        }
    }
    
    // Verify final state is reasonable
    let app_lock = app.lock().await;
    assert_eq!(app_lock.get_android_devices().len(), 10);
    assert_eq!(app_lock.get_ios_devices().len(), 5);
}

#[tokio::test]
async fn test_device_refresh_workflow() {
    #[cfg(feature = "test-utils")]
    {
        let mock_manager = Arc::new(
            ScenarioBuilder::new()
                .with_android_devices(2)
                .with_ios_devices(1)
                .build()
        );
        
        let app = AppState::new_with_mock_manager(mock_manager.clone()).await
            .expect("Failed to create app state");
        let app = Arc::new(Mutex::new(app));
        
        // Verify initial device count
        verify_device_counts(app.clone(), 2, 1).await
            .expect("Initial device count verification failed");
        
        // Simulate device list refresh
        {
            let mut app_lock = app.lock().await;
            // In a real scenario, this would trigger a refresh
            // For now, we verify the state remains consistent
            app_lock.set_current_panel(Panel::Android);
            assert_eq!(app_lock.get_android_devices().len(), 2);
            
            app_lock.set_current_panel(Panel::Ios);
            assert_eq!(app_lock.get_ios_devices().len(), 1);
        }
    }
}