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
            _ => Err(format!("Unknown platform: {}", s)),
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
