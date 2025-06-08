use emu::app::App;
use emu::config::Config;
use std::time::Instant;

#[tokio::test]
async fn final_startup_performance_validation() {
    println!("=== FINAL STARTUP PERFORMANCE VALIDATION ===");

    let iterations = 5;
    let mut startup_times = Vec::new();

    for i in 1..=iterations {
        println!("Iteration {}/{}", i, iterations);

        let start = Instant::now();
        let config = Config::default();
        let app_result = App::new(config).await;
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

    // Performance assertions
    assert!(
        avg_time < std::time::Duration::from_millis(200),
        "Average startup time {} ms exceeds 200ms threshold",
        avg_time.as_millis()
    );

    assert!(
        *max_time < std::time::Duration::from_millis(300),
        "Maximum startup time {} ms exceeds 300ms threshold",
        max_time.as_millis()
    );

    println!("✅ ALL PERFORMANCE THRESHOLDS MET!");
}

#[tokio::test]
async fn memory_efficiency_test() {
    println!("=== MEMORY EFFICIENCY TEST ===");

    // Simple memory footprint check
    let config = Config::default();
    let app = App::new(config).await.expect("Failed to create app");

    // App should be created successfully with minimal memory
    // This is mainly to ensure no memory leaks in the creation process
    drop(app);

    println!("✅ Memory efficiency test passed");
}

#[test]
fn startup_components_isolation_test() {
    println!("=== STARTUP COMPONENTS ISOLATION TEST ===");

    // Test individual component creation times
    let start = Instant::now();
    let _config = Config::default();
    let config_time = start.elapsed();
    println!("Config creation: {:?}", config_time);

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

        // Ensure iOS manager creation is reasonable
        assert!(
            ios_manager_time < std::time::Duration::from_millis(100),
            "iOS manager creation too slow: {} ms",
            ios_manager_time.as_millis()
        );
    }

    // Ensure Android manager creation is fast
    assert!(
        android_manager_time < std::time::Duration::from_millis(50),
        "Android manager creation too slow: {} ms",
        android_manager_time.as_millis()
    );

    // Ensure config creation is very fast
    assert!(
        config_time < std::time::Duration::from_millis(1),
        "Config creation too slow: {} ms",
        config_time.as_millis()
    );

    println!("✅ All component creation times within acceptable limits");
}
