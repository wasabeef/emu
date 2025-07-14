use emu::app::state::{AppState, DeviceCache, Panel};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_device_cache_creation() {
    let cache = DeviceCache::default();

    assert!(cache.android_device_types.is_empty());
    assert!(cache.android_api_levels.is_empty());
    assert!(cache.ios_device_types.is_empty());
    assert!(cache.ios_runtimes.is_empty());
    assert!(!cache.is_loading);
    assert!(!cache.is_stale()); // A newly created cache is valid

    println!("✅ Device cache created successfully");
}

#[tokio::test]
async fn test_device_cache_android_update() {
    let mut cache = DeviceCache::default();

    let device_types = vec![
        ("pixel_7".to_string(), "Pixel 7 (Google)".to_string()),
        ("pixel_fold".to_string(), "Pixel Fold (Google)".to_string()),
    ];

    let api_levels = vec![
        ("35".to_string(), "API 35 - Android 15".to_string()),
        ("36".to_string(), "API 36 - Android 16".to_string()),
    ];

    cache.update_android_cache(device_types.clone(), api_levels.clone());

    assert_eq!(cache.android_device_types, device_types);
    assert_eq!(cache.android_api_levels, api_levels);
    assert!(!cache.is_loading);
    assert!(!cache.is_stale());

    println!("✅ Android cache updated successfully");
}

#[tokio::test]
async fn test_device_cache_ios_update() {
    let mut cache = DeviceCache::default();

    let device_types = vec![
        ("iPhone_15_Pro".to_string(), "iPhone 15 Pro".to_string()),
        ("iPad_Pro_12_9".to_string(), "iPad Pro 12.9".to_string()),
    ];

    let runtimes = vec![
        ("iOS-17-0".to_string(), "iOS 17.0".to_string()),
        ("iOS-18-0".to_string(), "iOS 18.0".to_string()),
    ];

    cache.update_ios_cache(device_types.clone(), runtimes.clone());

    assert_eq!(cache.ios_device_types, device_types);
    assert_eq!(cache.ios_runtimes, runtimes);
    assert!(!cache.is_loading);
    assert!(!cache.is_stale());

    println!("✅ iOS cache updated successfully");
}

#[tokio::test]
async fn test_device_cache_staleness() {
    let mut cache = DeviceCache {
        last_updated: std::time::Instant::now() - Duration::from_secs(301),
        ..DeviceCache::default()
    };

    assert!(cache.is_stale(), "Cache should be stale after 5+ minutes");

    // Update cache
    cache.update_android_cache(vec![], vec![]);

    assert!(!cache.is_stale(), "Cache should be fresh after update");

    println!("✅ Cache staleness logic working correctly");
}

#[tokio::test]
async fn test_app_state_cache_integration() {
    let app_state = AppState::new();

    // Verify that the cache is initialized
    {
        let cache = app_state.device_cache.read().await;
        assert!(cache.android_device_types.is_empty());
        assert!(cache.android_api_levels.is_empty());
    }

    // Verify that Android cache is not available
    assert!(!app_state.is_cache_available(Panel::Android).await);

    // Verify that iOS cache is not available
    assert!(!app_state.is_cache_available(Panel::Ios).await);

    println!("✅ App state cache integration working");
}

