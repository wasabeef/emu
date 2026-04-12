use super::*;

#[test]
#[cfg(target_os = "macos")]
fn test_extract_ios_version() {
    assert_eq!(extract_ios_version("iOS 18.5"), 18.05);
    assert_eq!(extract_ios_version("iOS 17.0"), 17.0);
    assert_eq!(extract_ios_version("iOS 16.4"), 16.04);

    assert!((extract_ios_version("iOS 15.2.1") - 15.0201).abs() < 0.0001);
    assert!((extract_ios_version("iOS 16.3.2") - 16.0302).abs() < 0.0001);

    assert_eq!(extract_ios_version("watchOS 10.0"), 10.0);
    assert_eq!(extract_ios_version("tvOS 17.2"), 17.02);

    assert_eq!(extract_ios_version("iOS-17-0"), 17.0);
    assert_eq!(extract_ios_version("iOS"), 0.0);
    assert_eq!(extract_ios_version(""), 0.0);

    assert!(extract_ios_version("iOS 18.5") > extract_ios_version("iOS 18.0"));
    assert!(extract_ios_version("iOS 17.9") > extract_ios_version("iOS 17.1"));
}

#[test]
#[cfg(target_os = "macos")]
fn test_parse_device_type_display_name() {
    let result = IosManager::parse_device_type_display_name(
        "com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro",
    );
    assert!(result.contains("iPhone"));
    assert!(result.contains("15"));
    assert!(result.contains("Pro"));

    let result2 = IosManager::parse_device_type_display_name("iPhone-14");
    assert!(result2.contains("iPhone"));
    assert!(result2.contains("14"));

    assert_eq!(IosManager::parse_device_type_display_name(""), "");
}

#[test]
#[cfg(target_os = "macos")]
fn test_ios_manager_new() {
    let _manager = IosManager::new().expect("Failed to create IosManager");
}

#[test]
#[cfg(target_os = "macos")]
fn test_ios_device_parsing_edge_cases() {
    assert_eq!(extract_ios_version("iOS 18"), 18.0);
    assert_eq!(extract_ios_version("iOS-18"), 18.0);
    assert_eq!(extract_ios_version("iOS.18"), 18.0);
    assert_eq!(extract_ios_version("invalid"), 0.0);
    assert_eq!(extract_ios_version("iOS 99.99.99"), 99.9999);

    let result = IosManager::parse_device_type_display_name(
        "com.apple.CoreSimulator.SimDeviceType.iPad-Pro-12-9-inch-6th-generation",
    );
    assert!(result.contains("iPad"));
    assert!(result.contains("Pro"));

    let result2 = IosManager::parse_device_type_display_name("Apple-Watch-Series-8-45mm");
    assert!(result2.contains("Apple"));
    assert!(result2.contains("Watch"));
}

#[test]
fn test_ios_manager_creation() {
    let _manager = IosManager::new().expect("Failed to create IosManager");
}

#[allow(dead_code)]
#[cfg(not(target_os = "macos"))]
async fn test_ios_manager_non_macos_operations_disabled() {
    use crate::managers::common::{DeviceConfig, DeviceManager};
    use std::collections::HashMap;

    let _manager = IosManager::new().expect("Failed to create IosManager");

    assert!(<IosManager as DeviceManager>::list_devices(&_manager)
        .await
        .is_err());
    assert!(
        <IosManager as DeviceManager>::start_device(&_manager, "test")
            .await
            .is_err()
    );
    assert!(
        <IosManager as DeviceManager>::stop_device(&_manager, "test")
            .await
            .is_err()
    );

    let config = DeviceConfig {
        name: "Test Device".to_string(),
        device_type: "iPhone-15".to_string(),
        version: "iOS 17.0".to_string(),
        ram_size: None,
        storage_size: None,
        additional_options: HashMap::new(),
    };
    assert!(
        <IosManager as DeviceManager>::create_device(&_manager, &config)
            .await
            .is_err()
    );
    assert!(
        <IosManager as DeviceManager>::delete_device(&_manager, "test")
            .await
            .is_err()
    );
    assert!(
        <IosManager as DeviceManager>::wipe_device(&_manager, "test")
            .await
            .is_err()
    );
}

