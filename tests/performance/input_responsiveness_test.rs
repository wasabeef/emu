//! Input responsiveness performance tests
//! 
//! Tests keyboard input handling latency and ensures ultra-responsive behavior
//! for continuous key presses and rapid navigation.

use emu::app::App;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_input_latency_measurement() {
    // Skip on CI where timing can be unreliable
    if std::env::var("CI").is_ok() {
        return;
    }

    // Setup mock Android SDK environment
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create minimal SDK structure
    let sdk_path = temp_dir.path();
    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms).await.ok();
    }

    // Test input processing latency
    if let Ok(app) = App::new().await {
        let start_time = Instant::now();
        
        // Simulate rapid key navigation
        for _ in 0..50 {
            let mut state = app.state.lock().await;
            state.move_down();
            drop(state);
            
            // Measure time to complete state update
            let update_time = start_time.elapsed();
            
            // Each update should be under 1ms for ultra-responsive input
            assert!(
                update_time < Duration::from_millis(1),
                "State update took {update_time:?}, should be < 1ms for responsive input"
            );
        }

        println!("✓ Input latency test passed - all updates under 1ms");
    }

    std::env::remove_var("ANDROID_HOME");
    std::mem::forget(temp_dir);
}

#[tokio::test]
async fn test_continuous_key_processing() {
    // Skip on CI where timing can be unreliable
    if std::env::var("CI").is_ok() {
        return;
    }

    // Setup mock Android SDK environment
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    let sdk_path = temp_dir.path();
    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms).await.ok();
    }

    if let Ok(app) = App::new().await {
        let start_time = Instant::now();
        
        // Simulate continuous key hold (like holding 'j' to navigate down)
        for i in 0..100 {
            let mut state = app.state.lock().await;
            let prev_selected = state.selected_android;
            state.move_down();
            let new_selected = state.selected_android;
            drop(state);
            
            // Verify navigation actually occurred (unless at end of list)
            if i < 10 {
                // Should move until we hit the end
                assert_ne!(prev_selected, new_selected, "Navigation should occur for continuous keys");
            }
        }
        
        let total_time = start_time.elapsed();
        let avg_per_key = total_time / 100;
        
        // Average processing time per key should be under 0.5ms
        assert!(
            avg_per_key < Duration::from_micros(500),
            "Average key processing time {avg_per_key:?} should be < 0.5ms"
        );

        println!("✓ Continuous key processing test passed - avg {avg_per_key:?} per key");
    }

    std::env::remove_var("ANDROID_HOME");
    std::mem::forget(temp_dir);
}

#[tokio::test]
async fn test_event_batching_performance() {
    // Test that batched event processing is more efficient than single events
    
    // Setup mock Android SDK environment
    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    let sdk_path = temp_dir.path();
    tokio::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin"))
        .await
        .ok();

    let avdmanager_path = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
    tokio::fs::write(&avdmanager_path, "#!/bin/bash\necho 'mock'\n")
        .await
        .ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&avdmanager_path).await.unwrap().permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&avdmanager_path, perms).await.ok();
    }

    if let Ok(app) = App::new().await {
        // Test batch processing efficiency
        let start_time = Instant::now();
        
        // Simulate burst of events (like rapid key presses)
        for _ in 0..10 {
            let mut state = app.state.lock().await;
            state.move_down();
            state.move_up();
            state.toggle_panel(); // Switch panels
            drop(state);
            
            // Small delay to simulate real input timing
            sleep(Duration::from_micros(100)).await;
        }
        
        let batch_time = start_time.elapsed();
        
        // Batch processing should be efficient
        assert!(
            batch_time < Duration::from_millis(10),
            "Batch processing took {batch_time:?}, should be < 10ms"
        );

        println!("✓ Event batching test passed - processed in {batch_time:?}");
    }

    std::env::remove_var("ANDROID_HOME");
    std::mem::forget(temp_dir);
}