//! iOS simulator management
//!
//! This module provides comprehensive iOS Simulator management by interfacing with Xcode's
//! `xcrun simctl` command-line tool. All device types, runtimes, and configurations are
//! discovered dynamically to ensure compatibility with new iOS versions and device types.
//!
//! # Key Features
//!
//! - **Dynamic Discovery**: Device types and runtimes discovered at runtime from Xcode
//! - **Smart Prioritization**: Devices sorted by model type and version automatically
//! - **Graceful Error Handling**: Handles already-booted and already-shutdown states
//! - **Cross-Platform Safety**: Compile-time stubs for non-macOS platforms

mod details;
mod discovery;
mod lifecycle;

#[cfg(target_os = "macos")]
use std::path::Path;

// # xcrun simctl Command Reference
//
// ## Device Listing (`xcrun simctl list devices --json`)
// ```json
// {
//   "devices": {
//     "com.apple.CoreSimulator.SimRuntime.iOS-17-0": [
//       {
//         "lastBootedAt": "2024-01-15T10:30:00Z",
//         "dataPath": "/Users/.../CoreSimulator/Devices/{UUID}/data",
//         "logPath": "/Users/.../CoreSimulator/Devices/{UUID}/device.log",
//         "udid": "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6",
//         "isAvailable": true,
//         "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
//         "state": "Booted",
//         "name": "iPhone 15"
//       }
//     ]
//   }
// }
// ```
//
// **Device States**:
// - `Booted`: Device is running
// - `Shutdown`: Device is stopped
// - `Creating`: Device is being created
// - `Booting`: Device is starting up
// - `Shutting Down`: Device is stopping
//
// ## Device Types (`xcrun simctl list devicetypes --json`)
// ```json
// {
//   "devicetypes": [
//     {
//       "minRuntimeVersion": 917504,
//       "bundlePath": "/Applications/Xcode.app/.../iPhone 15.simdevicetype",
//       "maxRuntimeVersion": 4294967295,
//       "name": "iPhone 15",
//       "identifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
//       "productFamily": "iPhone"
//     }
//   ]
// }
// ```
//
// **Device Type Naming**:
// - Identifier format: `com.apple.CoreSimulator.SimDeviceType.{Device-Name}`
// - Display names extracted from JSON `name` field or parsed from identifier
// - Special handling for sizes: "12.9-inch" → "12.9\""
//
// ## Runtimes (`xcrun simctl list runtimes --json`)
// ```json
// {
//   "runtimes": [
//     {
//       "bundlePath": "/Library/.../iOS 17.0.simruntime",
//       "buildversion": "21A342",
//       "platform": "iOS",
//       "runtimeRoot": "/Library/.../iOS 17.0.simruntime/Contents/Resources/RuntimeRoot",
//       "identifier": "com.apple.CoreSimulator.SimRuntime.iOS-17-0",
//       "version": "17.0",
//       "isInternal": false,
//       "isAvailable": true,
//       "name": "iOS 17.0",
//       "supportedDeviceTypes": [
//         "com.apple.CoreSimulator.SimDeviceType.iPhone-SE-3rd-generation",
//         "com.apple.CoreSimulator.SimDeviceType.iPhone-15"
//       ]
//     }
//   ]
// }
// ```
//
// ## Common Operations
//
// ### Create Device
// ```bash
// xcrun simctl create "My iPhone" com.apple.CoreSimulator.SimDeviceType.iPhone-15 com.apple.CoreSimulator.SimRuntime.iOS-17-0
// # Returns: UUID of created device
// ```
//
// ### Boot Device
// ```bash
// xcrun simctl boot {UUID}
// # Note: Returns error if already booted - handled gracefully
// ```
//
// ### Shutdown Device
// ```bash
// xcrun simctl shutdown {UUID}
// # Note: Returns error if already shutdown - handled gracefully
// ```
//
// ### Delete Device
// ```bash
// xcrun simctl delete {UUID}
// # Note: Automatically shuts down device first if needed
// ```
//
// ### Erase Device (Wipe)
// ```bash
// xcrun simctl erase {UUID}
// # Resets device to factory settings
// ```
//
// ## Device Priority System
//
// Devices are sorted with the following priority (lower number = higher priority):
//
// 1. **iPhone** (0-99):
//    - Pro Max: 0
//    - Pro: 10
//    - Plus/Max: 20
//    - Mini: 30
//    - SE: 40
//    - Regular (by version): 50 - version
//
// 2. **iPad** (100-199):
//    - Pro 12.9": 100
//    - Pro 11": 110
//    - Air: 130
//    - Mini: 140
//    - Regular: 150
//
// 3. **Apple TV** (200-299):
//    - 4K: 200
//    - HD: 210
//
// 4. **Apple Watch** (300-399):
//    - Ultra: 300
//    - Series (by version): 310 - series_number
//    - SE: 330
//
// ## Error Handling
//
// ### Common Errors and Solutions
// - "Unable to boot device in current state: Booted" → Device already running (ignored)
// - "Unable to shutdown device in current state: Shutdown" → Device already stopped (ignored)
// - "Invalid device type" → Check available types with `xcrun simctl list devicetypes`
// - "Invalid runtime" → Check available runtimes with `xcrun simctl list runtimes`
//
// ### Simulator App Integration
// - Boot operations attempt to open Simulator.app automatically
// - Failures to open Simulator.app are logged but don't fail the operation
// - Devices can run in "headless" mode without the Simulator app
// - Automatic cleanup: Simulator.app quits when last device stops
// - Graceful shutdown: Uses AppleScript with killall fallback
//
// ## Log Streaming
//
// iOS simulator logs can be streamed using multiple approaches:
//
// ### Method 1: Direct simulator spawn (most reliable)
// ```bash
// xcrun simctl spawn {UUID} log stream
// ```
//
// ### Method 2: System log filtering
// ```bash
// log stream --predicate 'processImagePath contains "Simulator"'
// ```
//
// ### Method 3: Console app logs
// ```bash
// log stream --style compact
// ```
//
// **Log Level Detection**:
// - Keywords "error" or "Error" → ERROR level
// - Keywords "warning" or "Warning" → WARN level
// - All other logs → INFO level
//
// ## Implementation Notes
//
// ### State Detection Optimization
// The `start_device` method pre-checks device state to avoid redundant boot commands,
// preventing unnecessary error messages and improving user experience.
//
// ### Cross-Platform Safety
// - All iOS-specific code is gated with `#[cfg(target_os = "macos")]`
// - Non-macOS platforms get stub implementations that return appropriate errors
// - The `which` crate is used to verify `xcrun` availability at runtime

