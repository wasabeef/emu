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
        let display_name = format!("API {api} ({version})");
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
            "google_apis_playstore" => format!("Google Play Store ({architecture})"),
            "google_apis" => format!("Google APIs ({architecture})"),
            "default" => format!("Default ({architecture})"),
            _ => format!("{variant} ({architecture})"),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test ApiLevel::new()
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

    /// Test ApiLevel with variants
    #[test]
    fn test_api_level_with_variants() {
        let mut api_level = ApiLevel::new(
            33,
            "Android 13".to_string(),
            "system-images;android-33;google_apis;x86_64".to_string(),
        );

        let variant1 = SystemImageVariant::new(
            "google_apis".to_string(),
            "x86_64".to_string(),
            "system-images;android-33;google_apis;x86_64".to_string(),
        );

        let variant2 = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            "arm64-v8a".to_string(),
            "system-images;android-33;google_apis_playstore;arm64-v8a".to_string(),
        );

        api_level.variants.push(variant1);
        api_level.variants.push(variant2);

        assert_eq!(api_level.variants.len(), 2);
        assert_eq!(api_level.variants[0].variant, "google_apis");
        assert_eq!(api_level.variants[1].variant, "google_apis_playstore");
    }

    /// Test get_recommended_variant prioritization
    #[test]
    fn test_get_recommended_variant() {
        let mut api_level = ApiLevel::new(33, "Android 13".to_string(), "test".to_string());

        // Add variants in reverse priority order to test selection logic
        let default_variant = SystemImageVariant::new(
            "default".to_string(),
            "x86_64".to_string(),
            "system-images;android-33;default;x86_64".to_string(),
        );

        let google_apis_variant = SystemImageVariant::new(
            "google_apis".to_string(),
            "x86_64".to_string(),
            "system-images;android-33;google_apis;x86_64".to_string(),
        );

        let playstore_variant = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            "x86_64".to_string(),
            "system-images;android-33;google_apis_playstore;x86_64".to_string(),
        );

        // Test with only default variant
        api_level.variants.push(default_variant.clone());
        assert_eq!(
            api_level.get_recommended_variant().unwrap().variant,
            "default"
        );

        // Add google_apis - should be preferred over default
        api_level.variants.push(google_apis_variant.clone());
        assert_eq!(
            api_level.get_recommended_variant().unwrap().variant,
            "google_apis"
        );

        // Add playstore - should be preferred over google_apis
        api_level.variants.push(playstore_variant.clone());
        assert_eq!(
            api_level.get_recommended_variant().unwrap().variant,
            "google_apis_playstore"
        );
    }

    /// Test get_recommended_variant with empty variants
    #[test]
    fn test_get_recommended_variant_empty() {
        let api_level = ApiLevel::new(33, "Android 13".to_string(), "test".to_string());
        assert!(api_level.get_recommended_variant().is_none());
    }

    /// Test SystemImageVariant::new()
    #[test]
    fn test_system_image_variant_new() {
        let variant = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            "arm64-v8a".to_string(),
            "system-images;android-34;google_apis_playstore;arm64-v8a".to_string(),
        );

        assert_eq!(variant.variant, "google_apis_playstore");
        assert_eq!(variant.architecture, "arm64-v8a");
        assert_eq!(
            variant.package_id,
            "system-images;android-34;google_apis_playstore;arm64-v8a"
        );
        assert!(!variant.is_installed);
        assert_eq!(variant.display_name, "Google Play Store (arm64-v8a)");
    }

    /// Test SystemImageVariant display names
    #[test]
    fn test_system_image_variant_display_names() {
        let playstore = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            "x86_64".to_string(),
            "test".to_string(),
        );
        assert_eq!(playstore.display_name, "Google Play Store (x86_64)");

        let google_apis = SystemImageVariant::new(
            "google_apis".to_string(),
            "arm64-v8a".to_string(),
            "test".to_string(),
        );
        assert_eq!(google_apis.display_name, "Google APIs (arm64-v8a)");

        let default =
            SystemImageVariant::new("default".to_string(), "x86".to_string(), "test".to_string());
        assert_eq!(default.display_name, "Default (x86)");

        let custom = SystemImageVariant::new(
            "custom_variant".to_string(),
            "mips".to_string(),
            "test".to_string(),
        );
        assert_eq!(custom.display_name, "custom_variant (mips)");
    }

    /// Test InstallProgress
    #[test]
    fn test_install_progress() {
        let progress = InstallProgress {
            operation: "Downloading system image".to_string(),
            percentage: 45,
            eta_seconds: Some(120),
        };

        assert_eq!(progress.operation, "Downloading system image");
        assert_eq!(progress.percentage, 45);
        assert_eq!(progress.eta_seconds, Some(120));
    }

    /// Test preferred architecture detection
    #[test]
    fn test_preferred_architecture() {
        let arch = ApiLevel::get_preferred_architecture();

        // Should return one of the known architectures
        assert!(arch == "arm64-v8a" || arch == "x86_64");

        // Test that the function is consistent
        assert_eq!(arch, ApiLevel::get_preferred_architecture());

        // Test alias function
        assert_eq!(arch, SystemImageVariant::host_preferred_architecture());
    }

    /// Test serialization
    #[test]
    fn test_api_level_serialization() {
        let api_level = ApiLevel::new(34, "Android 14".to_string(), "test-id".to_string());

        let json = serde_json::to_string(&api_level).unwrap();
        let deserialized: ApiLevel = serde_json::from_str(&json).unwrap();

        assert_eq!(api_level.api, deserialized.api);
        assert_eq!(api_level.version, deserialized.version);
        assert_eq!(api_level.display_name, deserialized.display_name);
        assert_eq!(api_level.is_installed, deserialized.is_installed);
    }

    /// Test variant serialization
    #[test]
    fn test_system_image_variant_serialization() {
        let variant = SystemImageVariant::new(
            "google_apis".to_string(),
            "x86_64".to_string(),
            "test-package-id".to_string(),
        );

        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: SystemImageVariant = serde_json::from_str(&json).unwrap();

        assert_eq!(variant.variant, deserialized.variant);
        assert_eq!(variant.architecture, deserialized.architecture);
        assert_eq!(variant.package_id, deserialized.package_id);
        assert_eq!(variant.is_installed, deserialized.is_installed);
        assert_eq!(variant.display_name, deserialized.display_name);
    }

    /// Test architecture priority in selection
    #[test]
    fn test_architecture_priority() {
        let mut api_level = ApiLevel::new(33, "Android 13".to_string(), "test".to_string());
        let preferred_arch = ApiLevel::get_preferred_architecture();

        // Add non-preferred architecture first
        let non_preferred_arch = if preferred_arch == "arm64-v8a" {
            "x86_64"
        } else {
            "arm64-v8a"
        };

        let variant1 = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            non_preferred_arch.to_string(),
            "test1".to_string(),
        );

        let variant2 = SystemImageVariant::new(
            "google_apis_playstore".to_string(),
            preferred_arch.to_string(),
            "test2".to_string(),
        );

        api_level.variants.push(variant1);
        api_level.variants.push(variant2);

        // Should prefer native architecture
        let recommended = api_level.get_recommended_variant().unwrap();
        assert_eq!(recommended.architecture, preferred_arch);
    }
}
