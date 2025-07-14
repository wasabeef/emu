//! Comprehensive tests for models::api_level module
//!
//! These tests ensure complete coverage of ApiLevel, SystemImageVariant,
//! and InstallProgress structures with all methods and edge cases.

use emu::models::api_level::{ApiLevel, InstallProgress, SystemImageVariant};

#[test]
fn test_api_level_new() {
    let api_level = ApiLevel::new(
        34,
        "Android 14".to_string(),
        "system-images;android-34;google_apis;x86_64".to_string(),
    );

    assert_eq!(api_level.api, 34);
    assert_eq!(api_level.version, "Android 14");
    assert_eq!(api_level.display_name, "API 34 (Android 14)");
    assert_eq!(
        api_level.system_image_id,
        "system-images;android-34;google_apis;x86_64"
    );
    assert!(!api_level.is_installed);
    assert!(api_level.variants.is_empty());
}

#[test]
fn test_api_level_display_name_formatting() {
    let api_30 = ApiLevel::new(30, "Android 11".to_string(), "test".to_string());
    let api_33 = ApiLevel::new(33, "Android 13".to_string(), "test".to_string());
    let api_34 = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    assert_eq!(api_30.display_name, "API 30 (Android 11)");
    assert_eq!(api_33.display_name, "API 33 (Android 13)");
    assert_eq!(api_34.display_name, "API 34 (Android 14)");
}

#[test]
fn test_api_level_get_recommended_variant_empty() {
    let api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    assert!(api_level.get_recommended_variant().is_none());
}

#[test]
fn test_api_level_get_recommended_variant_single() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());
    let variant = SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    );
    api_level.variants.push(variant);

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().variant, "google_apis");
    assert_eq!(recommended.unwrap().architecture, "x86_64");
}

#[test]
fn test_api_level_get_recommended_variant_priority_google_apis_playstore() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add variants in different order
    api_level.variants.push(SystemImageVariant::new(
        "default".to_string(),
        "x86_64".to_string(),
        "default-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "apis-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "x86_64".to_string(),
        "playstore-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().variant, "google_apis_playstore");
}

#[test]
fn test_api_level_get_recommended_variant_priority_google_apis() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add variants without playstore
    api_level.variants.push(SystemImageVariant::new(
        "default".to_string(),
        "x86_64".to_string(),
        "default-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "apis-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().variant, "google_apis");
}

#[test]
fn test_api_level_get_recommended_variant_priority_default() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Only default variant
    api_level.variants.push(SystemImageVariant::new(
        "default".to_string(),
        "x86_64".to_string(),
        "default-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().variant, "default");
}

#[cfg(target_arch = "x86_64")]
#[test]
fn test_api_level_architecture_preference_x86_64() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add same variant with different architectures
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "arm64-v8a".to_string(),
        "arm-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "x86-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().architecture, "x86_64");
}

#[cfg(target_arch = "aarch64")]
#[test]
fn test_api_level_architecture_preference_aarch64() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add same variant with different architectures
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "x86-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "arm64-v8a".to_string(),
        "arm-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().architecture, "arm64-v8a");
}

#[test]
fn test_api_level_architecture_fallback_x86_64() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add variant with non-preferred architecture
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86".to_string(),
        "x86-package".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "x86_64-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    // Should fallback to x86_64 if preferred arch not available
    assert_eq!(recommended.unwrap().architecture, "x86_64");
}

#[test]
fn test_api_level_get_recommended_variant_first_fallback() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add variant with architecture that doesn't match any preference
    api_level.variants.push(SystemImageVariant::new(
        "custom".to_string(),
        "mips".to_string(),
        "mips-package".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());
    assert_eq!(recommended.unwrap().variant, "custom");
    assert_eq!(recommended.unwrap().architecture, "mips");
}

#[test]
fn test_api_level_complex_variant_selection() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Add multiple variants with complex priority testing
    api_level.variants.push(SystemImageVariant::new(
        "default".to_string(),
        "arm64-v8a".to_string(),
        "default-arm".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "arm64-v8a".to_string(),
        "apis-arm".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "arm64-v8a".to_string(),
        "playstore-arm".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "default".to_string(),
        "x86_64".to_string(),
        "default-x86".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "apis-x86".to_string(),
    ));
    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "x86_64".to_string(),
        "playstore-x86".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());

    // Should prefer google_apis_playstore with preferred architecture
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!(recommended.unwrap().variant, "google_apis_playstore");
        assert_eq!(recommended.unwrap().architecture, "x86_64");
    }

    #[cfg(target_arch = "aarch64")]
    {
        assert_eq!(recommended.unwrap().variant, "google_apis_playstore");
        assert_eq!(recommended.unwrap().architecture, "arm64-v8a");
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        // Should fallback to x86_64 for other architectures
        assert_eq!(recommended.unwrap().variant, "google_apis_playstore");
        assert_eq!(recommended.unwrap().architecture, "x86_64");
    }
}