#[cfg(target_os = "macos")]
use crate::constants::{
    commands::{SIMCTL, XCRUN},
    limits::{IOS_NAME_PARTS_MINIMUM, SINGLE_VERSION_PART},
    numeric::{VERSION_DEFAULT, VERSION_MINOR_DIVISOR, VERSION_PATCH_DIVISOR},
};
use crate::managers::common::{DeviceConfig, DeviceManager};
use crate::models::IosDevice;
#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::{bail, Result};

#[cfg(target_os = "macos")]
use crate::utils::command::CommandRunner;
#[cfg(target_os = "macos")]
use crate::utils::command_executor::CommandExecutor;
#[cfg(target_os = "macos")]
use std::sync::Arc;
#[cfg(target_os = "macos")]
use which;

#[cfg(target_os = "macos")]
/// Extract iOS version number from display string for sorting
///
/// Parses version numbers from runtime display names:
/// - "iOS 18.5" → 18.5
/// - "iOS 18.1" → 18.1
/// - "iOS 17.0" → 17.0
/// - "iOS 16.4" → 16.4
/// - "iOS 15.2.1" → 15.21 (patch version as decimal)
fn extract_ios_version(display_name: &str) -> f32 {
    // Extract version from strings like "iOS 17.0", "iOS 16.4", etc.
    if let Some(version_start) = display_name.find(char::is_numeric) {
        let version_str: String = display_name[version_start..]
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.')
            .collect();

        // Split version into parts
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() >= IOS_NAME_PARTS_MINIMUM {
            // Parse major.minor[.patch]
            let major = parts[0].parse::<f32>().unwrap_or(VERSION_DEFAULT);
            let minor = parts[1].parse::<f32>().unwrap_or(VERSION_DEFAULT) / VERSION_MINOR_DIVISOR;
            let patch = if parts.len() > 2 {
                parts[2].parse::<f32>().unwrap_or(VERSION_DEFAULT) / VERSION_PATCH_DIVISOR
            } else {
                VERSION_DEFAULT
            };
            major + minor + patch
        } else if parts.len() == SINGLE_VERSION_PART {
            // Just major version
            parts[0].parse::<f32>().unwrap_or(VERSION_DEFAULT)
        } else {
            VERSION_DEFAULT
        }
    } else {
        VERSION_DEFAULT
    }
}

#[cfg(target_os = "macos")]
/// iOS Simulator manager implementation for macOS.
///
/// This struct provides comprehensive management of iOS simulators through
/// Xcode's `xcrun simctl` command-line interface. It handles device discovery,
/// creation, lifecycle management, and status monitoring.
///
/// # Requirements
/// - macOS only (compile-time gated)
/// - Xcode or Xcode Command Line Tools installed
/// - `xcrun simctl` available in PATH
///
/// # Key Features
/// - Dynamic discovery of device types and runtimes
/// - Graceful handling of already-booted/shutdown states
/// - Automatic Simulator.app integration
/// - Real-time device status monitoring
#[derive(Clone)]
pub struct IosManager {
    /// Command executor for executing xcrun simctl commands (abstracted for testability)
    command_executor: Arc<dyn CommandExecutor>,
}

#[cfg(target_os = "macos")]
impl IosManager {
    // Inherent methods
    pub fn new() -> Result<Self> {
        Self::with_executor(Arc::new(CommandRunner::new()))
    }

