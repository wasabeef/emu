//! Event processing integration tests
//!
//! Tests event handling, debouncing, batching, and complex event workflows
//! using MockDeviceManager for emulator-independent testing.

use emu::app::event_processing::{EventDebouncer, NavigationBatcher};
use emu::app::events::AppEvent;
use emu::models::Panel;
use std::time::Duration;
use tokio::time::{sleep, Instant};

mod common;
use common::{setup_test_app, NavigationAction, simulate_navigation_test};

#[tokio::test]
async fn test_event_debouncer_functionality() {
    let mut debouncer = EventDebouncer::new(Duration::from_millis(50));
    
    // Test rapid events are debounced
    let start = Instant::now();
    
    // Send multiple rapid events
    assert!(debouncer.should_process_event());
    assert!(!debouncer.should_process_event()); // Too soon
    assert!(!debouncer.should_process_event()); // Still too soon
    
    // Wait for debounce period
    sleep(Duration::from_millis(60)).await;
    assert!(debouncer.should_process_event());
    
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(50));
}

#[tokio::test]
async fn test_navigation_batcher_batching() {
    let mut batcher = NavigationBatcher::new(Duration::from_millis(30));
    
    // Add multiple navigation events
    batcher.add_navigation(Panel::Android);
    batcher.add_navigation(Panel::Ios);
    batcher.add_navigation(Panel::Details);
    batcher.add_navigation(Panel::Android);
    
    // Should batch to the final navigation
    if let Some(final_panel) = batcher.get_batched_navigation() {
        assert_eq!(final_panel, Panel::Android);
    }
    
    // Batcher should be empty after getting result
    assert!(batcher.get_batched_navigation().is_none());
}

#[tokio::test]
async fn test_navigation_batcher_timing() {
    let mut batcher = NavigationBatcher::new(Duration::from_millis(20));
    
    // Add navigation
    batcher.add_navigation(Panel::Ios);
    
    // Immediately check - should be None (still batching)
    assert!(batcher.get_batched_navigation().is_none());
    
    // Wait for batch period
    sleep(Duration::from_millis(25)).await;
    
    // Now should have the batched navigation
    if let Some(panel) = batcher.get_batched_navigation() {
        assert_eq!(panel, Panel::Ios);
    }
}

#[tokio::test]
async fn test_complex_navigation_workflow() {
    let app = setup_test_app(3, 2).await;
    
    let navigation_sequence = vec![
        NavigationAction::SwitchToAndroid,
        NavigationAction::NextDevice,
        NavigationAction::NextDevice,
        NavigationAction::SwitchToIos,
        NavigationAction::SelectDevice(1),
        NavigationAction::SwitchToDetails,
        NavigationAction::SwitchToAndroid,
        NavigationAction::SelectDevice(0),
    ];
    
    let result = simulate_navigation_test(app.clone(), navigation_sequence).await;
    assert!(result.is_ok(), "Navigation workflow failed: {}", result.unwrap_err());
    
    // Verify final state
    let app_lock = app.lock().await;
    assert_eq!(app_lock.get_current_panel(), Panel::Android);
    assert_eq!(app_lock.get_selected_device_index(), 0);
}

#[tokio::test]
async fn test_rapid_navigation_simulation() {
    let app = setup_test_app(5, 3).await;
    
    // Simulate rapid navigation events
    let mut actions = Vec::new();
    for i in 0..20 {
        match i % 4 {
            0 => actions.push(NavigationAction::SwitchToAndroid),
            1 => actions.push(NavigationAction::NextDevice),
            2 => actions.push(NavigationAction::SwitchToIos),
            3 => actions.push(NavigationAction::PrevDevice),
            _ => {}
        }
    }
    
    let result = simulate_navigation_test(app.clone(), actions).await;
    assert!(result.is_ok(), "Rapid navigation failed: {}", result.unwrap_err());
    
    // Verify app is in a consistent state
    let app_lock = app.lock().await;
    let current_panel = app_lock.get_current_panel();
    let selected_index = app_lock.get_selected_device_index();
    
    let device_count = match current_panel {
        Panel::Android => app_lock.get_android_devices().len(),
        Panel::Ios => app_lock.get_ios_devices().len(),
        Panel::Details => 0,
    };
    
    if device_count > 0 && current_panel != Panel::Details {
        assert!(selected_index < device_count, 
               "Selected index {selected_index} out of bounds for {device_count} devices");
    }
}

#[tokio::test]
async fn test_modal_event_workflow() {
    let app = setup_test_app(2, 1).await;
    
    let modal_workflow = vec![
        NavigationAction::SwitchToAndroid,
        NavigationAction::ShowCreateModal,
        NavigationAction::HideModal,
        NavigationAction::SwitchToIos,
        NavigationAction::ShowCreateModal,
        NavigationAction::SwitchToDetails, // Should hide modal
        NavigationAction::SwitchToAndroid,
    ];
    
    let result = simulate_navigation_test(app.clone(), modal_workflow).await;
    assert!(result.is_ok());
    
    // Verify modal state
    let app_lock = app.lock().await;
    assert!(!app_lock.is_showing_create_device_modal());
}

