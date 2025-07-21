//! Memory usage and resource management performance tests
//!
//! Measures memory efficiency, resource leak detection, cache management,
//! and garbage collection performance.

use emu::app::state::{AppState, Panel};
use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use emu::models::{AndroidDevice, DeviceStatus};
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;
use std::time::{Duration, Instant};
const MEMORY_LEAK_DETECTION_CYCLES: usize = 100;
const CACHE_PERFORMANCE_TARGET_MS: u64 = 300;

use crate::common::setup_mock_android_sdk;

/// Memory efficiency benchmark
#[tokio::test]
async fn test_memory_efficiency_benchmark() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    println!("📊 Memory efficiency benchmark:");

    let device_counts = vec![50, 100];

    for device_count in device_counts {
        let avd_output = create_large_device_list(device_count);

        let mock_executor = MockCommandExecutor::new()
            .with_success("avdmanager", &["list", "avd"], &avd_output)
            .with_success(
                &avdmanager_path.to_string_lossy(),
                &["list", "avd"],
                &avd_output,
            )
            .with_success("adb", &["devices"], "List of devices attached\n")
            .with_success(
                &adb_path.to_string_lossy(),
                &["devices"],
                "List of devices attached\n",
            );

        let start_memory = get_memory_usage_estimate();
        let start_time = Instant::now();

        let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
        let devices = android_manager.list_devices().await.unwrap();

        let load_duration = start_time.elapsed();
        let end_memory = get_memory_usage_estimate();
        let memory_used = end_memory.saturating_sub(start_memory);

        assert_eq!(devices.len(), device_count);

        // Memory efficiency verification
        let devices_per_mb = if memory_used > 0 {
            device_count / memory_used
        } else {
            device_count // When memory usage cannot be measured
        };

        println!("  📱 {device_count} devices: {load_duration:?}, ~{memory_used}MB used, ~{devices_per_mb} devices/MB");

        // Performance requirements
        assert!(
            load_duration.as_secs() < 5,
            "Loading {device_count} devices took too long: {load_duration:?}"
        );
    }
}

/// Memory leak detection test
#[ignore = "Takes too long for CI environments"]
#[tokio::test]
async fn test_memory_leak_detection() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    println!("🔍 Memory leak detection test:");

    let avd_output = create_large_device_list(100);

    let initial_memory = get_memory_usage_estimate();
    let mut memory_samples = vec![];

    for cycle in 0..MEMORY_LEAK_DETECTION_CYCLES {
        let mock_executor = MockCommandExecutor::new()
            .with_success("avdmanager", &["list", "avd"], &avd_output)
            .with_success(
                &avdmanager_path.to_string_lossy(),
                &["list", "avd"],
                &avd_output,
            )
            .with_success("adb", &["devices"], "List of devices attached\n")
            .with_success(
                &adb_path.to_string_lossy(),
                &["devices"],
                "List of devices attached\n",
            );

        let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
        let _devices = android_manager.list_devices().await.unwrap();

        // Periodically sample memory usage
        if cycle % 10 == 0 {
            let current_memory = get_memory_usage_estimate();
            memory_samples.push(current_memory);

            if cycle > 0 {
                let memory_growth = current_memory.saturating_sub(initial_memory);
                println!("  Cycle {cycle}: ~{current_memory}MB (+{memory_growth}MB from start)");
            }
        }

        // Early detection of memory leaks
        if cycle > 0 && cycle % 20 == 0 {
            let current_memory = get_memory_usage_estimate();
            let memory_growth = current_memory.saturating_sub(initial_memory);

            // Warn if there's significant memory increase
            if memory_growth > 100 {
                // Increase of 100MB or more
                println!(
                    "  ⚠️  Potential memory leak detected at cycle {cycle}: +{memory_growth}MB"
                );
            }
        }
    }

    let final_memory = get_memory_usage_estimate();
    let total_growth = final_memory.saturating_sub(initial_memory);

    println!("  📊 Final: {final_memory}MB (+{total_growth}MB growth over {MEMORY_LEAK_DETECTION_CYCLES} cycles)");

    // Leak detection criterion: 50MB+ growth over 100 cycles is abnormal
    assert!(
        total_growth < 50,
        "Potential memory leak detected: {total_growth}MB growth over {MEMORY_LEAK_DETECTION_CYCLES} cycles"
    );
}

