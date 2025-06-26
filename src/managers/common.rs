//! Common manager utilities
//!
//! This module provides shared traits, types, and utilities for device managers.
//! It defines the common interface that both Android and iOS managers implement,
//! along with helper functions for device name sanitization and tool discovery.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Unified interface for managing virtual devices across platforms.
///
/// This trait provides a common API for device operations that works
/// consistently across Android AVDs and iOS simulators. All methods
/// are async to handle potentially long-running operations.
///
/// # Type Parameters
/// * `Device` - The platform-specific device type (AndroidDevice, IosDevice)
///
/// # Device Lifecycle
/// 1. `list_devices()` - Discover available devices
/// 2. `create_device()` - Create new devices with configuration
/// 3. `start_device()` / `stop_device()` - Control device state
/// 4. `wipe_device()` - Reset device to factory state
/// 5. `delete_device()` - Remove device permanently
///
/// # Error Handling
/// All methods return `Result<T>` to handle platform-specific errors
/// like missing SDKs, invalid configurations, or device conflicts.
pub trait DeviceManager {
    /// The platform-specific device type this manager handles
    type Device;

    /// Lists all available devices for this platform.
    ///
    /// This includes both running and stopped devices. The returned
    /// list is sorted by device priority for optimal user experience.
    ///
    /// # Returns
    /// * `Ok(Vec<Device>)` - List of available devices
    /// * `Err(anyhow::Error)` - If device discovery fails
    fn list_devices(&self) -> impl std::future::Future<Output = Result<Vec<Self::Device>>> + Send;

    /// Starts a virtual device by its identifier.
    ///
    /// For Android, this launches the AVD using the emulator command.
    /// For iOS, this boots the simulator using simctl. The operation
    /// may take several seconds to complete.
    ///
    /// # Arguments
    /// * `identifier` - Device identifier (AVD name for Android, UDID for iOS)
    ///
    /// # Returns
    /// * `Ok(())` - If device starts successfully
    /// * `Err(anyhow::Error)` - If device start fails or device not found
    fn start_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Stops a running virtual device.
    ///
    /// Gracefully shuts down the device, saving any unsaved state.
    /// If the device is already stopped, this operation succeeds silently.
    ///
    /// # Arguments
    /// * `identifier` - Device identifier to stop
    ///
    /// # Returns
    /// * `Ok(())` - If device stops successfully or was already stopped
    /// * `Err(anyhow::Error)` - If stop operation fails
    fn stop_device(&self, identifier: &str)
        -> impl std::future::Future<Output = Result<()>> + Send;

    /// Creates a new virtual device with the specified configuration.
    ///
    /// The device is created but not started. Use `start_device()` after
    /// creation to launch the new device.
    ///
    /// # Arguments
    /// * `config` - Device configuration including name, type, and system image
    ///
    /// # Returns
    /// * `Ok(())` - If device creation succeeds
    /// * `Err(anyhow::Error)` - If creation fails (name conflict, missing image, etc.)
    fn create_device(
        &self,
        config: &DeviceConfig,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Permanently deletes a virtual device.
    ///
    /// The device is stopped if running, then completely removed from
    /// the system. This operation cannot be undone.
    ///
    /// # Arguments
    /// * `identifier` - Device identifier to delete
    ///
    /// # Returns
    /// * `Ok(())` - If device deletion succeeds
    /// * `Err(anyhow::Error)` - If deletion fails or device not found
    fn delete_device(
        &self,
        identifier: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Wipes a virtual device, resetting it to factory state.
    ///
    /// This clears all user data, installed apps, and settings,
    /// returning the device to its initial configuration.
    ///
    /// # Arguments
    /// * `identifier` - Device identifier to wipe
    ///
    /// # Returns
    /// * `Ok(())` - If device wipe succeeds
    /// * `Err(anyhow::Error)` - If wipe operation fails
    fn wipe_device(&self, identifier: &str)
        -> impl std::future::Future<Output = Result<()>> + Send;

    /// Checks if the platform's development tools are available.
    ///
    /// For Android, this verifies Android SDK installation.
    /// For iOS, this checks for Xcode command line tools.
    ///
    /// # Returns
    /// * `true` - If platform tools are available and functional
    /// * `false` - If platform is not supported or tools are missing
    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send;
}

/// Configuration for creating new virtual devices.
///
/// DeviceConfig encapsulates all the settings needed to create a new
/// Android AVD or iOS simulator. It uses the builder pattern for
/// easy configuration with optional parameters.
///
/// # Examples
/// ```rust
/// use emu::managers::common::DeviceConfig;
///
/// // Basic device configuration
/// let config = DeviceConfig::new(
///     "My Device".to_string(),
///     "pixel_7".to_string(),
///     "android-34".to_string()
/// );
///
/// // Android device with custom RAM and storage
/// let name = "My Phone".to_string();
/// let device_type = "pixel_7".to_string();
/// let version = "android-34".to_string();
/// let android_config = DeviceConfig::new(name, device_type, version)
///     .with_ram("4096".to_string())
///     .with_storage("8192".to_string());
/// ```

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    /// Human-readable device name
    pub name: String,
    /// Platform-specific device type identifier
    pub device_type: String,
    /// System image version (API level for Android, iOS version for iOS)
    pub version: String,
    /// RAM allocation in MB (Android only)
    pub ram_size: Option<String>,
    /// Storage size in MB (Android only)
    pub storage_size: Option<String>,
    /// Additional platform-specific configuration options
    pub additional_options: HashMap<String, String>,
}

impl DeviceConfig {
    /// Creates a new device configuration with required parameters.
    ///
    /// # Arguments
    /// * `name` - Display name for the device
    /// * `device_type` - Platform-specific device type identifier
    /// * `version` - System image version to use
    ///
    /// # Returns
    /// A new DeviceConfig instance ready for customization
    pub fn new(name: String, device_type: String, version: String) -> Self {
        Self {
            name,
            device_type,
            version,
            ram_size: None,
            storage_size: None,
            additional_options: HashMap::new(),
        }
    }

