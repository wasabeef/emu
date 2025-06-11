use emu::models::device_info::DynamicDeviceConfig;
use std::collections::HashMap;

// Test helper to expose private methods from DynamicDeviceConfig for testing
struct TestHelper;

impl TestHelper {
    fn extract_android_device_version(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        // Check if this is a Pixel device (special handling)
        if combined.contains("pixel") && !combined.contains("nexus") {
            // For Pixel devices, the priority is the extracted version directly
            return DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);
        }

        // For non-Pixel devices, calculate version component
        let full_priority =
            DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name);
        let category_priority = Self::get_category_priority(device_id, display_name);
        let oem_priority = Self::get_oem_priority(display_name);

        // Version bonus is what's left after removing category and OEM priorities
        // For phones (category 0), OEM bonus is divided by 2 in the actual calculation
        // For other categories, OEM bonus is multiplied by 2
        if category_priority == 0 {
            full_priority
                .saturating_sub(category_priority)
                .saturating_sub(oem_priority / 2)
        } else {
            full_priority
                .saturating_sub(category_priority)
                .saturating_sub(oem_priority * 2)
        }
    }

    fn get_category_priority(device_id: &str, display_name: &str) -> u32 {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            display_name.to_lowercase()
        );

        // Match the logic from infer_device_category - order matters!
        // Foldable devices (check first to avoid phone categorization)
        if combined.contains("fold") || combined.contains("flip") {
            20
        } else if combined.contains("tablet")
            || combined.contains("pad")
            || (combined.contains("10") && combined.contains("inch"))
            || (combined.contains("11") && combined.contains("inch"))
            || (combined.contains("12") && combined.contains("inch"))
        {
            100
        } else if combined.contains("phone")
            || (combined.contains("pixel")
                && !combined.contains("fold")
                && !combined.contains("tablet"))
            || (combined.contains("galaxy")
                && !combined.contains("fold")
                && !combined.contains("tablet"))
            || combined.contains("oneplus")
            || (combined.contains("5") && combined.contains("inch"))
            || (combined.contains("6") && combined.contains("inch"))
            || (combined.contains("pro")
                && !combined.contains("tablet")
                && !combined.contains("fold"))
        {
            0
        } else if combined.contains("tv") || combined.contains("1080p") || combined.contains("4k") {
            200
        } else if combined.contains("wear")
            || combined.contains("watch")
            || (combined.contains("round") && !combined.contains("tablet"))
        {
            300
        } else if combined.contains("auto") || combined.contains("car") {
            400
        } else {
            500
        }
    }

    fn get_oem_priority(display_name: &str) -> u32 {
        let combined = display_name.to_lowercase();

        if combined.contains("google") || combined.contains("pixel") {
            0
        } else if combined.contains("samsung") || combined.contains("galaxy") {
            10
        } else if combined.contains("oneplus") {
            20
        } else if let Some(start) = display_name.find('(') {
            if let Some(end) = display_name.find(')') {
                let oem_part = &display_name[start + 1..end].to_lowercase();
                match oem_part.as_str() {
                    "xiaomi" => 30,
                    "asus" => 35,
                    "oppo" => 40,
                    "vivo" => 45,
                    "huawei" => 50,
                    "motorola" => 55,
                    "lenovo" => 60,
                    "sony" => 65,
                    _ => 100,
                }
            } else {
                100
            }
        } else {
            100
        }
    }

    fn calculate_android_device_priority(device_id: &str, display_name: &str) -> u32 {
        DynamicDeviceConfig::calculate_android_device_priority(device_id, display_name)
    }

    fn calculate_ios_device_priority(display_name: &str) -> u32 {
        DynamicDeviceConfig::calculate_ios_device_priority(display_name)
    }
}

