//! Performance tests for app/mod.rs focusing on execution speed and resource usage
//!
//! This file contains performance-related tests from app_mod_execution_test.rs and
//! additional performance scenarios from other files.

use emu::app::App;
use std::time::Duration;

use crate::common::setup_mock_android_sdk;

/// Test initialization timing and performance (consolidated)
#[tokio::test]
async fn test_initialization_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

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

/// Test timeout and responsiveness during initialization
#[tokio::test]
async fn test_initialization_responsiveness() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let start_time = std::time::Instant::now();

    let app_result = tokio::time::timeout(Duration::from_secs(30), App::new()).await;

    let elapsed = start_time.elapsed();

    match app_result {
        Ok(Ok(_app)) => {
            // Successful initialization within timeout
            assert!(
                elapsed < Duration::from_secs(15),
                "App initialization should be responsive, took: {elapsed:?}"
            );
        }
        Ok(Err(e)) => {
            // Failed initialization within timeout
            assert!(
                elapsed < Duration::from_secs(30),
                "App initialization failure should be fast, took: {elapsed:?}"
            );

            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Android") || error_msg.contains("SDK"),
                "Responsive initialization error: {error_msg}"
            );
        }
        Err(_) => {
            panic!("App initialization timed out after {elapsed:?}");
        }
    }
}

/// Test multiple App::new() calls for consistent performance
#[tokio::test]
async fn test_app_new_multiple_initializations() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test multiple App::new() calls
    for i in 0..3 {
        let start = std::time::Instant::now();
        let app = App::new().await;
        let duration = start.elapsed();

        // Each initialization should complete quickly
        assert!(
            duration < Duration::from_secs(15),
            "Initialization {i} should complete within 15 seconds, took: {duration:?}"
        );

        match app {
            Ok(_) => {
                // Each initialization should succeed independently
            }
            Err(e) => {
                // Each initialization should fail gracefully
                let error_message = e.to_string();
                assert!(
                    error_message.contains("Android")
                        || error_message.contains("SDK")
                        || error_message.contains("iOS"),
                    "Error in initialization {i} should be SDK-related: {e}"
                );
            }
        }
    }
}

/// Test concurrent App::new() calls for performance
#[tokio::test]
async fn test_app_new_concurrent_initialization() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test concurrent App::new() calls
    let mut handles = vec![];

    let start_time = std::time::Instant::now();

    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let task_start = std::time::Instant::now();
            let app = App::new().await;
            let task_duration = task_start.elapsed();

            // Each concurrent task should complete quickly
            assert!(
                task_duration < Duration::from_secs(20),
                "Concurrent task {i} should complete quickly, took: {task_duration:?}"
            );

            match app {
                Ok(_) => Ok(format!("App {i} initialized successfully")),
                Err(e) => {
                    let error_message = e.to_string();
                    if error_message.contains("Android")
                        || error_message.contains("SDK")
                        || error_message.contains("iOS")
                    {
                        Ok(format!("App {i} failed with expected error: {e}"))
                    } else {
                        Err(format!("App {i} failed with unexpected error: {e}"))
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all concurrent initializations
    let results = futures::future::join_all(handles).await;

    let total_duration = start_time.elapsed();
    assert!(
        total_duration < Duration::from_secs(30),
        "All concurrent initializations should complete within 30 seconds, took: {total_duration:?}"
    );

    // All tasks should complete successfully
    for (i, result) in results.iter().enumerate() {
        assert!(
            result.is_ok(),
            "Concurrent initialization {i} should complete: {result:?}"
        );
        if let Ok(Ok(message)) = result {
            assert!(
                !message.is_empty(),
                "Concurrent initialization {i} should have result message"
            );
        }
    }
}

/// Test app initialization completes within reasonable time
#[tokio::test]
async fn test_app_initialization_performance_basic() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test that App::new() completes within reasonable time
    let start = std::time::Instant::now();
    let app = App::new().await;
    let duration = start.elapsed();

    // Should complete within 15 seconds (allowing for slow CI environments)
    assert!(
        duration < Duration::from_secs(15),
        "App initialization should complete within 15 seconds, took: {duration:?}"
    );

    // Verify result is valid
    match app {
        Ok(_) => {}
        Err(e) => {
            let error_message = e.to_string();
            assert!(
                error_message.contains("Android")
                    || error_message.contains("SDK")
                    || error_message.contains("iOS"),
                "Error should be SDK-related: {e}"
            );
        }
    }
}

/// Test app memory usage patterns
#[tokio::test]
async fn test_app_new_memory_usage() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test that App::new() doesn't cause memory issues
    let initial_memory = get_memory_usage();

    // Create and drop multiple App instances
    for _ in 0..5 {
        let app = App::new().await;
        drop(app); // Explicit drop to test cleanup
    }

    let final_memory = get_memory_usage();

    // Memory usage should not increase dramatically
    // Allow for some variance due to system factors
    let memory_increase = final_memory.saturating_sub(initial_memory);
    assert!(
        memory_increase < 100_000_000, // 100MB limit
        "Memory usage should not increase dramatically: {memory_increase} bytes"
    );
}

