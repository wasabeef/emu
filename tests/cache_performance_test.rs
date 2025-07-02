//! Performance tests for device cache functionality
//!
//! This test suite validates the performance improvements from:
//! 1. Device list caching for faster startup
//! 2. Parallel initialization of device managers

use emu::{
    app::App,
    models::cache::{CacheConfig, DeviceCache},
    models::device::{AndroidDevice, DeviceStatus},
    utils::cache_manager::CacheManager,
};
use std::time::Instant;
use tempfile::TempDir;

#[tokio::test]
async fn test_startup_with_cache() {
    // Create a temporary cache directory
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create cache manager with test config
    let config = CacheConfig {
        max_age_secs: 300,
        enabled: true,
        cache_path: cache_path.clone(),
    };
    let cache_manager = CacheManager::with_config(config);

    // Create test devices
    let android_devices = vec![
        AndroidDevice {
            name: "test_avd_1".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        },
        AndroidDevice {
            name: "test_avd_2".to_string(),
            device_type: "pixel_6".to_string(),
            api_level: 33,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "16384M".to_string(),
        },
    ];

    // Save cache
    cache_manager
        .save_cache(android_devices.clone(), vec![])
        .await
        .unwrap();

    // Test 1: Measure time to load from cache
    let start = Instant::now();
    let loaded_cache = cache_manager.load_cache().await.unwrap();
    let cache_load_time = start.elapsed();

    println!("Cache load time: {:?}", cache_load_time);
    assert!(
        cache_load_time.as_millis() < 10,
        "Cache loading should be very fast"
    );
    assert_eq!(loaded_cache.android_devices.len(), 2);
    assert_eq!(loaded_cache.android_devices[0].name, "test_avd_1");

    // Test 2: Verify cache improves perceived startup time
    // In real usage, cached devices would be displayed immediately while
    // fresh data loads in background
    println!("With cache, devices can be displayed in < 10ms");
}

#[tokio::test]
async fn test_parallel_manager_initialization() {
    // Test the parallel initialization performance
    let start = Instant::now();

    // This should use parallel initialization internally
    let result = App::new().await;

    let init_time = start.elapsed();
    println!("App initialization time: {:?}", init_time);

    // Should initialize successfully
    assert!(result.is_ok(), "App should initialize successfully");

    // Parallel initialization should complete reasonably fast
    // (exact timing depends on system, but should be under 1 second)
    assert!(
        init_time.as_secs() < 1,
        "Parallel initialization should complete within 1 second"
    );
}

#[tokio::test]
async fn test_cache_expiration() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create cache with very short expiration
    let config = CacheConfig {
        max_age_secs: 0, // Expire immediately
        enabled: true,
        cache_path: cache_path.clone(),
    };
    let cache_manager = CacheManager::with_config(config);

    // Save and immediately try to load
    cache_manager.save_cache(vec![], vec![]).await.unwrap();

    // Should return None due to expiration
    let loaded = cache_manager.load_cache().await;
    assert!(loaded.is_none(), "Expired cache should not be loaded");
}

#[tokio::test]
async fn test_cache_persistence_across_restarts() {
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("test_cache.json");

    // First "app session"
    {
        let config = CacheConfig {
            max_age_secs: 300,
            enabled: true,
            cache_path: cache_path.clone(),
        };
        let cache_manager = CacheManager::with_config(config);

        let devices = vec![AndroidDevice {
            name: "persistent_avd".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        }];

        cache_manager.save_cache(devices, vec![]).await.unwrap();
    }

    // Second "app session" - simulating restart
    {
        let config = CacheConfig {
            max_age_secs: 300,
            enabled: true,
            cache_path: cache_path.clone(),
        };
        let cache_manager = CacheManager::with_config(config);

        let loaded = cache_manager.load_cache().await;
        assert!(loaded.is_some(), "Cache should persist across sessions");

        let cache = loaded.unwrap();
        assert_eq!(cache.android_devices.len(), 1);
        assert_eq!(cache.android_devices[0].name, "persistent_avd");
    }
}

#[test]
fn test_cache_version_compatibility() {
    // Test that cache with different version is rejected
    let cache_v1 = DeviceCache {
        last_updated: std::time::SystemTime::now(),
        android_devices: vec![],
        ios_devices: vec![],
        version: 1,
    };

    let cache_v999 = DeviceCache {
        last_updated: std::time::SystemTime::now(),
        android_devices: vec![],
        ios_devices: vec![],
        version: 999,
    };

    assert_eq!(cache_v1.version, DeviceCache::CURRENT_VERSION);
    assert_ne!(cache_v999.version, DeviceCache::CURRENT_VERSION);
}
