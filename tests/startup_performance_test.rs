use emu::app::App;
use emu::managers::common::DeviceManager;
use std::env;
use std::time::Instant;

#[tokio::test]
async fn test_startup_performance() {
    println!("=== EMU STARTUP PERFORMANCE TEST ===");

    // Test App initialization
    let start = Instant::now();
    let app_result = App::new().await;
    let app_init_duration = start.elapsed();
    println!("App initialization: {app_init_duration:?}");

    match app_result {
        Ok(_app) => {
            println!("✅ App created successfully");
        }
        Err(e) => {
            println!("❌ App creation failed: {e}");
            panic!("App creation failed");
        }
    }

    println!("📊 Total startup time: {app_init_duration:?}");

    // Performance thresholds
    let total_time = app_init_duration;
    if total_time > std::time::Duration::from_millis(500) {
        println!(
            "⚠️  SLOW STARTUP WARNING: {} > 500ms threshold",
            total_time.as_millis()
        );
    } else {
        println!(
            "✅ Startup performance acceptable: {} <= 500ms",
            total_time.as_millis()
        );
    }
}

#[tokio::test]
async fn test_app_components_performance() {
    println!("=== EMU COMPONENT PERFORMANCE TEST ===");

    // Component performance test

    // Test individual component initialization times
    let start = Instant::now();
    let android_manager_result = emu::managers::AndroidManager::new();
    let android_manager_duration = start.elapsed();
    println!("1. AndroidManager creation: {android_manager_duration:?}");

    match android_manager_result {
        Ok(android_manager) => {
            // Test device listing performance
            let start = Instant::now();
            let devices_result = android_manager.list_devices().await;
            let list_devices_duration = start.elapsed();
            println!("2. Android list_devices(): {list_devices_duration:?}");

            match devices_result {
                Ok(devices) => {
                    println!("   Found {} Android devices", devices.len());
                }
                Err(e) => {
                    println!("   Android device listing failed: {e}");
                }
            }
        }
        Err(e) => {
            println!("   AndroidManager creation failed: {e}");
        }
    }

    // Test iOS manager if on macOS
    #[cfg(target_os = "macos")]
    {
        let start = Instant::now();
        let ios_manager_result = emu::managers::IosManager::new();
        let ios_manager_duration = start.elapsed();
        println!("3. IosManager creation: {ios_manager_duration:?}");

        match ios_manager_result {
            Ok(ios_manager) => {
                let start = Instant::now();
                let devices_result = ios_manager.list_devices().await;
                let list_devices_duration = start.elapsed();
                println!("4. iOS list_devices(): {list_devices_duration:?}");

                match devices_result {
                    Ok(devices) => {
                        println!("   Found {} iOS devices", devices.len());
                    }
                    Err(e) => {
                        println!("   iOS device listing failed: {e}");
                    }
                }
            }
            Err(e) => {
                println!("   IosManager creation failed: {e}");
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        println!("3. IosManager: Skipped (not macOS)");
        println!("4. iOS list_devices(): Skipped (not macOS)");
    }
}

#[tokio::test]
async fn test_background_cache_performance() {
    println!("=== BACKGROUND CACHE PERFORMANCE TEST ===");

    // Create app and measure cache loading
    let start = Instant::now();
    let app_result = App::new().await;
    let total_duration = start.elapsed();

    match app_result {
        Ok(_app) => {
            println!(
                "✅ App with background cache created in: {:?}",
                total_duration
            );

            // Wait a bit for background cache to potentially complete
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!("📋 Background cache should be loading in parallel");
        }
        Err(e) => {
            println!("❌ App creation failed: {e}");
        }
    }
}

#[tokio::test]
async fn test_startup_without_device_loading() {
    println!("=== STARTUP WITHOUT DEVICE LOADING TEST ===");

    // This test simulates what startup should be like with minimal loading
    let start = Instant::now();

    // Just create the managers without listing devices
    let _android_manager_result = emu::managers::AndroidManager::new();
    let android_duration = start.elapsed();
    println!("1. AndroidManager only: {android_duration:?}");

    #[cfg(target_os = "macos")]
    {
        let start = Instant::now();
        let _ios_manager_result = emu::managers::IosManager::new();
        let ios_duration = start.elapsed();
        println!("2. IosManager only: {ios_duration:?}");
    }

    println!("📊 Manager creation only: {android_duration:?}");

    if android_duration > std::time::Duration::from_millis(100) {
        println!(
            "⚠️  Manager creation is slow: {} > 100ms",
            android_duration.as_millis()
        );
    } else {
        println!(
            "✅ Manager creation fast: {} <= 100ms",
            android_duration.as_millis()
        );
    }
}

#[tokio::test]
async fn test_parallel_command_performance() {
    println!("=== PARALLEL COMMAND EXECUTION PERFORMANCE TEST ===");

    // Test Android device listing with parallel commands
    match emu::managers::AndroidManager::new() {
        Ok(android_manager) => {
            // Test normal execution
            env::remove_var("EMU_PARALLEL_COMMANDS");
            let start = Instant::now();
            let normal_result = android_manager.list_devices().await;
            let normal_duration = start.elapsed();
            println!("1. Normal list_devices(): {normal_duration:?}");
            if let Ok(devices) = normal_result {
                println!("   Found {} devices", devices.len());
            }

            // Test parallel execution
            env::set_var("EMU_PARALLEL_COMMANDS", "true");
            let start = Instant::now();
            let parallel_result = android_manager.list_devices().await;
            let parallel_duration = start.elapsed();
            println!("2. Parallel list_devices(): {parallel_duration:?}");
            if let Ok(devices) = parallel_result {
                println!("   Found {} devices", devices.len());
            }

            // Calculate improvement
            if normal_duration > parallel_duration {
                let improvement = ((normal_duration.as_millis() - parallel_duration.as_millis())
                    as f64
                    / normal_duration.as_millis() as f64)
                    * 100.0;
                println!("✅ Performance improved by {improvement:.1}%");
            } else {
                println!("⚠️  No performance improvement detected");
            }

            // Clean up
            env::remove_var("EMU_PARALLEL_COMMANDS");
        }
        Err(e) => {
            println!("❌ AndroidManager creation failed: {e}");
        }
    }
}

#[tokio::test]
async fn test_incremental_refresh_performance() {
    println!("=== INCREMENTAL REFRESH PERFORMANCE TEST ===");

    // Create app and test refresh performance
    match App::new().await {
        Ok(mut _app) => {
            // Warm up - initial device load
            println!("Warming up with initial device load...");
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            // Test normal refresh
            env::remove_var("EMU_INCREMENTAL_REFRESH");
            let start = Instant::now();
            // We can't directly call refresh_devices_smart, but we can measure through the app
            // This is a placeholder for the actual test
            let normal_duration = start.elapsed();
            println!("1. Normal refresh: {normal_duration:?}");

            // Test incremental refresh
            env::set_var("EMU_INCREMENTAL_REFRESH", "true");
            let start = Instant::now();
            // Placeholder for actual incremental refresh test
            let incremental_duration = start.elapsed();
            println!("2. Incremental refresh: {incremental_duration:?}");

            // Clean up
            env::remove_var("EMU_INCREMENTAL_REFRESH");

            println!("Note: Full incremental refresh testing requires running the app");
        }
        Err(e) => {
            println!("❌ App creation failed: {e}");
        }
    }
}
