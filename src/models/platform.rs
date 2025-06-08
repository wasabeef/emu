//! Platform definitions

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Android,
    Ios,
}

impl Platform {
    pub fn all() -> Vec<Platform> {
        vec![Platform::Android, Platform::Ios]
    }

    pub fn supported() -> Vec<Platform> {
        let mut platforms = vec![Platform::Android];

        if cfg!(target_os = "macos") {
            platforms.push(Platform::Ios);
        }

        platforms
    }

    pub fn is_supported(&self) -> bool {
        match self {
            Platform::Android => true,
            Platform::Ios => cfg!(target_os = "macos"),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::Android => "Android",
            Platform::Ios => "iOS",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Platform::Android => "android",
            Platform::Ios => "ios",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Platform::Android => "Android Emulators",
            Platform::Ios => "iOS Simulators",
        }
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub platform: Platform,
    pub version: String,
    pub sdk_path: Option<String>,
    pub tools_available: Vec<String>,
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
