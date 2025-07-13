use emu::models::device_info::{
    DeviceCategory, DeviceInfo, DynamicDeviceConfig, ApiLevelInfo, DeviceSpecifications,
    FALLBACK_ANDROID_DEVICES, test_constants::*
};

#[test]
fn test_device_info_creation() {
    let device = DeviceInfo {
        id: "pixel_8".to_string(),
        display_name: "Pixel 8".to_string(),
        oem: Some("Google".to_string()),
        category: DeviceCategory::Phone,
    };
    
    assert_eq!(device.id, "pixel_8");
    assert_eq!(device.display_name, "Pixel 8");
    assert_eq!(device.oem, Some("Google".to_string()));
    assert_eq!(device.category, DeviceCategory::Phone);
}

#[test]
fn test_device_category_equality() {
    assert_eq!(DeviceCategory::Phone, DeviceCategory::Phone);
    assert_ne!(DeviceCategory::Phone, DeviceCategory::Tablet);
    
    let categories = vec![
        DeviceCategory::Phone,
        DeviceCategory::Tablet,
        DeviceCategory::Wear,
        DeviceCategory::TV,
        DeviceCategory::Automotive,
        DeviceCategory::Foldable,
        DeviceCategory::Unknown,
    ];
    
    for (i, category1) in categories.iter().enumerate() {
        for (j, category2) in categories.iter().enumerate() {
            if i == j {
                assert_eq!(category1, category2);
            } else {
                assert_ne!(category1, category2);
            }
        }
    }
}

#[test]
fn test_api_level_info_creation() {
    let api_info = ApiLevelInfo {
        level: 34,
        version_name: "Android 14".to_string(),
        available_tags: vec!["google_apis".to_string(), "google_apis_playstore".to_string()],
    };
    
    assert_eq!(api_info.level, 34);
    assert_eq!(api_info.version_name, "Android 14");
    assert_eq!(api_info.available_tags.len(), 2);
    assert!(api_info.available_tags.contains(&"google_apis".to_string()));
}

#[test]
fn test_dynamic_device_config_new() {
    let config = DynamicDeviceConfig::new();
    
    // Test that new config is properly initialized
    let priority = config.get_device_priority("non-existent-device");
    assert_eq!(priority, 999); // Unknown devices go to the end
}

#[test]
fn test_dynamic_device_config_default() {
    let config1 = DynamicDeviceConfig::new();
    let config2 = DynamicDeviceConfig::default();
    
    // Both should behave the same
    assert_eq!(
        config1.get_device_priority("test"),
        config2.get_device_priority("test")
    );
}

#[test]
fn test_calculate_android_device_priority() {
    // Test Pixel devices get highest priority
    let pixel_priority = DynamicDeviceConfig::calculate_android_device_priority(
        "pixel_9", 
        "Pixel 9 (Google)"
    );
    let other_priority = DynamicDeviceConfig::calculate_android_device_priority(
        "galaxy_s23", 
        "Galaxy S23 (Samsung)"
    );
    
    assert!(pixel_priority < other_priority, 
           "Pixel devices should have higher priority (lower number)");
}

#[test]
fn test_calculate_android_device_priority_versioned() {
    // Test that newer Pixel devices get better priority
    let pixel_9_priority = DynamicDeviceConfig::calculate_android_device_priority(
        "pixel_9", 
        "Pixel 9 (Google)"
    );
    let pixel_7_priority = DynamicDeviceConfig::calculate_android_device_priority(
        "pixel_7", 
        "Pixel 7 (Google)"
    );
    
    assert!(pixel_9_priority < pixel_7_priority, 
           "Newer Pixel devices should have better priority");
}

#[test]
fn test_calculate_ios_device_priority() {
    // Test iPhone priorities
    let iphone_pro_max = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15 Pro Max");
    let iphone_pro = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15 Pro");
    let iphone_regular = DynamicDeviceConfig::calculate_ios_device_priority("iPhone 15");
    
    assert!(iphone_pro_max < iphone_pro);
    assert!(iphone_pro < iphone_regular);
    
    // Test iPad priorities are higher numbers (lower priority) than iPhone
    let ipad_pro = DynamicDeviceConfig::calculate_ios_device_priority("iPad Pro 12.9");
    assert!(iphone_regular < ipad_pro);
}

#[test]
fn test_extract_name_parts() {
    let config = DynamicDeviceConfig::new();
    
    // Test with manufacturer in parentheses
    let parts = config.parse_device_name("Pixel 9 Pro (Google)");
    assert!(parts.contains(&"Pixel".to_string()));
    assert!(parts.contains(&"9".to_string()));
    assert!(parts.contains(&"Pro".to_string()));
    assert!(!parts.iter().any(|p| p.contains("Google")));
    
    // Test without parentheses
    let parts = config.parse_device_name("Nexus 5X");
    assert!(parts.contains(&"Nexus".to_string()));
    assert!(parts.contains(&"5X".to_string()));
    
    // Test with special modifiers
    let parts = config.parse_device_name("Pixel 9 Pro Fold (Google)");
    assert!(parts.contains(&"Pixel".to_string()));
    assert!(parts.contains(&"9".to_string()));
    assert!(parts.contains(&"Pro".to_string()));
    assert!(parts.contains(&"Fold".to_string()));
}

