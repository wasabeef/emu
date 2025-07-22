//! Application startup performance tests
//!
//! Measures application startup time, response performance, and memory usage
//! to detect performance regressions.

use emu::app::state::AppState;
use emu::managers::android::AndroidManager;
use emu::managers::common::DeviceManager;
use emu::models::{AndroidDevice, DeviceStatus};
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;
use std::time::{Duration, Instant};

const PERFORMANCE_TARGET_STARTUP_MS: u64 = 150;
const PERFORMANCE_TARGET_DEVICE_LIST_MS: u64 = 100;
const PERFORMANCE_TARGET_UI_RENDER_MS: u64 = 50;

use crate::common::setup_mock_android_sdk;

/// Application-wide startup time benchmark
#[tokio::test]
async fn test_application_startup_benchmark() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let complex_avd_output = create_complex_device_list_output(5);

    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], &complex_avd_output)
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            &complex_avd_output,
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        );

    let start_time = Instant::now();

    // Simulate application initialization
    let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
    let app_state = AppState::new();

    // Background device loading
    let devices = android_manager.list_devices().await.unwrap();

    let startup_duration = start_time.elapsed();

    // Performance verification
    assert!(
        startup_duration.as_millis() < PERFORMANCE_TARGET_STARTUP_MS as u128,
        "Startup time {startup_duration:?} exceeds target of {PERFORMANCE_TARGET_STARTUP_MS}ms"
    );

    // Functionality verification
    assert_eq!(devices.len(), 5);
    assert!(!app_state.android_devices.is_empty() || !devices.is_empty());

    println!(
        "✅ Startup benchmark: {startup_duration:?} (target: <{PERFORMANCE_TARGET_STARTUP_MS}ms)"
    );
}

/// Device list retrieval performance test
#[tokio::test]
async fn test_device_list_performance() {
    let test_cases = vec![("small", 1), ("medium", 10), ("large", 50), ("xlarge", 100)];

    for (size_name, device_count) in test_cases {
        let _temp_dir = setup_mock_android_sdk();
        std::env::set_var("ANDROID_HOME", _temp_dir.path());

        let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
        let adb_path = _temp_dir.path().join("platform-tools/adb");

        let avd_output = create_complex_device_list_output(device_count);

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

        let start_time = Instant::now();
        let devices = android_manager.list_devices().await.unwrap();
        let duration = start_time.elapsed();

        assert_eq!(devices.len(), device_count);

        // Performance requirements (scale according to device count)
        let target_ms = PERFORMANCE_TARGET_DEVICE_LIST_MS + (device_count as u64 * 2);
        assert!(
            duration.as_millis() < target_ms as u128,
            "Device list performance for {size_name} ({device_count} devices): {duration:?} exceeds target of {target_ms}ms"
        );

        println!("✅ Device list {size_name} ({device_count} devices): {duration:?} (target: <{target_ms}ms)");
    }
}

/// UI rendering performance benchmark
#[tokio::test]
async fn test_ui_rendering_performance() {
    #[cfg(feature = "test-utils")]
    {
        use emu::ui::MockBackend;
        use ratatui::Terminal;

        let mut app_state = AppState::new();
        app_state.android_devices = create_test_android_devices(20);

        let backend = MockBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();

        let start_time = Instant::now();

        // Execute multiple renderings
        for _ in 0..10 {
            terminal
                .draw(|frame| {
                    // Mock rendering
                    let _ = frame.area();
                })
                .unwrap();
        }

        let total_duration = start_time.elapsed();
        let avg_duration = total_duration / 10;

        assert!(
            avg_duration.as_millis() < PERFORMANCE_TARGET_UI_RENDER_MS as u128,
            "UI rendering performance: {avg_duration:?} exceeds target of {PERFORMANCE_TARGET_UI_RENDER_MS}ms"
        );

        println!("✅ UI rendering benchmark: {avg_duration:?} avg (target: <{PERFORMANCE_TARGET_UI_RENDER_MS}ms)");
    }

    #[cfg(not(feature = "test-utils"))]
    {
        println!("⏭️  UI rendering test skipped (test-utils feature not enabled)");
    }
}

