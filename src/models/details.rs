//! Shared device detail model.
//!
//! This module contains the cross-platform representation used to render
//! detailed device information in the UI. It intentionally depends only on
//! shared model types so lower layers do not need to import application state.

use crate::models::Platform;

/// Detailed information about a device.
/// Used in the device details panel to show comprehensive device information.
#[derive(Debug, Clone)]
pub struct DeviceDetails {
    /// Device display name
    pub name: String,
    /// Current status (Running/Stopped/Booted/Shutdown)
    pub status: String,
    /// Platform the device belongs to
    pub platform: Platform,
    /// Device type (e.g., "Pixel 4", "iPhone 15")
    pub device_type: String,
    /// API level (Android) or iOS version
    pub api_level_or_version: String,
    /// RAM size in MB (Android only)
    pub ram_size: Option<String>,
    /// Storage size in MB (Android only)
    pub storage_size: Option<String>,
    /// Screen resolution (e.g., "1080x1920")
    pub resolution: Option<String>,
    /// DPI value (Android) or scale factor (iOS)
    pub dpi: Option<String>,
    /// Full path to device files on disk
    pub device_path: Option<String>,
    /// System image path or identifier (Android only)
    pub system_image: Option<String>,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub identifier: String,
}
