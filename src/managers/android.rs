//! Android emulator management
//!
//! This module provides comprehensive Android Virtual Device (AVD) management by interfacing
//! with Android SDK command-line tools. All device information, API levels, and configurations
//! are retrieved dynamically from the Android SDK to ensure compatibility with future updates.
//!
//! # Key Implementation Details
//!
//! - **Dynamic Discovery**: All device types, API levels, and system images are discovered at runtime
//! - **Name Normalization**: AVD names with spaces are handled via underscore conversion for compatibility
//! - **Multi-Method Detection**: API levels use config.ini parsing with multiple fallback strategies
//! - **Smart Prioritization**: Devices sorted by category, version, and manufacturer dynamically
//!
//! # Android SDK Tools Integration
//!
//! ## avdmanager Command Reference
//!
//! The `avdmanager` tool is the primary interface for AVD operations:
//!
//! ### Device Listing (`avdmanager list device`)
//! ```text
//! Available Android Virtual Devices:
//! ========
//!     id: 0 or "tv_1080p"
//!     Name: Android TV (1080p)
//!     OEM : Google
//! ---------
//!     id: 1 or "tv_720p"
//!     Name: Android TV (720p)
//!     OEM : Google
//! ---------
//!     id: 2 or "wear_round"
//!     Name: Android Wear Round
//!     OEM : Google
//! ---------
//!     id: 3 or "wear_square"
//!     Name: Android Wear Square  
//!     OEM : Google
//! ---------
//!     id: 4 or "pixel_7"
//!     Name: Pixel 7
//!     OEM : Google
//! ---------
//! ```
//!
//! **Device Specification Fields** (parsed dynamically):
//! - `id`: Device identifier (used for --device parameter)
//! - `Name`: Human-readable device name
//! - `OEM`: Original Equipment Manufacturer
//! - Screen size, resolution, and density (in device details)
//!
//! ### AVD Listing (`avdmanager list avd`)
//! ```text
//! Available Android Virtual Devices:
//!     Name: Pixel_7_API_34
//!     Device: pixel_7 (Pixel 7)
//!     Path: /Users/user/.android/avd/Pixel_7_API_34.avd
//!     Target: Google APIs (Google Inc.)
//!             Based on: Android 14.0 (API level 34) Tag/ABI: google_apis_playstore/arm64-v8a
//! ---------
//! ```
//!
//! **AVD Information Fields**:
//! - `Name`: AVD instance name
//! - `Device`: Device type reference
//! - `Path`: Filesystem path to AVD configuration
//! - `Target`: Target platform and API information
//! - `Based on`: Android version and API level
//! - `Tag/ABI`: System image tag and processor architecture
//!
//! ### API Level Detection Strategy
//!
//! API levels are detected using multiple fallback methods:
//!
//! 1. **Primary**: `config.ini` analysis
//!    ```ini
//!    image.sysdir.1=system-images/android-34/google_apis_playstore/arm64-v8a/
//!    target=android-34
//!    ```
//!
//! 2. **Secondary**: Boot property queries
//!    ```bash
//!    adb -s emulator-5554 shell getprop ro.boot.qemu.avd_name
//!    adb -s emulator-5554 shell getprop ro.kernel.qemu.avd_name
//!    ```
//!
//! 3. **Tertiary**: avdmanager target parsing
//!    - Regex: `Based on:\s*Android\s*([\d.]+)`
//!    - Regex: `API level (\d+)`
//!    - Regex: `android-(\d+)`
//!
//! ## sdkmanager Command Reference
//!
//! The `sdkmanager` tool provides system image and platform information:
//!
//! ### System Images Listing (`sdkmanager --list --verbose`)
//! ```text
//! Installed packages:=====================] 100% Fetch remote repository...
//!   Path                                        | Version | Description                    | Location
//!   -------                                     | ------- | -------                        | -------
//!   build-tools;34.0.0                          | 34.0.0  | Android SDK Build-Tools 34     | build-tools/34.0.0
//!   platforms;android-34                        | 3       | Android SDK Platform 34        | platforms/android-34
//!   system-images;android-34;google_apis;arm64-v8a | 14      | Google APIs ARM 64 v8a System Image | system-images/android-34/google_apis/arm64-v8a
//!   system-images;android-34;google_apis_playstore;arm64-v8a | 14 | Google Play ARM 64 v8a System Image | system-images/android-34/google_apis_playstore/arm64-v8a
//! ```
//!
//! **System Image Package Format**: `system-images;android-{API_LEVEL};{TAG};{ABI}`
//! - API_LEVEL: Android API level (e.g., 34, 33, 32)
//! - TAG: Image variant (google_apis, google_apis_playstore, default, etc.)
//! - ABI: Processor architecture (arm64-v8a, x86_64, x86, etc.)
//!
//! ### Platform Information Parsing
//! ```text
//! platforms;android-34 | 3 | Android SDK Platform 34 | Android API 34, revision 2 | Android 14
//! ```
//!
//! **Android Version Name Extraction**:
//! - Pattern: `Android API {level}, revision {rev} | {version_name}`
//! - Example: "Android API 34, revision 2 | Android 14"
//! - Fallback: Derive from API level using known mappings
//!
//! ## Device Categorization Algorithm
//!
//! Devices are categorized dynamically based on naming patterns and device characteristics:
//!
//! ### Dynamic Pattern Matching
//! The `get_device_category()` method analyzes device IDs and display names to determine categories:
//!
//! 1. **Phone** (highest priority):
//!    - Keywords: "phone", "pixel", "galaxy", "oneplus"
//!    - Screen sizes: 5-6 inch displays
//!    - Excludes: foldables and tablets
//!
//! 2. **Tablet**:
//!    - Keywords: "tablet", "pad"
//!    - Screen sizes: 10+ inch displays
//!
//! 3. **Wearable**:
//!    - Keywords: "wear", "watch", "round", "square"
//!
//! 4. **TV**:
//!    - Keywords: "tv", "1080p", "720p", "4k"
//!
//! 5. **Automotive**:
//!    - Keywords: "auto", "car", "automotive"
//!
//! 6. **Desktop**:
//!    - Keywords: "desktop", "foldable"
//!    - Large screen sizes: 15+ inches
//!
//! ## Running State Detection
//!
//! AVD running state is determined through ADB device enumeration:
//!
//! ### ADB Devices Query (`adb devices`)
//! ```text
//! List of devices attached
//! emulator-5554   device
//! emulator-5556   device
//! ```
//!
//! ### AVD Name Resolution
//! Multiple methods are used to map emulator instances to AVD names:
//!
//! 1. **Boot property** (most reliable):
//!    ```bash
//!    adb -s emulator-5554 shell getprop ro.boot.qemu.avd_name
//!    # Returns: Pixel_7_API_34
//!    ```
//!
//! 2. **EMU console command**:
//!    ```bash
//!    adb -s emulator-5554 emu avd name
//!    # Returns: Pixel_7_API_34
//!    # Note: First line only, ignore "OK" status
//!    ```
//!
//! 3. **Kernel property** (fallback):
//!    ```bash
//!    adb -s emulator-5554 shell getprop ro.kernel.qemu.avd_name
//!    ```
//!
//! **Important**: The manager also stores normalized versions (spaces → underscores) to handle
//! AVDs created with spaces in their names, ensuring compatibility with both naming conventions.
//!
//! ## Error Handling Patterns
//!
//! ### Common avdmanager Errors
//! - License not accepted: Run `sdkmanager --licenses`
//! - System image not found: Install with `sdkmanager "system-images;android-{level};{tag};{abi}"`
//! - Invalid device name: Contains unsupported characters
//! - Existing AVD: Delete first or use --force flag
//! - Skin not found: Falls back to creation without skin parameter
//!
//! ### ADB Error Handling
//! - "error" in response: Command failed
//! - "KO" response: Command not supported
//! - "unknown command": Emulator doesn't support operation
//! - Empty response: Property not set or device offline
//!
//! ### Diagnostic System
//! The `diagnose_avd_creation_issues()` method provides detailed troubleshooting:
//! - Checks system image availability
//! - Verifies device type existence
//! - Suggests specific commands to fix issues
//! - Provides compact error messages suitable for TUI display
//!
//! ## Configuration File Format
//!
//! AVD configurations are stored in `{AVD_NAME}.avd/config.ini`:
//!
//! ```ini
//! # Critical configuration fields
//! avd.ini.encoding=UTF-8
//! avd.ini.displayname=Pixel 7 API 34
//! AvdId=Pixel_7_API_34
//! hw.device.name=pixel_7
//! hw.ramSize=2048
//! image.sysdir.1=system-images/android-34/google_apis_playstore/arm64-v8a/
//! target=android-34
//! disk.dataPartition.size=8192M
//! ```
//!
//! **Key Configuration Fields**:
//! - `avd.ini.displayname`: Human-readable name (with spaces)
//! - `AvdId`: Internal identifier (underscores instead of spaces)
//! - `hw.device.name`: Device type reference
//! - `hw.ramSize`: RAM allocation in MB
//! - `image.sysdir.1`: System image path (contains API level)
//! - `target`: Target platform identifier
//! - `disk.dataPartition.size`: Storage allocation
//!
//! ## Performance Optimizations
//!
//! ### Emulator Launch Arguments
//! ```bash
//! emulator -avd {name} -no-audio -no-snapshot-save -no-boot-anim -netfast
//! ```
//!
//! **Optimization Flags**:
//! - `-no-audio`: Disable audio subsystem (reduces overhead)
//! - `-no-snapshot-save`: Skip snapshot saving on exit
//! - `-no-boot-anim`: Skip boot animation for faster startup
//! - `-netfast`: Use faster network emulation
//! - `-wipe-data`: Cold boot with data wipe (for reset operations)
//!
//! ### Background Operations
//! - Device cache loading runs in true background tasks
//! - Log streaming uses debounced updates (500ms delay)
//! - Category filtering uses in-memory filtering when cache available
//!
//! ## Dynamic Prioritization System
//!
//! Device prioritization is calculated using three factors:
//!
//! 1. **Category Priority** (0-500 range):
//!    - Phones: 0 (highest priority)
//!    - Foldables: 20
//!    - Tablets: 100
//!    - TV: 200
//!    - Wear: 300
//!    - Automotive: 400
//!    - Generic/Unknown: 500
//!
//! 2. **Version Extraction** (0-150 range):
//!    - Extracts numeric versions from device names (e.g., "Pixel 8" → 8)
//!    - Newer versions get lower priority numbers (50 - version) * 3
//!    - Handles special cases like "8a", "15pro"
//!
//! 3. **Manufacturer Priority** (0-120 range):
//!    - Google/Pixel: 0 (highest)
//!    - Samsung/Galaxy: 10
//!    - OnePlus: 20
//!    - Other known brands: 30-60
//!    - Alphabetical for unknown: 70-110
//!
//! This ensures consistent, predictable device ordering without hardcoded device lists.
//!