#[test]
fn test_device_version_extraction() {
    let test_cases = vec![
        // (device_id, display_name, is_android)
        ("pixel_8_pro", "Pixel 8 Pro (Google)", true),
        ("pixel_7", "Pixel 7 (Google)", true),
        ("pixel_9", "Pixel 9 (Google)", true),
        ("galaxy_s24", "Galaxy S24 (Samsung)", true),
        ("galaxy_s23", "Galaxy S23 (Samsung)", true),
    ];

    let mut priorities = HashMap::new();

    for (device_id, display_name, _is_android) in &test_cases {
        let version_bonus = TestHelper::extract_android_device_version(device_id, display_name);
        priorities.insert((*device_id, *display_name), version_bonus);
        println!(
            "Device: {} {} -> Version Bonus: {}",
            device_id, display_name, version_bonus
        );
    }

    // Newer devices should have lower version bonus numbers (100 - version)
    assert!(
        priorities[&("pixel_9", "Pixel 9 (Google)")]
            < priorities[&("pixel_8_pro", "Pixel 8 Pro (Google)")],
        "Pixel 9 should have lower version bonus than Pixel 8 Pro"
    );
    assert!(
        priorities[&("pixel_8_pro", "Pixel 8 Pro (Google)")]
            < priorities[&("pixel_7", "Pixel 7 (Google)")],
        "Pixel 8 Pro should have lower version bonus than Pixel 7"
    );
    assert!(
        priorities[&("galaxy_s24", "Galaxy S24 (Samsung)")]
            < priorities[&("galaxy_s23", "Galaxy S23 (Samsung)")],
        "Galaxy S24 should have lower version bonus than Galaxy S23"
    );

    // Test iOS devices separately
    let ios_test_cases = vec![
        ("iPhone 15", 50 - 15), // Version 15 -> 50 - 15 = 35
        ("iPhone 14", 50 - 14), // Version 14 -> 50 - 14 = 36
    ];

    for (display_name, expected_priority) in ios_test_cases {
        let priority = TestHelper::calculate_ios_device_priority(display_name);
        println!(
            "iOS Device: {} -> Priority: {} (expected around {})",
            display_name, priority, expected_priority
        );
        assert!(
            priority <= expected_priority + 10,
            "iOS device {} priority should be close to expected",
            display_name
        );
    }
}

#[test]
fn test_oem_priority() {
    let test_cases = vec![
        ("Pixel 8 (Google)", 0),      // Google highest
        ("Galaxy S24 (Samsung)", 10), // Samsung second
        ("OnePlus 11", 20),           // OnePlus third
        ("Mi 13 (Xiaomi)", 30),       // Xiaomi fourth
        ("Unknown Device", 100),      // Unknown lowest
    ];

    for (display_name, expected_priority) in test_cases {
        let priority = TestHelper::get_oem_priority(display_name);
        println!(
            "Device: {} -> OEM Priority: {} (expected: {})",
            display_name, priority, expected_priority
        );
        assert_eq!(
            priority, expected_priority,
            "OEM priority mismatch for {}",
            display_name
        );
    }
}

#[test]
fn test_category_priority() {
    let test_cases = vec![
        ("pixel_8", "Pixel 8 (Google)", 0),        // Phone category
        ("pixel_fold", "Pixel Fold (Google)", 20), // Foldable
        ("pixel_tablet", "Pixel Tablet", 100),     // Tablet
        ("android_tv", "Android TV (1080p)", 200), // TV
        ("wear_round", "Android Wear Round", 300), // Wear
    ];

    for (device_id, display_name, expected_category) in test_cases {
        let category_priority = TestHelper::get_category_priority(device_id, display_name);
        println!(
            "Device: {} (id: {}) -> Category Priority: {} (expected: {})",
            display_name, device_id, category_priority, expected_category
        );

        assert_eq!(
            category_priority, expected_category,
            "Category priority mismatch for {}",
            display_name
        );
    }
}

