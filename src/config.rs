//! Configuration management
//!
//! This module handles application configuration including UI themes and
//! platform-specific default values for device creation. Configuration
//! can be loaded from files or use sensible defaults.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration structure.
///
/// Contains global settings like theme selection and platform-specific
/// configuration for Android and iOS device management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// UI theme name (e.g., "dark", "light")
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Android-specific configuration
    #[serde(default)]
    pub android: AndroidConfig,

    /// iOS-specific configuration  
    #[serde(default)]
    pub ios: IosConfig,
}

/// Configuration specific to Android device management.
///
/// Contains default values used when creating new Android Virtual Devices (AVDs).
/// These defaults can be overridden by the user during device creation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AndroidConfig {
    /// Default RAM allocation for new devices (e.g., "2048")
    pub default_ram: String,
    /// Default storage size for new devices (e.g., "4096")
    pub default_storage: String,
    /// Default Android API level for new devices
    pub default_api_level: u32,
}

/// Configuration specific to iOS simulator management.
///
/// Contains default values used when creating new iOS simulators.
/// These defaults can be overridden by the user during device creation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IosConfig {
    /// Default device type identifier (e.g., "iPhone-15")
    pub default_device_type: String,
    /// Default iOS version (e.g., "17.0")
    pub default_ios_version: String,
}

impl Config {
    /// Loads configuration from a file or creates default configuration.
    ///
    /// Currently returns default configuration as file loading is not yet implemented.
    /// In the future, this will load settings from TOML/JSON configuration files.
    ///
    /// # Arguments
    /// * `_path` - Optional path to configuration file (currently unused)
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded or default configuration
    /// * `Err(anyhow::Error)` - If file loading fails (future implementation)
    ///
    /// # Examples
    /// ```rust
    /// let config = Config::load(None)?; // Use defaults
    /// let config = Config::load(Some(PathBuf::from("emu.toml")))?; // From file
    /// ```
    pub fn load(_path: Option<PathBuf>) -> Result<Self> {
        // TODO: Implement actual file loading
        // For now, always return defaults
        Ok(Self::default())
    }

    /// Returns the UI theme based on the configuration.
    ///
    /// Converts the theme name string into a concrete Theme object
    /// that can be used by the UI rendering system.
    ///
    /// # Returns
    /// A Theme object configured according to the theme setting
    ///
    /// # Current Implementation
    /// Always returns dark theme regardless of configuration.
    /// Future versions will support multiple themes.
    pub fn theme(&self) -> crate::ui::Theme {
        crate::ui::Theme::dark()
    }
}

impl Default for Config {
    /// Creates a default configuration with sensible defaults.
    ///
    /// # Returns
    /// A Config instance with:
    /// - Dark theme
    /// - Empty Android configuration (will use runtime defaults)
    /// - Empty iOS configuration (will use runtime defaults)
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            android: AndroidConfig::default(),
            ios: IosConfig::default(),
        }
    }
}

/// Returns the default theme name for serde deserialization.
///
/// This function provides the default value when the theme field
/// is missing from a configuration file.
///
/// # Returns
/// The string "dark" as the default theme
fn default_theme() -> String {
    "dark".to_string()
}
