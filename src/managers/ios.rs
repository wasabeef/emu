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

//! # xcrun simctl Command Reference
//!
//! ## Device Listing (`xcrun simctl list devices --json`)
//! ```json
//! {
//!   "devices": {
//!     "com.apple.CoreSimulator.SimRuntime.iOS-17-0": [
//!       {
//!         "lastBootedAt": "2024-01-15T10:30:00Z",
//!         "dataPath": "/Users/.../CoreSimulator/Devices/{UUID}/data",
//!         "logPath": "/Users/.../CoreSimulator/Devices/{UUID}/device.log",
//!         "udid": "A1B2C3D4-E5F6-G7H8-I9J0-K1L2M3N4O5P6",
//!         "isAvailable": true,
//!         "deviceTypeIdentifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
//!         "state": "Booted",
//!         "name": "iPhone 15"
//!       }
//!     ]
//!   }
//! }
//! ```
//!
//! **Device States**:
//! - `Booted`: Device is running
//! - `Shutdown`: Device is stopped
//! - `Creating`: Device is being created
//! - `Booting`: Device is starting up
//! - `Shutting Down`: Device is stopping
//!
//! ## Device Types (`xcrun simctl list devicetypes --json`)
//! ```json
//! {
//!   "devicetypes": [
//!     {
//!       "minRuntimeVersion": 917504,
//!       "bundlePath": "/Applications/Xcode.app/.../iPhone 15.simdevicetype",
//!       "maxRuntimeVersion": 4294967295,
//!       "name": "iPhone 15",
//!       "identifier": "com.apple.CoreSimulator.SimDeviceType.iPhone-15",
//!       "productFamily": "iPhone"
//!     }
//!   ]
//! }
//! ```
//!
//! **Device Type Naming**:
//! - Identifier format: `com.apple.CoreSimulator.SimDeviceType.{Device-Name}`
//! - Display names extracted from JSON `name` field or parsed from identifier
//! - Special handling for sizes: "12.9-inch" → "12.9\""
//!
//! ## Runtimes (`xcrun simctl list runtimes --json`)
//! ```json
//! {
//!   "runtimes": [
//!     {
//!       "bundlePath": "/Library/.../iOS 17.0.simruntime",
//!       "buildversion": "21A342",
//!       "platform": "iOS",
//!       "runtimeRoot": "/Library/.../iOS 17.0.simruntime/Contents/Resources/RuntimeRoot",
//!       "identifier": "com.apple.CoreSimulator.SimRuntime.iOS-17-0",
//!       "version": "17.0",
//!       "isInternal": false,
//!       "isAvailable": true,
//!       "name": "iOS 17.0",
//!       "supportedDeviceTypes": [
//!         "com.apple.CoreSimulator.SimDeviceType.iPhone-SE-3rd-generation",
//!         "com.apple.CoreSimulator.SimDeviceType.iPhone-15"
//!       ]
//!     }
//!   ]
//! }
//! ```
//!
//! ## Common Operations
//!
//! ### Create Device
//! ```bash
//! xcrun simctl create "My iPhone" com.apple.CoreSimulator.SimDeviceType.iPhone-15 com.apple.CoreSimulator.SimRuntime.iOS-17-0
//! # Returns: UUID of created device
//! ```
//!
//! ### Boot Device
//! ```bash
//! xcrun simctl boot {UUID}
//! # Note: Returns error if already booted - handled gracefully
//! ```
//!
//! ### Shutdown Device
//! ```bash
//! xcrun simctl shutdown {UUID}
//! # Note: Returns error if already shutdown - handled gracefully
//! ```
//!
//! ### Delete Device
//! ```bash
//! xcrun simctl delete {UUID}
//! # Note: Automatically shuts down device first if needed
//! ```
//!
//! ### Erase Device (Wipe)
//! ```bash
//! xcrun simctl erase {UUID}
//! # Resets device to factory settings
//! ```
//!
//! ## Device Priority System
//!
//! Devices are sorted with the following priority (lower number = higher priority):
//!
//! 1. **iPhone** (0-99):
//!    - Pro Max: 0
//!    - Pro: 10
//!    - Plus/Max: 20
//!    - Mini: 30
//!    - SE: 40
//!    - Regular (by version): 50 - version
//!
//! 2. **iPad** (100-199):
//!    - Pro 12.9": 100
//!    - Pro 11": 110
//!    - Air: 130
//!    - Mini: 140
//!    - Regular: 150
//!
//! 3. **Apple TV** (200-299):
//!    - 4K: 200
//!    - HD: 210
//!
//! 4. **Apple Watch** (300-399):
//!    - Ultra: 300
//!    - Series (by version): 310 - series_number
//!    - SE: 330
//!
//! ## Error Handling
//!
//! ### Common Errors and Solutions
//! - "Unable to boot device in current state: Booted" → Device already running (ignored)
//! - "Unable to shutdown device in current state: Shutdown" → Device already stopped (ignored)
//! - "Invalid device type" → Check available types with `xcrun simctl list devicetypes`
//! - "Invalid runtime" → Check available runtimes with `xcrun simctl list runtimes`
//!
//! ### Simulator App Integration
//! - Boot operations attempt to open Simulator.app automatically
//! - Failures to open Simulator.app are logged but don't fail the operation
//! - Devices can run in "headless" mode without the Simulator app
//! - Automatic cleanup: Simulator.app quits when last device stops
//! - Graceful shutdown: Uses AppleScript with killall fallback
//!
//! ## Log Streaming
//!
//! iOS simulator logs can be streamed using multiple approaches:
//!
//! ### Method 1: Direct simulator spawn (most reliable)
//! ```bash
//! xcrun simctl spawn {UUID} log stream
//! ```
//!
//! ### Method 2: System log filtering
//! ```bash
//! log stream --predicate 'processImagePath contains "Simulator"'
//! ```
//!
//! ### Method 3: Console app logs
//! ```bash
//! log stream --style compact
//! ```
//!
//! **Log Level Detection**:
//! - Keywords "error" or "Error" → ERROR level
//! - Keywords "warning" or "Warning" → WARN level
//! - All other logs → INFO level
//!
//! ## Implementation Notes
//!
//! ### State Detection Optimization
//! The `start_device` method pre-checks device state to avoid redundant boot commands,
//! preventing unnecessary error messages and improving user experience.
//!
//! ### Cross-Platform Safety
//! - All iOS-specific code is gated with `#[cfg(target_os = "macos")]`
//! - Non-macOS platforms get stub implementations that return appropriate errors
//! - The `which` crate is used to verify `xcrun` availability at runtime

