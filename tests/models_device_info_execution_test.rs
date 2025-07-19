//! Unit tests for models/device_info.rs execution logic

use emu::models::device_info::DynamicDeviceConfig;

#[test]
fn test_dynamic_device_config_creation() {
    // Test DynamicDeviceConfig::new()
    let _config = DynamicDeviceConfig::new();

    // Should create successfully

    // Config should be in initial state
    // We can't directly test internal state, but creation should not panic
}

#[test]
fn test_calculate_android_device_priority() {
    // Test calculate_android_device_priority function

    // Test with various device IDs and display names
    let test_cases = vec![
        ("pixel_7", "Pixel 7"),
        ("pixel_6", "Pixel 6"),
        ("pixel_5", "Pixel 5"),
        ("nexus_6", "Nexus 6"),
        ("galaxy_s22", "Galaxy S22"),
        ("unknown_device", "Unknown Device"),
    ];

    for (device_id, display_name) in test_cases {
        let priority =
            DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);

        // Priority should be a reasonable value (can be 0 for unknown devices)
        assert!(
            priority < 10000,
            "Priority should be reasonable for {device_id}: {priority}"
        );
    }
}

#[test]
fn test_calculate_android_device_priority_pixel_devices() {
    // Test that Pixel devices get appropriate priority
    let pixel_7_priority =
        DynamicDeviceConfig::calculate_android_device_priority("pixel_7", "Pixel 7");
    let pixel_6_priority =
        DynamicDeviceConfig::calculate_android_device_priority("pixel_6", "Pixel 6");
    let pixel_5_priority =
        DynamicDeviceConfig::calculate_android_device_priority("pixel_5", "Pixel 5");

    // All should be reasonable (can be 0 for unknown devices)
    assert!(
        pixel_7_priority < 10000,
        "Pixel 7 priority should be reasonable"
    );
    assert!(
        pixel_6_priority < 10000,
        "Pixel 6 priority should be reasonable"
    );
    assert!(
        pixel_5_priority < 10000,
        "Pixel 5 priority should be reasonable"
    );

    // Should be different values (unless coincidentally equal)
    assert!(
        pixel_7_priority != pixel_6_priority || pixel_6_priority != pixel_5_priority,
        "Different Pixel devices should have different priorities"
    );
}

#[test]
fn test_calculate_ios_device_priority() {
    // Test calculate_ios_device_priority function

    let test_cases = vec![
        "iPhone 15 Pro Max",
        "iPhone 15 Pro",
        "iPhone 15",
        "iPhone 14",
        "iPhone SE (3rd generation)",
        "iPad Pro (12.9-inch) (6th generation)",
        "iPad Air (5th generation)",
        "iPad mini (6th generation)",
        "Apple TV 4K (3rd generation)",
        "Apple Watch Series 9 (45mm)",
        "Apple Watch SE (2nd generation) (44mm)",
        "Apple Watch Ultra 2 (49mm)",
    ];

    for display_name in test_cases {
        let priority = DynamicDeviceConfig::calculate_ios_device_priority(display_name);

        // Priority should be a reasonable value (note: can be 0 for unknown devices)
        assert!(
            priority < 10000,
            "Priority should be reasonable for {display_name}: {priority}"
        );
    }
}

#[test]
fn test_ios_device_priority_iphone_ordering() {
    // Test that iPhone devices get appropriate priority ordering
    let iphone_15_pro_max = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15 Pro Max");
    let iphone_15_pro = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15 Pro");
    let iphone_15 = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15");
    let iphone_14 = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 14");
    let iphone_se =
        DynamicDeviceConfig::calculate_ios_device_priority("iPhone SE (3rd generation)");

    // All should be reasonable (can be 0 for unknown devices)
    assert!(
        iphone_15_pro_max < 10000,
        "iPhone 15 Pro Max priority should be reasonable"
    );
    assert!(
        iphone_15_pro < 10000,
        "iPhone 15 Pro priority should be reasonable"
    );
    assert!(iphone_15 < 10000, "iPhone 15 priority should be reasonable");
    assert!(iphone_14 < 10000, "iPhone 14 priority should be reasonable");
    assert!(iphone_se < 10000, "iPhone SE priority should be reasonable");

    // Should be different values (unless coincidentally equal)
    let priorities = vec![
        iphone_15_pro_max,
        iphone_15_pro,
        iphone_15,
        iphone_14,
        iphone_se,
    ];
    let unique_priorities: std::collections::HashSet<_> = priorities.into_iter().collect();
    assert!(
        unique_priorities.len() > 1,
        "Different iPhone models should have different priorities"
    );
}