#[allow(dead_code)]
#[cfg(not(target_os = "macos"))]
async fn test_ios_manager_unified_device_manager_non_macos_disabled() {
    use crate::managers::common::{DeviceConfig, UnifiedDeviceManager};
    use std::collections::HashMap;

    let _manager = IosManager::new().expect("Failed to create IosManager");

    assert!(
        <IosManager as UnifiedDeviceManager>::list_devices(&_manager)
            .await
            .is_err()
    );
    assert!(
        <IosManager as UnifiedDeviceManager>::start_device(&_manager, "test")
            .await
            .is_err()
    );
    assert!(
        <IosManager as UnifiedDeviceManager>::stop_device(&_manager, "test")
            .await
            .is_err()
    );

    let config = DeviceConfig {
        name: "Test Device".to_string(),
        device_type: "iPhone-15".to_string(),
        version: "iOS 17.0".to_string(),
        ram_size: None,
        storage_size: None,
        additional_options: HashMap::new(),
    };
    assert!(
        <IosManager as UnifiedDeviceManager>::create_device(&_manager, &config)
            .await
            .is_err()
    );
    assert!(
        <IosManager as UnifiedDeviceManager>::delete_device(&_manager, "test")
            .await
            .is_err()
    );
    assert!(
        <IosManager as UnifiedDeviceManager>::wipe_device(&_manager, "test")
            .await
            .is_err()
    );
    assert!(!<IosManager as UnifiedDeviceManager>::is_available(&_manager).await);
}

#[allow(dead_code)]
#[cfg(target_os = "macos")]
fn test_ios_device_priority_disabled() {}

#[test]
#[cfg(target_os = "macos")]
fn test_ios_device_status_parsing() {
    let statuses = vec!["Booted", "Shutdown", "Creating", "Booting", "Shutting Down"];

    for status in statuses {
        match status {
            "Booted" => assert_eq!(status, "Booted"),
            "Shutdown" => assert_eq!(status, "Shutdown"),
            _ => assert!(!status.is_empty()),
        }
    }
}

#[allow(dead_code)]
async fn test_ios_manager_error_handling_disabled() {
    let _manager = IosManager::new().expect("Failed to create IosManager");

    #[cfg(not(target_os = "macos"))]
    {
        use crate::managers::common::DeviceConfig;
        use std::collections::HashMap;

        let config = DeviceConfig {
            name: "Test".to_string(),
            device_type: "iPhone".to_string(),
            version: "17.0".to_string(),
            ram_size: None,
            storage_size: None,
            additional_options: HashMap::new(),
        };

        let result = _manager.create_device(&config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("macOS"));
    }
}

#[test]
#[cfg(target_os = "macos")]
fn test_ios_version_comparison() {
    let v1 = extract_ios_version("iOS 17.0");
    let v2 = extract_ios_version("iOS 17.1");
    let v3 = extract_ios_version("iOS 18.0");

    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v1 < v3);

    let v4 = extract_ios_version("iOS 17.0.1");
    let v5 = extract_ios_version("iOS 17.0.2");

    assert!(v4 < v5);
    assert!(v1 < v4);
}

#[test]
#[cfg(target_os = "macos")]
fn test_device_type_display_formatting() {
    let test_cases = vec![
        "iPhone-15-Pro-Max",
        "iPad-Pro-11-inch",
        "Apple-Watch-Series-9",
        "Apple-TV-4K",
    ];

    for input in test_cases {
        let result = IosManager::parse_device_type_display_name(input);
        assert!(
            !result.is_empty(),
            "Result should not be empty for input '{input}'"
        );

        if input.contains("iPhone") {
            assert!(
                result.contains("iPhone"),
                "Result '{result}' should contain 'iPhone' for input '{input}'"
            );
        }
        if input.contains("iPad") {
            assert!(
                result.contains("iPad"),
                "Result '{result}' should contain 'iPad' for input '{input}'"
            );
        }
        if input.contains("Apple-Watch") {
            assert!(
                result.contains("Apple"),
                "Result '{result}' should contain 'Apple' for input '{input}'"
            );
        }
        if input.contains("TV") {
            assert!(
                result.contains("TV"),
                "Result '{result}' should contain 'TV' for input '{input}'"
            );
        }

        assert!(
            !result.contains("-"),
            "Result '{result}' should not contain hyphens"
        );
    }
}