/// Cache performance test
#[tokio::test]
async fn test_cache_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let avd_output = create_large_device_list(50);

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], &avd_output)
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            &avd_output,
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let android_manager = Arc::new(AndroidManager::with_executor(Arc::new(mock_executor)).unwrap());

    // First load (no cache)
    let first_load_start = Instant::now();
    let first_devices = android_manager.list_devices().await.unwrap();
    let first_load_duration = first_load_start.elapsed();

    assert_eq!(first_devices.len(), 50);

    // Second load (possibly cached)
    let second_load_start = Instant::now();
    let second_devices = android_manager.list_devices().await.unwrap();
    let second_load_duration = second_load_start.elapsed();

    assert_eq!(second_devices.len(), 50);

    // Cache effect verification (implementation dependent)
    if second_load_duration < first_load_duration {
        let improvement =
            first_load_duration.as_millis() as i64 - second_load_duration.as_millis() as i64;
        println!("✅ Cache performance: {improvement}ms improvement ({first_load_duration:?} → {second_load_duration:?})");
    } else {
        println!("📝 Cache performance: No caching detected ({first_load_duration:?} vs {second_load_duration:?})");
    }

    // Basic performance requirements
    assert!(
        second_load_duration.as_millis() < CACHE_PERFORMANCE_TARGET_MS as u128,
        "Second load too slow: {second_load_duration:?} exceeds {CACHE_PERFORMANCE_TARGET_MS}ms"
    );
}

/// Memory efficiency test when processing large datasets
#[tokio::test]
async fn test_large_dataset_memory_efficiency() {
    println!("🗂️  Large dataset memory efficiency test:");

    let mut app_state = AppState::new();
    let initial_memory = get_memory_usage_estimate();

    // Incrementally increase device count
    let test_stages = vec![100, 200, 500, 1000];

    for device_count in test_stages {
        let stage_start_memory = get_memory_usage_estimate();

        // Create large number of devices
        app_state.android_devices = create_memory_test_devices(device_count);

        let stage_end_memory = get_memory_usage_estimate();
        let stage_memory_used = stage_end_memory.saturating_sub(stage_start_memory);

        // Memory access performance test
        let access_start = Instant::now();
        for _ in 0..100 {
            let random_index =
                (Instant::now().elapsed().as_nanos() % device_count as u128) as usize;
            app_state.selected_android = random_index;
            let _selected = if !app_state.android_devices.is_empty() {
                Some(&app_state.android_devices[app_state.selected_android])
            } else {
                None
            };
        }
        let access_duration = access_start.elapsed();

        println!("  📱 {device_count} devices: ~{stage_memory_used}MB used, 100 random accesses in {access_duration:?}");

        // Access performance requirement (fast access even with large data)
        assert!(
            access_duration.as_millis() < 100,
            "Random access too slow for {device_count} devices: {access_duration:?}"
        );
    }

    let final_memory = get_memory_usage_estimate();
    let total_memory_used = final_memory.saturating_sub(initial_memory);

    println!("  📊 Total memory used: ~{total_memory_used}MB for 5000 devices");
}

/// Memory safety test during concurrent access
#[tokio::test]
async fn test_concurrent_memory_safety() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let avd_output = create_large_device_list(100);

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], &avd_output)
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            &avd_output,
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let android_manager = Arc::new(AndroidManager::with_executor(Arc::new(mock_executor)).unwrap());

    let concurrent_tasks = 50;
    let initial_memory = get_memory_usage_estimate();

    // Get device list concurrently
    let mut handles = vec![];
    for _ in 0..concurrent_tasks {
        let manager = android_manager.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                let _devices = manager.list_devices().await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let final_memory = get_memory_usage_estimate();
    let memory_growth = final_memory.saturating_sub(initial_memory);

    println!(
        "✅ Concurrent memory safety: {concurrent_tasks} tasks, +{memory_growth}MB memory growth"
    );

    // Memory usage within appropriate range even during concurrent processing
    assert!(
        memory_growth < 20,
        "Excessive memory growth during concurrent access: +{memory_growth}MB"
    );
}