#[cfg(target_os = "macos")]
use crate::constants::ios::{
    IOS_ALREADY_BOOTED_ERROR, IOS_ALREADY_SHUTDOWN_ERROR, IOS_DEVICE_STATUS_BOOTED,
    IOS_DEVICE_STATUS_CREATING, IOS_DEVICE_STATUS_SHUTDOWN, IOS_DEVICE_TYPE_PREFIX,
    IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT, IOS_RUNTIME_PREFIX, SIMULATOR_APP_NAME,
    SIMULATOR_OPEN_FLAG, SIMULATOR_QUIT_COMMAND,
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
use log;
#[cfg(target_os = "macos")]
use serde_json::Value;
#[cfg(target_os = "macos")]
use which;

#[cfg(target_os = "macos")]
/// Extract iOS version number from display string for sorting
///
/// Parses version numbers from runtime display names:
/// - "iOS 17.0" → 17.0
/// - "iOS 16.4" → 16.4
/// - "iOS 15.2.1" → 15.2 (major.minor only)
fn extract_ios_version(display_name: &str) -> f32 {
    // Extract version from strings like "iOS 17.0", "iOS 16.4", etc.
    if let Some(version_start) = display_name.find(char::is_numeric) {
        let version_str: String = display_name[version_start..]
            .chars()
            .take_while(|c| c.is_numeric() || *c == '.')
            .collect();

        // Parse "17.0" -> 17.0, "16.4" -> 16.4
        version_str.parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
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
    /// Command runner for executing xcrun simctl commands
    command_runner: CommandRunner,
}

#[cfg(target_os = "macos")]
impl IosManager {
    // Inherent methods
    pub fn new() -> Result<Self> {
        if which::which("xcrun").is_err() {
            bail!("Xcode Command Line Tools not found. Please install Xcode or run 'xcode-select --install'.")
        }

        // Verify simctl is available
        let runner = CommandRunner::new();
        if let Err(e) = std::process::Command::new("xcrun")
            .args(["simctl", "help"])
            .output()
        {
            bail!(
                "Failed to access iOS Simulator: {}. Make sure Xcode is properly installed.",
                e
            )
        }

        Ok(Self {
            command_runner: runner,
        })
    }

    fn parse_device_from_json(
        &self,
        device_json: &Value,
        runtime_str: &str,
    ) -> Result<Option<IosDevice>> {
        let name = device_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
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
            .unwrap_or("Unknown");
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
    pub async fn erase_device(&self, udid: &str) -> Result<()> {
        self.command_runner
            .run("xcrun", &["simctl", "erase", udid])
            .await
            .context(format!("Failed to erase iOS device {udid}"))?;
        Ok(())
    }

    pub async fn list_device_types(&self) -> Result<Vec<String>> {
        let output = self
            .command_runner
            .run("xcrun", &["simctl", "list", "devicetypes", "--json"])
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
            .command_runner
            .run("xcrun", &["simctl", "list", "devicetypes", "--json"])
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

        // Special case handling
        let display = cleaned.replace(IOS_INCH_PATTERN, IOS_INCH_REPLACEMENT); // 12.9 inch -> 12.9"

        // Capitalize properly
        display
            .split_whitespace()
            .map(|word| {
                if word == "inch"
                    || word == "se"
                    || word == "mini"
                    || (word.starts_with("i")
                        && (word.starts_with("iPhone")
                            || word.starts_with("iPad")
                            || word.starts_with("iPod")))
                {
                    word.to_string() // Preserve these special cases as-is
                } else {
                    // Capitalize first letter
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
            .command_runner
            .run("xcrun", &["simctl", "list", "runtimes", "--json"])
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
                        .command_runner
                        .run("osascript", &["-e", SIMULATOR_QUIT_COMMAND])
                        .await
                    {
                        log::warn!("Failed to quit Simulator.app gracefully: {e}");

                        // Fallback: Force quit using killall
                        if let Err(e2) = self
                            .command_runner
                            .run("killall", &[SIMULATOR_APP_NAME])
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
            .command_runner
            .run("xcrun", &["simctl", "list", "devices", "--json"])
            .await
            .context("Failed to list iOS devices")?;
        let json: Value =
            serde_json::from_str(&output).context("Failed to parse simctl JSON output")?;
        let mut devices = Vec::new();
        if let Some(devices_obj) = json.get("devices") {
            if let Some(devices_map) = devices_obj.as_object() {
                for (runtime, device_list_json) in devices_map {
                    if let Some(device_array_json) = device_list_json.as_array() {
                        for device_json_val in device_array_json {
                            if let Some(parsed_device) =
                                self.parse_device_from_json(device_json_val, runtime)?
                            {
                                devices.push(parsed_device);
                            }
                        }
                    }
                }
            }
        }
        Ok(devices)
    }

    async fn start_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to start iOS device: {identifier}");

        // Check if device is already booted
        let status_output = self
            .command_runner
            .run("xcrun", &["simctl", "list", "devices", "-j"])
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
                .command_runner
                .run("xcrun", &["simctl", "boot", identifier])
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
            .command_runner
            .spawn("open", &[SIMULATOR_OPEN_FLAG, SIMULATOR_APP_NAME])
            .await
        {
            log::warn!("Failed to open Simulator app: {e}. Device might be booting in headless mode or Simulator app needs to be opened manually.");
        }
        Ok(())
    }

    async fn stop_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to stop iOS device: {identifier}");

        let shutdown_result = self
            .command_runner
            .run("xcrun", &["simctl", "shutdown", identifier])
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
            .command_runner
            .run(
                "xcrun",
                &[
                    "simctl",
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
            .command_runner
            .run("xcrun", &["simctl", "shutdown", identifier])
            .await; // Ignore errors as device might already be shut down

        // Now delete the device
        self.command_runner
            .run("xcrun", &["simctl", "delete", identifier])
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
