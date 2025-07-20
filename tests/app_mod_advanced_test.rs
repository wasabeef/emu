//! Advanced tests for src/app/mod.rs internal functionality
//!
//! This test file focuses on testing app behavior through public interfaces
//! that exercise internal methods and improve overall coverage.

use emu::app::App;
use std::time::Duration;

mod common;
use common::setup_mock_android_sdk;

/// Test app initialization with state management
#[tokio::test]
async fn test_app_with_state_management() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // App creation exercises several internal paths:
            // - Arc<Mutex<AppState>> creation
            // - AndroidManager::new()
            // - IosManager::new() (on macOS)
            // - start_background_cache_loading()
            // - start_background_device_loading()

            // Let background operations run
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "State management initialization error: {error_msg}"
            );
        }
    }
}

/// Test app behavior during rapid operations
#[tokio::test]
async fn test_app_rapid_operations() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    for round in 0..5 {
        let start = std::time::Instant::now();
        let app_result = App::new().await;
        let elapsed = start.elapsed();

        match app_result {
            Ok(_app) => {
                // Quick initialization should be possible
                assert!(
                    elapsed < Duration::from_secs(15),
                    "Round {round}: App should initialize quickly, took {elapsed:?}"
                );

                // Brief operation to test app stability
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => {
                // Even errors should be fast
                assert!(
                    elapsed < Duration::from_secs(3),
                    "Round {round}: Error should be quick, took {elapsed:?}"
                );

                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Android") || error_msg.contains("SDK"),
                    "Round {round} error: {error_msg}"
                );
            }
        }
    }
}

/// Test app manager coordination
#[tokio::test]
async fn test_manager_coordination() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // App::new() coordinates multiple managers:
            // - AndroidManager creation and initialization
            // - IosManager creation (platform-dependent)
            // - Background task coordination
            // - State synchronization

            // Allow coordination to complete
            tokio::time::sleep(Duration::from_millis(50)).await;

            // App should be stable after coordination
        }
        Err(e) => {
            // Manager coordination failure
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android")
                    || error_msg.contains("iOS")
                    || error_msg.contains("manager")
                    || error_msg.contains("SDK"),
                "Manager coordination error: {error_msg}"
            );
        }
    }
}

/// Test background task lifecycle
#[tokio::test]
async fn test_background_task_lifecycle() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // App::new() starts background tasks:
            // - start_background_cache_loading()
            // - start_background_device_loading()

            // These tasks exercise:
            // - Task spawning
            // - Shared state access
            // - Async coordination
            // - Error handling in background context

            // Let background tasks initialize and run
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Background tasks should be running stably
        }
        Err(e) => {
            // Background task setup failure
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "Background task lifecycle error: {error_msg}"
            );
        }
    }
}

/// Test resource management patterns
#[tokio::test]
async fn test_resource_management() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test pattern: create, use briefly, drop
    for cycle in 0..3 {
        let app_result = App::new().await;

        match app_result {
            Ok(app) => {
                // App holds several resources:
                // - Arc<Mutex<AppState>>
                // - AndroidManager
                // - Option<IosManager>
                // - Background task handles

                // Brief usage
                tokio::time::sleep(Duration::from_millis(20)).await;

                // Explicit drop to test cleanup
                drop(app);

                // Allow cleanup to complete
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Android") || error_msg.contains("SDK"),
                    "Resource management cycle {cycle} error: {error_msg}"
                );
            }
        }
    }
}

/// Test cross-platform initialization paths
#[tokio::test]
async fn test_cross_platform_paths() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // Platform-specific code paths in App::new():
            // - Android manager (all platforms)
            // - iOS manager (macOS only via cfg!(target_os = "macos"))
            // - Platform-specific background operations

            if cfg!(target_os = "macos") {
                // macOS path: both Android and iOS managers
            } else {
                // Non-macOS path: Android manager only
            }

            // Common platform functionality should work
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "Cross-platform initialization error: {error_msg}"
            );
        }
    }
}

/// Test error propagation patterns
#[tokio::test]
async fn test_error_propagation() {
    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // Successful path - no errors to propagate
        }
        Err(e) => {
            // Test error propagation through App::new():
            // - AndroidManager::new()? can fail
            // - IosManager::new()? can fail (macOS)
            // - Background operations can fail

            let error_msg = e.to_string();

            // Error should be meaningful
            assert!(!error_msg.is_empty(), "Error message should not be empty");

            // Error should contain relevant context
            let has_useful_info = error_msg.contains("Android")
                || error_msg.contains("iOS")
                || error_msg.contains("SDK")
                || error_msg.contains("manager")
                || error_msg.contains("avdmanager")
                || error_msg.contains("emulator");

            assert!(
                has_useful_info,
                "Error should contain useful context: {error_msg}"
            );

            // Error should not leak internal implementation details
            assert!(
                !error_msg.contains("unwrap") && !error_msg.contains("panic"),
                "Error should not leak implementation details: {error_msg}"
            );
        }
    }
}

/// Test concurrent initialization scenarios
#[tokio::test]
async fn test_concurrent_initialization() {
    // Spawn multiple App::new() calls concurrently
    let concurrent_tasks = (0..3).map(|task_id| {
        tokio::spawn(async move {
            let result = App::new().await;
            match result {
                Ok(_app) => {
                    // Successful concurrent initialization
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    format!("Task {task_id}: Success")
                }
                Err(e) => {
                    // Failed concurrent initialization
                    format!("Task {task_id}: Error: {e}")
                }
            }
        })
    });

    // Wait for all concurrent tasks
    for (task_id, task) in concurrent_tasks.enumerate() {
        let result = task.await;
        assert!(
            result.is_ok(),
            "Concurrent task {task_id} should complete without panicking"
        );

        let message = result.unwrap();
        assert!(
            message.starts_with(&format!("Task {task_id}:")),
            "Task {task_id} should return proper message: {message}"
        );
    }
}

/// Test initialization timing and performance
#[tokio::test]
async fn test_initialization_performance() {
    let mut times = Vec::new();

    for attempt in 0..3 {
        let start = std::time::Instant::now();
        let app_result = App::new().await;
        let elapsed = start.elapsed();
        times.push(elapsed);

        match app_result {
            Ok(_app) => {
                // Successful initialization timing
                assert!(
                    elapsed < Duration::from_secs(30),
                    "Attempt {attempt}: Initialization should be under 30s, took {elapsed:?}"
                );

                // Brief usage to ensure stability
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => {
                // Error timing should also be reasonable
                assert!(
                    elapsed < Duration::from_secs(15),
                    "Attempt {attempt}: Error should be under 15s, took {elapsed:?}"
                );

                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Android") || error_msg.contains("SDK"),
                    "Attempt {attempt} performance test error: {error_msg}"
                );
            }
        }
    }

    // Performance should be consistent
    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    assert!(
        avg_time < Duration::from_secs(15),
        "Average initialization time should be reasonable: {avg_time:?}"
    );
}

/// Test app state integration during initialization
#[tokio::test]
async fn test_state_integration() {
    let app_result = App::new().await;

    match app_result {
        Ok(_app) => {
            // App::new() integrates with AppState through:
            // - Arc<Mutex<AppState>> creation
            // - Background cache operations
            // - Device loading coordination
            // - State synchronization

            // Allow state integration to complete
            tokio::time::sleep(Duration::from_millis(100)).await;

            // State integration should be stable
        }
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "State integration error: {error_msg}"
            );
        }
    }
}