#[test]
fn test_overall_priority_sorting() {
    let mut devices = vec![
        ("pixel_7", "Pixel 7 (Google)"),
        ("pixel_8", "Pixel 8 (Google)"),
        ("pixel_9", "Pixel 9 (Google)"),
        ("galaxy_s23", "Galaxy S23 (Samsung)"),
        ("galaxy_s24", "Galaxy S24 (Samsung)"),
        ("android_tv", "Android TV (1080p)"),
        ("wear_round", "Android Wear Round"),
        ("pixel_fold", "Pixel Fold (Google)"),
    ];

    // Sort by priority (lower number = higher priority)
    devices.sort_by(|a, b| {
        let priority_a = TestHelper::calculate_android_device_priority(a.0, a.1);
        let priority_b = TestHelper::calculate_android_device_priority(b.0, b.1);
        priority_a.cmp(&priority_b)
    });

    println!("Sorted devices (highest priority first):");
    for (i, (device_id, display_name)) in devices.iter().enumerate() {
        let priority = TestHelper::calculate_android_device_priority(device_id, display_name);
        let category = TestHelper::get_category_priority(device_id, display_name);
        let oem = TestHelper::get_oem_priority(display_name);
        // Calculate displayed OEM and version based on category
        let (displayed_oem, version) = if category == 0 {
            // For phones, OEM is divided by 2
            let displayed_oem = oem / 2;
            let version = priority
                .saturating_sub(category)
                .saturating_sub(displayed_oem);
            (displayed_oem, version)
        } else {
            // For other categories, OEM is multiplied by 2
            let displayed_oem = oem * 2;
            let version = priority
                .saturating_sub(category)
                .saturating_sub(displayed_oem);
            (displayed_oem, version)
        };
        println!(
            "  {}. {} -> Priority: {} (category: {}, oem: {}, version: {})",
            i + 1,
            display_name,
            priority,
            category,
            displayed_oem,
            version
        );
    }

    // Check that Pixel 9 comes before Pixel 8, which comes before Pixel 7
    let pixel_9_pos = devices.iter().position(|(id, _)| id == &"pixel_9").unwrap();
    let pixel_8_pos = devices.iter().position(|(id, _)| id == &"pixel_8").unwrap();
    let pixel_7_pos = devices.iter().position(|(id, _)| id == &"pixel_7").unwrap();

    assert!(
        pixel_9_pos < pixel_8_pos,
        "Pixel 9 should come before Pixel 8"
    );
    assert!(
        pixel_8_pos < pixel_7_pos,
        "Pixel 8 should come before Pixel 7"
    );

    // Verify the expected order based on user requirements:
    // 1. Pixel devices get highest priority regardless of version vs other manufacturers
    // 2. Within Pixel devices, newer versions come first

    // Pixel 9 should come before Galaxy S24 due to Pixel priority
    let galaxy_s24_pos = devices
        .iter()
        .position(|(_, name)| name.contains("Galaxy S24"))
        .unwrap();
    let pixel_9_pos = devices.iter().position(|(id, _)| id == &"pixel_9").unwrap();

    // This demonstrates that Pixel devices have highest priority
    assert!(
        pixel_9_pos < galaxy_s24_pos,
        "Pixel 9 should come before Galaxy S24 (Pixel priority)"
    );

    // But within the same version range, Google should come before Samsung
    // (This is already handled by the OEM priority in the calculation)

    // Check that phones come before TV and Wear devices
    let phone_positions: Vec<usize> = devices
        .iter()
        .enumerate()
        .filter(|(_, (_, name))| {
            name.contains("Pixel") && !name.contains("TV") && !name.contains("Wear")
        })
        .map(|(i, _)| i)
        .collect();

    let tv_position = devices
        .iter()
        .position(|(_, name)| name.contains("TV"))
        .unwrap();
    let wear_position = devices
        .iter()
        .position(|(_, name)| name.contains("Wear"))
        .unwrap();

    for phone_pos in phone_positions {
        assert!(
            phone_pos < tv_position,
            "Phones should come before TV devices"
        );
        assert!(
            phone_pos < wear_position,
            "Phones should come before Wear devices"
        );
    }

    // Check that Pixel Fold is categorized as foldable, not phone
    let fold_pos = devices
        .iter()
        .position(|(id, _)| id == &"pixel_fold")
        .unwrap();
    let fold_priority =
        TestHelper::calculate_android_device_priority("pixel_fold", "Pixel Fold (Google)");
    let fold_category = TestHelper::get_category_priority("pixel_fold", "Pixel Fold (Google)");
    println!(
        "Pixel Fold position: {}, priority: {}, category: {}",
        fold_pos, fold_priority, fold_category
    );
    assert_eq!(
        fold_category, 20,
        "Pixel Fold should be categorized as foldable (priority 20)"
    );
}

#[test]
fn test_simple_pixel_priority() {
    // Simple test to see what priority "Pixel" gets
    let pixel_priority = TestHelper::calculate_android_device_priority("pixel", "Pixel (Google)");
    println!("Simple Pixel priority: {}", pixel_priority);

    // This should be 25 according to our logic
    assert_eq!(
        pixel_priority, 25,
        "Pixel should get priority 25 (unversioned)"
    );
}