#[test]
fn test_device_specifications_default() {
    let specs = DeviceSpecifications::default();
    assert_eq!(specs.screen_size_inches, 0.0);
    assert_eq!(specs.screen_width, 0);
    assert_eq!(specs.screen_height, 0);
    assert_eq!(specs.screen_density, 0);
    assert_eq!(specs.manufacturer, "");
}

#[test]
fn test_device_specifications_creation() {
    let specs = DeviceSpecifications {
        screen_size_inches: 6.1,
        screen_width: 1080,
        screen_height: 2400,
        screen_density: 420,
        manufacturer: "Google".to_string(),
    };
    
    assert_eq!(specs.screen_size_inches, 6.1);
    assert_eq!(specs.screen_width, 1080);
    assert_eq!(specs.screen_height, 2400);
    assert_eq!(specs.screen_density, 420);
    assert_eq!(specs.manufacturer, "Google");
}

#[test]
fn test_fallback_android_devices() {
    assert!(!FALLBACK_ANDROID_DEVICES.is_empty());
    assert!(FALLBACK_ANDROID_DEVICES.contains(&"pixel_7"));
    assert!(FALLBACK_ANDROID_DEVICES.contains(&"pixel_6"));
    assert!(FALLBACK_ANDROID_DEVICES.contains(&"pixel_5"));
    
    // Verify all devices are reasonable fallbacks
    for device in FALLBACK_ANDROID_DEVICES {
        assert!(!device.is_empty());
        assert!(!device.contains(' ')); // Device IDs shouldn't have spaces
    }
}

#[test]
fn test_test_constants() {
    assert_eq!(TEST_ANDROID_DEVICE, "pixel_7");
    assert_eq!(TEST_IOS_DEVICE, "com.apple.CoreSimulator.SimDeviceType.iPhone-15");
    
    // Constants should be valid device identifiers
    assert!(!TEST_ANDROID_DEVICE.is_empty());
    assert!(!TEST_IOS_DEVICE.is_empty());
}

#[test]
fn test_device_category_debug() {
    let category = DeviceCategory::Phone;
    let debug_str = format!("{category:?}");
    assert_eq!(debug_str, "Phone");
    
    let category = DeviceCategory::Foldable;
    let debug_str = format!("{category:?}");
    assert_eq!(debug_str, "Foldable");
}

#[test]
fn test_device_info_clone() {
    let original = DeviceInfo {
        id: "test_device".to_string(),
        display_name: "Test Device".to_string(),
        oem: Some("Test OEM".to_string()),
        category: DeviceCategory::Tablet,
    };
    
    let cloned = original.clone();
    assert_eq!(original.id, cloned.id);
    assert_eq!(original.display_name, cloned.display_name);
    assert_eq!(original.oem, cloned.oem);
    assert_eq!(original.category, cloned.category);
}

#[test]
fn test_api_level_info_clone() {
    let original = ApiLevelInfo {
        level: 33,
        version_name: "Android 13".to_string(),
        available_tags: vec!["default".to_string()],
    };
    
    let cloned = original.clone();
    assert_eq!(original.level, cloned.level);
    assert_eq!(original.version_name, cloned.version_name);
    assert_eq!(original.available_tags, cloned.available_tags);
}

#[test]
fn test_device_info_debug() {
    let device = DeviceInfo {
        id: "debug_test".to_string(),
        display_name: "Debug Test Device".to_string(),
        oem: None,
        category: DeviceCategory::Unknown,
    };
    
    let debug_str = format!("{device:?}");
    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("Debug Test Device"));
    assert!(debug_str.contains("Unknown"));
}

#[test]
fn test_device_cache_operations() {
    let mut config = DynamicDeviceConfig::new();
    
    let devices = vec![
        DeviceInfo {
            id: "test1".to_string(),
            display_name: "Test Device 1".to_string(),
            oem: Some("Test OEM".to_string()),
            category: DeviceCategory::Phone,
        },
        DeviceInfo {
            id: "test2".to_string(),
            display_name: "Test Device 2".to_string(),
            oem: Some("Test OEM".to_string()),
            category: DeviceCategory::Tablet,
        },
    ];
    
    config.load_device_cache(devices);
    
    // Cached devices should have better priority than unknown
    let cached_priority = config.get_device_priority("test1");
    let unknown_priority = config.get_device_priority("unknown");
    
    assert!(cached_priority < unknown_priority);
    assert_eq!(unknown_priority, 999);
}

#[test]
fn test_api_cache_operations() {
    let mut config = DynamicDeviceConfig::new();
    
    let api_levels = vec![
        ApiLevelInfo {
            level: 34,
            version_name: "Android 14".to_string(),
            available_tags: vec!["google_apis".to_string()],
        },
        ApiLevelInfo {
            level: 33,
            version_name: "Android 13".to_string(),
            available_tags: vec!["default".to_string()],
        },
    ];
    
    config.load_api_cache(api_levels);
    
    let version_name = config.get_android_version_name(34);
    assert_eq!(version_name, "Android 14");
    
    let fallback_name = config.get_android_version_name(999);
    assert_eq!(fallback_name, "API 999");
}