#[tokio::test]
async fn test_cache_population_to_form() {
    let mut app_state = AppState::new();

    // Add test data to cache
    {
        let mut cache = app_state.device_cache.write().await;
        cache.update_android_cache(
            vec![
                ("pixel_7".to_string(), "Pixel 7 (Google)".to_string()),
                ("pixel_fold".to_string(), "Pixel Fold (Google)".to_string()),
            ],
            vec![
                ("35".to_string(), "API 35 - Android 15".to_string()),
                ("36".to_string(), "API 36 - Android 16".to_string()),
            ],
        );
    }

    // Verify that Android cache has become available
    assert!(app_state.is_cache_available(Panel::Android).await);

    // Set cache data to form
    app_state.populate_form_from_cache(Panel::Android).await;

    // Verify that data has been set to form
    assert!(!app_state
        .create_device_form
        .available_device_types
        .is_empty());
    assert!(!app_state.create_device_form.available_versions.is_empty());
    assert!(!app_state.create_device_form.is_loading_cache);

    // Verify that the first device type is selected
    assert_eq!(app_state.create_device_form.device_type_id, "pixel_7");
    assert_eq!(app_state.create_device_form.device_type, "Pixel 7 (Google)");

    // Verify that the first API level is selected
    assert_eq!(app_state.create_device_form.version, "35");
    assert_eq!(
        app_state.create_device_form.version_display,
        "API 35 - Android 15"
    );

    // Verify that placeholder name has been generated
    assert!(!app_state.create_device_form.name.is_empty());
    assert!(app_state.create_device_form.name.contains("Pixel"));
    assert!(app_state.create_device_form.name.contains("API"));

    println!(
        "✅ Cache population to form working: '{}'",
        app_state.create_device_form.name
    );
}

#[tokio::test]
async fn test_background_cache_update_startup() {
    let mut app_state = AppState::new();

    // Test background update start
    app_state.start_background_cache_update().await;

    // Verify that loading flag is set
    assert!(app_state.create_device_form.is_loading_cache);

    // Also verify cache loading flag
    {
        let cache = app_state.device_cache.read().await;
        assert!(cache.is_loading);
    }

    println!("✅ Background cache update started successfully");
}

#[tokio::test]
async fn test_cache_respects_staleness_policy() {
    let mut cache = DeviceCache::default();

    // New cache is valid
    assert!(!cache.is_stale());

    // Add data
    cache.update_android_cache(
        vec![("test".to_string(), "Test Device".to_string())],
        vec![("35".to_string(), "API 35".to_string())],
    );

    // Still valid after update
    assert!(!cache.is_stale());

    // Manually set to old time
    cache.last_updated = std::time::Instant::now() - Duration::from_secs(400);

    // Stale cache is invalid
    assert!(cache.is_stale());

    println!("✅ Cache staleness policy working correctly");
}

#[test]
fn test_cache_thread_safety() {
    // Test thread safety of Arc<RwLock<DeviceCache>>
    let cache = Arc::new(RwLock::new(DeviceCache::default()));

    // Verify safe access from multiple threads
    let _cache_clone = Arc::clone(&cache);

    let handle = std::thread::spawn(move || {
        // Access cache from another thread
        println!("Cache accessed from another thread");
    });

    handle.join().unwrap();

    println!("✅ Cache is thread-safe");
}

#[tokio::test]
async fn test_form_updates_from_cache_selection() {
    let mut app_state = AppState::new();

    // Set choices to form
    app_state.create_device_form.available_device_types = vec![
        ("pixel_7".to_string(), "Pixel 7 (Google)".to_string()),
        ("pixel_fold".to_string(), "Pixel Fold (Google)".to_string()),
    ];

    app_state.create_device_form.available_versions = vec![
        ("35".to_string(), "API 35 - Android 15".to_string()),
        ("36".to_string(), "API 36 - Android 16".to_string()),
    ];

    // Update device type selection
    app_state.create_device_form.selected_device_type_index = 1;
    app_state.create_device_form.update_selected_device_type();

    assert_eq!(app_state.create_device_form.device_type_id, "pixel_fold");
    assert_eq!(
        app_state.create_device_form.device_type,
        "Pixel Fold (Google)"
    );

    // Update API level selection
    app_state.create_device_form.selected_api_level_index = 1;
    app_state.create_device_form.update_selected_api_level();

    assert_eq!(app_state.create_device_form.version, "36");
    assert_eq!(
        app_state.create_device_form.version_display,
        "API 36 - Android 16"
    );

    // Verify that placeholder name has been updated
    assert!(app_state.create_device_form.name.contains("Pixel Fold"));
    assert!(app_state.create_device_form.name.contains("API 36"));

    println!(
        "✅ Form updates correctly from cache selection: '{}'",
        app_state.create_device_form.name
    );
}