#[test]
fn test_pixel_versioned_vs_unversioned_priority() {
    // Test versioned vs unversioned Pixel devices
    let pixel_priority = TestHelper::calculate_android_device_priority("pixel", "Pixel (Google)");
    let pixel_c_priority =
        TestHelper::calculate_android_device_priority("pixel_c", "Pixel C (Google)");
    let pixel_fold_priority =
        TestHelper::calculate_android_device_priority("pixel_fold", "Pixel Fold (Google)");
    let pixel_9_priority =
        TestHelper::calculate_android_device_priority("pixel_9", "Pixel 9 (Google)");
    let pixel_9_pro_priority =
        TestHelper::calculate_android_device_priority("pixel_9_pro", "Pixel 9 Pro (Google)");
    let pixel_9_fold_priority =
        TestHelper::calculate_android_device_priority("pixel_9_fold", "Pixel 9 Fold (Google)");

    println!("Pixel priority: {}", pixel_priority);
    println!("Pixel C priority: {}", pixel_c_priority);
    println!("Pixel Fold priority: {}", pixel_fold_priority);
    println!("Pixel 9 priority: {}", pixel_9_priority);
    println!("Pixel 9 Pro priority: {}", pixel_9_pro_priority);
    println!("Pixel 9 Fold priority: {}", pixel_9_fold_priority);

    // Debug version extraction - call the actual method to see what's happening
    println!("=== DEBUGGING VERSION EXTRACTION ===");

    // Test what extract_device_version returns for these cases
    // We can't call it directly, so let's look at the actual priority calculation

    // Let's manually check what happens inside the pixel special handling
    let combined_pixel = format!(
        "{} {}",
        "pixel".to_lowercase(),
        "Pixel (Google)".to_lowercase()
    );
    let combined_pixel_9 = format!(
        "{} {}",
        "pixel_9".to_lowercase(),
        "Pixel 9 (Google)".to_lowercase()
    );

    println!("Combined pixel: '{}'", combined_pixel);
    println!("Combined pixel_9: '{}'", combined_pixel_9);
    println!("Contains pixel: {}", combined_pixel.contains("pixel"));
    println!("Contains nexus: {}", combined_pixel.contains("nexus"));

    // Debug version extraction
    let pixel_version = TestHelper::extract_android_device_version("pixel", "Pixel (Google)");
    let pixel_9_version = TestHelper::extract_android_device_version("pixel_9", "Pixel 9 (Google)");
    println!("Pixel version extraction: {}", pixel_version);
    println!("Pixel 9 version extraction: {}", pixel_9_version);

    // Versioned Pixel devices should come before unversioned ones
    assert!(
        pixel_9_priority < pixel_priority,
        "Pixel 9 ({}) should come before Pixel ({})",
        pixel_9_priority,
        pixel_priority
    );
    assert!(
        pixel_9_priority < pixel_c_priority,
        "Pixel 9 should come before Pixel C"
    );
    assert!(
        pixel_9_priority < pixel_fold_priority,
        "Pixel 9 should come before Pixel Fold"
    );
    assert!(
        pixel_9_pro_priority < pixel_priority,
        "Pixel 9 Pro should come before Pixel"
    );
    assert!(
        pixel_9_fold_priority < pixel_fold_priority,
        "Pixel 9 Fold should come before Pixel Fold"
    );
}

#[test]
fn test_galaxy_nexus_priority() {
    // Test Galaxy Nexus specifically
    let galaxy_nexus_priority =
        TestHelper::calculate_android_device_priority("galaxy_nexus", "Galaxy Nexus (Google)");
    let pixel_7_priority =
        TestHelper::calculate_android_device_priority("pixel_7", "Pixel 7 (Google)");
    let pixel_8_priority =
        TestHelper::calculate_android_device_priority("pixel_8", "Pixel 8 (Google)");

    println!("Galaxy Nexus priority: {}", galaxy_nexus_priority);
    println!("Pixel 7 priority: {}", pixel_7_priority);
    println!("Pixel 8 priority: {}", pixel_8_priority);

    // Galaxy Nexus should come after Pixel devices
    assert!(
        pixel_7_priority < galaxy_nexus_priority,
        "Pixel 7 should come before Galaxy Nexus"
    );
    assert!(
        pixel_8_priority < galaxy_nexus_priority,
        "Pixel 8 should come before Galaxy Nexus"
    );
}

