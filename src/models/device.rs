//! Device model definitions for Android and iOS virtual devices.
//!
//! This module contains the core data structures representing virtual devices
//! in the application. Each platform has its own device type with platform-specific
//! fields, while sharing common status enumerations.

use crate::constants::{android::DEFAULT_STORAGE_FALLBACK, defaults::DEFAULT_RAM_MB};
use serde::{Deserialize, Serialize};

/// Common interface for all device types.
///
/// This trait provides a unified interface for accessing device properties
/// across different platforms (Android and iOS).
pub trait Device: Send + Sync + std::fmt::Debug {
    /// Returns the unique identifier for the device
    fn id(&self) -> &str;

    /// Returns the display name of the device
    fn name(&self) -> &str;

    /// Returns the current status of the device
    fn status(&self) -> &DeviceStatus;

    /// Returns whether the device is currently running
    fn is_running(&self) -> bool;
}

/// Represents an Android Virtual Device (AVD).
///
/// Contains all information needed to display and manage an Android emulator
/// instance, including its configuration and current runtime status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDevice {
    /// AVD name (unique identifier)
    pub name: String,
    /// Device type identifier (e.g., "pixel_7", "tv_1080p")
    pub device_type: String,
    /// Android API level (e.g., 34 for Android 14)
    pub api_level: u32,
    /// Current device status
    pub status: DeviceStatus,
    /// Whether the emulator is currently running
    pub is_running: bool,
    /// RAM allocation in MB (e.g., "2048")
    pub ram_size: String,
    /// Storage size (e.g., "8192M", "4G")
    pub storage_size: String,
}

/// Represents an iOS Simulator device.
///
/// Contains all information needed to display and manage an iOS simulator
/// instance, including its unique identifier and runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosDevice {
    /// Display name of the simulator
    pub name: String,
    /// Unique device identifier (UUID)
    pub udid: String,
    /// Device type (e.g., "iPhone 15", "iPad Pro")
    pub device_type: String,
    /// iOS version number (e.g., "17.0")
    pub ios_version: String,
    /// Full runtime version string (e.g., "iOS 17.0")
    pub runtime_version: String,
    /// Current device status
    pub status: DeviceStatus,
    /// Whether the simulator is currently booted
    pub is_running: bool,
    /// Whether the device is available for use (not corrupted)
    pub is_available: bool,
}

/// Represents the current operational state of a virtual device.
///
/// Used by both Android and iOS devices to indicate their current status
/// in a platform-agnostic way.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    /// Device is running and ready for use
    Running,
    /// Device is stopped/shutdown
    Stopped,
    /// Device is in the process of starting
    Starting,
    /// Device is in the process of stopping
    Stopping,
    /// Device is being created
    Creating,
    /// Device is in an error state
    Error,
    /// Device status cannot be determined
    Unknown,
}