    /// Sets the RAM allocation for the device (Android only).
    ///
    /// # Arguments
    /// * `ram` - RAM size in MB as a string
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_ram(mut self, ram: String) -> Self {
        self.ram_size = Some(ram);
        self
    }

    /// Sets the storage size for the device (Android only).
    ///
    /// # Arguments
    /// * `storage` - Storage size in MB as a string
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_storage(mut self, storage: String) -> Self {
        self.storage_size = Some(storage);
        self
    }

    /// Adds a custom configuration option.
    ///
    /// This allows platform-specific options to be passed through
    /// to the underlying device creation tools.
    ///
    /// # Arguments
    /// * `key` - Option name
    /// * `value` - Option value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_option(mut self, key: String, value: String) -> Self {
        self.additional_options.insert(key, value);
        self
    }
}

/// Parses JSON output from device management commands.
///
/// This function safely parses JSON strings returned by commands like
/// `adb devices` or `xcrun simctl list devices --json`.
///
/// # Arguments
/// * `output` - JSON string from command output
///
/// # Returns
/// * `Ok(Value)` - Parsed JSON value
/// * `Err(anyhow::Error)` - If JSON parsing fails
///
/// # Examples
/// ```rust
/// use emu::managers::common::parse_json_devices;
/// # fn main() -> anyhow::Result<()> {
/// let json = parse_json_devices(r#"{"devices": []}"#)?;
/// # Ok(())
/// # }
/// ```
pub fn parse_json_devices(output: &str) -> Result<Value> {
    let json: Value = serde_json::from_str(output)?;
    Ok(json)
}

/// Extracts a device list from nested JSON using a path.
///
/// Navigates through JSON object hierarchy using the provided path
/// and returns the array value if found.
///
/// # Arguments
/// * `json` - Root JSON value to search
/// * `path` - Array of keys to navigate the JSON hierarchy
///
/// # Returns
/// * `Some(&Value)` - If the path leads to an array
/// * `None` - If the path is invalid or doesn't lead to an array
///
/// # Examples
/// ```rust
/// use emu::managers::common::extract_device_list;
/// use serde_json::json;
///
/// let json = json!({"devices": {"android": []}});
/// let devices = extract_device_list(&json, &["devices", "android"]);
/// ```
pub fn extract_device_list<'a>(json: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = json;
    for key in path {
        current = current.get(key)?;
    }
    current.as_array().map(|_| current)
}

/// Formats a device name to fit within a specified length.
///
/// Truncates long device names and appends "..." to indicate truncation.
/// Useful for display in fixed-width UI components.
///
/// # Arguments
/// * `name` - Original device name
/// * `max_length` - Maximum allowed length including "..."
///
/// # Returns
/// Formatted device name that fits within the specified length
///
/// # Examples
/// ```rust
/// use emu::managers::common::format_device_name;
///
/// assert_eq!(format_device_name("Long Device Name", 10), "Long De...");
/// assert_eq!(format_device_name("Short", 10), "Short");
/// ```
pub fn format_device_name(name: &str, max_length: usize) -> String {
    if name.len() <= max_length {
        name.to_string()
    } else {
        format!("{}...", &name[..max_length - 3])
    }
}

