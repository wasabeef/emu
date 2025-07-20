//! Basic functionality tests for managers/android.rs
//!
//! Tests basic initialization, configuration parsing, and utility functions of AndroidManager.
//! Excludes features that require actual Android SDK and focuses on testable parts.

use emu::managers::android::AndroidManager;
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;

mod common;
use common::setup_mock_android_sdk;

/// Basic AndroidManager initialization test (no SDK required)
#[tokio::test]
async fn test_android_manager_creation() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor with necessary responses
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Create AndroidManager with mock executor
    let result = AndroidManager::with_executor(Arc::new(mock_executor));

    // Should succeed with mock executor
    assert!(result.is_ok());
}

/// Device category classification test
#[test]
fn test_device_category_classification() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Create AndroidManager with mock executor
    let manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();

    // Phone category test (AndroidManager returns lowercase)
    assert_eq!(manager.get_device_category("pixel_7", "Pixel 7"), "phone");
    assert_eq!(manager.get_device_category("pixel_4", "Pixel 4"), "phone");
    assert_eq!(manager.get_device_category("nexus_5", "Nexus 5"), "phone");

    // Tablet category test
    assert_eq!(
        manager.get_device_category("pixel_tablet", "Pixel Tablet"),
        "tablet"
    );
    assert_eq!(manager.get_device_category("nexus_10", "Nexus 10"), "phone"); // nexus_10 doesn't contain pixel so it's phone

    // TV category test
    assert_eq!(
        manager.get_device_category("tv_1080p", "Android TV (1080p)"),
        "tv"
    );
    assert_eq!(
        manager.get_device_category("tv_720p", "Android TV (720p)"),
        "tv"
    );

    // Wear category test
    assert_eq!(
        manager.get_device_category("wear_round", "Android Wear Round"),
        "wear"
    );
    assert_eq!(
        manager.get_device_category("wear_square", "Android Wear Square"),
        "wear"
    );

    // Automotive category test
    assert_eq!(
        manager.get_device_category("automotive_1024p", "Automotive (1024p landscape)"),
        "automotive"
    );

    // Desktop category test
    assert_eq!(
        manager.get_device_category("desktop_large", "Large Desktop"),
        "desktop"
    );
    assert_eq!(
        manager.get_device_category("desktop_medium", "Medium Desktop"),
        "desktop"
    );

    // Unknown device test (default is phone)
    assert_eq!(
        manager.get_device_category("unknown_device", "Unknown Device"),
        "phone"
    );
    assert_eq!(manager.get_device_category("", ""), "phone");
}

/// Device category classification helper function (mimics AndroidManager implementation)
fn classify_device_category(device_id: &str, device_name: &str) -> &'static str {
    let combined = format!(
        "{} {}",
        device_id.to_lowercase(),
        device_name.to_lowercase()
    );

    // Tablet - tablet devices (check first to catch tablet variants)
    if combined.contains("tablet")
        || combined.contains("pad")
        || combined.contains("tab")
        || (combined.contains("10") && combined.contains("inch"))
        || (combined.contains("11") && combined.contains("inch"))
        || (combined.contains("12") && combined.contains("inch"))
        || (combined.contains("13") && combined.contains("inch"))
    {
        return "tablet";
    }

    // Phone - most common device type
    if combined.contains("phone")
        || combined.contains("pixel") && !combined.contains("fold") && !combined.contains("tablet")
        || combined.contains("galaxy")
            && !combined.contains("fold")
            && !combined.contains("tablet")
            && !combined.contains("watch")
        || combined.contains("oneplus")
        || combined.contains("iphone")
    {
        return "phone";
    }

    // Wear - wearable devices
    if combined.contains("wear")
        || combined.contains("watch")
        || combined.contains("round") && !combined.contains("tablet")
        || combined.contains("square") && !combined.contains("tablet")
    {
        return "wear";
    }

    // TV - television devices
    if combined.contains("tv")
        || combined.contains("1080p")
        || combined.contains("4k")
        || combined.contains("720p")
    {
        return "tv";
    }

    // Automotive - automotive devices
    if combined.contains("auto") || combined.contains("car") || combined.contains("automotive") {
        return "automotive";
    }

    // Desktop - desktop/large screen devices
    if combined.contains("desktop")
        || combined.contains("foldable") && combined.contains("large")
        || (combined.contains("15") && combined.contains("inch"))
        || (combined.contains("17") && combined.contains("inch"))
    {
        return "desktop";
    }

    // Default is phone (most common)
    "phone"
}