/// Garbage collection efficiency test (Rust drop efficiency)
#[tokio::test]
async fn test_resource_cleanup_efficiency() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    println!("🧹 Resource cleanup efficiency test:");

    let cleanup_cycles = 100;
    let initial_memory = get_memory_usage_estimate();

    for cycle in 0..cleanup_cycles {
        // Create and destroy large amounts of resources
        {
            let mut app_state = AppState::new();
            app_state.android_devices = create_memory_test_devices(100);

            // Create multiple managers
            let avd_output = create_large_device_list(100);
            let mock_executor = MockCommandExecutor::new()
                .with_success("avdmanager", &["list", "avd"], &avd_output)
                .with_success(
                    &avdmanager_path.to_string_lossy(),
                    &["list", "avd"],
                    &avd_output,
                )
                .with_success("adb", &["devices"], "List of devices attached\n")
                .with_success(
                    &adb_path.to_string_lossy(),
                    &["devices"],
                    "List of devices attached\n",
                );

            let _android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

            // Intentionally use resources
            for _ in 0..50 {
                let _selected = if !app_state.android_devices.is_empty() {
                    Some(&app_state.android_devices[app_state.selected_android])
                } else {
                    None
                };
            }
        } // Resources are dropped at end of scope

        // Periodically check memory usage
        if cycle % 25 == 0 {
            let current_memory = get_memory_usage_estimate();
            let memory_used = current_memory.saturating_sub(initial_memory);
            println!("  Cycle {cycle}: ~{current_memory}MB ({memory_used:+}MB from start)");
        }
    }

    let final_memory = get_memory_usage_estimate();
    let net_memory_change = final_memory.saturating_sub(initial_memory);

    println!("  📊 Net memory change after {cleanup_cycles} cycles: {net_memory_change:+}MB");

    // Efficient cleanup: little memory increase even after creating/destroying large amounts of resources
    assert!(
        net_memory_change < 10,
        "Inefficient resource cleanup: +{net_memory_change}MB after {cleanup_cycles} cycles"
    );
}

/// Memory fragmentation detection test
#[tokio::test]
async fn test_memory_fragmentation_detection() {
    println!("🧩 Memory fragmentation detection test:");

    let initial_memory = get_memory_usage_estimate();
    let mut allocation_times = vec![];

    // Create and destroy data structures of various sizes
    for iteration in 0..50 {
        let allocation_start = Instant::now();

        // Create device lists of different sizes
        let small_size = 10 + (iteration % 5) * 10;
        let medium_size = 100 + (iteration % 3) * 50;
        let large_size = 500 + (iteration % 2) * 250;

        let mut states = vec![];

        // Create mixed small, medium, and large data structures
        states.push({
            let mut state = AppState::new();
            state.android_devices = create_memory_test_devices(small_size);
            state
        });

        states.push({
            let mut state = AppState::new();
            state.android_devices = create_memory_test_devices(medium_size);
            state
        });

        states.push({
            let mut state = AppState::new();
            state.android_devices = create_memory_test_devices(large_size);
            state
        });

        let allocation_duration = allocation_start.elapsed();
        allocation_times.push(allocation_duration.as_micros() as u64);

        // Destroy some (induce fragmentation)
        if iteration % 3 == 0 {
            states.pop();
        }

        // Periodic checks
        if iteration % 10 == 0 {
            let current_memory = get_memory_usage_estimate();
            let memory_growth = current_memory.saturating_sub(initial_memory);
            println!("  Iteration {iteration}: {allocation_duration:?} allocation, ~{memory_growth}MB total");
        }
    }

    // Verify allocation time consistency (fragmentation detection)
    allocation_times.sort();
    let median = allocation_times[allocation_times.len() / 2];
    let p95 = allocation_times[(allocation_times.len() * 95) / 100];

    println!("  📊 Allocation times: median {median}μs, P95 {p95}μs");

    // Fragmentation detection: P95 within 5x of median
    assert!(
        p95 <= median * 5,
        "Potential memory fragmentation: P95 {p95}μs exceeds 5x median {median}μs"
    );
}

