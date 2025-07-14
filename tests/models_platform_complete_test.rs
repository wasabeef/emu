//! Comprehensive tests for models::platform module
//!
//! These tests ensure complete coverage of Platform enum, PlatformInfo struct,
//! and all platform-specific functionality including serialization and traits.

use anyhow::Result;
use emu::models::platform::{Platform, PlatformInfo};
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::test]
async fn test_platform_all() -> Result<()> {
    let all_platforms = Platform::all();

    assert_eq!(all_platforms.len(), 2);
    assert!(all_platforms.contains(&Platform::Android));
    assert!(all_platforms.contains(&Platform::Ios));

    Ok(())
}

#[tokio::test]
async fn test_platform_supported() -> Result<()> {
    let supported = Platform::supported();

    // Android should always be supported
    assert!(supported.contains(&Platform::Android));

    // iOS support depends on platform
    if cfg!(target_os = "macos") {
        assert!(supported.contains(&Platform::Ios));
        assert_eq!(supported.len(), 2);
    } else {
        assert!(!supported.contains(&Platform::Ios));
        assert_eq!(supported.len(), 1);
    }

    Ok(())
}

#[tokio::test]
async fn test_platform_is_supported() -> Result<()> {
    // Android should always be supported
    assert!(Platform::Android.is_supported());

    // iOS support depends on platform
    if cfg!(target_os = "macos") {
        assert!(Platform::Ios.is_supported());
    } else {
        assert!(!Platform::Ios.is_supported());
    }

    Ok(())
}

#[tokio::test]
async fn test_platform_display_name() -> Result<()> {
    assert_eq!(Platform::Android.display_name(), "Android");
    assert_eq!(Platform::Ios.display_name(), "iOS");

    Ok(())
}

#[tokio::test]
async fn test_platform_short_name() -> Result<()> {
    assert_eq!(Platform::Android.short_name(), "android");
    assert_eq!(Platform::Ios.short_name(), "ios");

    Ok(())
}

#[tokio::test]
async fn test_platform_description() -> Result<()> {
    assert_eq!(Platform::Android.description(), "Android Emulators");
    assert_eq!(Platform::Ios.description(), "iOS Simulators");

    Ok(())
}

#[tokio::test]
async fn test_platform_requirements() -> Result<()> {
    let android_reqs = Platform::Android.requirements();
    assert_eq!(android_reqs.len(), 2);
    assert!(android_reqs.contains(&"Android SDK"));
    assert!(android_reqs.contains(&"ANDROID_HOME environment variable"));

    let ios_reqs = Platform::Ios.requirements();
    assert_eq!(ios_reqs.len(), 2);
    assert!(ios_reqs.contains(&"Xcode"));
    assert!(ios_reqs.contains(&"macOS"));

    Ok(())
}

#[tokio::test]
async fn test_platform_display_trait() -> Result<()> {
    assert_eq!(format!("{}", Platform::Android), "Android");
    assert_eq!(format!("{}", Platform::Ios), "iOS");

    Ok(())
}

#[tokio::test]
async fn test_platform_from_str() -> Result<()> {
    // Test valid parsing
    assert_eq!(Platform::from_str("android").unwrap(), Platform::Android);
    assert_eq!(Platform::from_str("Android").unwrap(), Platform::Android);
    assert_eq!(Platform::from_str("ANDROID").unwrap(), Platform::Android);

    assert_eq!(Platform::from_str("ios").unwrap(), Platform::Ios);
    assert_eq!(Platform::from_str("iOS").unwrap(), Platform::Ios);
    assert_eq!(Platform::from_str("IOS").unwrap(), Platform::Ios);

    // Test invalid parsing
    assert!(Platform::from_str("windows").is_err());
    assert!(Platform::from_str("linux").is_err());
    assert!(Platform::from_str("").is_err());
    assert!(Platform::from_str("invalid").is_err());

    let error = Platform::from_str("invalid").unwrap_err();
    assert!(error.contains("Unknown platform: invalid"));

    Ok(())
}

#[tokio::test]
async fn test_platform_serialization() -> Result<()> {
    // Test Android serialization
    let android_json = serde_json::to_string(&Platform::Android)?;
    let android_deserialized: Platform = serde_json::from_str(&android_json)?;
    assert_eq!(android_deserialized, Platform::Android);

    // Test iOS serialization
    let ios_json = serde_json::to_string(&Platform::Ios)?;
    let ios_deserialized: Platform = serde_json::from_str(&ios_json)?;
    assert_eq!(ios_deserialized, Platform::Ios);

    Ok(())
}

#[tokio::test]
async fn test_platform_traits() -> Result<()> {
    // Test Debug
    assert_eq!(format!("{:?}", Platform::Android), "Android");
    assert_eq!(format!("{:?}", Platform::Ios), "Ios");

    // Test Clone
    let android = Platform::Android;
    let android_clone = android;
    assert_eq!(android, android_clone);

    // Test Copy
    let ios = Platform::Ios;
    let ios_copy = ios;
    assert_eq!(ios, ios_copy);

    // Test PartialEq
    assert_eq!(Platform::Android, Platform::Android);
    assert_ne!(Platform::Android, Platform::Ios);

    // Test Eq
    assert!(Platform::Android == Platform::Android);
    assert!(Platform::Android != Platform::Ios);

    Ok(())
}

