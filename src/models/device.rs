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
