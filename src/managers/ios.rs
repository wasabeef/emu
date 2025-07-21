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
use crate::constants::ios::{
    IOS_ALREADY_BOOTED_ERROR, IOS_ALREADY_SHUTDOWN_ERROR, IOS_DEVICE_STATUS_BOOTED,
    IOS_DEVICE_STATUS_CREATING, IOS_DEVICE_STATUS_SHUTDOWN, IOS_DEVICE_TYPE_PREFIX,
    IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT, IOS_RUNTIME_PREFIX, SIMULATOR_APP_NAME,
    SIMULATOR_OPEN_FLAG, SIMULATOR_QUIT_COMMAND,
};
#[cfg(target_os = "macos")]
use crate::constants::{
    commands::{KILLALL, OSASCRIPT, SIMCTL, XCRUN},
    defaults::UNKNOWN_VALUE,
    ios_devices::{
        DEVICE_KEYWORD_AIR, DEVICE_KEYWORD_IPAD, DEVICE_KEYWORD_IPHONE, DEVICE_KEYWORD_MINI,
        DEVICE_KEYWORD_PLUS, DEVICE_KEYWORD_PRO, DEVICE_KEYWORD_PRO_MAX, DEVICE_KEYWORD_SE,
        DEVICE_SIZE_11, DEVICE_SIZE_12_9, DEVICE_VERSION_13, DEVICE_VERSION_14, DEVICE_VERSION_15,
        DEVICE_VERSION_16, *,
    },
    limits::{IOS_NAME_PARTS_MINIMUM, SINGLE_VERSION_PART},
    numeric::{
        BYTES_PER_MB, IOS_DEVICE_PARSE_BATCH_SIZE, VERSION_DEFAULT, VERSION_MINOR_DIVISOR,
        VERSION_PATCH_DIVISOR,
    },
    patterns::text_patterns::{
        APPLE_DEVICE_IPAD, APPLE_DEVICE_IPHONE, APPLE_DEVICE_IPOD, APPLE_DEVICE_PREFIX_I,
        CHIP_PREFIX_A, CHIP_PREFIX_M, INCH_INDICATOR, MEMORY_CLOSE_BRACKET, MEMORY_OPEN_BRACKET,
    },
    resolutions::*,
};
use crate::managers::common::{DeviceConfig, DeviceManager};
#[cfg(target_os = "macos")]
use crate::models::device_info::DynamicDeviceConfig;
#[cfg(target_os = "macos")]
use crate::models::DeviceStatus;
use crate::models::IosDevice;
#[cfg(target_os = "macos")]
use anyhow::Context;
use anyhow::{bail, Result};