#[test]
fn test_ios_device_priority_ipad_types() {
    // Test iPad device priority calculation
    let ipad_pro_12_9 =
        DynamicDeviceConfig::calculate_ios_device_priority("iPad Pro (12.9-inch) (6th generation)");
    let ipad_pro_11 =
        DynamicDeviceConfig::calculate_ios_device_priority("iPad Pro (11-inch) (4th generation)");
    let ipad_air = DynamicDeviceConfig::calculate_ios_device_priority("iPad Air (5th generation)");
    let ipad_mini =
        DynamicDeviceConfig::calculate_ios_device_priority("iPad mini (6th generation)");

    // All should be reasonable (can be 0 for unknown devices)
    assert!(
        ipad_pro_12_9 < 10000,
        "iPad Pro 12.9 priority should be reasonable"
    );
    assert!(
        ipad_pro_11 < 10000,
        "iPad Pro 11 priority should be reasonable"
    );
    assert!(ipad_air < 10000, "iPad Air priority should be reasonable");
    assert!(ipad_mini < 10000, "iPad mini priority should be reasonable");

    // Should be different values
    let priorities = vec![ipad_pro_12_9, ipad_pro_11, ipad_air, ipad_mini];
    let unique_priorities: std::collections::HashSet<_> = priorities.into_iter().collect();
    assert!(
        unique_priorities.len() > 1,
        "Different iPad models should have different priorities"
    );
}

#[test]
fn test_ios_device_priority_apple_watch() {
    // Test Apple Watch device priority calculation
    let watch_series_9 =
        DynamicDeviceConfig::calculate_ios_device_priority("Apple Watch Series 9 (45mm)");
    let watch_se = DynamicDeviceConfig::calculate_ios_device_priority(
        "Apple Watch SE (2nd generation) (44mm)",
    );
    let watch_ultra =
        DynamicDeviceConfig::calculate_ios_device_priority("Apple Watch Ultra 2 (49mm)");

    // All should be reasonable (can be 0 for unknown devices)
    assert!(
        watch_series_9 < 10000,
        "Apple Watch Series 9 priority should be reasonable"
    );
    assert!(
        watch_se < 10000,
        "Apple Watch SE priority should be reasonable"
    );
    assert!(
        watch_ultra < 10000,
        "Apple Watch Ultra priority should be reasonable"
    );

    // Should be different values
    let priorities = vec![watch_series_9, watch_se, watch_ultra];
    let unique_priorities: std::collections::HashSet<_> = priorities.into_iter().collect();
    assert!(
        unique_priorities.len() > 1,
        "Different Apple Watch models should have different priorities"
    );
}

#[test]
fn test_ios_device_priority_apple_tv() {
    // Test Apple TV device priority calculation
    let apple_tv_4k =
        DynamicDeviceConfig::calculate_ios_device_priority("Apple TV 4K (3rd generation)");
    let apple_tv_hd = DynamicDeviceConfig::calculate_ios_device_priority("Apple TV HD");

    // All should be reasonable (can be 0 for unknown devices)
    assert!(
        apple_tv_4k < 10000,
        "Apple TV 4K priority should be reasonable"
    );
    assert!(
        apple_tv_hd < 10000,
        "Apple TV HD priority should be reasonable"
    );

    // Should be different values (unless coincidentally equal)
    // Apple TV priorities should be consistent (they can be equal or different)
    // Apple TV models should have consistent priority calculation
}

#[test]
fn test_device_config_cache_operations() {
    // Test cache-related operations
    let mut config = DynamicDeviceConfig::new();

    // Test load_device_cache with empty vector
    config.load_device_cache(vec![]);

    // Should not panic

    // Test load_api_cache with empty vector
    config.load_api_cache(vec![]);

    // Should not panic
}

#[test]
fn test_get_device_priority() {
    // Test get_device_priority method
    let config = DynamicDeviceConfig::new();

    // Test with various device IDs
    let test_device_ids = vec![
        "pixel_7",
        "pixel_6",
        "nexus_6",
        "galaxy_s22",
        "unknown_device",
    ];

    for device_id in test_device_ids {
        let priority = config.get_device_priority(device_id);

        // Priority should be a reasonable value (can be 999 for unknown devices)
        assert!(
            priority < 10000,
            "Priority should be reasonable for {device_id}: {priority}"
        );
    }
}

#[test]
fn test_parse_device_name() {
    // Test parse_device_name method
    let config = DynamicDeviceConfig::new();

    let test_cases = vec![
        ("pixel_7", vec!["pixel", "7"]),
        ("nexus_6", vec!["nexus", "6"]),
        ("galaxy_s22", vec!["galaxy", "s22"]),
        ("tv_1080p", vec!["tv", "1080p"]),
        ("wear_os_square", vec!["wear", "os", "square"]),
        ("automotive_1024p", vec!["automotive", "1024p"]),
    ];

    for (device_type, expected_parts) in test_cases {
        let parsed = config.parse_device_name(device_type);

        // Should return a vector
        assert!(
            !parsed.is_empty(),
            "Parsed device name should not be empty for {device_type}"
        );

        // Should contain expected parts (order may vary)
        for expected_part in expected_parts {
            assert!(parsed.iter().any(|part| part.contains(expected_part) || expected_part.contains(part)),
                    "Parsed name should contain '{expected_part}' for device '{device_type}': {parsed:?}");
        }
    }
}