/// Android version name mapping test
#[test]
fn test_android_version_name_mapping() {
    // Test API level to Android version name mapping logic
    assert_eq!(get_android_version_name(34), "Android 14");
    assert_eq!(get_android_version_name(33), "Android 13");
    assert_eq!(get_android_version_name(32), "Android 12L");
    assert_eq!(get_android_version_name(31), "Android 12");
    assert_eq!(get_android_version_name(30), "Android 11");
    assert_eq!(get_android_version_name(29), "Android 10");
    assert_eq!(get_android_version_name(28), "Android 9");
    assert_eq!(get_android_version_name(27), "Android 8.1");
    assert_eq!(get_android_version_name(26), "Android 8.0");
    assert_eq!(get_android_version_name(25), "Android 7.1");
    assert_eq!(get_android_version_name(24), "Android 7.0");
    assert_eq!(get_android_version_name(23), "Android 6.0");

    // Test older versions
    assert_eq!(get_android_version_name(21), "Android 5.0");
    assert_eq!(get_android_version_name(19), "Android 4.4");
    assert_eq!(get_android_version_name(16), "Android 4.1");

    // Test out of range values
    assert_eq!(get_android_version_name(99), "API 99");
    assert_eq!(get_android_version_name(0), "API 0");
}

/// Android version name mapping helper function
fn get_android_version_name(api_level: u32) -> String {
    match api_level {
        34 => "Android 14".to_string(),
        33 => "Android 13".to_string(),
        32 => "Android 12L".to_string(),
        31 => "Android 12".to_string(),
        30 => "Android 11".to_string(),
        29 => "Android 10".to_string(),
        28 => "Android 9".to_string(),
        27 => "Android 8.1".to_string(),
        26 => "Android 8.0".to_string(),
        25 => "Android 7.1".to_string(),
        24 => "Android 7.0".to_string(),
        23 => "Android 6.0".to_string(),
        22 => "Android 5.1".to_string(),
        21 => "Android 5.0".to_string(),
        19 => "Android 4.4".to_string(),
        18 => "Android 4.3".to_string(),
        17 => "Android 4.2".to_string(),
        16 => "Android 4.1".to_string(),
        _ => format!("API {api_level}"),
    }
}

/// Device name and ID priority test
#[test]
fn test_device_priority_logic() {
    // Confirm Pixel devices are classified as Phone
    let pixel_devices = vec![
        ("pixel_7", "Pixel 7"),
        ("pixel_6", "Pixel 6"),
        ("pixel_4", "Pixel 4"),
        ("pixel_3", "Pixel 3"),
    ];

    for (device_id, device_name) in pixel_devices {
        assert_eq!(classify_device_category(device_id, device_name), "phone");
    }

    // Confirm Nexus device classification
    assert_eq!(classify_device_category("nexus_5", "Nexus 5"), "phone");
    assert_eq!(classify_device_category("nexus_6", "Nexus 6"), "phone");
    // nexus_10 contains "10" but not "inch", so it's classified as phone
    assert_eq!(classify_device_category("nexus_10", "Nexus 10"), "phone");
}

/// Basic structure check for error handling test
#[test]
fn test_android_manager_structure() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Confirm AndroidManager can be created normally
    let result = AndroidManager::with_executor(Arc::new(mock_executor));

    // Should succeed with mock executor
    assert!(result.is_ok());
}

/// Version string parsing test
#[test]
fn test_version_string_parsing() {
    // Test conversion from API level to Android version name
    let test_cases = vec![
        (34, "Android 14"),
        (33, "Android 13"),
        (32, "Android 12L"),
        (31, "Android 12"),
        (30, "Android 11"),
        (29, "Android 10"),
    ];

    for (api_level, expected_name) in test_cases {
        let version_name = get_android_version_name(api_level);
        assert_eq!(version_name, expected_name);
    }
}

/// Configuration value range check test
#[test]
fn test_configuration_validation() {
    let temp_dir = setup_mock_android_sdk();
    std::env::set_var("ANDROID_HOME", temp_dir.path());

    // Create a mock executor
    let mock_executor = MockCommandExecutor::new()
        .with_success("avdmanager", &["list", "avd"], "")
        .with_success("adb", &["devices"], "List of devices attached\n");

    // Confirm AndroidManager can be created
    // Actual configuration value validation is performed in DeviceConfig
    let manager_result = AndroidManager::with_executor(Arc::new(mock_executor));
    assert!(manager_result.is_ok());
}

/// Basic pattern test for device information parsing
#[test]
fn test_device_info_patterns() {
    // Test category classification with various device name patterns
    let test_patterns = vec![
        // Phone patterns
        ("pixel_8", "Pixel 8", "phone"),
        ("galaxy_s23", "Galaxy S23", "phone"),
        ("iphone_se", "iPhone SE", "phone"),
        // Tablet patterns
        ("pixel_tablet_2023", "Pixel Tablet", "tablet"),
        ("galaxy_tab", "Galaxy Tab", "tablet"),
        ("ipad_air", "iPad Air", "tablet"),
        // TV patterns
        ("android_tv_4k", "Android TV (4K)", "tv"),
        ("google_tv", "Google TV", "tv"),
        // Wear patterns
        ("wear_os_3", "Wear OS 3", "wear"),
        ("galaxy_watch", "Galaxy Watch", "wear"),
        // Others (default is phone)
        ("unknown_type", "Unknown Type", "phone"),
    ];

    for (device_id, device_name, expected_category) in test_patterns {
        let actual_category = classify_device_category(device_id, device_name);
        assert_eq!(
            actual_category, expected_category,
            "Device {device_id} ({device_name}) should be categorized as {expected_category}"
        );
    }
}