impl Device for AndroidDevice {
    fn id(&self) -> &str {
        &self.name
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> &DeviceStatus {
        &self.status
    }

    fn is_running(&self) -> bool {
        self.is_running
    }
}

impl Device for IosDevice {
    fn id(&self) -> &str {
        &self.udid
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> &DeviceStatus {
        &self.status
    }

    fn is_running(&self) -> bool {
        self.is_running
    }
}

impl Default for AndroidDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_type: String::new(),
            api_level: 0,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: DEFAULT_RAM_MB.to_string(),
            storage_size: DEFAULT_STORAGE_FALLBACK.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android_device_trait_impl() {
        let device = AndroidDevice {
            name: "test_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };

        assert_eq!(device.id(), "test_device");
        assert_eq!(device.name(), "test_device");
        assert_eq!(*device.status(), DeviceStatus::Running);
        assert!(device.is_running());
    }

    #[test]
    fn test_ios_device_trait_impl() {
        let device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "ABC123".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        assert_eq!(device.id(), "ABC123");
        assert_eq!(device.name(), "iPhone 15");
        assert_eq!(*device.status(), DeviceStatus::Stopped);
        assert!(!device.is_running());
    }

    #[test]
    fn test_device_status_equality() {
        assert_eq!(DeviceStatus::Running, DeviceStatus::Running);
        assert_ne!(DeviceStatus::Running, DeviceStatus::Stopped);
        assert_eq!(DeviceStatus::Unknown, DeviceStatus::Unknown);
    }

    #[test]
    fn test_android_device_default() {
        let device = AndroidDevice::default();
        assert_eq!(device.name, "");
        assert_eq!(device.device_type, "");
        assert_eq!(device.api_level, 0);
        assert_eq!(device.status, DeviceStatus::Stopped);
        assert!(!device.is_running);
        assert_eq!(device.ram_size, DEFAULT_RAM_MB.to_string());
        assert_eq!(device.storage_size, DEFAULT_STORAGE_FALLBACK);
    }

    #[test]
    fn test_device_status_display() {
        // Test all DeviceStatus variants
        let statuses = [
            DeviceStatus::Running,
            DeviceStatus::Stopped,
            DeviceStatus::Starting,
            DeviceStatus::Stopping,
            DeviceStatus::Creating,
            DeviceStatus::Error,
            DeviceStatus::Unknown,
        ];

        // Debug formatting should work for all statuses
        for status in &statuses {
            let debug_str = format!("{status:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_device_status_serialization() {
        use serde_json;

        // Test that DeviceStatus can be serialized/deserialized
        let status = DeviceStatus::Running;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: DeviceStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);

        // Test all status variants
        let statuses = [
            DeviceStatus::Running,
            DeviceStatus::Stopped,
            DeviceStatus::Starting,
            DeviceStatus::Stopping,
            DeviceStatus::Creating,
            DeviceStatus::Error,
            DeviceStatus::Unknown,
        ];

        for original_status in &statuses {
            let json = serde_json::to_string(original_status).unwrap();
            let parsed: DeviceStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*original_status, parsed);
        }
    }

    #[test]
    fn test_android_device_serialization() {
        use serde_json;

        let device = AndroidDevice {
            name: "test_device".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };

        let json = serde_json::to_string(&device).unwrap();
        let parsed: AndroidDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(device.name, parsed.name);
        assert_eq!(device.device_type, parsed.device_type);
        assert_eq!(device.api_level, parsed.api_level);
        assert_eq!(device.status, parsed.status);
        assert_eq!(device.is_running, parsed.is_running);
        assert_eq!(device.ram_size, parsed.ram_size);
        assert_eq!(device.storage_size, parsed.storage_size);
    }

    #[test]
    fn test_ios_device_serialization() {
        use serde_json;

        let device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "ABC123-DEF456-GHI789".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        let json = serde_json::to_string(&device).unwrap();
        let parsed: IosDevice = serde_json::from_str(&json).unwrap();

        assert_eq!(device.name, parsed.name);
        assert_eq!(device.udid, parsed.udid);
        assert_eq!(device.device_type, parsed.device_type);
        assert_eq!(device.ios_version, parsed.ios_version);
        assert_eq!(device.runtime_version, parsed.runtime_version);
        assert_eq!(device.status, parsed.status);
        assert_eq!(device.is_running, parsed.is_running);
        assert_eq!(device.is_available, parsed.is_available);
    }

    #[test]
    fn test_android_device_clone() {
        let device = AndroidDevice {
            name: "original".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };

        let cloned = device.clone();

        assert_eq!(device.name, cloned.name);
        assert_eq!(device.device_type, cloned.device_type);
        assert_eq!(device.api_level, cloned.api_level);
        assert_eq!(device.status, cloned.status);
        assert_eq!(device.is_running, cloned.is_running);
        assert_eq!(device.ram_size, cloned.ram_size);
        assert_eq!(device.storage_size, cloned.storage_size);

        // Ensure they are independent objects
        assert_ne!(&device.name as *const String, &cloned.name as *const String);
    }

    #[test]
    fn test_ios_device_clone() {
        let device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "ABC123-DEF456-GHI789".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        let cloned = device.clone();

        assert_eq!(device.name, cloned.name);
        assert_eq!(device.udid, cloned.udid);
        assert_eq!(device.device_type, cloned.device_type);
        assert_eq!(device.ios_version, cloned.ios_version);
        assert_eq!(device.runtime_version, cloned.runtime_version);
        assert_eq!(device.status, cloned.status);
        assert_eq!(device.is_running, cloned.is_running);
        assert_eq!(device.is_available, cloned.is_available);

        // Ensure they are independent objects
        assert_ne!(&device.udid as *const String, &cloned.udid as *const String);
    }

    #[test]
    fn test_device_debug_formatting() {
        let android_device = AndroidDevice {
            name: "test_android".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };

        let debug_str = format!("{android_device:?}");
        assert!(debug_str.contains("test_android"));
        assert!(debug_str.contains("pixel_7"));
        assert!(debug_str.contains("34"));
        assert!(debug_str.contains("Running"));

        let ios_device = IosDevice {
            name: "iPhone 15".to_string(),
            udid: "ABC123".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        let debug_str = format!("{ios_device:?}");
        assert!(debug_str.contains("iPhone 15"));
        assert!(debug_str.contains("ABC123"));
        assert!(debug_str.contains("17.0"));
        assert!(debug_str.contains("Stopped"));
    }

    #[test]
    fn test_device_trait_consistency() {
        let android_device = AndroidDevice {
            name: "android_test".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 34,
            status: DeviceStatus::Running,
            is_running: true,
            ram_size: "4096".to_string(),
            storage_size: "8192".to_string(),
        };

        let ios_device = IosDevice {
            name: "iOS Test".to_string(),
            udid: "iOS-123".to_string(),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        };

        // Test that both devices implement the Device trait consistently
        let android_as_device: &dyn Device = &android_device;
        let ios_as_device: &dyn Device = &ios_device;

        // Android device ID should be the name
        assert_eq!(android_as_device.id(), "android_test");
        assert_eq!(android_as_device.name(), "android_test");

        // iOS device ID should be the UDID
        assert_eq!(ios_as_device.id(), "iOS-123");
        assert_eq!(ios_as_device.name(), "iOS Test");

        // Status and running state should match
        assert_eq!(*android_as_device.status(), DeviceStatus::Running);
        assert!(android_as_device.is_running());

        assert_eq!(*ios_as_device.status(), DeviceStatus::Stopped);
        assert!(!ios_as_device.is_running());
    }

    #[test]
    fn test_device_status_copy_semantics() {
        let status1 = DeviceStatus::Running;
        let status2 = status1; // Copy, not move

        // Both should be usable after copy
        assert_eq!(status1, DeviceStatus::Running);
        assert_eq!(status2, DeviceStatus::Running);
        assert_eq!(status1, status2);

        // Test with all variants
        let statuses = [
            DeviceStatus::Running,
            DeviceStatus::Stopped,
            DeviceStatus::Starting,
            DeviceStatus::Stopping,
            DeviceStatus::Creating,
            DeviceStatus::Error,
            DeviceStatus::Unknown,
        ];

        for original in &statuses {
            let copied = *original;
            assert_eq!(*original, copied);
        }
    }

    #[test]
    fn test_device_with_edge_case_values() {
        // Test Android device with edge case values
        let android = AndroidDevice {
            name: "".to_string(), // Empty name
            device_type: "very_long_device_type_name_that_might_cause_issues".to_string(),
            api_level: 0, // Minimum API level
            status: DeviceStatus::Unknown,
            is_running: false,
            ram_size: "0".to_string(),
            storage_size: "".to_string(),
        };

        assert_eq!(android.id(), "");
        assert_eq!(android.name(), "");
        assert!(!android.is_running());

        // Test iOS device with edge case values
        let ios = IosDevice {
            name: "📱 Emoji Device".to_string(), // Unicode characters
            udid: "".to_string(),                // Empty UDID
            device_type: "Unknown Device Type".to_string(),
            ios_version: "0.0".to_string(),
            runtime_version: "".to_string(),
            status: DeviceStatus::Error,
            is_running: true, // Contradictory state
            is_available: false,
        };

        assert_eq!(ios.id(), "");
        assert_eq!(ios.name(), "📱 Emoji Device");
        assert!(ios.is_running()); // Should respect is_running field
        assert_eq!(*ios.status(), DeviceStatus::Error);
    }

    #[test]
    fn test_device_status_inequalities() {
        // Test comprehensive inequality checks
        let running = DeviceStatus::Running;
        let stopped = DeviceStatus::Stopped;
        let starting = DeviceStatus::Starting;
        let stopping = DeviceStatus::Stopping;
        let creating = DeviceStatus::Creating;
        let error = DeviceStatus::Error;
        let unknown = DeviceStatus::Unknown;

        // Test that different statuses are not equal
        assert_ne!(running, stopped);
        assert_ne!(running, starting);
        assert_ne!(running, stopping);
        assert_ne!(running, creating);
        assert_ne!(running, error);
        assert_ne!(running, unknown);

        assert_ne!(stopped, starting);
        assert_ne!(stopped, stopping);
        assert_ne!(stopped, creating);
        assert_ne!(stopped, error);
        assert_ne!(stopped, unknown);

        // Test self-equality (should pass but ensures Copy trait works)
        assert_eq!(running, running);
        assert_eq!(stopped, stopped);
        assert_eq!(unknown, unknown);
    }
}