#[tokio::test]
async fn test_platform_hash() -> Result<()> {
    use std::collections::HashSet;

    let mut platforms = HashSet::new();
    platforms.insert(Platform::Android);
    platforms.insert(Platform::Ios);
    platforms.insert(Platform::Android); // Duplicate should not increase size

    assert_eq!(platforms.len(), 2);
    assert!(platforms.contains(&Platform::Android));
    assert!(platforms.contains(&Platform::Ios));

    // Test in HashMap
    let mut platform_map = HashMap::new();
    platform_map.insert(Platform::Android, "Android SDK");
    platform_map.insert(Platform::Ios, "Xcode");

    assert_eq!(platform_map.get(&Platform::Android), Some(&"Android SDK"));
    assert_eq!(platform_map.get(&Platform::Ios), Some(&"Xcode"));

    Ok(())
}

#[tokio::test]
async fn test_platform_info_new() -> Result<()> {
    let android_info = PlatformInfo::new(Platform::Android);

    assert_eq!(android_info.platform, Platform::Android);
    assert_eq!(android_info.version, "");
    assert_eq!(android_info.sdk_path, None);
    assert_eq!(android_info.tools_available.len(), 0);
    assert!(!android_info.is_configured);

    let ios_info = PlatformInfo::new(Platform::Ios);
    assert_eq!(ios_info.platform, Platform::Ios);

    Ok(())
}

#[tokio::test]
async fn test_platform_info_builder_pattern() -> Result<()> {
    let info = PlatformInfo::new(Platform::Android)
        .version("34.0.0".to_string())
        .sdk_path(Some("/opt/android-sdk".to_string()))
        .tools(vec!["adb".to_string(), "emulator".to_string()])
        .configured(true);

    assert_eq!(info.platform, Platform::Android);
    assert_eq!(info.version, "34.0.0");
    assert_eq!(info.sdk_path, Some("/opt/android-sdk".to_string()));
    assert_eq!(info.tools_available, vec!["adb", "emulator"]);
    assert!(info.is_configured);

    Ok(())
}

#[tokio::test]
async fn test_platform_info_partial_builder() -> Result<()> {
    let info = PlatformInfo::new(Platform::Ios)
        .version("15.0".to_string())
        .configured(false);

    assert_eq!(info.platform, Platform::Ios);
    assert_eq!(info.version, "15.0");
    assert_eq!(info.sdk_path, None);
    assert_eq!(info.tools_available.len(), 0);
    assert!(!info.is_configured);

    Ok(())
}

#[tokio::test]
async fn test_platform_info_serialization() -> Result<()> {
    let info = PlatformInfo::new(Platform::Android)
        .version("33.0.2".to_string())
        .sdk_path(Some("/Users/test/Android/Sdk".to_string()))
        .tools(vec!["avdmanager".to_string(), "sdkmanager".to_string()])
        .configured(true);

    // Test serialization
    let json = serde_json::to_string(&info)?;
    assert!(json.contains("Android"));
    assert!(json.contains("33.0.2"));
    assert!(json.contains("/Users/test/Android/Sdk"));
    assert!(json.contains("avdmanager"));

    // Test deserialization
    let deserialized: PlatformInfo = serde_json::from_str(&json)?;
    assert_eq!(deserialized.platform, info.platform);
    assert_eq!(deserialized.version, info.version);
    assert_eq!(deserialized.sdk_path, info.sdk_path);
    assert_eq!(deserialized.tools_available, info.tools_available);
    assert_eq!(deserialized.is_configured, info.is_configured);

    Ok(())
}

#[tokio::test]
async fn test_platform_info_clone() -> Result<()> {
    let info = PlatformInfo::new(Platform::Ios)
        .version("17.0".to_string())
        .tools(vec!["xcrun".to_string(), "simctl".to_string()]);

    let cloned = info.clone();

    assert_eq!(cloned.platform, info.platform);
    assert_eq!(cloned.version, info.version);
    assert_eq!(cloned.sdk_path, info.sdk_path);
    assert_eq!(cloned.tools_available, info.tools_available);
    assert_eq!(cloned.is_configured, info.is_configured);

    Ok(())
}

#[tokio::test]
async fn test_platform_info_debug() -> Result<()> {
    let info = PlatformInfo::new(Platform::Android)
        .version("debug_version".to_string())
        .configured(true);

    let debug_output = format!("{info:?}");
    assert!(debug_output.contains("PlatformInfo"));
    assert!(debug_output.contains("Android"));
    assert!(debug_output.contains("debug_version"));
    assert!(debug_output.contains("true"));

    Ok(())
}

#[tokio::test]
async fn test_platform_info_edge_cases() -> Result<()> {
    // Test with empty strings and vectors
    let empty_info = PlatformInfo::new(Platform::Android)
        .version(String::new())
        .sdk_path(Some(String::new()))
        .tools(Vec::new());

    assert_eq!(empty_info.version, "");
    assert_eq!(empty_info.sdk_path, Some("".to_string()));
    assert_eq!(empty_info.tools_available.len(), 0);

    // Test with very long strings
    let long_version = "a".repeat(1000);
    let long_path = "/very/long/path/".repeat(100);
    let long_info = PlatformInfo::new(Platform::Ios)
        .version(long_version.clone())
        .sdk_path(Some(long_path.clone()));

    assert_eq!(long_info.version, long_version);
    assert_eq!(long_info.sdk_path, Some(long_path));

    Ok(())
}