#[test]
fn test_realistic_device_ordering() {
    // Realistic device list similar to what avdmanager might return
    let mut devices = vec![
        ("pixel_8_pro", "Pixel 8 Pro (Google)"),
        ("pixel_7a", "Pixel 7a (Google)"),
        ("pixel_fold", "Pixel Fold (Google)"),
        ("galaxy_s24_ultra", "Galaxy S24 Ultra (Samsung)"),
        ("galaxy_a54", "Galaxy A54 (Samsung)"),
        ("medium_phone", "Medium Phone (Generic)"),
        ("tv_1080p", "Android TV (1080p)"),
        ("wear_round", "Android Wear Round"),
        ("automotive_1024p", "Automotive (1024p)"),
        ("2.7_qvga", "2.7\" QVGA"),
    ];

    devices.sort_by(|a, b| {
        let priority_a = TestHelper::calculate_android_device_priority(a.0, a.1);
        let priority_b = TestHelper::calculate_android_device_priority(b.0, b.1);
        priority_a.cmp(&priority_b)
    });

    println!("Realistic device ordering:");
    for (i, (device_id, display_name)) in devices.iter().enumerate() {
        let priority = TestHelper::calculate_android_device_priority(device_id, display_name);
        println!(
            "  {}. {} (ID: {}) -> Priority: {}",
            i + 1,
            display_name,
            device_id,
            priority
        );
    }

    // The first few devices should be phones, with Google having preference
    assert!(
        devices[0].1.contains("Pixel") || devices[0].1.contains("Galaxy"),
        "First device should be a major phone brand"
    );

    // At least one of the first 3 should be a Pixel (Google priority)
    let first_three_has_pixel = devices
        .iter()
        .take(3)
        .any(|(_, name)| name.contains("Pixel"));
    assert!(
        first_three_has_pixel,
        "First three devices should include a Pixel"
    );

    // TV and Wear devices should be near the end
    let tv_position = devices
        .iter()
        .position(|(_, name)| name.contains("TV"))
        .unwrap();
    let wear_position = devices
        .iter()
        .position(|(_, name)| name.contains("Wear"))
        .unwrap();

    assert!(
        tv_position > devices.len() / 2,
        "TV devices should be in second half"
    );
    assert!(
        wear_position > devices.len() / 2,
        "Wear devices should be in second half"
    );
}
#[test]
fn test_pixel_version_ordering() {
    // Test that Pixel devices are ordered by version correctly
    let mut pixel_devices = [
        ("pixel_7", "Pixel 7 (Google)"),
        ("pixel_8", "Pixel 8 (Google)"),
        ("pixel_9", "Pixel 9 (Google)"),
        ("pixel_10", "Pixel 10 (Google)"), // Future device
        ("pixel_8_pro", "Pixel 8 Pro (Google)"),
        ("pixel_7a", "Pixel 7a (Google)"),
    ];

    pixel_devices.sort_by(|a, b| {
        let priority_a = TestHelper::calculate_android_device_priority(a.0, a.1);
        let priority_b = TestHelper::calculate_android_device_priority(b.0, b.1);
        priority_a.cmp(&priority_b)
    });

    println!("Pixel device ordering:");
    for (i, (device_id, display_name)) in pixel_devices.iter().enumerate() {
        let priority = TestHelper::calculate_android_device_priority(device_id, display_name);
        println!("  {}. {} -> Priority: {}", i + 1, display_name, priority);
    }

    // Check that newer Pixels come first
    let pixel_10_pos = pixel_devices
        .iter()
        .position(|(id, _)| id == &"pixel_10")
        .unwrap();
    let pixel_9_pos = pixel_devices
        .iter()
        .position(|(id, _)| id == &"pixel_9")
        .unwrap();
    let pixel_8_pos = pixel_devices
        .iter()
        .position(|(id, _)| id == &"pixel_8")
        .unwrap();
    let pixel_7_pos = pixel_devices
        .iter()
        .position(|(id, _)| id == &"pixel_7")
        .unwrap();

    assert!(
        pixel_10_pos < pixel_9_pos,
        "Pixel 10 should come before Pixel 9"
    );
    assert!(
        pixel_9_pos < pixel_8_pos,
        "Pixel 9 should come before Pixel 8"
    );
    assert!(
        pixel_8_pos < pixel_7_pos,
        "Pixel 8 should come before Pixel 7"
    );
}
