//! Platform definitions and platform-specific information.
//!
//! This module defines the supported platforms (Android and iOS) and provides
//! utilities for checking platform availability, requirements, and configuration.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the supported virtual device platforms.
///
/// Each platform has different requirements and availability based on
/// the host operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    /// Android platform - available on all operating systems
    Android,
    /// iOS platform - only available on macOS
    Ios,
}

impl Platform {
    /// Returns all possible platform variants.
    pub fn all() -> Vec<Platform> {
        vec![Platform::Android, Platform::Ios]
    }

    /// Returns only the platforms supported on the current operating system.
    ///
    /// Android is supported on all platforms, while iOS is only supported on macOS.
    pub fn supported() -> Vec<Platform> {
        let mut platforms = vec![Platform::Android];

        if cfg!(target_os = "macos") {
            platforms.push(Platform::Ios);
        }

        platforms
    }

    /// Checks if this platform is supported on the current operating system.
    ///
    /// # Returns
    /// - `true` for Android on all systems
    /// - `true` for iOS only on macOS
    /// - `false` for iOS on non-macOS systems
    pub fn is_supported(&self) -> bool {
        match self {
            Platform::Android => true,
            Platform::Ios => cfg!(target_os = "macos"),
        }
    }

    /// Returns the human-readable display name of the platform.
    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::Android => "Android",
            Platform::Ios => "iOS",
        }
    }

    /// Returns the lowercase short name of the platform.
    pub fn short_name(&self) -> &'static str {
        match self {
            Platform::Android => "android",
            Platform::Ios => "ios",
        }
    }

    /// Returns a descriptive name for the platform's virtual devices.
    pub fn description(&self) -> &'static str {
        match self {
            Platform::Android => "Android Emulators",
            Platform::Ios => "iOS Simulators",
        }
    }

    /// Returns the requirements for using this platform.
    ///
    /// These are the tools and environment needed for the platform to work.
    pub fn requirements(&self) -> Vec<&'static str> {
        match self {
            Platform::Android => vec!["Android SDK", "ANDROID_HOME environment variable"],
            Platform::Ios => vec!["Xcode", "macOS"],
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "android" => Ok(Platform::Android),
            "ios" => Ok(Platform::Ios),
            _ => Err(format!("Unknown platform: {s}")),
        }
    }
}

/// Information about a platform's configuration and available tools.
///
/// This struct contains details about whether a platform is properly
/// configured, what tools are available, and where the SDK is located.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// The platform this information describes
    pub platform: Platform,
    /// SDK or tool version (e.g., "34.0.0" for Android SDK)
    pub version: String,
    /// Path to the SDK installation
    pub sdk_path: Option<String>,
    /// List of available command-line tools
    pub tools_available: Vec<String>,
    /// Whether the platform is properly configured and ready to use
    pub is_configured: bool,
}