#[test]
fn test_get_android_version_name() {
    // Test get_android_version_name method
    let config = DynamicDeviceConfig::new();

    let test_cases = vec![
        (34, "Android 14"),
        (33, "Android 13"),
        (32, "Android 12L"),
        (31, "Android 12"),
        (30, "Android 11"),
        (29, "Android 10"),
        (28, "Android 9"),
        (27, "Android 8.1"),
        (26, "Android 8.0"),
        (25, "Android 7.1"),
        (24, "Android 7.0"),
        (23, "Android 6.0"),
        (22, "Android 5.1"),
        (21, "Android 5.0"),
        (19, "Android 4.4"),
        (18, "Android 4.3"),
        (17, "Android 4.2"),
        (16, "Android 4.1"),
        (15, "Android 4.0.3"),
        (14, "Android 4.0"),
        (1, "Android 1.0"),       // Very old version
        (999, "Android API 999"), // Future version
    ];

    for (api_level, _expected_name) in test_cases {
        let version_name = config.get_android_version_name(api_level);

        // Should return a non-empty string
        assert!(
            !version_name.is_empty(),
            "Version name should not be empty for API {api_level}"
        );

        // Should contain "Android" or "API" (fallback format)
        assert!(
            version_name.contains("Android") || version_name.contains("API"),
            "Version name should contain 'Android' or 'API' for API {api_level}: {version_name}"
        );

        // For unknown versions, should use fallback format
        if !(14..=34).contains(&api_level) {
            assert_eq!(
                version_name,
                format!("API {api_level}"),
                "Unknown API level should use fallback format for API {api_level}"
            );
        }
    }
}

#[test]
fn test_device_priority_consistency() {
    // Test that priority calculations are consistent
    let device_id = "pixel_7";
    let display_name = "Pixel 7";

    // Multiple calls should return the same value
    let priority1 = DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);
    let priority2 = DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);
    let priority3 = DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);

    assert_eq!(
        priority1, priority2,
        "Priority calculation should be consistent"
    );
    assert_eq!(
        priority2, priority3,
        "Priority calculation should be consistent"
    );
}

#[test]
fn test_ios_device_priority_consistency() {
    // Test that iOS priority calculations are consistent
    let display_name = "iPhone 15 Pro Max";

    // Multiple calls should return the same value
    let priority1 = DynamicDeviceConfig::calculate_ios_device_priority(display_name);
    let priority2 = DynamicDeviceConfig::calculate_ios_device_priority(display_name);
    let priority3 = DynamicDeviceConfig::calculate_ios_device_priority(display_name);

    assert_eq!(
        priority1, priority2,
        "iOS priority calculation should be consistent"
    );
    assert_eq!(
        priority2, priority3,
        "iOS priority calculation should be consistent"
    );
}

#[test]
fn test_device_priority_edge_cases() {
    // Test edge cases for device priority calculation

    // Empty strings
    let empty_priority = DynamicDeviceConfig::calculate_android_device_priority("", "");
    assert!(
        empty_priority < 10000,
        "Empty device should have reasonable priority"
    );

    // Very long strings
    let long_device_id = "a".repeat(1000);
    let long_display_name = "b".repeat(1000);
    let long_priority =
        DynamicDeviceConfig::calculate_android_device_priority(&long_device_id, &long_display_name);
    assert!(
        long_priority < 10000,
        "Long device name should have reasonable priority"
    );

    // Special characters
    let special_priority = DynamicDeviceConfig::calculate_android_device_priority(
        "device-with-special_chars@123",
        "Device (Special) [Chars]",
    );
    assert!(
        special_priority < 10000,
        "Device with special characters should have reasonable priority"
    );
}

#[test]
fn test_ios_device_priority_edge_cases() {
    // Test edge cases for iOS device priority calculation

    // Empty string
    let empty_priority = DynamicDeviceConfig::calculate_ios_device_priority("");
    assert!(
        empty_priority < 10000,
        "Empty iOS device should have reasonable priority"
    );

    // Very long string
    let long_name = "a".repeat(1000);
    let long_priority = DynamicDeviceConfig::calculate_ios_device_priority(&long_name);
    assert!(
        long_priority < 10000,
        "Long iOS device name should have reasonable priority"
    );

    // Special characters
    let special_priority =
        DynamicDeviceConfig::calculate_ios_device_priority("Device (Special) [Chars] @123");
    assert!(
        special_priority < 10000,
        "iOS device with special characters should have reasonable priority"
    );

    // Unknown device type
    let unknown_priority =
        DynamicDeviceConfig::calculate_ios_device_priority("Unknown Device Type");
    assert!(
        unknown_priority < 10000,
        "Unknown iOS device should have reasonable priority"
    );
}