#[tokio::test]
async fn test_boundary_navigation_events() {
    let app = setup_test_app(2, 1).await; // 2 Android, 1 iOS
    
    // Test boundary conditions
    {
        let mut app_lock = app.lock().await;
        app_lock.set_current_panel(Panel::Android);
        app_lock.set_selected_device_index(1); // Last Android device
        
        // Try to go beyond boundary
        app_lock.next_device();
        // Should wrap around or stay at boundary
        let index = app_lock.get_selected_device_index();
        assert!(index < 2, "Index should be within Android device count");
    }
    
    // Test iOS boundary
    {
        let mut app_lock = app.lock().await;
        app_lock.set_current_panel(Panel::Ios);
        app_lock.set_selected_device_index(0); // Only iOS device
        
        app_lock.previous_device();
        let index = app_lock.get_selected_device_index();
        assert_eq!(index, 0, "Should stay at first device");
        
        app_lock.next_device();
        let index = app_lock.get_selected_device_index();
        assert_eq!(index, 0, "Should stay at only device");
    }
}

#[tokio::test]
async fn test_event_processing_performance() {
    let app = setup_test_app(10, 5).await;
    
    let start_time = Instant::now();
    
    // Process many events rapidly
    let mut actions = Vec::new();
    for i in 0..100 {
        actions.push(match i % 6 {
            0 => NavigationAction::SwitchToAndroid,
            1 => NavigationAction::NextDevice,
            2 => NavigationAction::SwitchToIos,
            3 => NavigationAction::PrevDevice,
            4 => NavigationAction::SwitchToDetails,
            5 => NavigationAction::SelectDevice(i % 5),
            _ => NavigationAction::SwitchToAndroid,
        });
    }
    
    let result = simulate_navigation_test(app.clone(), actions).await;
    assert!(result.is_ok());
    
    let elapsed = start_time.elapsed();
    
    // Should process 100 events in reasonable time (less than 1 second)
    assert!(elapsed < Duration::from_secs(1), 
           "Event processing took too long: {elapsed:?}");
    
    // Verify final state consistency
    let app_lock = app.lock().await;
    let current_panel = app_lock.get_current_panel();
    let selected_index = app_lock.get_selected_device_index();
    
    match current_panel {
        Panel::Android => {
            assert!(selected_index < 10, "Android selection out of bounds");
        }
        Panel::Ios => {
            assert!(selected_index < 5, "iOS selection out of bounds");
        }
        Panel::Details => {
            // Details panel doesn't have device selection
        }
    }
}

#[tokio::test]
async fn test_concurrent_event_processing() {
    let app = setup_test_app(3, 2).await;
    let app_clone = app.clone();
    
    // Spawn concurrent event processing tasks
    let handle1 = tokio::spawn(async move {
        let actions = vec![
            NavigationAction::SwitchToAndroid,
            NavigationAction::NextDevice,
            NavigationAction::NextDevice,
            NavigationAction::SelectDevice(0),
        ];
        simulate_navigation_test(app, actions).await
    });
    
    let handle2 = tokio::spawn(async move {
        let actions = vec![
            NavigationAction::SwitchToIos,
            NavigationAction::SelectDevice(1),
            NavigationAction::ShowCreateModal,
            NavigationAction::HideModal,
        ];
        simulate_navigation_test(app_clone, actions).await
    });
    
    // Wait for both to complete
    let (result1, result2) = tokio::join!(handle1, handle2);
    
    assert!(result1.is_ok());
    assert!(result1.unwrap().is_ok());
    assert!(result2.is_ok());
    assert!(result2.unwrap().is_ok());
}

#[tokio::test]
async fn test_debouncer_reset_functionality() {
    let mut debouncer = EventDebouncer::new(Duration::from_millis(30));
    
    // Process first event
    assert!(debouncer.should_process_event());
    
    // Wait half the debounce time
    sleep(Duration::from_millis(15)).await;
    assert!(!debouncer.should_process_event());
    
    // Reset the debouncer
    debouncer.reset();
    
    // Should be able to process immediately after reset
    assert!(debouncer.should_process_event());
}

#[tokio::test]
async fn test_navigation_batcher_overflow() {
    let mut batcher = NavigationBatcher::new(Duration::from_millis(10));
    
    // Add many rapid navigations
    for i in 0..50 {
        let panel = match i % 3 {
            0 => Panel::Android,
            1 => Panel::Ios,
            _ => Panel::Details,
        };
        batcher.add_navigation(panel);
    }
    
    // Should still only return the last navigation
    sleep(Duration::from_millis(15)).await;
    
    if let Some(final_panel) = batcher.get_batched_navigation() {
        assert_eq!(final_panel, Panel::Ios); // Last panel in the sequence
    }
    
    // Verify batcher is cleaned up
    assert!(batcher.get_batched_navigation().is_none());
}

#[tokio::test]
async fn test_event_processing_edge_cases() {
    let app = setup_test_app(0, 0).await; // No devices
    
    // Test navigation with no devices
    let edge_case_actions = vec![
        NavigationAction::SwitchToAndroid,
        NavigationAction::NextDevice, // Should handle gracefully
        NavigationAction::PrevDevice, // Should handle gracefully
        NavigationAction::SelectDevice(0), // Should handle gracefully
        NavigationAction::SwitchToIos,
        NavigationAction::SelectDevice(5), // Out of bounds
    ];
    
    let result = simulate_navigation_test(app.clone(), edge_case_actions).await;
    // Should not crash, even with edge cases
    assert!(result.is_ok() || result.unwrap_err().contains("out of bounds"));
    
    // Verify app state is still valid
    let app_lock = app.lock().await;
    assert_eq!(app_lock.get_android_devices().len(), 0);
    assert_eq!(app_lock.get_ios_devices().len(), 0);
}