impl PlatformInfo {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            version: String::new(),
            sdk_path: None,
            tools_available: Vec::new(),
            is_configured: false,
        }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = version;
        self
    }

    pub fn sdk_path(mut self, path: Option<String>) -> Self {
        self.sdk_path = path;
        self
    }

    pub fn tools(mut self, tools: Vec<String>) -> Self {
        self.tools_available = tools;
        self
    }

    pub fn configured(mut self, configured: bool) -> Self {
        self.is_configured = configured;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// Test Platform enum basic functionality
    #[test]
    fn test_platform_enum() {
        let android = Platform::Android;
        let ios = Platform::Ios;

        assert_eq!(android.display_name(), "Android");
        assert_eq!(ios.display_name(), "iOS");

        assert_eq!(android.short_name(), "android");
        assert_eq!(ios.short_name(), "ios");
    }

    /// Test Platform::all() returns all variants
    #[test]
    fn test_platform_all() {
        let all_platforms = Platform::all();
        assert_eq!(all_platforms.len(), 2);
        assert!(all_platforms.contains(&Platform::Android));
        assert!(all_platforms.contains(&Platform::Ios));
    }

    /// Test Platform::supported() respects operating system
    #[test]
    fn test_platform_supported() {
        let supported = Platform::supported();
        assert!(supported.contains(&Platform::Android));

        #[cfg(target_os = "macos")]
        assert!(supported.contains(&Platform::Ios));

        #[cfg(not(target_os = "macos"))]
        assert!(!supported.contains(&Platform::Ios));
    }

    /// Test Platform::is_supported() method
    #[test]
    fn test_platform_is_supported() {
        assert!(Platform::Android.is_supported());

        #[cfg(target_os = "macos")]
        assert!(Platform::Ios.is_supported());

        #[cfg(not(target_os = "macos"))]
        assert!(!Platform::Ios.is_supported());
    }

    /// Test Platform descriptions
    #[test]
    fn test_platform_descriptions() {
        assert_eq!(Platform::Android.description(), "Android Emulators");
        assert_eq!(Platform::Ios.description(), "iOS Simulators");
    }

    /// Test Platform requirements
    #[test]
    fn test_platform_requirements() {
        let android_reqs = Platform::Android.requirements();
        assert!(!android_reqs.is_empty());
        assert!(android_reqs.contains(&"Android SDK"));
        assert!(android_reqs.contains(&"ANDROID_HOME environment variable"));

        let ios_reqs = Platform::Ios.requirements();
        assert!(!ios_reqs.is_empty());
        assert!(ios_reqs.contains(&"Xcode"));
        assert!(ios_reqs.contains(&"macOS"));
    }

    /// Test Platform Display trait
    #[test]
    fn test_platform_display() {
        assert_eq!(format!("{}", Platform::Android), "Android");
        assert_eq!(format!("{}", Platform::Ios), "iOS");
    }

    /// Test Platform FromStr trait
    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("android").unwrap(), Platform::Android);
        assert_eq!(Platform::from_str("Android").unwrap(), Platform::Android);
        assert_eq!(Platform::from_str("ANDROID").unwrap(), Platform::Android);

        assert_eq!(Platform::from_str("ios").unwrap(), Platform::Ios);
        assert_eq!(Platform::from_str("iOS").unwrap(), Platform::Ios);
        assert_eq!(Platform::from_str("IOS").unwrap(), Platform::Ios);

        assert!(Platform::from_str("unknown").is_err());
        assert!(Platform::from_str("").is_err());
    }

    /// Test Platform equality and hash
    #[test]
    fn test_platform_equality() {
        assert_eq!(Platform::Android, Platform::Android);
        assert_eq!(Platform::Ios, Platform::Ios);
        assert_ne!(Platform::Android, Platform::Ios);

        // Test that platforms can be used in collections
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Platform::Android);
        set.insert(Platform::Ios);
        assert_eq!(set.len(), 2);
    }

    /// Test PlatformInfo::new()
    #[test]
    fn test_platform_info_new() {
        let android_info = PlatformInfo::new(Platform::Android);
        assert_eq!(android_info.platform, Platform::Android);
        assert!(android_info.version.is_empty());
        assert!(android_info.sdk_path.is_none());
        assert!(android_info.tools_available.is_empty());
        assert!(!android_info.is_configured);

        let ios_info = PlatformInfo::new(Platform::Ios);
        assert_eq!(ios_info.platform, Platform::Ios);
        assert!(!ios_info.is_configured);
    }

    /// Test PlatformInfo builder methods
    #[test]
    fn test_platform_info_builder() {
        let info = PlatformInfo::new(Platform::Android)
            .version("34.0.0".to_string())
            .sdk_path(Some("/usr/local/android-sdk".to_string()))
            .tools(vec!["adb".to_string(), "emulator".to_string()])
            .configured(true);

        assert_eq!(info.platform, Platform::Android);
        assert_eq!(info.version, "34.0.0");
        assert_eq!(info.sdk_path, Some("/usr/local/android-sdk".to_string()));
        assert_eq!(info.tools_available, vec!["adb", "emulator"]);
        assert!(info.is_configured);
    }

    /// Test PlatformInfo with iOS
    #[test]
    fn test_platform_info_ios() {
        let info = PlatformInfo::new(Platform::Ios)
            .version("15.0".to_string())
            .sdk_path(Some("/Applications/Xcode.app".to_string()))
            .tools(vec!["simctl".to_string(), "xcrun".to_string()])
            .configured(true);

        assert_eq!(info.platform, Platform::Ios);
        assert_eq!(info.version, "15.0");
        assert!(info.is_configured);
    }

    /// Test serialization and deserialization
    #[test]
    fn test_platform_serialization() {
        let android = Platform::Android;
        let ios = Platform::Ios;

        // Test that platforms can be serialized/deserialized
        let android_json = serde_json::to_string(&android).unwrap();
        let ios_json = serde_json::to_string(&ios).unwrap();

        let android_deserialized: Platform = serde_json::from_str(&android_json).unwrap();
        let ios_deserialized: Platform = serde_json::from_str(&ios_json).unwrap();

        assert_eq!(android, android_deserialized);
        assert_eq!(ios, ios_deserialized);
    }

    /// Test PlatformInfo serialization
    #[test]
    fn test_platform_info_serialization() {
        let info = PlatformInfo::new(Platform::Android)
            .version("34.0.0".to_string())
            .configured(true);

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: PlatformInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.platform, deserialized.platform);
        assert_eq!(info.version, deserialized.version);
        assert_eq!(info.is_configured, deserialized.is_configured);
    }
}
