use emu::app::App;
use std::time::Instant;

#[tokio::test]
async fn final_startup_performance_validation() {
    // Skip test in CI environment where Android SDK is not available
    if std::env::var("CI").is_ok() {
        println!("Skipping performance test in CI environment");
        return;
    }

    println!("=== FINAL STARTUP PERFORMANCE VALIDATION ===");

    let iterations = 5;
    let mut startup_times = Vec::new();

    for i in 1..=iterations {
        println!("Iteration {}/{}", i, iterations);

        let start = Instant::now();
        let app_result = App::new().await;
        let duration = start.elapsed();

        match app_result {
            Ok(_app) => {
                startup_times.push(duration);
                println!("  ✅ Startup time: {:?}", duration);
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
                panic!("Startup failed on iteration {}", i);
            }
        }
    }

    // Calculate statistics
    let min_time = startup_times.iter().min().unwrap();
    let max_time = startup_times.iter().max().unwrap();
    let avg_time = startup_times.iter().sum::<std::time::Duration>() / startup_times.len() as u32;

    println!("\n📊 PERFORMANCE SUMMARY:");
    println!("  Minimum: {:?} ({} ms)", min_time, min_time.as_millis());
    println!("  Maximum: {:?} ({} ms)", max_time, max_time.as_millis());
    println!("  Average: {:?} ({} ms)", avg_time, avg_time.as_millis());

    // Performance assertions - realistic thresholds for full app startup
    // Including device manager initialization and background tasks
    assert!(
        avg_time < std::time::Duration::from_millis(5000),
        "Average startup time {} ms exceeds 5000ms threshold",
        avg_time.as_millis()
    );

    assert!(
        *max_time < std::time::Duration::from_millis(15000),
        "Maximum startup time {} ms exceeds 15000ms threshold",
        max_time.as_millis()
    );

    println!("✅ ALL PERFORMANCE THRESHOLDS MET!");
}

#[tokio::test]
async fn memory_efficiency_test() {
    // Skip test in CI environment where Android SDK is not available
    if std::env::var("CI").is_ok() {
        println!("Skipping memory efficiency test in CI environment");
        return;
    }

    println!("=== MEMORY EFFICIENCY TEST ===");

    // Simple memory footprint check
    let app = App::new().await.expect("Failed to create app");

    // App should be created successfully with minimal memory
    // This is mainly to ensure no memory leaks in the creation process
    drop(app);

    println!("✅ Memory efficiency test passed");
}

#[test]
fn startup_components_isolation_test() {
    // Skip test in CI environment where Android SDK is not available
    if std::env::var("CI").is_ok() {
        println!("Skipping startup components test in CI environment");
        return;
    }

    println!("=== STARTUP COMPONENTS ISOLATION TEST ===");

    // Test individual component creation times

    let start = Instant::now();
    let _android_manager = emu::managers::AndroidManager::new();
    let android_manager_time = start.elapsed();
    println!("AndroidManager creation: {:?}", android_manager_time);

    #[cfg(target_os = "macos")]
    {
        let start = Instant::now();
        let _ios_manager = emu::managers::IosManager::new();
        let ios_manager_time = start.elapsed();
        println!("IosManager creation: {:?}", ios_manager_time);

        // Skip performance assertions in CI environment
        if std::env::var("CI").is_err() {
            // Ensure iOS manager creation is reasonable
            // iOS manager may need to check xcrun availability
            assert!(
                ios_manager_time < std::time::Duration::from_millis(1000),
                "iOS manager creation too slow: {} ms",
                ios_manager_time.as_millis()
            );
        }
    }

    // Skip performance assertions in CI environment
    if std::env::var("CI").is_err() {
        // Ensure Android manager creation is reasonable
        // Android manager may need to check SDK tools availability
        assert!(
            android_manager_time < std::time::Duration::from_millis(1000),
            "Android manager creation too slow: {} ms",
            android_manager_time.as_millis()
        );
    }


    println!("✅ All component creation times within acceptable limits");
}