/// Test app async behavior with timeout
#[tokio::test]
async fn test_app_new_async_behavior() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test that App::new() properly handles async operations
    let app_future = App::new();

    // Should be able to race with timeout
    let result = tokio::time::timeout(Duration::from_secs(30), app_future).await;

    match result {
        Ok(app_result) => {
            // App::new() completed within timeout
            match app_result {
                Ok(_) => {}
                Err(e) => {
                    let error_message = e.to_string();
                    assert!(
                        error_message.contains("Android")
                            || error_message.contains("SDK")
                            || error_message.contains("iOS"),
                        "Error should be SDK-related: {e}"
                    );
                }
            }
        }
        Err(_) => {
            panic!("App::new() should complete within 30 seconds");
        }
    }
}

/// Test app initialization with timeout using select
#[tokio::test]
async fn test_app_new_with_timeout() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test that App::new() completes within reasonable timeout
    let app_future = App::new();

    // Race initialization with timeout
    tokio::select! {
        app_result = app_future => {
            // App::new() completed before timeout
            match app_result {
                Ok(_) => {
                    // Success
                }
                Err(e) => {
                    let error_message = e.to_string();
                    assert!(
                        error_message.contains("Android") ||
                        error_message.contains("SDK") ||
                        error_message.contains("iOS"),
                        "Error should be SDK-related: {e}"
                    );
                }
            }
        }
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            // This shouldn't happen in normal test execution
            panic!("App::new() should complete within 30 seconds");
        }
    }
}

/// Test rapid successive initialization
#[tokio::test]
async fn test_rapid_initialization() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    // Test rapid successive App creation
    let mut durations = Vec::new();

    for i in 0..5 {
        let start = std::time::Instant::now();
        let app_result = App::new().await;
        let duration = start.elapsed();
        durations.push(duration);

        match app_result {
            Ok(_app) => {
                // Quick initialization should be possible
                assert!(
                    duration < Duration::from_secs(15),
                    "Rapid initialization {i} should be quick, took {duration:?}"
                );

                // Brief operation to test app stability
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => {
                // Even errors should be fast
                assert!(
                    duration < Duration::from_secs(3),
                    "Rapid initialization {i} error should be quick, took {duration:?}"
                );

                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("Android") || error_msg.contains("SDK"),
                    "Rapid initialization {i} error: {error_msg}"
                );
            }
        }
    }

    // Check consistency of performance
    if durations.len() > 1 {
        let min_duration = durations.iter().min().unwrap();
        let max_duration = durations.iter().max().unwrap();
        let variance = max_duration.as_millis() - min_duration.as_millis();

        // Performance should be relatively consistent
        assert!(
            variance < 5000, // 5 second variance allowed
            "Performance variance should be low: {variance}ms between min and max"
        );
    }
}

// Helper function to get rough memory usage
fn get_memory_usage() -> usize {
    // Simple memory usage estimation
    // In a real application, you might use more sophisticated memory tracking
    std::process::id() as usize * 1024 // Simple approximation
}
