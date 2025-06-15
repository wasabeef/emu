//! Android API level management structures.

use serde::{Deserialize, Serialize};

/// Represents an Android API level with its installation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiLevel {
    /// API level number (e.g., 34)
    pub api: u32,
    /// Android version name (e.g., "Android 12L")
    pub version: String,
    /// Full display name (e.g., "Android 12L (API 32)")
    pub display_name: String,
    /// System image ID (e.g., "system-images;android-32;google_apis;x86_64")
    pub system_image_id: String,
    /// Whether this API level is installed
    pub is_installed: bool,
    /// Available variants for this API level
    pub variants: Vec<SystemImageVariant>,
}

/// Represents a system image variant (e.g., google_apis, google_apis_playstore).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemImageVariant {
    /// Variant type (e.g., "google_apis", "google_apis_playstore", "default")
    pub variant: String,
    /// Architecture (e.g., "x86_64", "arm64-v8a")
    pub architecture: String,
    /// Full package ID
    pub package_id: String,
    /// Whether this variant is installed
    pub is_installed: bool,
    /// Display name for UI
    pub display_name: String,
}

/// API level installation progress information.
#[derive(Debug, Clone)]
pub struct InstallProgress {
    /// Current operation description
    pub operation: String,
    /// Progress percentage (0-100)
    pub percentage: u8,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u32>,
}

impl ApiLevel {
    /// Creates a new API level entry.
    pub fn new(api: u32, version: String, system_image_id: String) -> Self {
        let display_name = format!("{} (API {})", version, api);
        Self {
            api,
            version,
            display_name,
            system_image_id,
            is_installed: false,
            variants: Vec::new(),
        }
    }

    /// Gets the recommended variant for this API level.
    pub fn get_recommended_variant(&self) -> Option<&SystemImageVariant> {
        // Detect host architecture for optimal performance
        let preferred_arch = Self::get_preferred_architecture();

        // Priority: google_apis_playstore > google_apis > default
        // Within each variant type, prefer native architecture
        self.variants
            .iter()
            .find(|v| v.variant == "google_apis_playstore" && v.architecture == preferred_arch)
            .or_else(|| {
                self.variants
                    .iter()
                    .find(|v| v.variant == "google_apis_playstore" && v.architecture == "x86_64")
            })
            .or_else(|| {
                self.variants
                    .iter()
                    .find(|v| v.variant == "google_apis" && v.architecture == preferred_arch)
            })
            .or_else(|| {
                self.variants
                    .iter()
                    .find(|v| v.variant == "google_apis" && v.architecture == "x86_64")
            })
            .or_else(|| {
                self.variants
                    .iter()
                    .find(|v| v.variant == "default" && v.architecture == preferred_arch)
            })
            .or_else(|| {
                self.variants
                    .iter()
                    .find(|v| v.variant == "default" && v.architecture == "x86_64")
            })
            .or_else(|| self.variants.first())
    }

    /// Determines the preferred architecture based on the host system.
    fn get_preferred_architecture() -> &'static str {
        #[cfg(target_arch = "x86_64")]
        {
            "x86_64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            "arm64-v8a"
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // Default to x86_64 for other architectures
            "x86_64"
        }
    }
}

impl SystemImageVariant {
    /// Creates a new system image variant.
    pub fn new(variant: String, architecture: String, package_id: String) -> Self {
        let display_name = match variant.as_str() {
            "google_apis_playstore" => format!("Google Play ({})", architecture),
            "google_apis" => format!("Google APIs ({})", architecture),
            "default" => format!("Default ({})", architecture),
            _ => format!("{} ({})", variant, architecture),
        };

        Self {
            variant,
            architecture,
            package_id,
            is_installed: false,
            display_name,
        }
    }

    /// Returns the host system's preferred architecture for Android emulation.
    /// This is useful for UI display or debugging purposes.
    pub fn host_preferred_architecture() -> &'static str {
        ApiLevel::get_preferred_architecture()
    }
}