#[test]
fn test_api_level_serialization() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test-image".to_string());
    api_level.is_installed = true;
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    ));

    // Test serialization
    let serialized = serde_json::to_string(&api_level).unwrap();
    assert!(serialized.contains("\"api\":34"));
    assert!(serialized.contains("\"version\":\"Android 14\""));
    assert!(serialized.contains("\"is_installed\":true"));

    // Test deserialization
    let deserialized: ApiLevel = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.api, 34);
    assert_eq!(deserialized.version, "Android 14");
    assert!(deserialized.is_installed);
    assert_eq!(deserialized.variants.len(), 1);
}

#[test]
fn test_api_level_debug_formatting() {
    let api_level = ApiLevel::new(34, "Android 14".to_string(), "test-image".to_string());

    let debug_output = format!("{api_level:?}");
    assert!(debug_output.contains("ApiLevel"));
    assert!(debug_output.contains("api: 34"));
    assert!(debug_output.contains("Android 14"));
    assert!(debug_output.contains("test-image"));
}

#[test]
fn test_api_level_clone() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test-image".to_string());
    api_level.is_installed = true;
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    ));

    let cloned = api_level.clone();
    assert_eq!(cloned.api, api_level.api);
    assert_eq!(cloned.version, api_level.version);
    assert_eq!(cloned.is_installed, api_level.is_installed);
    assert_eq!(cloned.variants.len(), api_level.variants.len());
}

#[test]
fn test_system_image_variant_new() {
    let variant = SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "system-images;android-34;google_apis;x86_64".to_string(),
    );

    assert_eq!(variant.variant, "google_apis");
    assert_eq!(variant.architecture, "x86_64");
    assert_eq!(
        variant.package_id,
        "system-images;android-34;google_apis;x86_64"
    );
    assert!(!variant.is_installed);
    assert_eq!(variant.display_name, "Google APIs (x86_64)");
}

#[test]
fn test_system_image_variant_display_names() {
    let playstore = SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "x86_64".to_string(),
        "test".to_string(),
    );
    let apis = SystemImageVariant::new(
        "google_apis".to_string(),
        "arm64-v8a".to_string(),
        "test".to_string(),
    );
    let default =
        SystemImageVariant::new("default".to_string(), "x86".to_string(), "test".to_string());
    let custom = SystemImageVariant::new(
        "custom_variant".to_string(),
        "mips".to_string(),
        "test".to_string(),
    );

    assert_eq!(playstore.display_name, "Google Play (x86_64)");
    assert_eq!(apis.display_name, "Google APIs (arm64-v8a)");
    assert_eq!(default.display_name, "Default (x86)");
    assert_eq!(custom.display_name, "custom_variant (mips)");
}

#[test]
fn test_system_image_variant_host_preferred_architecture() {
    let arch = SystemImageVariant::host_preferred_architecture();

    #[cfg(target_arch = "x86_64")]
    assert_eq!(arch, "x86_64");

    #[cfg(target_arch = "aarch64")]
    assert_eq!(arch, "arm64-v8a");

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    assert_eq!(arch, "x86_64");
}

#[test]
fn test_system_image_variant_serialization() {
    let mut variant = SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    );
    variant.is_installed = true;

    // Test serialization
    let serialized = serde_json::to_string(&variant).unwrap();
    assert!(serialized.contains("\"variant\":\"google_apis\""));
    assert!(serialized.contains("\"architecture\":\"x86_64\""));
    assert!(serialized.contains("\"is_installed\":true"));

    // Test deserialization
    let deserialized: SystemImageVariant = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.variant, "google_apis");
    assert_eq!(deserialized.architecture, "x86_64");
    assert!(deserialized.is_installed);
}

#[test]
fn test_system_image_variant_debug_formatting() {
    let variant = SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    );

    let debug_output = format!("{variant:?}");
    assert!(debug_output.contains("SystemImageVariant"));
    assert!(debug_output.contains("google_apis"));
    assert!(debug_output.contains("x86_64"));
    assert!(debug_output.contains("test-package"));
}

#[test]
fn test_system_image_variant_clone() {
    let mut variant = SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "test-package".to_string(),
    );
    variant.is_installed = true;

    let cloned = variant.clone();
    assert_eq!(cloned.variant, variant.variant);
    assert_eq!(cloned.architecture, variant.architecture);
    assert_eq!(cloned.package_id, variant.package_id);
    assert_eq!(cloned.is_installed, variant.is_installed);
    assert_eq!(cloned.display_name, variant.display_name);
}

#[test]
fn test_install_progress_creation() {
    let progress = InstallProgress {
        operation: "Downloading system image".to_string(),
        percentage: 75,
        eta_seconds: Some(120),
    };

    assert_eq!(progress.operation, "Downloading system image");
    assert_eq!(progress.percentage, 75);
    assert_eq!(progress.eta_seconds, Some(120));
}

