//! Tests for architecture detection functionality

use emu::models::api_level::{ApiLevel, SystemImageVariant};

#[test]
fn test_host_architecture_detection() {
    let host_arch = SystemImageVariant::host_preferred_architecture();

    // On Apple Silicon Macs, should prefer arm64-v8a
    #[cfg(target_arch = "aarch64")]
    assert_eq!(
        host_arch, "arm64-v8a",
        "Apple Silicon should prefer arm64-v8a"
    );

    // On Intel Macs, should prefer x86_64
    #[cfg(target_arch = "x86_64")]
    assert_eq!(host_arch, "x86_64", "Intel processors should prefer x86_64");

    println!("Detected host architecture preference: {}", host_arch);
}

#[test]
fn test_recommended_variant_selection() {
    // Create test API level with multiple variants
    let mut api_level = ApiLevel::new(
        34,
        "Android 14".to_string(),
        "system-images;android-34;google_apis_playstore;arm64-v8a".to_string(),
    );

    // Add variants in non-optimal order to test prioritization
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "system-images;android-34;google_apis;x86_64".to_string(),
    ));

    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "arm64-v8a".to_string(),
        "system-images;android-34;google_apis_playstore;arm64-v8a".to_string(),
    ));

    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "x86_64".to_string(),
        "system-images;android-34;google_apis_playstore;x86_64".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());

    let variant = recommended.unwrap();

    // On Apple Silicon, should prefer arm64-v8a with google_apis_playstore
    #[cfg(target_arch = "aarch64")]
    {
        assert_eq!(variant.variant, "google_apis_playstore");
        assert_eq!(variant.architecture, "arm64-v8a");
    }

    // On Intel, should prefer x86_64 with google_apis_playstore
    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!(variant.variant, "google_apis_playstore");
        assert_eq!(variant.architecture, "x86_64");
    }
}

#[test]
fn test_fallback_when_preferred_arch_not_available() {
    let mut api_level = ApiLevel::new(
        34,
        "Android 14".to_string(),
        "system-images;android-34;google_apis;x86_64".to_string(),
    );

    // Only add x86_64 variants (no arm64 variants)
    api_level.variants.push(SystemImageVariant::new(
        "google_apis".to_string(),
        "x86_64".to_string(),
        "system-images;android-34;google_apis;x86_64".to_string(),
    ));

    api_level.variants.push(SystemImageVariant::new(
        "google_apis_playstore".to_string(),
        "x86_64".to_string(),
        "system-images;android-34;google_apis_playstore;x86_64".to_string(),
    ));

    let recommended = api_level.get_recommended_variant();
    assert!(recommended.is_some());

    let variant = recommended.unwrap();
    // Should fall back to x86_64 with google_apis_playstore even on ARM
    assert_eq!(variant.variant, "google_apis_playstore");
    assert_eq!(variant.architecture, "x86_64");
}

#[test]
fn test_default_abi_function() {
    use emu::constants::android_packages;

    let default_abi = android_packages::default_abi();

    #[cfg(target_arch = "aarch64")]
    assert_eq!(default_abi, "arm64-v8a");

    #[cfg(target_arch = "x86_64")]
    assert_eq!(default_abi, "x86_64");

    println!("Default ABI for this host: {}", default_abi);
}