#[cfg(target_os = "macos")]
use crate::utils::command::CommandRunner;
#[cfg(target_os = "macos")]
use crate::utils::command_executor::CommandExecutor;
#[cfg(target_os = "macos")]
use log;
#[cfg(target_os = "macos")]
use serde_json::Value;
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
        if which::which(XCRUN).is_err() {
            bail!("Xcode Command Line Tools not found. Please install Xcode or run 'xcode-select --install'.")
        }

        // Verify simctl is available
        if let Err(e) = std::process::Command::new(XCRUN)
            .args([SIMCTL, "help"])
            .output()
        {
            bail!(
                "Failed to access iOS Simulator: {}. Make sure Xcode is properly installed.",
                e
            )
        }

        Ok(Self {
            command_executor: executor,
        })
    }

    fn parse_device_from_json(
        &self,
        device_json: &Value,
        runtime_str: &str,
    ) -> Result<Option<IosDevice>> {
        let device_name = device_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(UNKNOWN_VALUE);
        let udid = device_json
            .get("udid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if udid.is_empty() {
            return Ok(None);
        }

        let state_str = device_json
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or(UNKNOWN_VALUE);
        let is_available_json = device_json
            .get("isAvailable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let device_type_identifier = device_json
            .get("deviceTypeIdentifier")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ios_version_str = runtime_str
            .replace(IOS_RUNTIME_PREFIX, "")
            .replace("-", ".");

        // Format device name with iOS version for clarity
        let ios_version_display = ios_version_str.replace("iOS.", "");
        let name = format!("{device_name} (iOS {ios_version_display})");

        let status = match state_str {
            IOS_DEVICE_STATUS_BOOTED => DeviceStatus::Running,
            IOS_DEVICE_STATUS_SHUTDOWN => DeviceStatus::Stopped,
            IOS_DEVICE_STATUS_CREATING => DeviceStatus::Creating,
            _ => DeviceStatus::Unknown,
        };
        let is_running_bool = state_str == IOS_DEVICE_STATUS_BOOTED;

        Ok(Some(IosDevice {
            name,
            udid,
            device_type: device_type_identifier,
            ios_version: ios_version_str.clone(),
            runtime_version: ios_version_str,
            status,
            is_running: is_running_bool,
            is_available: is_available_json,
        }))
    }

    // These helper methods remain in the inherent impl block as they are specific to IosManager's way of handling things
    // and not directly part of the DeviceManager trait's public API contract for all managers.

    /// Get detailed device information for an iOS simulator
    pub async fn get_device_details(&self, udid: &str) -> Result<crate::app::state::DeviceDetails> {
        // Get device information
        let device_output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devices", "-j"])
            .await
            .context("Failed to get device list")?;

        let json: Value =
            serde_json::from_str(&device_output).context("Failed to parse device JSON")?;

        let mut device_details = None;

        // Find the specific device
        if let Some(devices) = json.get("devices").and_then(|v| v.as_object()) {
            for (runtime, device_list) in devices {
                if let Some(devices_array) = device_list.as_array() {
                    for device in devices_array {
                        if let Some(device_udid) = device.get("udid").and_then(|v| v.as_str()) {
                            if device_udid == udid {
                                // Extract device information
                                let name = device
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(UNKNOWN_VALUE)
                                    .to_string();

                                let state = device
                                    .get("state")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(UNKNOWN_VALUE)
                                    .to_string();

                                // Extract runtime version
                                let version = runtime
                                    .replace("com.apple.CoreSimulator.SimRuntime.iOS-", "")
                                    .replace("-", ".");

                                let device_type = device
                                    .get("deviceTypeIdentifier")
                                    .and_then(|v| v.as_str())
                                    .map(Self::parse_device_type_display_name)
                                    .unwrap_or_else(|| "Unknown".to_string());

                                // Storage information
                                let storage_size = device
                                    .get("dataPathSize")
                                    .and_then(|v| v.as_u64())
                                    .map(|size| format!("{} MB", size / BYTES_PER_MB));

                                // Paths
                                let device_path = device
                                    .get("dataPath")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());

                                // Get screen resolution from device type if possible
                                let resolution = self.get_device_resolution(&device_type);

                                device_details = Some(crate::app::state::DeviceDetails {
                                    name: name.clone(),
                                    status: state,
                                    platform: crate::app::state::Panel::Ios,
                                    device_type,
                                    api_level_or_version: format!("iOS {version}"),
                                    ram_size: None, // iOS simulators don't have configurable RAM
                                    storage_size,
                                    resolution,
                                    dpi: Some(RETINA_DISPLAY.to_string()), // iOS uses Retina display
                                    device_path,
                                    system_image: None, // Not applicable for iOS
                                    identifier: udid.to_string(),
                                });

                                break;
                            }
                        }
                    }
                    if device_details.is_some() {
                        break;
                    }
                }
            }
        }

        device_details.ok_or_else(|| anyhow::anyhow!("Device with UDID {} not found", udid))
    }

    /// Get approximate resolution for known iOS device types
    fn get_device_resolution(&self, device_type: &str) -> Option<String> {
        let device_lower = device_type.to_lowercase();

        // iPhone resolutions
        if device_lower.contains(DEVICE_KEYWORD_IPHONE) {
            if device_lower.contains(DEVICE_VERSION_16)
                || device_lower.contains(DEVICE_VERSION_15)
                || device_lower.contains(DEVICE_VERSION_14)
            {
                if device_lower.contains(DEVICE_KEYWORD_PRO_MAX) {
                    return Some(IPHONE_15_PRO_MAX_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_KEYWORD_PRO) {
                    return Some(IPHONE_15_PRO_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_KEYWORD_PLUS) {
                    return Some(IPHONE_15_PRO_MAX_RESOLUTION.to_string());
                } else {
                    return Some(IPHONE_15_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_SE) {
                return Some(IPHONE_SE_RESOLUTION.to_string());
            }
        }

        // iPad resolutions
        if device_lower.contains(DEVICE_KEYWORD_IPAD) {
            if device_lower.contains(DEVICE_KEYWORD_PRO) {
                if device_lower.contains(DEVICE_VERSION_13)
                    || device_lower.contains(DEVICE_SIZE_12_9)
                {
                    return Some(IPAD_PRO_12_9_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_SIZE_11) {
                    return Some(IPAD_PRO_11_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_AIR) {
                if device_lower.contains(DEVICE_VERSION_13) {
                    return Some(IPAD_AIR_13_RESOLUTION.to_string());
                } else {
                    return Some(IPAD_AIR_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_MINI) {
                return Some(IPAD_MINI_RESOLUTION.to_string());
            } else {
                return Some(IPAD_RESOLUTION.to_string()); // Regular iPad
            }
        }

        None
    }

    pub async fn erase_device(&self, udid: &str) -> Result<()> {
        self.command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "erase", udid])
            .await
            .context(format!("Failed to erase iOS device {udid}"))?;
        Ok(())
    }

    pub async fn list_device_types(&self) -> Result<Vec<String>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devicetypes", "--json"])
            .await
            .context("Failed to list device types")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse device types JSON")?;
        let mut device_types = Vec::new();
        if let Some(types_array) = json.get("devicetypes").and_then(|v| v.as_array()) {
            for device_type_json in types_array {
                if let Some(identifier) =
                    device_type_json.get("identifier").and_then(|v| v.as_str())
                {
                    device_types.push(identifier.to_string());
                }
            }
        }
        Ok(device_types)
    }

    /// Get device types with display names (similar to Android's approach)
    pub async fn list_device_types_with_names(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devicetypes", "--json"])
            .await
            .context("Failed to list device types")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse device types JSON")?;
        let mut device_types = Vec::new();

        if let Some(types_array) = json.get("devicetypes").and_then(|v| v.as_array()) {
            for device_type_json in types_array {
                if let Some(identifier) =
                    device_type_json.get("identifier").and_then(|v| v.as_str())
                {
                    // Try to get the name from JSON first
                    let display_name =
                        if let Some(name) = device_type_json.get("name").and_then(|v| v.as_str()) {
                            name.to_string()
                        } else {
                            // Fallback: parse from identifier
                            // com.apple.CoreSimulator.SimDeviceType.iPhone-15 -> iPhone 15
                            Self::parse_device_type_display_name(identifier)
                        };

                    device_types.push((identifier.to_string(), display_name));
                }
            }
        }

        // Sort devices by priority (similar to Android)
        device_types.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_ios_device_priority(&a.1);
            let priority_b = DynamicDeviceConfig::calculate_ios_device_priority(&b.1);
            priority_a.cmp(&priority_b)
        });

        Ok(device_types)
    }

    /// Parse device type identifier to display name
    ///
    /// Converts identifiers like "com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro"
    /// into human-readable names like "iPhone 15 Pro"
    fn parse_device_type_display_name(identifier: &str) -> String {
        // com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro -> iPhone 15 Pro
        let cleaned = identifier
            .replace(IOS_DEVICE_TYPE_PREFIX, "")
            .replace("-", " ")
            .replace("_", " ");

        // Special case handling for inch sizes and technical specifications
        let mut display = cleaned.replace(IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT); // 12.9 inch -> 12.9"

        // Handle additional inch sizes that might not be covered by the pattern
        display = display.replace(INCH_13_PATTERN, INCH_13_REPLACEMENT);
        display = display.replace(INCH_11_PATTERN, INCH_11_REPLACEMENT);

        // Handle memory specifications
        display = display.replace(MEMORY_8GB_PATTERN, MEMORY_8GB_REPLACEMENT);
        display = display.replace(MEMORY_16GB_PATTERN, MEMORY_16GB_REPLACEMENT);

        // Capitalize properly with enhanced special case handling
        display
            .split_whitespace()
            .map(|word| {
                let word_lower = word.to_lowercase();

                // Preserve special cases exactly as they should appear
                if word_lower == "inch"
                    || word_lower == "se"
                    || word_lower == "mini"
                    || word_lower == "max"
                    || word_lower == "plus"
                    || word_lower == "pro"
                    || word_lower == "air"
                    || word_lower == "ultra"
                    || word.contains(INCH_INDICATOR) // Inch measurements like 12.9"
                    || word.contains(MEMORY_OPEN_BRACKET) || word.contains(MEMORY_CLOSE_BRACKET) // Memory specs like (8GB)
                    || (word.starts_with(APPLE_DEVICE_PREFIX_I) &&
                        (word.starts_with(APPLE_DEVICE_IPHONE) || word.starts_with(APPLE_DEVICE_IPAD) || word.starts_with(APPLE_DEVICE_IPOD)))
                    || word_lower.starts_with(CHIP_PREFIX_M) && word.len() <= 3 // M4, M3, M2, M1 chips
                    || word_lower.starts_with(CHIP_PREFIX_A) && word.chars().nth(1).is_some_and(|c| c.is_ascii_digit()) // A17, A16 chips
                {
                    // For chip names, ensure proper capitalization
                    if word_lower.starts_with(CHIP_PREFIX_M) && word.len() <= 3 {
                        return word.to_uppercase(); // m4 -> M4
                    } else if word_lower.starts_with(CHIP_PREFIX_A) && word.chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
                        return word.to_uppercase(); // a17 -> A17
                    }

                    word.to_string() // Preserve these special cases as-is
                } else {
                    // Capitalize first letter for regular words
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub async fn list_runtimes(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "runtimes", "--json"])
            .await
            .context("Failed to list runtimes")?;
        let json: Value = serde_json::from_str(&output).context("Failed to parse runtimes JSON")?;
        let mut runtimes = Vec::new();
        if let Some(runtimes_array) = json.get("runtimes").and_then(|v| v.as_array()) {
            for runtime_json in runtimes_array {
                if let Some(identifier) = runtime_json.get("identifier").and_then(|v| v.as_str()) {
                    if runtime_json
                        .get("isAvailable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        // Extract display name from various possible fields
                        let display_name =
                            if let Some(name) = runtime_json.get("name").and_then(|v| v.as_str()) {
                                // Use the name field if available
                                name.to_string()
                            } else if let Some(version) =
                                runtime_json.get("version").and_then(|v| v.as_str())
                            {
                                // Use version if name is not available
                                format!("iOS {version}")
                            } else {
                                // Fallback: parse from identifier
                                // com.apple.CoreSimulator.SimRuntime.iOS-17-0 -> iOS 17.0
                                identifier
                                    .replace("com.apple.CoreSimulator.SimRuntime.", "")
                                    .replace("-", ".")
                                    .replace("iOS.", "iOS ")
                            };

                        runtimes.push((identifier.to_string(), display_name));
                    }
                }
            }
        }

        // Sort by version (newest first) - similar to Android's approach
        runtimes.sort_by(|a, b| {
            // Extract version numbers for sorting
            let version_a = extract_ios_version(&a.1);
            let version_b = extract_ios_version(&b.1);
            version_b
                .partial_cmp(&version_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(runtimes)
    }

    /// Helper method to quit Simulator.app if no devices are running
    ///
    /// This method checks if any iOS devices are still running after a shutdown operation.
    /// If no devices are running, it attempts to quit Simulator.app gracefully using
    /// AppleScript. If that fails, it falls back to using `killall` to force quit.
    ///
    /// This prevents the Simulator.app icon from lingering in the Dock when all
    /// devices have been stopped, providing a cleaner user experience.
    ///
    /// # Implementation Details
    ///
    /// 1. **Device Status Check**: Queries all devices to see if any are running
    /// 2. **Graceful Quit**: Attempts `osascript -e "tell application \"Simulator\" to quit"`
    /// 3. **Force Quit Fallback**: Uses `killall Simulator` if graceful quit fails
    /// 4. **Error Handling**: Logs warnings but doesn't fail the operation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Called automatically after device shutdown
    /// self.quit_simulator_if_no_running_devices().await;
    /// ```
    async fn quit_simulator_if_no_running_devices(&self) {
        // Check if there are any running devices
        match self.list_devices().await {
            Ok(devices) => {
                let has_running_devices = devices.iter().any(|device| device.is_running);

                if !has_running_devices {
                    log::info!("No iOS devices are running, quitting Simulator.app");

                    // Quit Simulator.app gracefully
                    if let Err(e) = self
                        .command_executor
                        .run(Path::new(OSASCRIPT), &["-e", SIMULATOR_QUIT_COMMAND])
                        .await
                    {
                        log::warn!("Failed to quit Simulator.app gracefully: {e}");

                        // Fallback: Force quit using killall
                        if let Err(e2) = self
                            .command_executor
                            .run(Path::new(KILLALL), &[SIMULATOR_APP_NAME])
                            .await
                        {
                            log::warn!("Failed to force quit Simulator.app: {e2}");
                        }
                    }
                } else {
                    log::debug!("Other iOS devices are still running, keeping Simulator.app open");
                }
            }
            Err(e) => {
                log::warn!("Failed to check device status before quitting Simulator.app: {e}");
            }
        }
    }
}

#[cfg(target_os = "macos")]
impl DeviceManager for IosManager {
    type Device = IosDevice;

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        let output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devices", "--json"])
            .await
            .context("Failed to list iOS devices")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse simctl JSON output")?;

        let mut devices = Vec::new();
        if let Some(devices_obj) = json.get("devices") {
            if let Some(devices_map) = devices_obj.as_object() {
                // Process devices more efficiently with batch parsing
                let mut raw_devices = Vec::new();

                // First, collect all device data without complex async operations
                for (runtime, device_list_json) in devices_map {
                    if let Some(device_array_json) = device_list_json.as_array() {
                        for device_json_val in device_array_json {
                            raw_devices.push((device_json_val, runtime));
                        }
                    }
                }

                // Parse devices in batches to improve performance
                for batch in raw_devices.chunks(IOS_DEVICE_PARSE_BATCH_SIZE) {
                    for (device_json_val, runtime) in batch {
                        if let Some(parsed_device) =
                            self.parse_device_from_json(device_json_val, runtime)?
                        {
                            devices.push(parsed_device);
                        }
                    }
                }
            }
        }

        // Sort devices by priority after parsing for consistent ordering
        devices.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_ios_device_priority(&a.name);
            let priority_b = DynamicDeviceConfig::calculate_ios_device_priority(&b.name);
            priority_a.cmp(&priority_b)
        });

        Ok(devices)
    }

    async fn start_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to start iOS device: {identifier}");

        // Check if device is already booted
        let status_output = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "list", "devices", "-j"])
            .await
            .context("Failed to get device status")?;

        let json: Value =
            serde_json::from_str(&status_output).context("Failed to parse device status")?;

        let mut is_already_booted = false;
        if let Some(devices) = json.get("devices").and_then(|v| v.as_object()) {
            for (_, device_list) in devices {
                if let Some(devices_array) = device_list.as_array() {
                    for device in devices_array {
                        if let Some(udid) = device.get("udid").and_then(|v| v.as_str()) {
                            if udid == identifier {
                                if let Some(state) = device.get("state").and_then(|v| v.as_str()) {
                                    if state == IOS_DEVICE_STATUS_BOOTED {
                                        is_already_booted = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_already_booted {
            log::info!("Device {identifier} is already booted");
        } else {
            // Boot the device
            let boot_result = self
                .command_executor
                .run(Path::new(XCRUN), &[SIMCTL, "boot", identifier])
                .await;

            match boot_result {
                Ok(_) => log::info!("Successfully booted iOS device {identifier}"),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains(IOS_ALREADY_BOOTED_ERROR) {
                        log::info!("Device {identifier} was already in the process of booting");
                    } else {
                        return Err(e).context(format!("Failed to boot iOS device {identifier}"));
                    }
                }
            }
        }

        // Attempt to open Simulator.app, but don't fail the whole operation if this specific step fails.
        if let Err(e) = self
            .command_executor
            .spawn(
                Path::new("open"),
                &[SIMULATOR_OPEN_FLAG, SIMULATOR_APP_NAME],
            )
            .await
        {
            log::warn!("Failed to open Simulator app: {e}. Device might be booting in headless mode or Simulator app needs to be opened manually.");
        }
        Ok(())
    }

    async fn stop_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to stop iOS device: {identifier}");

        let shutdown_result = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "shutdown", identifier])
            .await;

        match shutdown_result {
            Ok(_) => {
                log::info!("Successfully shut down iOS device {identifier}");

                // Check if all devices are now stopped, and if so, quit Simulator.app
                // This prevents the Simulator icon from lingering in the Dock
                self.quit_simulator_if_no_running_devices().await;

                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains(IOS_ALREADY_SHUTDOWN_ERROR) {
                    log::info!("Device {identifier} was already shut down");

                    // Even if device was already stopped, check if we should quit Simulator.app
                    self.quit_simulator_if_no_running_devices().await;

                    Ok(())
                } else {
                    Err(e).context(format!("Failed to shutdown iOS device {identifier}"))
                }
            }
        }
    }

    async fn create_device(&self, config: &DeviceConfig) -> Result<()> {
        log::info!(
            "Attempting to create iOS device: {} of type {} with runtime {}",
            config.name,
            config.device_type,
            config.version
        );
        // For iOS, config.version is the runtime identifier (e.g., com.apple.CoreSimulator.SimRuntime.iOS-17-0)
        // config.device_type is the device type identifier (e.g., com.apple.CoreSimulator.SimDeviceType.iPhone-15)
        let output = self
            .command_executor
            .run(
                Path::new(XCRUN),
                &[
                    SIMCTL,
                    "create",
                    &config.name,
                    &config.device_type,
                    &config.version,
                ],
            )
            .await
            .context(format!(
                "Failed to create iOS device '{}' with type '{}' and runtime '{}'",
                config.name, config.device_type, config.version
            ))?;
        log::info!("Successfully created iOS device. UDID: {}", output.trim());
        Ok(())
    }

    async fn delete_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to delete iOS device: {identifier}");

        // First try to shutdown the device if it's running
        let _ = self
            .command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "shutdown", identifier])
            .await; // Ignore errors as device might already be shut down

        // Now delete the device
        self.command_executor
            .run(Path::new(XCRUN), &[SIMCTL, "delete", identifier])
            .await
            .context(format!(
                "Failed to delete iOS device {identifier}. Make sure the device exists and is not in use."
            ))?;

        log::info!("Successfully deleted iOS device {identifier}");
        Ok(())
    }

    async fn wipe_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to wipe iOS device: {identifier}");
        // For iOS, we use the erase command which wipes all content and settings
        self.erase_device(identifier).await
    }

    async fn is_available(&self) -> bool {
        which::which("xcrun").is_ok()
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
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "macos")]
    fn test_extract_ios_version() {
        // Test version parse logic (actual implementation: major + minor/100 + patch/10000)
        assert_eq!(extract_ios_version("iOS 18.5"), 18.05); // 18 + 5/100
        assert_eq!(extract_ios_version("iOS 17.0"), 17.0); // 17 + 0/100
        assert_eq!(extract_ios_version("iOS 16.4"), 16.04); // 16 + 4/100

        // With patch version: major + minor/100 + patch/10000
        assert!((extract_ios_version("iOS 15.2.1") - 15.0201).abs() < 0.0001); // Consider floating point precision
        assert!((extract_ios_version("iOS 16.3.2") - 16.0302).abs() < 0.0001);

        // Other OS follow the same logic
        assert_eq!(extract_ios_version("watchOS 10.0"), 10.0);
        assert_eq!(extract_ios_version("tvOS 17.2"), 17.02);

        // Edge cases
        assert_eq!(extract_ios_version("iOS-17-0"), 17.0);
        assert_eq!(extract_ios_version("iOS"), 0.0);
        assert_eq!(extract_ios_version(""), 0.0);

        // Verify sort functionality
        assert!(extract_ios_version("iOS 18.5") > extract_ios_version("iOS 18.0"));
        assert!(extract_ios_version("iOS 17.9") > extract_ios_version("iOS 17.1"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_parse_device_type_display_name() {
        // Test basic device type conversion (not dependent on implementation details)
        let result = IosManager::parse_device_type_display_name(
            "com.apple.CoreSimulator.SimDeviceType.iPhone-15-Pro",
        );
        assert!(result.contains("iPhone"));
        assert!(result.contains("15"));
        assert!(result.contains("Pro"));

        let result2 = IosManager::parse_device_type_display_name("iPhone-14");
        assert!(result2.contains("iPhone"));
        assert!(result2.contains("14"));

        // Test empty string
        assert_eq!(IosManager::parse_device_type_display_name(""), "");
    }
}