/// Sanitizes device names for safe filesystem usage.
///
/// Replaces unsafe characters with underscores while preserving
/// alphanumeric characters, hyphens, underscores, and periods.
///
/// # Arguments
/// * `name` - Original device name
///
/// # Returns
/// Sanitized device name safe for filesystem operations
///
/// # Examples
/// ```rust
/// use emu::managers::common::sanitize_device_name;
///
/// assert_eq!(sanitize_device_name("My Device!"), "My_Device_");
/// assert_eq!(sanitize_device_name("Test-Device_123"), "Test-Device_123");
/// ```
pub fn sanitize_device_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Sanitizes device names for safe command line usage (more strict).
///
/// This function performs aggressive sanitization to ensure device names
/// are safe for use in shell commands. It removes quotes, spaces, and
/// other problematic characters that could cause command injection or
/// parsing issues.
///
/// # Arguments
/// * `name` - Original device name
///
/// # Returns
/// Heavily sanitized device name safe for command line usage
///
/// # Sanitization Rules
/// - Removes quotes, spaces, tabs, newlines completely
/// - Preserves alphanumeric, hyphens, underscores, periods
/// - Replaces other characters with underscores
/// - Trims leading/trailing non-alphanumeric characters
///
/// # Examples
/// ```rust
/// use emu::managers::common::sanitize_device_name_for_command;
///
/// assert_eq!(sanitize_device_name_for_command("2.7\" QVGA API 36"), "2.7QVGAAPI36");
/// assert_eq!(sanitize_device_name_for_command("My Device"), "MyDevice");
/// assert_eq!(sanitize_device_name_for_command("'Test'"), "Test");
/// ```
pub fn sanitize_device_name_for_command(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter_map(|c| match c {
            '"' | '\'' | ' ' | '\t' | '\n' | '\r' => None,
            c if c.is_alphanumeric() || matches!(c, '-' | '_' | '.') => Some(c),
            _ => Some('_'),
        })
        .collect();

    // Ensure the name doesn't start or end with special characters
    sanitized
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim_end_matches(|c: char| !c.is_alphanumeric())
        .to_string()
}

#[derive(Debug, Clone)]
pub struct ToolPath {
    pub name: String,
    pub path: std::path::PathBuf,
    pub version: Option<String>,
}

impl ToolPath {
    pub fn new(name: String, path: std::path::PathBuf) -> Self {
        Self {
            name,
            path,
            version: None,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

pub fn find_tools_in_paths(tool_name: &str, search_paths: &[std::path::PathBuf]) -> Vec<ToolPath> {
    let mut tools = Vec::new();

    for path in search_paths {
        let tool_path = path.join(tool_name);
        if tool_path.exists() {
            tools.push(ToolPath::new(tool_name.to_string(), tool_path));
        }

        // Also check with .exe extension on Windows
        #[cfg(windows)]
        {
            let exe_path = path.join(format!("{}.exe", tool_name));
            if exe_path.exists() {
                tools.push(ToolPath::new(tool_name.to_string(), exe_path));
            }
        }
    }

    tools
}

pub async fn check_tool_version(tool_path: &std::path::Path) -> Result<String> {
    let output = std::process::Command::new(tool_path)
        .arg("--version")
        .output()?;

    let version_output = String::from_utf8_lossy(&output.stdout);

    // Extract version number from output
    let version = version_output
        .lines()
        .next()
        .unwrap_or("unknown")
        .split_whitespace()
        .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_device_name() {
        assert_eq!(format_device_name("Short", 20), "Short");
        assert_eq!(
            format_device_name("This is a very long device name", 15),
            "This is a ve..."
        );
    }

    #[test]
    fn test_sanitize_device_name() {
        assert_eq!(sanitize_device_name("My Device"), "My_Device");
        assert_eq!(sanitize_device_name("Test-Device_123"), "Test-Device_123");
        assert_eq!(sanitize_device_name("Device@#$%"), "Device____");
    }

    #[test]
    fn test_sanitize_device_name_for_command() {
        // Test quote and space removal (the main issue from user feedback)
        assert_eq!(
            sanitize_device_name_for_command("2.7\" QVGA API 36"),
            "2.7QVGAAPI36"
        );
        assert_eq!(
            sanitize_device_name_for_command("'Single Quote'"),
            "SingleQuote"
        );

        // Test space removal
        assert_eq!(
            sanitize_device_name_for_command("My Device Name"),
            "MyDeviceName"
        );

        // Test special character handling (replace with underscore, then trim)
        assert_eq!(sanitize_device_name_for_command("Device@#$%"), "Device");
        assert_eq!(
            sanitize_device_name_for_command("Device@Test#"),
            "Device_Test"
        );

        // Test leading/trailing special character removal
        assert_eq!(
            sanitize_device_name_for_command("_Test_Device_"),
            "Test_Device"
        );
        assert_eq!(sanitize_device_name_for_command("...Test..."), "Test");

        // Test alphanumeric preservation
        assert_eq!(
            sanitize_device_name_for_command("Test-Device_123.4"),
            "Test-Device_123.4"
        );

        // Test empty result handling
        assert_eq!(sanitize_device_name_for_command("\"'\"'"), "");

        // Test the specific case from user's error (double quotes in device name)
        assert_eq!(
            sanitize_device_name_for_command("2.7\" QVGA API 36"),
            "2.7QVGAAPI36"
        );
    }
}
