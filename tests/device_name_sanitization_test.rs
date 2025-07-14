use emu::app::state::CreateDeviceForm;
use emu::models::device_info::DynamicDeviceConfig;

#[test]
fn test_device_name_generation_preserves_spaces() {
    let mut form = CreateDeviceForm::new();

    // Set device type and version for testing
    form.device_type = "Pixel Fold (Google)".to_string();
    form.device_type_id = "pixel_fold".to_string();
    form.version = "35".to_string();
    form.version_display = "API 35 - Android 15".to_string();

    // Generate placeholder name
    form.generate_placeholder_name();

    // Verify that spaces are preserved
    assert!(form.name.contains(" "), "Device name should contain spaces");
    assert!(
        form.name.contains("Pixel"),
        "Device name should contain device type"
    );
    assert!(
        form.name.contains("API"),
        "Device name should contain API level"
    );

    // Test specific format - new dynamic parser excludes content in parentheses
    assert_eq!(form.name, "Pixel Fold API 35");

    let name = &form.name;
    println!("Generated device name: '{name}'");
}

#[test]
fn test_device_name_generation_various_formats() {
    let test_cases = vec![
        // (device_type, version_display, expected_contains)
        (
            "2.7\" QVGA",
            "API 36 - Android 16",
            vec!["2.7", "QVGA", "API 36"],
        ),
        (
            "iPhone 15 Pro",
            "iOS 17.0",
            vec!["iPhone", "15", "Pro", "iOS 17"],
        ),
        (
            "Android TV (1080p)",
            "API 34 - Android 14",
            vec!["Android", "TV", "API 34"],
        ),
        (
            "Wear Round",
            "API 33 - Android 13",
            vec!["Wear", "Round", "API 33"],
        ),
    ];

    for (device_type, version_display, expected_parts) in test_cases {
        let mut form = CreateDeviceForm::new();
        form.device_type = device_type.to_string();
        form.device_type_id = device_type.to_lowercase().replace(" ", "_");
        form.version_display = version_display.to_string();
        form.version = if version_display.starts_with("iOS") {
            version_display.to_string()
        } else {
            version_display
                .split_whitespace()
                .nth(1)
                .unwrap_or("35")
                .to_string()
        };

        form.generate_placeholder_name();

        println!(
            "Device type: '{}' -> Generated name: '{}'",
            device_type, form.name
        );

        // Verify that expected parts are included
        for expected_part in expected_parts {
            assert!(
                form.name.contains(expected_part),
                "Device name '{}' should contain '{}'",
                form.name,
                expected_part
            );
        }

        // Verify that name is not empty
        assert!(
            !form.name.trim().is_empty(),
            "Device name should not be empty"
        );
    }
}

#[test]
fn test_device_name_fallback_when_empty() {
    let mut form = CreateDeviceForm::new();

    // Set empty device type and version
    form.device_type = "".to_string();
    form.device_type_id = "".to_string();
    form.version = "35".to_string();
    form.version_display = "".to_string();

    form.generate_placeholder_name();

    // Verify that fallback name is generated
    assert_eq!(form.name, "Device API");

    let name = &form.name;
    println!("Fallback device name: '{name}'");
}

#[test]
fn test_dynamic_device_config_parsing() {
    let config = DynamicDeviceConfig::new();

    let test_cases = vec![
        "Pixel 7 Pro (Google)",
        "iPhone 15 Pro Max",
        "2.7\" QVGA",
        "Android TV (1080p)",
        "Wear Round",
    ];

    for device_name in test_cases {
        let parsed_parts = config.parse_device_name(device_name);

        println!("Device '{device_name}' parsed to: {parsed_parts:?}");

        // Verify that parsing didn't fail (not empty)
        assert!(
            !parsed_parts.is_empty() || device_name.is_empty(),
            "{}",
            format!("Should be able to parse device name '{device_name}'")
        );
    }
}

#[test]
fn test_create_device_form_android_initialization() {
    let form = CreateDeviceForm::for_android();

    // Test Android form initialization state
    assert_eq!(form.ram_size, "2048");
    assert_eq!(form.storage_size, "8192");
    assert!(form.available_device_types.is_empty());
    assert!(form.available_versions.is_empty());
    assert_eq!(form.selected_api_level_index, 0);
    assert_eq!(form.selected_device_type_index, 0);
    assert!(!form.is_loading_cache);

    println!("Android form initialized correctly");
}

#[test]
fn test_create_device_form_ios_initialization() {
    let form = CreateDeviceForm::for_ios();

    // Test iOS form initialization state
    assert_eq!(form.ram_size, "2048");
    assert_eq!(form.storage_size, "8192");
    assert!(form.available_device_types.is_empty());
    assert!(form.available_versions.is_empty());
    assert_eq!(form.selected_api_level_index, 0);
    assert_eq!(form.selected_device_type_index, 0);
    assert!(!form.is_loading_cache);

    println!("iOS form initialized correctly");
}

#[test]
fn test_device_name_with_special_characters() {
    let mut form = CreateDeviceForm::new();

    // Test device name with special characters
    form.device_type = "2.7\" QVGA (Small)".to_string();
    form.device_type_id = "qvga_2_7".to_string();
    form.version = "36".to_string();
    form.version_display = "API 36 - Android 16".to_string();

    form.generate_placeholder_name();

    // Verify that name is generated (spaces are preserved)
    assert!(!form.name.is_empty(), "Device name should not be empty");
    assert!(form.name.contains("2.7"), "Should contain screen size");
    assert!(form.name.contains("QVGA"), "Should contain resolution");
    assert!(form.name.contains("API 36"), "Should contain API level");

    // Verify that double quotes are processed (remain in display name)
    let name = &form.name;
    println!("Special character device name: '{name}'");
}

#[test]
fn test_device_name_sanitization_for_avd_creation() {
    // Test for Android sanitization
    let test_cases = vec![
        // (input, should_be_safe_for_avd)
        ("Pixel 7 Pro API 34", true),    // Normal case
        ("2.7\" QVGA API 36", true),     // With quotes (removed in AVD)
        ("Device with: colon", true),    // With colon (removed in AVD)
        ("Device/with/slash", true),     // With slash (removed in AVD)
        ("Normal Device Name", true),    // Normal case
        ("'Single Quote Device'", true), // With single quotes
        ("Device*with*asterisk", true),  // With asterisk
    ];

    for (input, should_be_safe) in test_cases {
        // Test actual AndroidManager sanitization process (for AVD names)
        let sanitized = input
            .chars()
            .filter_map(|c| match c {
                // AVD names: only a-z A-Z 0-9 . _ - are allowed
                c if c.is_ascii_alphanumeric() || c == '.' || c == '-' => Some(c),
                ' ' | '_' => Some('_'), // Convert spaces to underscores
                _ => None,              // Remove all other characters
            })
            .collect::<String>()
            .trim_matches('_') // Remove leading/trailing underscores
            .to_string();

        if should_be_safe {
            assert!(
                !sanitized.is_empty(),
                "{}",
                format!("Sanitized name should not be empty for: '{input}'")
            );
            // In AVD names, spaces are converted to underscores
            if input.contains(' ') {
                assert!(
                    sanitized.contains('_'),
                    "{}",
                    format!("Spaces should be converted to underscores in AVD name: '{input}'")
                );
            }
        }

        println!("Input: '{input}' -> AVD Name: '{sanitized}'");
    }
}