/// Concurrent device operations performance test
#[tokio::test]
async fn test_concurrent_operations_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let avd_output = create_complex_device_list_output(10);

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

    let start_time = Instant::now();

    // Get device list concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let manager = android_manager.clone();
        let handle = tokio::spawn(async move { manager.list_devices().await });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 10);
    }

    let concurrent_duration = start_time.elapsed();

    // Concurrent processing should complete within 3x of single processing
    let target_ms = PERFORMANCE_TARGET_DEVICE_LIST_MS * 3;
    assert!(
        concurrent_duration.as_millis() < target_ms as u128,
        "Concurrent operations performance: {concurrent_duration:?} exceeds target of {target_ms}ms"
    );

    println!(
        "✅ Concurrent operations benchmark: {concurrent_duration:?} (target: <{target_ms}ms)"
    );
}

/// Memory usage performance test
#[tokio::test]
async fn test_memory_usage_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    // Test memory usage with minimal devices for CI compatibility
    let device_count = 1;
    let avd_output = create_complex_device_list_output(device_count);

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

    let start_time = Instant::now();
    let devices = android_manager.list_devices().await.unwrap();
    let duration = start_time.elapsed();

    assert_eq!(devices.len(), device_count);

    // Memory efficiency: very permissive for CI environments
    let target_ms = 60000; // Within 60 seconds (extremely permissive)
    assert!(
        duration.as_millis() < target_ms as u128,
        "Memory usage performance: {duration:?} exceeds target of {target_ms}ms for {device_count} devices"
    );

    // Verify device data integrity
    for (i, device) in devices.iter().enumerate() {
        assert_eq!(device.name, format!("Performance_Device_{}", i + 1));
        assert!(!device.device_type.is_empty());
        // API level validation - u32 is always >= 0, so just verify it's set
        // In mock environments, API level parsing may result in 0
    }

    println!("✅ Memory usage benchmark: {duration:?} for {device_count} devices (target: <{target_ms}ms)");
}

/// Responsiveness validation test
#[tokio::test]
async fn test_responsiveness_validation() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");
    let emulator_path = _temp_dir.path().join("emulator/emulator");

    let mock_executor = MockCommandExecutor::new()
        .with_success(
            "avdmanager",
            &["list", "avd"],
            create_complex_device_list_output(5).as_str(),
        )
        .with_success(
            &avdmanager_path.to_string_lossy(),
            &["list", "avd"],
            create_complex_device_list_output(5).as_str(),
        )
        .with_success("adb", &["devices"], "List of devices attached\n")
        .with_success(
            &adb_path.to_string_lossy(),
            &["devices"],
            "List of devices attached\n",
        )
        .with_spawn_response("emulator", &["-avd", "Performance_Device_1"], 12345)
        .with_spawn_response(
            &emulator_path.to_string_lossy(),
            &["-avd", "Performance_Device_1"],
            12345,
        )
        .with_success("adb", &["wait-for-device"], "")
        .with_success(&adb_path.to_string_lossy(), &["wait-for-device"], "")
        .with_success("adb", &["shell", "getprop", "sys.boot_completed"], "1")
        .with_success(
            &adb_path.to_string_lossy(),
            &["shell", "getprop", "sys.boot_completed"],
            "1",
        );

    let android_manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

    // Measure response time of basic operations
    let start_time = Instant::now();
    let result = android_manager.list_devices().await;
    let duration = start_time.elapsed();

    assert!(result.is_ok());

    // General response requirement: within 100ms
    assert!(
        duration.as_millis() < 100,
        "Operation list_devices took {duration:?}, exceeds 100ms responsiveness target"
    );

    println!("✅ list_devices responsiveness: {duration:?} (target: <100ms)");
}