#[test]
fn test_install_progress_without_eta() {
    let progress = InstallProgress {
        operation: "Installing...".to_string(),
        percentage: 100,
        eta_seconds: None,
    };

    assert_eq!(progress.operation, "Installing...");
    assert_eq!(progress.percentage, 100);
    assert!(progress.eta_seconds.is_none());
}

#[test]
fn test_install_progress_debug_formatting() {
    let progress = InstallProgress {
        operation: "Extracting files".to_string(),
        percentage: 50,
        eta_seconds: Some(60),
    };

    let debug_output = format!("{progress:?}");
    assert!(debug_output.contains("InstallProgress"));
    assert!(debug_output.contains("Extracting files"));
    assert!(debug_output.contains("percentage: 50"));
    assert!(debug_output.contains("Some(60)"));
}

#[test]
fn test_install_progress_clone() {
    let progress = InstallProgress {
        operation: "Verifying download".to_string(),
        percentage: 90,
        eta_seconds: Some(30),
    };

    let cloned = progress.clone();
    assert_eq!(cloned.operation, progress.operation);
    assert_eq!(cloned.percentage, progress.percentage);
    assert_eq!(cloned.eta_seconds, progress.eta_seconds);
}

#[test]
fn test_install_progress_percentage_bounds() {
    let min_progress = InstallProgress {
        operation: "Starting...".to_string(),
        percentage: 0,
        eta_seconds: None,
    };

    let max_progress = InstallProgress {
        operation: "Complete".to_string(),
        percentage: 100,
        eta_seconds: Some(0),
    };

    assert_eq!(min_progress.percentage, 0);
    assert_eq!(max_progress.percentage, 100);
    assert_eq!(max_progress.eta_seconds, Some(0));
}

#[test]
fn test_api_level_edge_cases() {
    // Test with empty strings
    let api_level = ApiLevel::new(0, String::new(), String::new());
    assert_eq!(api_level.api, 0);
    assert_eq!(api_level.version, "");
    assert_eq!(api_level.display_name, "API 0 ()");
    assert_eq!(api_level.system_image_id, "");

    // Test with very long strings
    let long_version = "A".repeat(1000);
    let long_image_id = "B".repeat(1000);
    let api_level = ApiLevel::new(9999, long_version.clone(), long_image_id.clone());
    assert_eq!(api_level.api, 9999);
    assert_eq!(api_level.version, long_version);
    assert_eq!(api_level.display_name, format!("API 9999 ({long_version})"));
    assert_eq!(api_level.system_image_id, long_image_id);
}

#[test]
fn test_system_image_variant_edge_cases() {
    // Test with empty strings
    let variant = SystemImageVariant::new(String::new(), String::new(), String::new());
    assert_eq!(variant.variant, "");
    assert_eq!(variant.architecture, "");
    assert_eq!(variant.package_id, "");
    assert_eq!(variant.display_name, " ()");

    // Test with special characters
    let variant = SystemImageVariant::new(
        "variant-with_special.chars".to_string(),
        "arch@test".to_string(),
        "package#id".to_string(),
    );
    assert_eq!(variant.variant, "variant-with_special.chars");
    assert_eq!(variant.architecture, "arch@test");
    assert_eq!(variant.package_id, "package#id");
    assert_eq!(
        variant.display_name,
        "variant-with_special.chars (arch@test)"
    );
}

#[test]
fn test_comprehensive_variant_priority_testing() {
    let mut api_level = ApiLevel::new(34, "Android 14".to_string(), "test".to_string());

    // Test all possible combinations and priority orders
    let test_cases = vec![
        ("default", "x86"),
        ("default", "x86_64"),
        ("default", "arm64-v8a"),
        ("google_apis", "x86"),
        ("google_apis", "x86_64"),
        ("google_apis", "arm64-v8a"),
        ("google_apis_playstore", "x86"),
        ("google_apis_playstore", "x86_64"),
        ("google_apis_playstore", "arm64-v8a"),
    ];

    for (variant, arch) in test_cases {
        api_level.variants.push(SystemImageVariant::new(
            variant.to_string(),
            arch.to_string(),
            format!("{variant}-{arch}"),
        ));
    }

    let recommended = api_level.get_recommended_variant().unwrap();

    // Verify highest priority variant is selected
    assert_eq!(recommended.variant, "google_apis_playstore");

    // Architecture should match host preference or fallback to x86_64
    #[cfg(target_arch = "x86_64")]
    assert_eq!(recommended.architecture, "x86_64");

    #[cfg(target_arch = "aarch64")]
    assert_eq!(recommended.architecture, "arm64-v8a");

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    assert_eq!(recommended.architecture, "x86_64");
}