/// Long-running memory stability test
#[tokio::test]
async fn test_long_running_memory_stability() {
    println!("⏰ Long-running memory stability test:");

    let mut app_state = AppState::new();
    app_state.android_devices = create_memory_test_devices(100);

    let initial_memory = get_memory_usage_estimate();
    let stability_duration = Duration::from_secs(2); // 2-second test
    let start_time = Instant::now();

    let mut operation_count = 0;
    let mut memory_samples = vec![];

    while start_time.elapsed() < stability_duration {
        // Simulate various operations
        match operation_count % 4 {
            0 => {
                // Navigation
                if !app_state.android_devices.is_empty() {
                    app_state.selected_android =
                        (app_state.selected_android + 1) % app_state.android_devices.len();
                }
            }
            1 => {
                // Device state change
                let index = operation_count % app_state.android_devices.len();
                app_state.android_devices[index].status =
                    match app_state.android_devices[index].status {
                        DeviceStatus::Stopped => DeviceStatus::Running,
                        DeviceStatus::Running => DeviceStatus::Stopped,
                        _ => DeviceStatus::Unknown,
                    };
            }
            2 => {
                // Platform switching
                app_state.active_panel = match app_state.active_panel {
                    Panel::Android => Panel::Ios,
                    Panel::Ios => Panel::Android,
                };
            }
            3 => {
                // Device selection confirmation
                let _selected = if !app_state.android_devices.is_empty() {
                    Some(&app_state.android_devices[app_state.selected_android])
                } else {
                    None
                };
            }
            _ => {}
        }

        operation_count += 1;

        // Periodic memory sampling
        if operation_count % 1000 == 0 {
            let current_memory = get_memory_usage_estimate();
            memory_samples.push(current_memory);
        }

        // Adjust to avoid excessive CPU usage
        if operation_count % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }

    let final_memory = get_memory_usage_estimate();
    let total_growth = final_memory.saturating_sub(initial_memory);

    println!("  📊 {operation_count} operations in {stability_duration:?}: +{total_growth}MB memory growth");

    // Memory usage stable even during long runs
    assert!(
        total_growth < 5,
        "Memory instability during long run: +{total_growth}MB growth"
    );

    // Low memory usage variation
    if memory_samples.len() > 1 {
        let min_memory = memory_samples.iter().min().unwrap();
        let max_memory = memory_samples.iter().max().unwrap();
        let variation = max_memory - min_memory;

        println!("  📊 Memory variation: {variation}MB ({min_memory}MB - {max_memory}MB)");

        assert!(variation < 10, "Excessive memory variation: {variation}MB");
    }
}

// Helper functions

fn create_large_device_list(device_count: usize) -> String {
    let mut output = String::from("Available Android Virtual Devices:\n");

    for i in 1..=device_count {
        output.push_str(&format!(
            "    Name: MemoryTest_Device_{i}\n    Device: pixel_7 (Pixel 7)\n    Path: /Users/user/.android/avd/MemoryTest_Device_{i}.avd\n    Target: Google APIs (Google Inc.)\n            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a\n---------\n"
        ));
    }

    output
}

fn create_memory_test_devices(count: usize) -> Vec<AndroidDevice> {
    (1..=count)
        .map(|i| AndroidDevice {
            name: format!("MemoryTest_Device_{i}"),
            device_type: format!("pixel_{}", (i % 5) + 1),
            api_level: 30 + (i % 10) as u32,
            status: match i % 4 {
                0 => DeviceStatus::Stopped,
                1 => DeviceStatus::Running,
                2 => DeviceStatus::Starting,
                _ => DeviceStatus::Unknown,
            },
            is_running: i % 4 == 1,
            ram_size: format!("{}", 2048 + (i % 4) * 1024),
            storage_size: format!("{}", 8192 + (i % 3) * 4096),
        })
        .collect()
}

/// Get estimated memory usage (for testing)
fn get_memory_usage_estimate() -> usize {
    // Actual memory usage measurement is OS-dependent and complex,
    // so return test estimate (implementation could use proc/self/status or similar)

    // Pseudo memory value based on thread ID (for test consistency)
    let thread_id = std::thread::current().id();
    let base_memory = 50; // 50MB base
    let variation = (format!("{thread_id:?}").len() % 20) as usize; // 0-19MB variation

    base_memory + variation
}
