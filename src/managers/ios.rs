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

use crate::managers::common::{DeviceConfig, DeviceManager};
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
            .replace("com.apple.CoreSimulator.SimRuntime.iOS-", "")
            .replace("-", ".");

        let status = match state_str {
            "Booted" => DeviceStatus::Running,
            "Shutdown" => DeviceStatus::Stopped,
            "Creating" => DeviceStatus::Creating,
            _ => DeviceStatus::Unknown,
        };
        let is_running_bool = state_str == "Booted";

        Ok(Some(IosDevice {
            name,
            udid,
            device_type: device_type_identifier,
            ios_version: ios_version_str.clone(),
            runtime_version: ios_version_str,
            status,
            is_running: is_running_bool,
            is_available: is_available_json,
            is_physical: false,
        }))
    }

    // These helper methods remain in the inherent impl block as they are specific to IosManager's way of handling things
    // and not directly part of the DeviceManager trait's public API contract for all managers.
    pub async fn erase_device(&self, udid: &str) -> Result<()> {
        self.command_runner
            .run("xcrun", &["simctl", "erase", udid])
            .await
            .context(format!("Failed to erase iOS device {}", udid))?;
        Ok(())
    }

    /// Lists available iOS device types with their identifiers and display names.
    pub async fn list_device_types_with_names(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_runner
            .run("xcrun", &["simctl", "list", "devicetypes", "-j"])
            .await
            .context("Failed to list iOS device types")?;

        let json: Value = serde_json::from_str(&output).context("Failed to parse device types")?;

        let mut device_types = Vec::new();

        if let Some(types_array) = json.get("devicetypes").and_then(|v| v.as_array()) {
            for device_type in types_array {
                if let (Some(identifier), Some(name)) = (
                    device_type.get("identifier").and_then(|v| v.as_str()),
                    device_type.get("name").and_then(|v| v.as_str()),
                ) {
                    device_types.push((identifier.to_string(), name.to_string()));
                }
            }
        }

        Ok(device_types)
    }

    /// Lists available iOS runtimes.
    pub async fn list_runtimes(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_runner
            .run("xcrun", &["simctl", "list", "runtimes", "-j"])
            .await
            .context("Failed to list iOS runtimes")?;

        let json: Value = serde_json::from_str(&output).context("Failed to parse runtimes")?;

        let mut runtimes = Vec::new();

        if let Some(runtimes_array) = json.get("runtimes").and_then(|v| v.as_array()) {
            for runtime in runtimes_array {
                if let (Some(identifier), Some(name), Some(is_available)) = (
                    runtime.get("identifier").and_then(|v| v.as_str()),
                    runtime.get("name").and_then(|v| v.as_str()),
                    runtime.get("isAvailable").and_then(|v| v.as_bool()),
                ) {
                    if is_available {
                        runtimes.push((identifier.to_string(), name.to_string()));
                    }
                }
            }
        }

        // Sort runtimes by version (newest first)
        runtimes.sort_by(|a, b| {
            let version_a = extract_ios_version(&a.1);
            let version_b = extract_ios_version(&b.1);
            version_b
                .partial_cmp(&version_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(runtimes)
    }

    /// Lists all connected physical iOS devices.
    ///
    /// Uses `xcrun devicectl` (Xcode 15+) or `instruments` (older Xcode) to discover
    /// connected physical iOS devices.
    ///
    /// # Returns
    /// Vector of IosDevice structs representing physical devices
    pub async fn list_physical_devices(&self) -> Result<Vec<IosDevice>> {
        let mut devices = Vec::new();

        // Try using xcdevice first (works with recent Xcode versions)
        if let Ok(output) = self
            .command_runner
            .run("xcrun", &["xcdevice", "list"])
            .await
        {
            if let Ok(json) = serde_json::from_str::<Value>(&output) {
                if let Some(devices_array) = json.as_array() {
                    for device_json in devices_array {
                        // Filter for physical iOS devices only
                        if let Some(simulator) =
                            device_json.get("simulator").and_then(|v| v.as_bool())
                        {
                            if simulator {
                                continue; // Skip simulators
                            }
                        }

                        if let Some(platform) = device_json.get("platform").and_then(|v| v.as_str())
                        {
                            if !platform.contains("iphoneos") && !platform.contains("ipados") {
                                continue; // Skip non-iOS devices (like Mac)
                            }
                        }

                        if let (Some(udid), Some(name), Some(model)) = (
                            device_json.get("identifier").and_then(|v| v.as_str()),
                            device_json.get("name").and_then(|v| v.as_str()),
                            device_json.get("modelName").and_then(|v| v.as_str()),
                        ) {
                            let ios_version = device_json
                                .get("operatingSystemVersion")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string();

                            // Extract version number from format like "16.6 (20G75)"
                            let version_number = ios_version
                                .split_whitespace()
                                .next()
                                .unwrap_or(&ios_version)
                                .to_string();

                            devices.push(IosDevice {
                                name: name.to_string(),
                                udid: udid.to_string(),
                                device_type: model.to_string(),
                                ios_version: version_number.clone(),
                                runtime_version: format!("iOS {}", version_number),
                                status: DeviceStatus::Running,
                                is_running: true,
                                is_available: true,
                                is_physical: true,
                            });
                        }
                    }
                }
            }
        } else {
            // Fallback to instruments for older Xcode versions
            if let Ok(output) = self
                .command_runner
                .run("instruments", &["-s", "devices"])
                .await
            {
                for line in output.lines() {
                    // Skip simulators and header lines
                    if line.contains("Simulator")
                        || line.contains("Known Devices")
                        || line.trim().is_empty()
                    {
                        continue;
                    }

                    // Parse lines like: "iPhone 14 Pro (17.0) [UUID]"
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line.find(']') {
                            let udid = line[start + 1..end].to_string();
                            let device_info = line[..start].trim();

                            // Extract device name and iOS version
                            if let Some(paren_start) = device_info.rfind('(') {
                                if let Some(paren_end) = device_info.rfind(')') {
                                    let name = device_info[..paren_start].trim().to_string();
                                    let ios_version =
                                        device_info[paren_start + 1..paren_end].trim().to_string();

                                    devices.push(IosDevice {
                                        name: name.clone(),
                                        udid,
                                        device_type: name,
                                        ios_version: ios_version.clone(),
                                        runtime_version: format!("iOS {}", ios_version),
                                        status: DeviceStatus::Running,
                                        is_running: true,
                                        is_available: true,
                                        is_physical: true,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(devices)
    }

    /// Lists all iOS simulators.
    ///
    /// # Returns
    /// Vector of IosDevice structs representing available simulators
    pub async fn list_simulators(&self) -> Result<Vec<IosDevice>> {
        let output = self
            .command_runner
            .run("xcrun", &["simctl", "list", "devices", "-j"])
            .await
            .context("Failed to list iOS devices")?;

        let json: Value = serde_json::from_str(&output).context("Failed to parse device list")?;

        let mut devices = Vec::new();

        if let Some(devices_map) = json.get("devices").and_then(|v| v.as_object()) {
            for (runtime_str, device_list) in devices_map {
                if let Some(devices_array) = device_list.as_array() {
                    for device_json in devices_array {
                        if let Ok(Some(device)) =
                            self.parse_device_from_json(device_json, runtime_str)
                        {
                            devices.push(device);
                        }
                    }
                }
            }
        }

        // Sort devices by priority
        devices.sort_by_key(|d| self.calculate_device_priority(&d.device_type, &d.name));

        Ok(devices)
    }

    /// Calculates priority for device sorting.
    fn calculate_device_priority(&self, device_type: &str, name: &str) -> u32 {
        let lower_type = device_type.to_lowercase();
        let lower_name = name.to_lowercase();

        // iPhone priority (0-99)
        if lower_type.contains("iphone") || lower_name.contains("iphone") {
            if lower_name.contains("pro max") {
                0
            } else if lower_name.contains("pro") {
                10
            } else if lower_name.contains("plus") || lower_name.contains("max") {
                20
            } else if lower_name.contains("mini") {
                30
            } else if lower_name.contains("se") {
                40
            } else {
                // Extract version number for regular iPhones
                let version = extract_ios_version(&lower_name);
                50 - (version as u32).min(50)
            }
        }
        // iPad priority (100-199)
        else if lower_type.contains("ipad") || lower_name.contains("ipad") {
            if lower_name.contains("pro") && lower_name.contains("12.9") {
                100
            } else if lower_name.contains("pro") && lower_name.contains("11") {
                110
            } else if lower_name.contains("pro") {
                120
            } else if lower_name.contains("air") {
                130
            } else if lower_name.contains("mini") {
                140
            } else {
                150
            }
        }
        // Apple TV priority (200-299)
        else if lower_type.contains("tv") || lower_name.contains("tv") {
            if lower_name.contains("4k") {
                200
            } else {
                210
            }
        }
        // Apple Watch priority (300-399)
        else if lower_type.contains("watch") || lower_name.contains("watch") {
            if lower_name.contains("ultra") {
                300
            } else if lower_name.contains("series") {
                // Extract series number
                let series = extract_ios_version(&lower_name) as u32;
                310 - series.min(10)
            } else if lower_name.contains("se") {
                330
            } else {
                340
            }
        }
        // Other devices
        else {
            999
        }
    }

    async fn start_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to start iOS device: {}", identifier);

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
                                    if state == "Booted" {
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
            log::info!("Device {} is already booted", identifier);
        } else {
            // Boot the device
            let boot_result = self
                .command_runner
                .run("xcrun", &["simctl", "boot", identifier])
                .await;

            match boot_result {
                Ok(_) => log::info!("Successfully booted iOS device {}", identifier),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("Unable to boot device in current state: Booted") {
                        log::info!(
                            "Device {} was already in the process of booting",
                            identifier
                        );
                    } else {
                        return Err(e).context(format!("Failed to boot iOS device {}", identifier));
                    }
                }
            }
        }

        // Attempt to open Simulator.app, but don't fail the whole operation if this specific step fails.
        if let Err(e) = self
            .command_runner
            .spawn("open", &["-a", "Simulator"])
            .await
        {
            log::warn!("Failed to open Simulator app: {}. Device might be booting in headless mode or Simulator app needs to be opened manually.", e);
        }
        Ok(())
    }

    async fn stop_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to stop iOS device: {}", identifier);

        let shutdown_result = self
            .command_runner
            .run("xcrun", &["simctl", "shutdown", identifier])
            .await;

        match shutdown_result {
            Ok(_) => {
                log::info!("Successfully shut down iOS device {}", identifier);
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("Unable to shutdown device in current state: Shutdown") {
                    log::info!("Device {} was already shut down", identifier);
                    Ok(())
                } else {
                    Err(e).context(format!("Failed to shutdown iOS device {}", identifier))
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
        log::info!("Attempting to delete iOS device: {}", identifier);

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
                "Failed to delete iOS device {}. Make sure the device exists and is not in use.",
                identifier
            ))?;

        log::info!("Successfully deleted iOS device {}", identifier);
        Ok(())
    }

    async fn wipe_device(&self, identifier: &str) -> Result<()> {
        log::info!("Attempting to wipe iOS device: {}", identifier);
        // For iOS, we use the erase command which wipes all content and settings
        self.erase_device(identifier).await
    }

    async fn is_available(&self) -> bool {
        which::which("xcrun").is_ok()
    }
}

#[cfg(target_os = "macos")]
#[allow(clippy::manual_async_fn)]
impl DeviceManager for IosManager {
    type Device = IosDevice;

    fn list_devices(&self) -> impl std::future::Future<Output = Result<Vec<Self::Device>>> + Send {
        async {
            // Get both simulators and physical devices
            let mut all_devices = Vec::new();

            // Get simulators
            let simulators = self.list_simulators().await?;
            all_devices.extend(simulators);

            // Get physical devices
            let physical_devices = self.list_physical_devices().await?;
            all_devices.extend(physical_devices);

            Ok(all_devices)
        }
    }

    fn start_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let identifier = identifier.to_string();
        async move { self.start_device(&identifier).await }
    }

    fn stop_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let identifier = identifier.to_string();
        async move { self.stop_device(&identifier).await }
    }

    fn create_device(
        &self,
        config: &DeviceConfig,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let config = config.clone();
        async move { self.create_device(&config).await }
    }

    fn delete_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let identifier = identifier.to_string();
        async move { self.delete_device(&identifier).await }
    }

    fn wipe_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        let identifier = identifier.to_string();
        async move { self.wipe_device(&identifier).await }
    }

    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send {
        async { self.is_available().await }
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

    pub async fn list_simulators(&self) -> Result<Vec<IosDevice>> {
        bail!("iOS simulator management is only available on macOS")
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(clippy::manual_async_fn)]
impl DeviceManager for IosManager {
    type Device = IosDevice; // This will use the potentially simplified IosDevice from models.rs for non-macOS

    fn list_devices(&self) -> impl std::future::Future<Output = Result<Vec<Self::Device>>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn start_device(
        &self,
        _identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn stop_device(
        &self,
        _identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn create_device(
        &self,
        _config: &DeviceConfig,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn delete_device(
        &self,
        _identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn wipe_device(
        &self,
        _identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { bail!("iOS simulator management is only available on macOS") }
    }

    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send {
        async { false } // Not available on non-macOS
    }
}