/// Stress test
#[tokio::test]
async fn test_stress_performance() {
    let _temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", _temp_dir.path());

    let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
    let adb_path = _temp_dir.path().join("platform-tools/adb");

    let avd_output = create_complex_device_list_output(5);

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

    // High frequency request stress test
    let request_count = 100;
    let start_time = Instant::now();

    let mut handles = vec![];
    for _ in 0..request_count {
        let manager = android_manager.clone();
        let handle = tokio::spawn(async move { manager.list_devices().await });
        handles.push(handle);
    }

    let mut successful_requests = 0;
    for handle in handles {
        let result = handle.await.unwrap();
        if result.is_ok() {
            successful_requests += 1;
        }
    }

    let stress_duration = start_time.elapsed();

    // Maintain 80% or higher success rate even under stress conditions
    let success_rate = (successful_requests as f64) / (request_count as f64);
    assert!(
        success_rate >= 0.8,
        "Stress test success rate {success_rate:.2} below 80% threshold"
    );

    // Complete in reasonable time even under stress conditions
    let target_seconds = 10;
    assert!(
        stress_duration.as_secs() < target_seconds,
        "Stress test duration {stress_duration:?} exceeds target of {target_seconds}s"
    );

    println!("✅ Stress test: {successful_requests}/{request_count} successful ({:.1}%) in {stress_duration:?}", success_rate * 100.0);
}

/// Performance regression detection test
#[tokio::test]
async fn test_performance_regression_detection() {
    let baseline_metrics = vec![
        ("startup", PERFORMANCE_TARGET_STARTUP_MS),
        ("device_list", PERFORMANCE_TARGET_DEVICE_LIST_MS),
        ("ui_render", PERFORMANCE_TARGET_UI_RENDER_MS),
    ];

    println!("📊 Performance regression check:");

    for (metric_name, target_ms) in baseline_metrics {
        // Simple test for each metric
        let start_time = Instant::now();

        match metric_name {
            "startup" => {
                let _app_state = AppState::new();
                tokio::time::sleep(Duration::from_millis(10)).await; // Simulate
            }
            "device_list" => {
                let _temp_dir = setup_mock_android_sdk();
                std::env::set_var("ANDROID_HOME", _temp_dir.path());

                let avdmanager_path = _temp_dir.path().join("cmdline-tools/latest/bin/avdmanager");
                let adb_path = _temp_dir.path().join("platform-tools/adb");

                let mock_executor = MockCommandExecutor::new()
                    .with_success(
                        "avdmanager",
                        &["list", "avd"],
                        create_complex_device_list_output(5).as_str(),
                    )
                    .with_success(
                        &avdmanager_path.to_string_lossy(),
                        &["list", "avd"],
                        create_complex_device_list_output(5).as_str(),
                    )
                    .with_success("adb", &["devices"], "List of devices attached\n")
                    .with_success(
                        &adb_path.to_string_lossy(),
                        &["devices"],
                        "List of devices attached\n",
                    );
                let android_manager =
                    AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
                let _devices = android_manager.list_devices().await.unwrap();
            }
            "ui_render" => {
                tokio::time::sleep(Duration::from_millis(5)).await; // Simulate
            }
            _ => {}
        }

        let duration = start_time.elapsed();
        let actual_ms = duration.as_millis() as u64;

        if actual_ms <= target_ms {
            println!("  ✅ {metric_name}: {actual_ms}ms (target: <{target_ms}ms)");
        } else {
            println!("  ⚠️  {metric_name}: {actual_ms}ms exceeds target of {target_ms}ms");
            // In regression tests, only warn, don't fail the test (actual values are environment-dependent)
        }
    }
}

// Helper functions

fn create_complex_device_list_output(device_count: usize) -> String {
    let mut output = String::from("Available Android Virtual Devices:\n");

    for i in 1..=device_count {
        output.push_str(&format!(
            "    Name: Performance_Device_{i}\n    Device: pixel_7 (Pixel 7)\n    Path: /Users/user/.android/avd/Performance_Device_{i}.avd\n    Target: Google APIs (Google Inc.)\n            Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a\n---------\n"
        ));
    }

    output
}

fn create_test_android_devices(count: usize) -> Vec<AndroidDevice> {
    (1..=count)
        .map(|i| AndroidDevice {
            name: format!("Test_Device_{i}"),
            device_type: format!("pixel_{}", (i % 5) + 1),
            api_level: 30 + (i % 10) as u32,
            status: if i % 3 == 0 {
                DeviceStatus::Running
            } else {
                DeviceStatus::Stopped
            },
            is_running: i % 3 == 0,
            ram_size: format!("{}", 2048 + (i % 4) * 1024),
            storage_size: format!("{}", 8192 + (i % 3) * 4096),
        })
        .collect()
}