    /// Creates a new IosManager instance with a custom command executor.
    /// This is primarily used for testing with mock executors.
    pub fn with_executor(executor: Arc<dyn CommandExecutor>) -> Result<Self> {
        // Quick check for Xcode Command Line Tools
        if which::which(XCRUN).is_err() {
            bail!("Xcode Command Line Tools not found. Please install Xcode or run 'xcode-select --install'.")
        }

        // Skip expensive simctl verification at startup - will be validated on first use
        // This improves startup performance significantly (saves ~30 seconds)
        Ok(Self {
            command_executor: executor,
        })
    }

    // These helper methods remain in the inherent impl block as they are specific to IosManager's way of handling things
    // and not directly part of the DeviceManager trait's public API contract for all managers.

    pub async fn erase_device(&self, udid: &str) -> Result<()> {
        self.command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "erase", udid])
            .await
            .context(format!("Failed to erase iOS device {udid}"))?;
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl DeviceManager for IosManager {
    type Device = IosDevice;

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        self.list_devices_internal().await
    }

    async fn start_device(&self, identifier: &str) -> Result<()> {
        self.start_device_internal(identifier).await
    }

    async fn stop_device(&self, identifier: &str) -> Result<()> {
        self.stop_device_internal(identifier).await
    }

    async fn create_device(&self, config: &DeviceConfig) -> Result<()> {
        self.create_device_internal(config).await
    }

    async fn delete_device(&self, identifier: &str) -> Result<()> {
        self.delete_device_internal(identifier).await
    }

    async fn wipe_device(&self, identifier: &str) -> Result<()> {
        self.wipe_device_internal(identifier).await
    }

    async fn is_available(&self) -> bool {
        self.is_available_internal().await
    }
}

/// Implementation of UnifiedDeviceManager for IosManager (macOS)
#[cfg(target_os = "macos")]
#[async_trait::async_trait]
impl crate::managers::common::UnifiedDeviceManager for IosManager {
    async fn list_devices(&self) -> Result<Vec<Box<dyn crate::models::device::Device>>> {
        let devices = <Self as DeviceManager>::list_devices(self).await?;
        Ok(devices
            .into_iter()
            .map(|d| Box::new(d) as Box<dyn crate::models::device::Device>)
            .collect())
    }

    async fn start_device(&self, device_id: &str) -> Result<()> {
        <Self as DeviceManager>::start_device(self, device_id).await
    }

    async fn stop_device(&self, device_id: &str) -> Result<()> {
        <Self as DeviceManager>::stop_device(self, device_id).await
    }

    async fn create_device(&self, config: &crate::managers::common::DeviceConfig) -> Result<()> {
        <Self as DeviceManager>::create_device(self, config).await
    }

    async fn delete_device(&self, device_id: &str) -> Result<()> {
        <Self as DeviceManager>::delete_device(self, device_id).await
    }

    async fn wipe_device(&self, device_id: &str) -> Result<()> {
        <Self as DeviceManager>::wipe_device(self, device_id).await
    }

    async fn is_available(&self) -> bool {
        <Self as DeviceManager>::is_available(self).await
    }
}

// Stub implementation for non-macOS platforms
#[cfg(not(target_os = "macos"))]
/// iOS Simulator manager stub for non-macOS platforms.
///
/// This is a placeholder implementation that returns appropriate errors
/// when iOS simulator operations are attempted on non-macOS systems.
/// All methods return errors indicating that iOS simulator management
/// is only available on macOS.
#[derive(Clone)]
pub struct IosManager;

#[cfg(not(target_os = "macos"))]
impl IosManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self) // Allow creation, but is_available will be false
    }

    pub async fn list_device_types_with_names(&self) -> Result<Vec<(String, String)>> {
        bail!("iOS simulator management is only available on macOS")
    }

    pub async fn list_runtimes(&self) -> Result<Vec<(String, String)>> {
        bail!("iOS simulator management is only available on macOS")
    }

    pub async fn get_device_details(&self, _udid: &str) -> Result<crate::models::DeviceDetails> {
        bail!("iOS simulator management is only available on macOS")
    }
}

#[cfg(not(target_os = "macos"))]
impl DeviceManager for IosManager {
    type Device = IosDevice; // This will use the potentially simplified IosDevice from models.rs for non-macOS

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn start_device(&self, _identifier: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn stop_device(&self, _identifier: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn create_device(&self, _config: &DeviceConfig) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn delete_device(&self, _identifier: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn wipe_device(&self, _identifier: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn is_available(&self) -> bool {
        false // Not available on non-macOS
    }
}

/// Implementation of UnifiedDeviceManager for IosManager (non-macOS)
#[cfg(not(target_os = "macos"))]
#[async_trait::async_trait]
impl crate::managers::common::UnifiedDeviceManager for IosManager {
    async fn list_devices(&self) -> Result<Vec<Box<dyn crate::models::device::Device>>> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn start_device(&self, _device_id: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn stop_device(&self, _device_id: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn create_device(&self, _config: &crate::managers::common::DeviceConfig) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn delete_device(&self, _device_id: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn wipe_device(&self, _device_id: &str) -> Result<()> {
        bail!("iOS simulator management is only available on macOS")
    }

    async fn is_available(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
