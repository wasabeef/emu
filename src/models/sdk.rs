//! SDK management data structures.
//!
//! This module defines structures for managing Android SDK packages and API levels.

use serde::{Deserialize, Serialize};

/// Represents an Android SDK package available for installation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkPackage {
    /// Package identifier (e.g., "platforms;android-34")
    pub name: String,

    /// Package version string
    pub version: String,

    /// Human-readable description
    pub description: String,

    /// Whether the package is currently installed
    pub installed: bool,
}

/// Represents an Android API level that can be installed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiLevel {
    /// Numeric API level (e.g., 34)
    pub level: u32,

    /// Android version name (e.g., "Android 14")
    pub version_name: String,

    /// SDK package name for this API level
    pub package_name: String,

    /// Whether this API level is currently installed
    pub installed: bool,

    /// System image type (e.g., "Google Play", "Google APIs", "AOSP")
    pub image_type: Option<String>,
}

impl ApiLevel {
    /// Creates a new ApiLevel instance.
    ///
    /// # Arguments
    /// * `level` - The numeric API level
    /// * `version_name` - The Android version name
    /// * `installed` - Whether the API level is installed
    ///
    /// # Returns
    /// A new ApiLevel instance
    pub fn new(level: u32, version_name: String, installed: bool) -> Self {
        Self {
            level,
            version_name,
            package_name: format!("platforms;android-{}", level),
            installed,
            image_type: None,
        }
    }

    /// Returns a display string for the API level.
    ///
    /// # Returns
    /// Formatted string like "API 34 - Android 14 (Google Play)"
    pub fn display_name(&self) -> String {
        if let Some(ref image_type) = self.image_type {
            format!(
                "API {} - {} ({})",
                self.level, self.version_name, image_type
            )
        } else {
            format!("API {} - {}", self.level, self.version_name)
        }
    }

    /// Returns whether this API level is compatible with modern devices.
    ///
    /// # Returns
    /// True if API level >= 21 (Android 5.0)
    pub fn is_modern(&self) -> bool {
        self.level >= 21
    }
}

/// Represents the current status of an SDK installation operation.
#[derive(Debug, Clone, PartialEq)]
pub enum SdkInstallStatus {
    /// Installation not started
    Pending,

    /// Currently downloading/installing
    Installing {
        /// Progress percentage (0-100)
        progress: u8,

        /// Current operation description
        message: String,
    },

    /// Installation completed successfully
    Completed,

    /// Installation failed
    Failed {
        /// Error message
        error: String,
    },
}

impl SdkInstallStatus {
    /// Returns whether the installation is currently in progress.
    pub fn is_in_progress(&self) -> bool {
        matches!(self, SdkInstallStatus::Installing { .. })
    }

    /// Returns whether the installation completed successfully.
    pub fn is_completed(&self) -> bool {
        matches!(self, SdkInstallStatus::Completed)
    }

    /// Returns whether the installation failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, SdkInstallStatus::Failed { .. })
    }

    /// Returns the current progress percentage if installing.
    pub fn progress(&self) -> Option<u8> {
        if let SdkInstallStatus::Installing { progress, .. } = self {
            Some(*progress)
        } else {
            None
        }
    }

    /// Returns the current status message.
    pub fn message(&self) -> Option<&str> {
        match self {
            SdkInstallStatus::Installing { message, .. } => Some(message),
            SdkInstallStatus::Failed { error } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_level_creation() {
        let api_level = ApiLevel::new(34, "Android 14".to_string(), true);

        assert_eq!(api_level.level, 34);
        assert_eq!(api_level.version_name, "Android 14");
        assert_eq!(api_level.package_name, "platforms;android-34");
        assert!(api_level.installed);
    }

    #[test]
    fn test_api_level_display_name() {
        let api_level = ApiLevel::new(34, "Android 14".to_string(), false);
        assert_eq!(api_level.display_name(), "API 34 - Android 14");
    }

    #[test]
    fn test_api_level_is_modern() {
        let old_api = ApiLevel::new(19, "Android 4.4".to_string(), false);
        let modern_api = ApiLevel::new(34, "Android 14".to_string(), false);

        assert!(!old_api.is_modern());
        assert!(modern_api.is_modern());
    }

    #[test]
    fn test_sdk_install_status() {
        let pending = SdkInstallStatus::Pending;
        let installing = SdkInstallStatus::Installing {
            progress: 50,
            message: "Downloading...".to_string(),
        };
        let completed = SdkInstallStatus::Completed;
        let failed = SdkInstallStatus::Failed {
            error: "Network error".to_string(),
        };

        assert!(!pending.is_in_progress());
        assert!(installing.is_in_progress());
        assert!(!completed.is_in_progress());
        assert!(!failed.is_in_progress());

        assert_eq!(installing.progress(), Some(50));
        assert_eq!(pending.progress(), None);

        assert_eq!(installing.message(), Some("Downloading..."));
        assert_eq!(failed.message(), Some("Network error"));
        assert_eq!(pending.message(), None);
    }
}