use crate::{
    constants::{adb_commands, android_paths, android_version_to_api_level, commands, env_vars},
    managers::common::{DeviceConfig, DeviceManager},
    models::device_info::DeviceCategory,
    models::{AndroidDevice, ApiLevel, DeviceStatus, SdkPackage},
    utils::command::CommandRunner,
};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

lazy_static! {
    // Device listing regexes
    static ref ID_REGEX: Regex = Regex::new(r#"id:\s*\d+\s*or\s*\"(.+)\""#).unwrap();
    static ref NAME_REGEX: Regex = Regex::new(r"Name:\s*(.+)").unwrap();
    static ref OEM_REGEX: Regex = Regex::new(r"OEM\s*:\s*(.+)").unwrap();

    // AVD listing regexes
    static ref AVD_NAME_REGEX: Regex = Regex::new(r"Name:\s*(.+)").unwrap();
    static ref PATH_REGEX: Regex = Regex::new(r"Path:\s*(.+)").unwrap();
    static ref TARGET_REGEX: Regex = Regex::new(r"Target:\s*(.+)").unwrap();
    static ref ABI_REGEX: Regex = Regex::new(r"Tag/ABI:\s*(.+)").unwrap();
    static ref DEVICE_REGEX: Regex = Regex::new(r"Device:\s*(.+)").unwrap();
    static ref BASED_ON_REGEX: Regex = Regex::new(r"Based on:\s*Android\s*([\d.]+)").unwrap();

    // Config parsing regexes
    static ref IMAGE_SYSDIR_REGEX: Regex = Regex::new(r"image\.sysdir\.1=system-images/android-(\d+)/?").unwrap();
    static ref TARGET_CONFIG_REGEX: Regex = Regex::new(r"target=android-(\d+)").unwrap();
    static ref API_LEVEL_REGEX: Regex = Regex::new(r"API level (\d+)").unwrap();
    static ref ANDROID_VERSION_REGEX: Regex = Regex::new(r"android-(\d+)").unwrap();
    static ref AVD_DISPLAYNAME_REGEX: Regex = Regex::new(r"avd\.ini\.displayname=(.+)").unwrap();
    static ref NUMBER_PATTERN_REGEX: Regex = Regex::new(r"(\d{2,3})").unwrap();
    static ref API_OR_ANDROID_REGEX: Regex = Regex::new(r"(?:API level |android-)(\d+)").unwrap();
}
/// Android Virtual Device (AVD) manager implementation.
///
/// This struct provides comprehensive management of Android emulators through
/// the Android SDK command-line tools. It handles device discovery, creation,
/// lifecycle management, and real-time status monitoring.
///
/// # Key Responsibilities
/// - Discovers and manages Android SDK tools (avdmanager, emulator, adb)
/// - Lists available device types and system images dynamically
/// - Creates, starts, stops, and deletes AVDs
/// - Monitors running emulators and maps them to AVD names
/// - Provides detailed device information and logs
///
/// # Tool Integration
/// - **avdmanager**: For AVD creation, deletion, and listing
/// - **emulator**: For starting AVDs with optimized parameters
/// - **adb**: For device status, log streaming, and property queries
/// - **sdkmanager**: For system image discovery and API level information
#[derive(Clone)]
pub struct AndroidManager {
    /// Command runner for executing Android SDK tools
    command_runner: CommandRunner,
    /// Path to Android SDK home directory (from ANDROID_HOME or ANDROID_SDK_ROOT)
    _android_home: PathBuf,
    /// Path to avdmanager executable
    avdmanager_path: PathBuf,
    /// Path to emulator executable
    emulator_path: PathBuf,
}

impl AndroidManager {
    /// Creates a new AndroidManager instance.
    ///
    /// Discovers the Android SDK location from environment variables and
    /// locates required command-line tools (avdmanager, emulator).
    ///
    /// # Returns
    /// - `Ok(AndroidManager)` - If Android SDK and tools are found
    /// - `Err` - If Android SDK is not installed or tools are missing
    ///
    /// # Environment Variables
    /// Checks in order:
    /// 1. `ANDROID_HOME` - Primary Android SDK location
    /// 2. `ANDROID_SDK_ROOT` - Alternative SDK location
    pub fn new() -> Result<Self> {
        let android_home = Self::find_android_home()?;
        let avdmanager_path = Self::find_tool(&android_home, commands::AVDMANAGER)?;
        let emulator_path = Self::find_tool(&android_home, commands::EMULATOR)?;

        Ok(Self {
            command_runner: CommandRunner::new(),
            _android_home: android_home,
            avdmanager_path,
            emulator_path,
        })
    }

    /// Locates the Android SDK home directory from environment variables.
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Path to Android SDK
    /// - `Err` - If neither ANDROID_HOME nor ANDROID_SDK_ROOT is set
    fn find_android_home() -> Result<PathBuf> {
        if let Ok(path) = std::env::var(env_vars::ANDROID_HOME) {
            return Ok(PathBuf::from(path));
        }

        if let Ok(path) = std::env::var(env_vars::ANDROID_SDK_ROOT) {
            return Ok(PathBuf::from(path));
        }

        bail!("Android SDK not found. Please set ANDROID_HOME or ANDROID_SDK_ROOT")
    }

    /// Finds a specific tool within the Android SDK directory structure.
    ///
    /// Searches multiple possible locations in order:
    /// 1. cmdline-tools/latest/bin/
    /// 2. tools/bin/
    /// 3. emulator/ (for emulator tool)
    ///
    /// # Arguments
    /// * `android_home` - Android SDK root directory
    /// * `tool` - Tool name to find (e.g., "avdmanager", "emulator")
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Full path to the tool executable
    /// - `Err` - If tool is not found in any expected location
    fn find_tool(android_home: &Path, tool: &str) -> Result<PathBuf> {
        let paths = [
            android_home
                .join(android_paths::CMDLINE_TOOLS_LATEST_BIN)
                .join(tool),
            android_home.join(android_paths::TOOLS_BIN).join(tool),
            android_home.join(android_paths::EMULATOR_DIR).join(tool),
        ];

        for path in &paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        bail!("Tool '{}' not found in Android SDK", tool)
    }

    /// Maps running emulator instances to their AVD names.
    ///
    /// Uses multiple methods to resolve AVD names from emulator serial numbers:
    /// 1. Boot property: `ro.boot.qemu.avd_name` (most reliable)
    /// 2. EMU console command: `adb emu avd name`
    /// 3. Kernel property: `ro.kernel.qemu.avd_name` (fallback)
    ///
    /// Also handles AVD names with spaces by storing normalized versions
    /// (spaces replaced with underscores) for compatibility.
    ///
    /// # Returns
    /// HashMap mapping AVD names to emulator serial numbers (e.g., "emulator-5554")
    ///
    /// # Example
    /// ```
    /// // Returns: {"Pixel_7_API_34" => "emulator-5554", "Pixel 7 API 34" => "emulator-5554"}
    /// ```
    pub async fn get_running_avd_names(&self) -> Result<HashMap<String, String>> {
        let mut avd_map = HashMap::new();
        let mut normalized_map = HashMap::new();

        // Get list of running emulators
        let adb_output = self
            .command_runner
            .run(commands::ADB, &[adb_commands::DEVICES])
            .await
            .unwrap_or_default();

        for line in adb_output.lines() {
            if line.contains("emulator-") && line.contains("device") {
                if let Some(emulator_id) = line.split_whitespace().next() {
                    // log::debug!("Found emulator: {}", emulator_id);

                    // Try multiple methods to get AVD name

                    // Method 1: Try to get AVD name from boot properties (most reliable)
                    if let Ok(boot_prop_output) = self
                        .command_runner
                        .run(
                            "adb",
                            &[
                                "-s",
                                emulator_id,
                                "shell",
                                "getprop",
                                "ro.boot.qemu.avd_name",
                            ],
                        )
                        .await
                    {
                        let avd_name = boot_prop_output.trim().to_string();
                        // log::debug!("Method 1 - AVD name from boot property for {}: '{}'", emulator_id, avd_name);

                        if !avd_name.is_empty() {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            // Also store normalized version
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }

                    // Method 2: Try adb emu avd name (parse first line only)
                    if let Ok(avd_name_output) = self
                        .command_runner
                        .run("adb", &["-s", emulator_id, "emu", "avd", "name"])
                        .await
                    {
                        // Take only the first line to avoid "OK" or other status messages
                        let avd_name = avd_name_output
                            .lines()
                            .next()
                            .unwrap_or("")
                            .trim()
                            .to_string();

                        // log::debug!("Method 2 - AVD name output for {}: '{}'", emulator_id, avd_name);

                        // Check if this is a valid AVD name (not an error message)
                        if !avd_name.is_empty()
                            && !avd_name.contains("error")
                            && !avd_name.contains("KO")
                            && !avd_name.contains("unknown command")
                            && avd_name != "OK"
                        {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            // Also store normalized version
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }

                    // Method 3: Try to get AVD name from kernel properties
                    if let Ok(prop_output) = self
                        .command_runner
                        .run(
                            "adb",
                            &[
                                "-s",
                                emulator_id,
                                "shell",
                                "getprop",
                                "ro.kernel.qemu.avd_name",
                            ],
                        )
                        .await
                    {
                        let avd_name = prop_output.trim().to_string();
                        // log::debug!("Method 3 - AVD name from kernel property for {}: '{}'", emulator_id, avd_name);

                        if !avd_name.is_empty() {
                            avd_map.insert(avd_name.clone(), emulator_id.to_string());
                            // Also store normalized version
                            let normalized = avd_name.replace(' ', "_");
                            if normalized != avd_name {
                                normalized_map.insert(normalized, emulator_id.to_string());
                            }
                            continue;
                        }
                    }

                    // log::warn!("Could not determine AVD name for emulator {}", emulator_id);
                }
            }
        }

        // Merge normalized map into main map for any missing entries
        for (normalized_name, serial) in normalized_map {
            avd_map.entry(normalized_name).or_insert(serial);
        }

        Ok(avd_map)
    }

    /// Lists all connected physical Android devices.
    ///
    /// Uses `adb devices` to discover connected physical devices and retrieves
    /// their properties including device model, Android version, and API level.
    ///
    /// # Returns
    /// Vector of AndroidDevice structs representing physical devices
    pub async fn list_physical_devices(&self) -> Result<Vec<AndroidDevice>> {
        let mut devices = Vec::new();

        // Get list of all connected devices
        let adb_output = self
            .command_runner
            .run(commands::ADB, &[adb_commands::DEVICES])
            .await
            .unwrap_or_default();

        log::debug!("ADB devices output:\n{}", adb_output);

        for line in adb_output.lines() {
            if line.contains("device")
                && !line.contains("emulator-")
                && !line.contains("List of devices")
            {
                if let Some(device_id) = line.split_whitespace().next() {
                    log::debug!("Found physical device: {}", device_id);

                    // Get device properties
                    let model = self
                        .command_runner
                        .run(
                            commands::ADB,
                            &["-s", device_id, "shell", "getprop", "ro.product.model"],
                        )
                        .await
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    let manufacturer = self
                        .command_runner
                        .run(
                            commands::ADB,
                            &[
                                "-s",
                                device_id,
                                "shell",
                                "getprop",
                                "ro.product.manufacturer",
                            ],
                        )
                        .await
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    let api_level_str = self
                        .command_runner
                        .run(
                            commands::ADB,
                            &["-s", device_id, "shell", "getprop", "ro.build.version.sdk"],
                        )
                        .await
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    let api_level = api_level_str.parse::<u32>().unwrap_or(0);

                    let device_name = if !model.is_empty() && !manufacturer.is_empty() {
                        format!("{} {} ({})", manufacturer, model, device_id)
                    } else {
                        device_id.to_string()
                    };

                    devices.push(AndroidDevice {
                        name: device_name,
                        device_type: model.clone(),
                        api_level,
                        status: DeviceStatus::Running,
                        is_running: true,
                        ram_size: "N/A".to_string(),
                        storage_size: "N/A".to_string(),
                        is_physical: true,
                    });
                }
            }
        }

        Ok(devices)
    }

    async fn start_device(&self, identifier: &str) -> Result<()> {
        // Start emulator with reduced console output
        let args = vec![
            "-avd",
            identifier,
            "-no-audio",         // Disable audio for less output
            "-no-snapshot-save", // Don't save snapshot on exit
            "-no-boot-anim",     // Skip boot animation
            "-netfast",          // Faster network emulation
        ];

        self.command_runner
            .spawn(&self.emulator_path, &args)
            .await?;
        Ok(())
    }

    async fn stop_device(&self, identifier: &str) -> Result<()> {
        // log::info!("Attempting to stop Android emulator: {}", identifier);

        // Get running AVDs to find the emulator ID for the given AVD name
        let running_avds = self.get_running_avd_names().await?;

        if let Some(emulator_id) = running_avds.get(identifier) {
            // log::info!("Found emulator {} for AVD {}, stopping it", emulator_id, identifier);
            self.command_runner
                .run("adb", &["-s", emulator_id, "emu", "kill"])
                .await
                .context(format!("Failed to stop emulator {}", emulator_id))?;
        } else {
            // log::warn!("AVD '{}' is not currently running", identifier);
        }

        Ok(())
    }

    async fn create_device(&self, config: &DeviceConfig) -> Result<()> {
        // AVD names must follow strict rules: only a-z A-Z 0-9 . _ - are allowed
        // But preserve the original name for display purposes in config.ini
        let safe_name = config
            .name
            .chars()
            .filter_map(|c| match c {
                // Keep only allowed characters, replace spaces and others with underscores
                c if c.is_ascii_alphanumeric() || c == '.' || c == '-' => Some(c),
                ' ' | '_' => Some('_'), // Convert spaces to underscores
                _ => None,              // Remove all other characters
            })
            .collect::<String>()
            .trim_matches('_') // Remove leading/trailing underscores
            .to_string();

        if safe_name.is_empty() {
            return Err(anyhow::anyhow!(
                "Device name '{}' contains only invalid characters and cannot be used for AVD creation.",
                config.name
            ));
        }

        // Check if device with this name already exists
        let existing_devices = self.list_devices().await?;
        if existing_devices.iter().any(|d| d.name == safe_name) {
            return Err(anyhow::anyhow!(
                "Device with name '{}' already exists. Please choose a different name or delete the existing device first.",
                safe_name
            ));
        }

        // Try to find available system image for this API level
        let (tag, abi) = if let Some((found_tag, found_abi)) = self
            .get_first_available_system_image(&config.version)
            .await?
        {
            (found_tag, found_abi)
        } else {
            // If no system image found, try default values
            let default_tag = config
                .additional_options
                .get("tag")
                .map_or("google_apis_playstore", |s| s.as_str());
            let default_abi = config
                .additional_options
                .get("abi")
                .map_or("arm64-v8a", |s| s.as_str());
            (default_tag.to_string(), default_abi.to_string())
        };

        let package_path = format!(
            "system-images;android-{};{};{}",
            config.version, // API level
            tag,
            abi
        );

        // Check if system image is available
        let image_available = self
            .check_system_image_available(&config.version, &tag, &abi)
            .await
            .unwrap_or(false);

        if !image_available {
            let available_images = self.list_available_system_images().await?;
            return Err(anyhow::anyhow!(
                "System image '{}' not found. Install it with: sdkmanager \"{}\"\nAvailable images: {}",
                package_path, package_path, available_images.join(", ")
            ));
        }

        // **IMPROVED APPROACH**: Use proper device and skin parameters
        let mut args = vec!["create", "avd", "-n", &safe_name, "-k", &package_path];

        // Add device parameter if valid - use ID for better compatibility
        let device_param =
            if !config.device_type.is_empty() && config.device_type.to_lowercase() != "custom" {
                let available_devices = self.list_available_devices().await?;
                Self::find_matching_device_id(&available_devices, &config.device_type)
            } else {
                None
            };

        // デバイスパラメータが見つからない場合はデフォルトを使用
        if let Some(ref device_id) = device_param {
            args.push("--device");
            args.push(device_id);
        } else {
            // デバイスパラメータを省略 - avdmanager がデフォルトデバイスを使用
            log::warn!(
                "Device type '{}' not found, using default device",
                config.device_type
            );
        }

        // スキンを動的に取得して指定（エラー時はフォールバック戦略を使用）
        let skin_name = if let Some(ref device_id) = device_param {
            self.get_appropriate_skin(device_id, &config.device_type)
                .await
        } else {
            self.get_appropriate_skin(&config.device_type, &config.device_type)
                .await
        };

        if let Some(ref skin) = skin_name {
            args.push("--skin");
            args.push(skin);
        }

        let result = self.command_runner.run(&self.avdmanager_path, &args).await;

        // スキンエラーの場合はスキンなしで再試行
        let result = if result.is_err() && skin_name.is_some() {
            let error_str = result.as_ref().unwrap_err().to_string();
            if error_str.to_lowercase().contains("skin") {
                log::warn!(
                    "Skin '{}' failed, retrying without skin",
                    skin_name.as_ref().unwrap()
                );
                // スキンパラメータを除去して再試行
                let mut fallback_args =
                    vec!["create", "avd", "-n", &safe_name, "-k", &package_path];
                if let Some(ref device_id) = device_param {
                    fallback_args.push("--device");
                    fallback_args.push(device_id);
                }
                self.command_runner
                    .run(&self.avdmanager_path, &fallback_args)
                    .await
            } else {
                result
            }
        } else {
            result
        };

        match result {
            Ok(_output) => {
                // **OPTIMIZED**: Only update config if absolutely necessary
                // The avdmanager should have created a good base configuration with --device and --skin
                // We only need to fine-tune specific settings
                if let Err(e) = self
                    .fine_tune_avd_config(&safe_name, config, &tag, &abi)
                    .await
                {
                    // Log warning but don't fail the entire operation
                    eprintln!("Warning: Failed to fine-tune AVD configuration: {}", e);
                }

                Ok(())
            }
            Err(e) => {
                // Provide comprehensive error diagnostics
                let error_str = e.to_string();

                // Create ultra-compact diagnostic info for UI
                let mut diagnostic_info = Vec::new();
                // 短縮された名前（最大20文字）
                let short_name = if safe_name.len() > 20 {
                    format!("{}...", &safe_name[..17])
                } else {
                    safe_name.clone()
                };
                diagnostic_info.push(format!("AVD: {}", short_name));
                diagnostic_info.push(format!("API: {}", config.version));

                // 重要な情報を先頭に、簡潔なエラーメッセージを作成
                if error_str.contains("system image")
                    || error_str.contains("package path")
                    || error_str.contains("not installed")
                {
                    Err(anyhow::anyhow!(
                        "System image not installed for API {}\nRun: sdkmanager \"{}\"",
                        config.version,
                        package_path
                    ))
                } else if error_str.contains("license") || error_str.contains("accept") {
                    Err(anyhow::anyhow!(
                        "Android SDK licenses not accepted\nRun: sdkmanager --licenses"
                    ))
                } else if error_str.contains("already exists") {
                    Err(anyhow::anyhow!(
                        "AVD '{}' already exists\nDelete existing or choose different name",
                        config.name
                    ))
                } else if error_str.contains("device") && error_str.contains("not found") {
                    Err(anyhow::anyhow!(
                        "Device type '{}' not found\nCheck available types in device list",
                        config.device_type
                    ))
                } else {
                    // 汎用エラー - 最も重要な情報のみ
                    let key_error = if error_str.contains("Error:") {
                        error_str
                            .split("Error:")
                            .nth(1)
                            .unwrap_or(&error_str)
                            .trim()
                    } else if error_str.contains("failed") {
                        error_str
                            .split("failed")
                            .nth(0)
                            .unwrap_or(&error_str)
                            .trim()
                    } else {
                        &error_str
                    };

                    let short_error = if key_error.len() > 60 {
                        format!("{}...", &key_error[..57])
                    } else {
                        key_error.to_string()
                    };

                    Err(anyhow::anyhow!(
                        "AVD creation failed: {}\nAVD: {} | API: {}",
                        short_error,
                        short_name,
                        config.version
                    ))
                }
            }
        }
    }

    async fn delete_device(&self, identifier: &str) -> Result<()> {
        // Check if device is running and stop it first
        let running_avds = self.get_running_avd_names().await.unwrap_or_default();
        if running_avds.contains_key(identifier) {
            log::info!(
                "Device '{}' is running, stopping before deletion",
                identifier
            );
            if let Err(e) = self.stop_device(identifier).await {
                log::warn!(
                    "Failed to stop device '{}' before deletion: {}",
                    identifier,
                    e
                );
                // Continue with deletion even if stop fails
            }

            // Wait a bit for the device to fully stop
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        // Delete the AVD
        self.command_runner
            .run(&self.avdmanager_path, &["delete", "avd", "-n", identifier])
            .await
            .context(format!("Failed to delete Android AVD '{}'", identifier))?;
        Ok(())
    }

    async fn wipe_device(&self, identifier: &str) -> Result<()> {
        // For Android, wipe means clearing user data without starting the device
        // First, stop the device if it's running
        let running_avds = self.get_running_avd_names().await?;
        if running_avds.contains_key(identifier) {
            log::info!("Device '{}' is running, stopping before wipe", identifier);
            self.stop_device(identifier).await?;
            // Wait a bit for the emulator to shut down
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        // Directly delete user data files from AVD directory instead of starting emulator
        if let Ok(home_dir) = std::env::var("HOME") {
            let avd_path = std::path::PathBuf::from(home_dir)
                .join(".android")
                .join("avd")
                .join(format!("{}.avd", identifier));

            if avd_path.exists() {
                // Delete user data files that get recreated on next boot
                let files_to_delete = [
                    "userdata.img",
                    "userdata-qemu.img",
                    "cache.img",
                    "cache.img.qcow2",
                    "userdata.img.qcow2",
                    "sdcard.img",
                    "sdcard.img.qcow2",
                    "multiinstance.lock",
                ];

                for file_name in &files_to_delete {
                    let file_path = avd_path.join(file_name);
                    if file_path.exists() {
                        if let Err(e) = tokio::fs::remove_file(&file_path).await {
                            log::warn!("Failed to remove {}: {}", file_path.display(), e);
                        } else {
                            log::debug!("Removed user data file: {}", file_path.display());
                        }
                    }
                }

                // Also clear snapshots directory if it exists
                let snapshots_dir = avd_path.join("snapshots");
                if snapshots_dir.exists() {
                    if let Err(e) = tokio::fs::remove_dir_all(&snapshots_dir).await {
                        log::warn!("Failed to remove snapshots directory: {}", e);
                    } else {
                        log::debug!("Removed snapshots directory");
                    }
                }

                log::info!("Successfully wiped user data for device '{}'", identifier);
            } else {
                return Err(anyhow::anyhow!(
                    "AVD directory not found: {}",
                    avd_path.display()
                ));
            }
        } else {
            return Err(anyhow::anyhow!("HOME environment variable not set"));
        }

        Ok(())
    }

    async fn is_available(&self) -> bool {
        // Availability is determined by `new()` succeeding (tools found).
        true
    }
}

/// Extracts percentage from a line of text.
/// Looks for patterns like "50%" or "[50%]" or "50 %".
fn extract_percentage(line: &str) -> Option<u8> {
    // Try to find a number followed by %
    if let Some(pos) = line.find('%') {
        // Look backwards from % to find the number
        let prefix = &line[..pos];
        let number_str: String = prefix
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || c.is_whitespace())
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>()
            .trim()
            .to_string();

        if let Ok(percent) = number_str.parse::<u8>() {
            return Some(percent.min(100));
        }
    }
    None
}

impl AndroidManager {
    /// Lists all Android Virtual Devices (AVDs).
    ///
    /// # Returns
    /// Vector of AndroidDevice structs representing available AVDs
    pub async fn list_avds(&self) -> Result<Vec<AndroidDevice>> {
        let output = self
            .command_runner
            .run(&self.avdmanager_path, &["list", "avd"])
            .await
            .context("Failed to list Android AVDs")?;

        let mut devices = Vec::new();
        let mut current_avd = HashMap::<String, String>::new();
        let running_avds = self.get_running_avd_names().await.unwrap_or_default();

        // Parse the output
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed == "---------" {
                // End of current AVD info, process it
                if let Some(name) = current_avd.get("Name") {
                    let mut device_name = name.clone();
                    let api_level = self.extract_api_level_from_avd(&current_avd).unwrap_or(0);
                    let device_type = current_avd
                        .get("Device")
                        .map(|d| {
                            // Extract device type name from format like "pixel_7 (Pixel 7)"
                            if let Some(paren_idx) = d.find(" (") {
                                d[paren_idx + 2..d.len() - 1].to_string()
                            } else {
                                d.clone()
                            }
                        })
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Check if running - support both original name and normalized name
                    let is_running = running_avds.contains_key(&device_name)
                        || running_avds.contains_key(&device_name.replace(' ', "_"));

                    let status = if is_running {
                        DeviceStatus::Running
                    } else {
                        DeviceStatus::Stopped
                    };

                    // For devices with spaces in names, prefer the display name from config
                    let _avd_display_name = self
                        .get_avd_display_name(&device_name)
                        .await
                        .unwrap_or_else(|| device_name.clone());

                    // Always use the AVD ID (safe name) for operations
                    device_name = current_avd.get("Name").unwrap_or(&device_name).clone();

                    devices.push(AndroidDevice {
                        name: device_name,
                        device_type,
                        api_level,
                        status,
                        is_running,
                        ram_size: "2048".to_string(), // Default, will be updated from config
                        storage_size: "8192".to_string(), // Default, will be updated from config
                        is_physical: false,
                    });
                }
                current_avd.clear();
            } else if let Some((key, value)) = self.parse_key_value_line(trimmed) {
                current_avd.insert(key, value);
            }
        }

        // Process the last AVD if there's no trailing separator
        if let Some(name) = current_avd.get("Name") {
            let mut device_name = name.clone();
            let api_level = self.extract_api_level_from_avd(&current_avd).unwrap_or(0);
            let device_type = current_avd
                .get("Device")
                .map(|d| {
                    if let Some(paren_idx) = d.find(" (") {
                        d[paren_idx + 2..d.len() - 1].to_string()
                    } else {
                        d.clone()
                    }
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let is_running = running_avds.contains_key(&device_name)
                || running_avds.contains_key(&device_name.replace(' ', "_"));

            let status = if is_running {
                DeviceStatus::Running
            } else {
                DeviceStatus::Stopped
            };

            let _avd_display_name = self
                .get_avd_display_name(&device_name)
                .await
                .unwrap_or_else(|| device_name.clone());

            device_name = current_avd.get("Name").unwrap_or(&device_name).clone();

            devices.push(AndroidDevice {
                name: device_name,
                device_type,
                api_level,
                status,
                is_running,
                ram_size: "2048".to_string(),
                storage_size: "8192".to_string(),
                is_physical: false,
            });
        }

        Ok(devices)
    }

    /// Parses a key-value line from avdmanager output.
    fn parse_key_value_line(&self, line: &str) -> Option<(String, String)> {
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Some((parts[0].trim().to_string(), parts[1].trim().to_string()));
            }
        }
        None
    }

    /// Extracts API level from AVD information.
    fn extract_api_level_from_avd(&self, avd_info: &HashMap<String, String>) -> Option<u32> {
        // Try to extract from "Based on" field first
        if let Some(based_on) = avd_info.get("Based on") {
            if let Some(captures) = BASED_ON_REGEX.captures(based_on) {
                if let Some(version) = captures.get(1) {
                    // Convert Android version to API level
                    let version_str = version.as_str();
                    // Parse version string to float, then to u32
                    if let Ok(version_float) = version_str.parse::<f32>() {
                        let api_level = android_version_to_api_level(version_float as u32);
                        return Some(api_level);
                    }
                }
            }
            // Try direct API level extraction
            if let Some(captures) = API_LEVEL_REGEX.captures(based_on) {
                if let Some(api) = captures.get(1) {
                    return api.as_str().parse().ok();
                }
            }
        }

        // Try to extract from Target field
        if let Some(target) = avd_info.get("Target") {
            if let Some(captures) = ANDROID_VERSION_REGEX.captures(target) {
                if let Some(api) = captures.get(1) {
                    return api.as_str().parse().ok();
                }
            }
        }

        // Try to extract from Path (look for API level in path)
        if let Some(path) = avd_info.get("Path") {
            if let Some(api_match) = API_OR_ANDROID_REGEX.find(path) {
                let api_str = api_match.as_str();
                if let Some(captures) = API_OR_ANDROID_REGEX.captures(api_str) {
                    if let Some(api) = captures.get(1) {
                        return api.as_str().parse().ok();
                    }
                }
            }
        }

        None
    }

    /// Gets the display name for an AVD from its config.ini file.
    async fn get_avd_display_name(&self, avd_name: &str) -> Option<String> {
        if let Ok(home_dir) = std::env::var("HOME") {
            let config_path = std::path::PathBuf::from(home_dir)
                .join(".android")
                .join("avd")
                .join(format!("{}.avd", avd_name))
                .join("config.ini");

            if let Ok(content) = fs::read_to_string(&config_path).await {
                for line in content.lines() {
                    if let Some(captures) = AVD_DISPLAYNAME_REGEX.captures(line) {
                        if let Some(display_name) = captures.get(1) {
                            return Some(display_name.as_str().to_string());
                        }
                    }
                }
            }
        }
        None
    }
    /// SDK Manager functionality for installing API levels and system images.
    /// Lists all available packages from the SDK manager.
    ///
    /// # Returns
    /// - `Ok(Vec<SdkPackage>)` - List of available packages
    /// - `Err` - If sdkmanager command fails
    pub async fn list_sdk_packages(&self) -> Result<Vec<SdkPackage>> {
        let sdkmanager_path = self.find_sdkmanager_tool()?;

        let output = self
            .command_runner
            .run(&sdkmanager_path, &["--list", "--verbose"])
            .await
            .context("Failed to list SDK packages")?;

        let mut packages = Vec::new();
        let mut in_packages_section = false;
        let mut in_installed_section = false;
        let mut lines = output.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            // Track which section we're in
            if trimmed.contains("Installed packages:") {
                in_installed_section = true;
                in_packages_section = false;
                continue;
            } else if trimmed.contains("Available Packages:") {
                in_packages_section = true;
                in_installed_section = false;
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with("------") {
                continue;
            }

            // Parse package lines from both sections
            if in_packages_section || in_installed_section {
                // Check if this line looks like a package name (contains semicolons)
                if trimmed.contains(';')
                    && !trimmed.starts_with("Description:")
                    && !trimmed.starts_with("Version:")
                    && !trimmed.starts_with("Installed Location:")
                {
                    if let Some(mut package) =
                        self.parse_sdk_package_info(&mut lines, trimmed.to_string())
                    {
                        package.installed = in_installed_section;
                        packages.push(package);
                    }
                }
            }
        }

        log::debug!("Found {} SDK packages", packages.len());
        Ok(packages)
    }

    /// Lists available API levels that can be installed.
    ///
    /// # Returns
    /// - `Ok(Vec<ApiLevel>)` - List of available API levels
    /// - `Err` - If sdkmanager command fails
    pub async fn list_available_api_levels(&self) -> Result<Vec<ApiLevel>> {
        let packages = self.list_sdk_packages().await?;
        let mut api_levels = Vec::new();

        for package in packages {
            // Look for system-images instead of platforms
            if package.name.starts_with("system-images;android-") {
                if let Some(api_level) = self.extract_system_image_info(&package) {
                    api_levels.push(api_level);
                }
            }
        }

        // Group by API level and select the best system image for each level
        let mut best_images: std::collections::HashMap<u32, ApiLevel> =
            std::collections::HashMap::new();

        for api_level in api_levels {
            let key = api_level.level;
            let is_better = match best_images.get(&key) {
                None => true,
                Some(existing) => {
                    self.is_better_system_image(&api_level.package_name, &existing.package_name)
                }
            };

            if is_better {
                best_images.insert(key, api_level);
            }
        }

        // Convert back to Vec and sort by API level (descending - newest first)
        let mut result: Vec<_> = best_images.into_values().collect();
        result.sort_by(|a, b| b.level.cmp(&a.level));

        Ok(result)
    }

    /// Installs the best system image for a specific API level.
    ///
    /// # Arguments
    /// * `api_level` - The API level to install (e.g., 34 for Android 14)
    ///
    /// # Returns
    /// - `Ok(())` - If installation succeeds
    /// - `Err` - If installation fails
    pub async fn install_api_level(&self, api_level: u32) -> Result<()> {
        // Find the best system image for this API level
        let system_image_package = self.find_best_system_image_for_api_level(api_level).await?;

        let sdkmanager_path = self.find_sdkmanager_tool()?;

        log::info!(
            "Installing system image for API Level {} ({})",
            api_level,
            system_image_package
        );

        // Accept licenses first
        self.accept_licenses().await?;

        // Install the system image package
        let output = self
            .command_runner
            .run(&sdkmanager_path, &[&system_image_package])
            .await
            .context(format!(
                "Failed to install system image for API level {}",
                api_level
            ))?;

        log::debug!("sdkmanager install output: {}", output);

        // Verify installation
        if output.contains("Warning:") || output.contains("Error:") {
            return Err(anyhow::anyhow!(
                "SDK installation completed with warnings/errors: {}",
                output
            ));
        }

        log::info!(
            "Successfully installed system image for API Level {}",
            api_level
        );
        Ok(())
    }

    /// Installs a system image with progress reporting.
    ///
    /// # Arguments
    /// * `api_level` - The API level to install
    /// * `progress_callback` - Callback to report installation progress
    ///
    /// # Returns
    /// - `Ok(())` - If installation succeeds
    /// - `Err` - If installation fails
    pub async fn install_api_level_with_progress<F>(
        &self,
        api_level: u32,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(crate::models::SdkInstallStatus) + Send,
    {
        use crate::models::SdkInstallStatus;
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::process::Command;

        // Find the best system image for this API level
        let package_name = self.find_best_system_image_for_api_level(api_level).await?;
        let sdkmanager_path = self.find_sdkmanager_tool()?;

        log::info!(
            "Installing system image for API Level {} with progress tracking ({})",
            api_level,
            package_name
        );

        // Report initial status
        progress_callback(SdkInstallStatus::Installing {
            progress: 0,
            message: "Preparing installation...".to_string(),
        });

        // Accept licenses first
        progress_callback(SdkInstallStatus::Installing {
            progress: 5,
            message: "Accepting licenses...".to_string(),
        });

        if let Err(e) = self.accept_licenses().await {
            progress_callback(SdkInstallStatus::Failed {
                error: format!("Failed to accept licenses: {}", e),
            });
            return Err(e);
        }

        progress_callback(SdkInstallStatus::Installing {
            progress: 10,
            message: format!("Starting download of API Level {}...", api_level),
        });

        // Run sdkmanager with progress tracking
        let mut cmd = Command::new(&sdkmanager_path);
        cmd.arg(&package_name)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().context("Failed to start sdkmanager")?;

        // Create readers for stdout and stderr
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let stderr = child.stderr.take().expect("Failed to get stderr");
        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        // Track progress based on output
        let mut current_progress = 10;
        #[allow(unused_assignments)]
        let mut last_message = "Starting download...".to_string();

        // Use tokio::select! to read from both streams
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log::debug!("sdkmanager stdout: {}", line);

                            // Parse progress from output
                            if line.contains("Downloading") && current_progress < 20 {
                                current_progress = 20;
                                last_message = "Downloading system image...".to_string();
                                progress_callback(SdkInstallStatus::Installing {
                                    progress: current_progress,
                                    message: last_message.clone(),
                                });
                            } else if (line.contains("Unzipping") || line.contains("Extracting")) && current_progress < 60 {
                                current_progress = 60;
                                last_message = "Extracting system image...".to_string();
                                progress_callback(SdkInstallStatus::Installing {
                                    progress: current_progress,
                                    message: last_message.clone(),
                                });
                            } else if line.contains("Installing") && current_progress < 80 {
                                current_progress = 80;
                                last_message = "Installing system image...".to_string();
                                progress_callback(SdkInstallStatus::Installing {
                                    progress: current_progress,
                                    message: last_message.clone(),
                                });
                            } else if line.contains("%") {
                                // Try to extract percentage
                                if let Some(percent) = extract_percentage(&line) {
                                    // Round to nearest 5%
                                    let rounded_percent = ((percent + 2) / 5) * 5;
                                    // Map download percentage (0-100%) to progress (20-80%)
                                    let mapped_progress = 20 + (rounded_percent * 60 / 100).min(60);

                                    // Only update if progress changed by at least 5%
                                    if mapped_progress >= current_progress + 5 || mapped_progress < current_progress {
                                        current_progress = mapped_progress;
                                        last_message = format!("Downloading... {}%", rounded_percent);

                                        progress_callback(SdkInstallStatus::Installing {
                                            progress: current_progress,
                                            message: last_message.clone(),
                                        });
                                    }
                                }
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            log::warn!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            log::debug!("sdkmanager stderr: {}", line);

                            // Check for errors
                            if line.contains("Error:") || line.contains("Failed") {
                                progress_callback(SdkInstallStatus::Failed {
                                    error: line.clone(),
                                });
                            }
                        }
                        Ok(None) => break, // EOF
                        Err(e) => {
                            log::warn!("Error reading stderr: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child
            .wait()
            .await
            .context("Failed to wait for sdkmanager")?;

        if !status.success() {
            let error_msg = format!("sdkmanager exited with status: {}", status);
            progress_callback(SdkInstallStatus::Failed {
                error: error_msg.clone(),
            });
            return Err(anyhow::anyhow!(error_msg));
        }

        progress_callback(SdkInstallStatus::Installing {
            progress: 95,
            message: "Verifying installation...".to_string(),
        });

        // Verify the system image was installed
        let packages = self.list_sdk_packages().await?;
        let installed = packages
            .iter()
            .any(|p| p.name == package_name && p.installed);

        if !installed {
            let error_msg = "System image installation verification failed";
            progress_callback(SdkInstallStatus::Failed {
                error: error_msg.to_string(),
            });
            return Err(anyhow::anyhow!(error_msg));
        }

        progress_callback(SdkInstallStatus::Completed);
        log::info!("Successfully installed API Level {}", api_level);
        Ok(())
    }

    /// Uninstalls a system image for a specific API level.
    ///
    /// # Arguments
    /// * `api_level` - The API level to uninstall (e.g., 34 for Android 14)
    ///
    /// # Returns
    /// - `Ok(())` - If uninstallation succeeds
    /// - `Err` - If uninstallation fails
    pub async fn uninstall_api_level(&self, api_level: u32) -> Result<()> {
        // Find the installed system image for this API level
        let packages = self.list_sdk_packages().await?;
        let system_image_package = packages
            .iter()
            .find(|p| {
                p.installed
                    && p.name.starts_with("system-images;android-")
                    && p.name.contains(&format!("android-{}", api_level))
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No installed system image found for API level {}",
                    api_level
                )
            })?;

        let package_name = system_image_package.name.clone();
        let sdkmanager_path = self.find_sdkmanager_tool()?;

        log::info!(
            "Uninstalling system image for API Level {} ({})",
            api_level,
            package_name
        );

        // Uninstall the system image package
        let output = self
            .command_runner
            .run(&sdkmanager_path, &["--uninstall", &package_name])
            .await
            .context(format!(
                "Failed to uninstall system image for API level {}",
                api_level
            ))?;

        log::debug!("sdkmanager uninstall output: {}", output);

        // Verify uninstallation
        if output.contains("Warning:") || output.contains("Error:") {
            return Err(anyhow::anyhow!(
                "SDK uninstallation completed with warnings/errors: {}",
                output
            ));
        }

        log::info!(
            "Successfully uninstalled system image for API Level {}",
            api_level
        );
        Ok(())
    }

    /// Accepts all Android SDK licenses automatically.
    ///
    /// # Returns
    /// - `Ok(())` - If licenses accepted successfully
    /// - `Err` - If license acceptance fails
    pub async fn accept_licenses(&self) -> Result<()> {
        let sdkmanager_path = self.find_sdkmanager_tool()?;

        log::debug!("Accepting Android SDK licenses");

        // Use 'yes' command to automatically accept all licenses
        let output = self
            .command_runner
            .run_with_input(
                &sdkmanager_path,
                &["--licenses"],
                "y\ny\ny\ny\ny\ny\ny\ny\ny\ny\n",
            )
            .await
            .context("Failed to accept SDK licenses")?;

        log::debug!("License acceptance output: {}", output);
        Ok(())
    }

    /// Finds the sdkmanager tool path.
    ///
    /// # Returns
    /// - `Ok(PathBuf)` - Path to sdkmanager
    /// - `Err` - If sdkmanager not found
    fn find_sdkmanager_tool(&self) -> Result<PathBuf> {
        let android_home = Self::find_android_home()?;
        Self::find_tool(&android_home, "sdkmanager")
    }

    /// Parses SDK package information from sdkmanager output.
    ///
    /// The format is:
    /// ```text
    /// package-name
    ///     Description:        description text
    ///     Version:            version number
    /// ```
    ///
    /// # Arguments
    /// * `lines` - Iterator over lines from sdkmanager output
    /// * `package_name` - The package name found on the current line
    ///
    /// # Returns
    /// - `Some(SdkPackage)` - If package info can be parsed
    /// - `None` - If parsing fails
    fn parse_sdk_package_info(
        &self,
        lines: &mut std::iter::Peekable<std::str::Lines>,
        package_name: String,
    ) -> Option<SdkPackage> {
        let mut description = String::new();
        let mut version = String::new();

        // Look ahead at the next lines for Description and Version
        while let Some(&next_line) = lines.peek() {
            let trimmed = next_line.trim();

            if trimmed.starts_with("Description:") {
                lines.next(); // Consume the line
                description = trimmed.strip_prefix("Description:")?.trim().to_string();
            } else if trimmed.starts_with("Version:") {
                lines.next(); // Consume the line
                version = trimmed.strip_prefix("Version:")?.trim().to_string();
            } else if trimmed.is_empty() {
                lines.next(); // Consume empty lines
            } else {
                // We've hit the next package or some other content
                break;
            }
        }

        // We need at least a version to consider this valid
        if !version.is_empty() {
            Some(SdkPackage {
                name: package_name,
                version,
                description,
                installed: false, // Will be set by calling method based on section
            })
        } else {
            None
        }
    }

    /// Extracts API level information from a system image package.
    ///
    /// # Arguments
    /// * `package` - SDK package containing system image information
    ///
    /// # Returns
    /// - `Some(ApiLevel)` - If package contains valid API level
    /// - `None` - If package cannot be parsed
    fn extract_system_image_info(&self, package: &SdkPackage) -> Option<ApiLevel> {
        // Extract API level from system image package name
        // (e.g., "system-images;android-34;google_apis;x86_64")
        if let Some(captures) = regex::Regex::new(r"system-images;android-(\d+);([^;]+);(.+)")
            .ok()?
            .captures(&package.name)
        {
            let level_str = captures.get(1)?.as_str();
            let level = level_str.parse::<u32>().ok()?;
            let tag = captures.get(2)?.as_str();
            let abi = captures.get(3)?.as_str();

            // Create a short descriptive version name
            let version_name = match level {
                35 => "Android 15".to_string(),
                34 => "Android 14".to_string(),
                33 => "Android 13".to_string(),
                32 => "Android 12L".to_string(),
                31 => "Android 12".to_string(),
                30 => "Android 11".to_string(),
                29 => "Android 10".to_string(),
                28 => "Android 9".to_string(),
                _ => format!("API {}", level),
            };

            // Add architecture info for better understanding
            let arch_info = if abi.contains("x86_64") {
                " (x86_64)"
            } else if abi.contains("x86") {
                " (x86)"
            } else if abi.contains("arm64") {
                " (ARM64)"
            } else {
                ""
            };

            let final_version_name = format!("{}{}", version_name, arch_info);

            // Determine image type
            let image_type = if tag.contains("google_apis_playstore") {
                Some("Google Play".to_string())
            } else if tag.contains("google_apis") {
                Some("Google APIs".to_string())
            } else if tag == "default" {
                Some("AOSP".to_string())
            } else if tag.contains("android-tv") {
                Some("Android TV".to_string())
            } else if tag.contains("android-wear") {
                Some("Wear OS".to_string())
            } else if tag.contains("android-automotive") {
                Some("Automotive".to_string())
            } else {
                Some(tag.to_string())
            };

            Some(ApiLevel {
                level,
                version_name: final_version_name,
                package_name: package.name.clone(),
                installed: package.installed,
                image_type,
            })
        } else {
            None
        }
    }

    /// Determines which system image is better based on current system architecture and preferences.
    ///
    /// # Arguments
    /// * `new_package` - New system image package name
    /// * `existing_package` - Existing system image package name
    ///
    /// # Returns
    /// - `true` - If new package is better than existing
    /// - `false` - If existing package is better
    fn is_better_system_image(&self, new_package: &str, existing_package: &str) -> bool {
        let new_score = self.score_system_image(new_package);
        let existing_score = self.score_system_image(existing_package);
        new_score > existing_score
    }

    /// Scores a system image package based on compatibility and preference.
    /// Higher scores are better.
    ///
    /// # Arguments
    /// * `package_name` - System image package name
    ///
    /// # Returns
    /// - `i32` - Score for the package (higher is better)
    fn score_system_image(&self, package_name: &str) -> i32 {
        let mut score = 0;

        // Prefer Google APIs over others (for compatibility with Google Play services)
        if package_name.contains("google_apis_playstore") {
            score += 100; // Best: Has Google Play Store
        } else if package_name.contains("google_apis") {
            score += 80; // Good: Has Google APIs
        } else if package_name.contains("default") {
            score += 60; // OK: Default AOSP
        } else {
            score += 40; // Special images (TV, Wear, etc.)
        }

        // Prefer native architecture
        #[cfg(target_arch = "x86_64")]
        {
            if package_name.contains("x86_64") {
                score += 50; // Native architecture
            } else if package_name.contains("x86") {
                score += 30; // Compatible architecture
            } else {
                score += 10; // ARM (emulated, slower)
            }
        }

        #[cfg(target_arch = "x86")]
        {
            if package_name.contains("x86") && !package_name.contains("x86_64") {
                score += 50; // Native architecture
            } else if package_name.contains("x86_64") {
                score += 20; // Compatible but may have issues
            } else {
                score += 10; // ARM (emulated, slower)
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if package_name.contains("arm64-v8a") {
                score += 50; // Native architecture
            } else if package_name.contains("armeabi") {
                score += 30; // Compatible architecture
            } else {
                score += 10; // x86 (emulated, slower)
            }
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
        {
            // For other architectures, prefer x86_64 as it's most common
            if package_name.contains("x86_64") {
                score += 30;
            } else if package_name.contains("x86") {
                score += 20;
            } else {
                score += 10;
            }
        }

        score
    }

    /// Finds the best system image package for a specific API level.
    ///
    /// # Arguments
    /// * `api_level` - The API level to find a system image for
    ///
    /// # Returns
    /// - `Ok(String)` - The best system image package name
    /// - `Err` - If no suitable system image is found
    async fn find_best_system_image_for_api_level(&self, api_level: u32) -> Result<String> {
        let packages = self.list_sdk_packages().await?;
        let mut best_package: Option<String> = None;
        let mut best_score = 0;

        for package in packages {
            if package
                .name
                .starts_with(&format!("system-images;android-{};", api_level))
            {
                let score = self.score_system_image(&package.name);
                if score > best_score {
                    best_score = score;
                    best_package = Some(package.name);
                }
            }
        }

        best_package.ok_or_else(|| {
            anyhow::anyhow!(
                "No system image found for API level {}. Available system images can be listed with 'sdkmanager --list'",
                api_level
            )
        })
    }

    /// Lists all available Android device types.
    pub async fn list_available_devices(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_runner
            .run(&self.avdmanager_path, &["list", "device"])
            .await
            .context("Failed to list Android devices")?;

        let mut devices = Vec::new();
        let mut current_device_id = None;
        let mut current_device_name = None;

        for line in output.lines() {
            let trimmed = line.trim();
            if let Some(captures) = ID_REGEX.captures(trimmed) {
                current_device_id = Some(captures.get(1).unwrap().as_str().to_string());
            } else if let Some(captures) = NAME_REGEX.captures(trimmed) {
                current_device_name = Some(captures.get(1).unwrap().as_str().to_string());
            } else if trimmed.starts_with("--------")
                && current_device_id.is_some()
                && current_device_name.is_some()
            {
                devices.push((
                    current_device_id.take().unwrap(),
                    current_device_name.take().unwrap(),
                ));
            }
        }

        // Don't forget the last device if there's no trailing separator
        if let (Some(id), Some(name)) = (current_device_id, current_device_name) {
            devices.push((id, name));
        }

        Ok(devices)
    }

    /// Gets the appropriate skin for a device.
    async fn get_appropriate_skin(&self, device_id: &str, _device_type: &str) -> Option<String> {
        // Try to find a matching skin for the device
        // Common skins include: pixel_4, pixel_5, etc.
        if device_id.contains("pixel") {
            Some(device_id.to_string())
        } else if device_id.contains("wear") {
            Some("wear_round".to_string())
        } else if device_id.contains("tv") {
            Some("tv_1080p".to_string())
        } else {
            None
        }
    }

    /// Gets the first available system image for an API level.
    async fn get_first_available_system_image(
        &self,
        api_level: &str,
    ) -> Result<Option<(String, String)>> {
        let packages = self.list_sdk_packages().await?;

        for package in packages {
            if package.installed
                && package
                    .name
                    .starts_with(&format!("system-images;android-{};", api_level))
            {
                // Parse the package name to extract tag and abi
                let parts: Vec<&str> = package.name.split(';').collect();
                if parts.len() >= 4 {
                    return Ok(Some((parts[2].to_string(), parts[3].to_string())));
                }
            }
        }

        Ok(None)
    }

    /// Checks if a system image is available.
    async fn check_system_image_available(
        &self,
        api_level: &str,
        tag: &str,
        abi: &str,
    ) -> Result<bool> {
        let packages = self.list_sdk_packages().await?;
        let package_name = format!("system-images;android-{};{};{}", api_level, tag, abi);

        Ok(packages
            .iter()
            .any(|p| p.name == package_name && p.installed))
    }

    /// Fine-tunes AVD configuration after creation.
    async fn fine_tune_avd_config(
        &self,
        avd_name: &str,
        config: &DeviceConfig,
        _tag: &str,
        _abi: &str,
    ) -> Result<()> {
        if let Ok(home_dir) = std::env::var("HOME") {
            let config_path = std::path::PathBuf::from(home_dir)
                .join(".android")
                .join("avd")
                .join(format!("{}.avd", avd_name))
                .join("config.ini");

            if config_path.exists() {
                let mut content = fs::read_to_string(&config_path).await?;

                // Update RAM size if specified
                if let Some(ram) = config.additional_options.get("ram") {
                    content = content.replace("hw.ramSize=2048", &format!("hw.ramSize={}", ram));
                }

                // Update storage size if specified
                if let Some(storage) = config.additional_options.get("storage") {
                    let storage_mb = storage.parse::<u32>().unwrap_or(8192);
                    content = content.replace(
                        "disk.dataPartition.size=8192M",
                        &format!("disk.dataPartition.size={}M", storage_mb),
                    );
                }

                // Add display name if original name had spaces
                if config.name != avd_name && !content.contains("avd.ini.displayname=") {
                    content.push_str(&format!("\navd.ini.displayname={}", config.name));
                }

                fs::write(&config_path, content).await?;
            }
        }

        Ok(())
    }

    /// Finds a matching device ID from available devices.
    fn find_matching_device_id(
        available_devices: &[(String, String)],
        device_type: &str,
    ) -> Option<String> {
        let device_type_lower = device_type.to_lowercase();

        // Try exact match first
        for (id, name) in available_devices {
            if name.to_lowercase() == device_type_lower {
                return Some(id.clone());
            }
        }

        // Try contains match
        for (id, name) in available_devices {
            if name.to_lowercase().contains(&device_type_lower)
                || device_type_lower.contains(&name.to_lowercase())
            {
                return Some(id.clone());
            }
        }

        // Try ID match
        for (id, _) in available_devices {
            if id.to_lowercase() == device_type_lower {
                return Some(id.clone());
            }
        }

        None
    }

    /// Lists all available system images.
    pub async fn list_available_system_images(&self) -> Result<Vec<String>> {
        let packages = self.list_sdk_packages().await?;
        let mut images = Vec::new();

        for package in packages {
            if package.installed && package.name.starts_with("system-images;") {
                images.push(package.name);
            }
        }

        Ok(images)
    }

    /// Lists available API targets.
    pub async fn list_available_targets(&self) -> Result<Vec<(String, String)>> {
        let api_levels = self.list_available_api_levels().await?;
        Ok(api_levels
            .into_iter()
            .map(|api| (api.level.to_string(), api.version_name))
            .collect())
    }

    /// Lists devices filtered by category as name/type pairs.
    pub async fn list_devices_by_category(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<(String, String)>> {
        if let Some(cat) = category {
            let category_str = cat.to_lowercase();
            // Get available device types
            let device_types = self.list_available_devices().await?;
            Ok(device_types
                .into_iter()
                .filter(|(device_id, display_name)| {
                    let device_category = self.get_device_category(device_id, display_name);
                    match device_category {
                        DeviceCategory::Phone => {
                            category_str == "phone" || category_str == "generic"
                        }
                        DeviceCategory::Tablet => category_str == "tablet",
                        DeviceCategory::TV => category_str == "tv",
                        DeviceCategory::Wear => {
                            category_str == "wear" || category_str == "wearable"
                        }
                        DeviceCategory::Automotive => {
                            category_str == "automotive" || category_str == "auto"
                        }
                        DeviceCategory::Foldable => {
                            category_str == "foldable" || category_str == "desktop"
                        }
                        DeviceCategory::Unknown => category_str == "unknown",
                    }
                })
                .collect())
        } else {
            // Return all available device types
            self.list_available_devices().await
        }
    }

    /// Gets device category from device type and name.
    pub fn get_device_category(&self, device_id: &str, display_name: &str) -> DeviceCategory {
        let id_lower = device_id.to_lowercase();
        let name_lower = display_name.to_lowercase();

        // Check for specific categories based on keywords
        if id_lower.contains("tv") || name_lower.contains("tv") {
            DeviceCategory::TV
        } else if id_lower.contains("wear")
            || name_lower.contains("wear")
            || name_lower.contains("watch")
        {
            DeviceCategory::Wear
        } else if id_lower.contains("automotive")
            || id_lower.contains("auto")
            || name_lower.contains("automotive")
        {
            DeviceCategory::Automotive
        } else if id_lower.contains("tablet")
            || name_lower.contains("tablet")
            || name_lower.contains("pad")
        {
            DeviceCategory::Tablet
        } else if id_lower.contains("desktop")
            || name_lower.contains("desktop")
            || id_lower.contains("foldable")
        {
            DeviceCategory::Foldable
        } else {
            // Default to Phone for most common devices
            DeviceCategory::Phone
        }
    }

    /// Gets detailed device information.
    pub async fn get_device_details(&self, identifier: &str) -> Result<DeviceDetails> {
        use crate::app::state::DeviceDetails;

        // Get AVD path
        let home_dir = std::env::var("HOME")?;
        let avd_path = std::path::PathBuf::from(&home_dir)
            .join(".android")
            .join("avd")
            .join(format!("{}.avd", identifier));

        let config_path = avd_path.join("config.ini");

        let mut ram_mb = 2048;
        let mut storage_mb = 8192;
        let mut api_level = 0;
        let mut device_type = "Unknown".to_string();

        if let Ok(content) = fs::read_to_string(&config_path).await {
            for line in content.lines() {
                if line.starts_with("hw.ramSize=") {
                    if let Some(value) = line.split('=').nth(1) {
                        ram_mb = value.parse().unwrap_or(2048);
                    }
                } else if line.contains("disk.dataPartition.size=") {
                    if let Some(value) = line.split('=').nth(1) {
                        let size_str = value.trim_end_matches('M');
                        storage_mb = size_str.parse().unwrap_or(8192);
                    }
                } else if line.starts_with("hw.device.name=") {
                    if let Some(value) = line.split('=').nth(1) {
                        device_type = value.to_string();
                    }
                } else if let Some(captures) = IMAGE_SYSDIR_REGEX.captures(line) {
                    if let Some(api) = captures.get(1) {
                        api_level = api.as_str().parse().unwrap_or(0);
                    }
                } else if let Some(captures) = TARGET_CONFIG_REGEX.captures(line) {
                    if let Some(api) = captures.get(1) {
                        api_level = api.as_str().parse().unwrap_or(0);
                    }
                }
            }
        }

        Ok(DeviceDetails {
            name: identifier.to_string(),
            status: "Unknown".to_string(), // Will be updated by caller
            platform: crate::app::state::Panel::Android,
            device_type,
            api_level_or_version: api_level.to_string(),
            ram_size: Some(format!("{} MB", ram_mb)),
            storage_size: Some(format!("{} MB", storage_mb)),
            resolution: None,
            dpi: None,
            device_path: Some(avd_path.to_string_lossy().to_string()),
            system_image: None,
            identifier: identifier.to_string(),
        })
    }
}

use crate::app::state::DeviceDetails;

#[allow(clippy::manual_async_fn)]
impl DeviceManager for AndroidManager {
    type Device = AndroidDevice;

    fn list_devices(&self) -> impl std::future::Future<Output = Result<Vec<Self::Device>>> + Send {
        async {
            // Get both AVDs and physical devices
            let mut all_devices = Vec::new();

            // Get AVDs
            let avds = self.list_avds().await?;
            all_devices.extend(avds);

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
