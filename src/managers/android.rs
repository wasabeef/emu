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
    constants::{commands, defaults, env_vars, files},
    managers::common::{DeviceConfig, DeviceManager},
    models::device_info::{
        ApiLevelInfo, DeviceCategory, DeviceInfo, DynamicDeviceConfig, DynamicDeviceProvider,
    },
    models::{AndroidDevice, DeviceStatus},
    utils::command::CommandRunner,
};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
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
    android_home: PathBuf,
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
            android_home,
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
                .join(files::android::CMDLINE_TOOLS_LATEST_BIN)
                .join(tool),
            android_home.join(files::android::TOOLS_BIN).join(tool),
            android_home.join(files::android::EMULATOR_DIR).join(tool),
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
            .run(commands::ADB, &[commands::adb::DEVICES])
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

    fn parse_android_version_to_api_level(version: &str) -> u32 {
        // Extract major version number from strings like "15.0", "14.0", etc.
        let major_version = version
            .split('.')
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);

        // Map Android version to API level
        match major_version {
            15 => 35,
            14 => 34,
            13 => 33,
            12 => 32,
            11 => 30,
            10 => 29,
            9 => 28,
            8 => 26,
            7 => 24,
            6 => 23,
            5 => 21,
            4 => 15,
            _ => major_version, // Fallback to version number
        }
    }

    /// List available Android targets (API levels) based on installed system images
    pub async fn list_available_targets(&self) -> Result<Vec<(String, String)>> {
        // Get actually installed system images
        let installed_images = self.list_available_system_images().await?;

        // Also track available tags/ABIs for each API level
        let mut api_level_info: std::collections::HashMap<
            String,
            std::collections::HashSet<String>,
        > = std::collections::HashMap::new();

        let mut targets = std::collections::HashMap::new();

        // Extract API levels and their available tags/ABIs from installed system images
        for image in installed_images {
            // Parse "system-images;android-XX;tag;abi" format
            let parts: Vec<&str> = image.split(';').collect();
            if parts.len() >= 4 {
                if let Some(api_level) = parts.get(1).and_then(|p| p.strip_prefix("android-")) {
                    // Track available tags for this API level
                    let tag_abi = format!("{};{}", parts[2], parts[3]);
                    api_level_info
                        .entry(api_level.to_string())
                        .or_default()
                        .insert(tag_abi);

                    // Use simple version mapping to avoid recursion
                    let api_num: u32 = api_level.parse().unwrap_or(0);
                    let android_version = self.get_android_version_name(api_num);

                    let display = format!("API {api_level} - {android_version}");
                    targets.insert(api_level.to_string(), display);
                }
            }
        }

        // Convert to sorted vector
        let mut result: Vec<(String, String)> = targets.into_iter().collect();

        // Sort by API level (descending)
        result.sort_by(|a, b| {
            let api_a: u32 = a.0.parse().unwrap_or(0);
            let api_b: u32 = b.0.parse().unwrap_or(0);
            api_b.cmp(&api_a)
        });

        Ok(result)
    }

    /// Lists all available Android device definitions from the SDK.
    ///
    /// Parses output from `avdmanager list device` to discover all device types
    /// that can be used to create AVDs. Returns devices sorted by priority
    /// (phones first, then tablets, etc.) and version.
    ///
    /// # Returns
    /// Vector of tuples containing:
    /// - Device ID (e.g., "pixel_7")
    /// - Display name (e.g., "Pixel 7")
    ///
    /// # Device Priority
    /// Devices are sorted using dynamic priority calculation based on:
    /// 1. Category (phone > tablet > tv > wear > automotive)
    /// 2. Version number (newer versions first)
    /// 3. Manufacturer (Google/Pixel prioritized)
    pub async fn list_available_devices(&self) -> Result<Vec<(String, String)>> {
        let output = self
            .command_runner
            .run(&self.avdmanager_path, &["list", "device"])
            .await
            .context("Failed to list Android devices")?;

        let mut devices = Vec::new();
        let mut current_id = String::new();
        let mut current_name = String::new();
        let mut current_oem = String::new();

        for line in output.lines() {
            if let Some(caps) = ID_REGEX.captures(line) {
                current_id = caps[1].to_string();
            } else if let Some(caps) = NAME_REGEX.captures(line) {
                current_name = caps[1].to_string();
            } else if let Some(caps) = OEM_REGEX.captures(line) {
                current_oem = caps[1].to_string();
            } else if line.contains("-----") && !current_id.is_empty() {
                // Create display string
                let display = if !current_oem.is_empty() && current_oem != "Generic" {
                    format!("{} ({})", current_name, current_oem)
                } else {
                    current_name.clone()
                };

                devices.push((current_id.clone(), display));

                // Reset for next device
                current_id.clear();
                current_name.clear();
                current_oem.clear();
            }
        }

        // Don't forget the last device if exists
        if !current_id.is_empty() {
            let display = if !current_oem.is_empty() && current_oem != "Generic" {
                format!("{} ({})", current_name, current_oem)
            } else {
                current_name.clone()
            };
            devices.push((current_id, display));
        }

        // If no devices found, log warning but don't add hardcoded fallbacks
        if devices.is_empty() {
            log::warn!(
                "No Android device definitions found. Please check your Android SDK installation."
            );
        }

        // Sort devices by priority (Pixel devices first, then by version)
        devices.sort_by(|a, b| {
            let priority_a = DynamicDeviceConfig::calculate_android_device_priority(&a.0, &a.1);
            let priority_b = DynamicDeviceConfig::calculate_android_device_priority(&b.0, &b.1);
            priority_a.cmp(&priority_b)
        });

        Ok(devices)
    }

    /// Dynamically determine device category
    pub fn get_device_category(&self, device_id: &str, device_display: &str) -> String {
        let combined = format!(
            "{} {}",
            device_id.to_lowercase(),
            device_display.to_lowercase()
        );

        // Phone - most common device type
        if combined.contains("phone") || 
           combined.contains("pixel") && !combined.contains("fold") && !combined.contains("tablet") ||
           combined.contains("galaxy") && !combined.contains("fold") && !combined.contains("tablet") ||
           combined.contains("oneplus") ||
           combined.contains("iphone") ||
           // Determine by screen size (smartphone range)
           (combined.contains("5") && combined.contains("inch")) ||
           (combined.contains("6") && combined.contains("inch")) ||
           (combined.contains("pro") && !combined.contains("tablet") && !combined.contains("fold"))
        {
            return "phone".to_string();
        }

        // Tablet - tablet devices
        if combined.contains("tablet")
            || combined.contains("pad")
            || (combined.contains("10") && combined.contains("inch"))
            || (combined.contains("11") && combined.contains("inch"))
            || (combined.contains("12") && combined.contains("inch"))
            || (combined.contains("13") && combined.contains("inch"))
        {
            return "tablet".to_string();
        }

        // Wear - wearable devices
        if combined.contains("wear")
            || combined.contains("watch")
            || combined.contains("round") && !combined.contains("tablet")
            || combined.contains("square") && !combined.contains("tablet")
        {
            return "wear".to_string();
        }

        // TV - television devices
        if combined.contains("tv")
            || combined.contains("1080p")
            || combined.contains("4k")
            || combined.contains("720p")
        {
            return "tv".to_string();
        }

        // Automotive - automotive devices
        if combined.contains("auto") || combined.contains("car") || combined.contains("automotive")
        {
            return "automotive".to_string();
        }

        // Desktop - desktop/large screen devices
        if combined.contains("desktop")
            || combined.contains("foldable") && combined.contains("large")
            || (combined.contains("15") && combined.contains("inch"))
            || (combined.contains("17") && combined.contains("inch"))
        {
            return "desktop".to_string();
        }

        // Default is phone (most common)
        "phone".to_string()
    }

    /// Get device list filtered by category
    pub async fn list_devices_by_category(
        &self,
        category: Option<&str>,
    ) -> Result<Vec<(String, String)>> {
        let all_devices = self.list_available_devices().await?;

        if let Some(filter_category) = category {
            if filter_category == "all" {
                return Ok(all_devices);
            }

            let filtered_devices: Vec<(String, String)> = all_devices
                .into_iter()
                .filter(|(id, display)| {
                    let device_category = self.get_device_category(id, display);
                    device_category == filter_category
                })
                .collect();

            Ok(filtered_devices)
        } else {
            Ok(all_devices)
        }
    }

    /// Check if a system image is available for the given API level, tag, and ABI
    pub async fn check_system_image_available(
        &self,
        api_level: &str,
        tag: &str,
        abi: &str,
    ) -> Result<bool> {
        let package_path = format!("system-images;android-{api_level};{tag};{abi}");

        let installed_images = self.list_available_system_images().await?;
        let is_installed = installed_images.contains(&package_path);

        Ok(is_installed)
    }

    /// Get a list of available system images
    pub async fn list_available_system_images(&self) -> Result<Vec<String>> {
        let mut images = Vec::new();

        if let Ok(sdkmanager_path) = Self::find_tool(&self.android_home, "sdkmanager") {
            let output = self
                .command_runner
                .run(
                    &sdkmanager_path,
                    &["--list", "--verbose", "--include_obsolete"],
                )
                .await?;

            let mut in_installed_section = false;

            for line in output.lines() {
                let trimmed = line.trim();

                // Track when we're in the installed packages section
                if trimmed.starts_with("Installed packages:") {
                    in_installed_section = true;
                    continue;
                }

                // Track when we exit the installed section
                if in_installed_section
                    && (trimmed.starts_with("Available Packages:")
                        || trimmed.starts_with("Available Updates:"))
                {
                    in_installed_section = false;
                    continue;
                }

                // Only process lines in the installed section
                if in_installed_section && trimmed.starts_with("system-images;") {
                    // Parse the line to extract just the package path
                    if let Some(space_pos) = trimmed.find(' ') {
                        let package_path = &trimmed[..space_pos];
                        images.push(package_path.to_string());
                    } else {
                        // If no space found, the whole line might be the package path
                        images.push(trimmed.to_string());
                    }
                }
            }
        }

        Ok(images)
    }

    /// Get the first available system image for a given API level
    pub async fn get_first_available_system_image(
        &self,
        api_level: &str,
    ) -> Result<Option<(String, String)>> {
        let installed_images = self.list_available_system_images().await?;

        // Find system images for this API level
        for image in installed_images {
            let parts: Vec<&str> = image.split(';').collect();
            if parts.len() >= 4 {
                if let Some(android_part) = parts.get(1) {
                    if android_part == &format!("android-{api_level}") {
                        // Return first available tag and abi
                        return Ok(Some((parts[2].to_string(), parts[3].to_string())));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get the AVD directory path for a given AVD name
    async fn get_avd_path(&self, avd_name: &str) -> Result<Option<PathBuf>> {
        let avd_output = self
            .command_runner
            .run(&self.avdmanager_path, &["list", "avd"])
            .await
            .context("Failed to list Android AVDs")?;

        let mut current_name = String::new();

        for line in avd_output.lines() {
            let trimmed = line.trim();
            if let Some(caps) = AVD_NAME_REGEX.captures(trimmed) {
                current_name = caps[1].to_string();
            } else if let Some(caps) = PATH_REGEX.captures(trimmed) {
                if current_name == avd_name {
                    return Ok(Some(PathBuf::from(caps[1].to_string())));
                }
            }
        }

        Ok(None)
    }

    /// Get display name for a device type using dynamic parsing
    fn get_device_display_name(&self, device_type: &str) -> String {
        // Dynamic display name generation without hardcoded patterns
        let display_name = device_type
            .replace("_", " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        if display_name.is_empty() {
            "Generic Device".to_string()
        } else {
            display_name
        }
    }

    /// Fine-tune AVD configuration after creation with avdmanager
    async fn fine_tune_avd_config(
        &self,
        avd_name: &str,
        config: &DeviceConfig,
        _tag: &str,
        _abi: &str,
    ) -> Result<()> {
        if let Some(avd_path) = self.get_avd_path(avd_name).await? {
            let config_path = avd_path.join("config.ini");

            // Read existing config created by avdmanager
            let mut config_content = fs::read_to_string(&config_path)
                .await
                .context("Failed to read existing AVD configuration")?;

            // Use the user's original name instead of generating from device type
            let device_display_name = &config.name;

            // Only override specific settings if needed
            let ram_mb = if let Some(ram) = &config.ram_size {
                ram.parse::<u32>().unwrap_or(0)
            } else {
                0
            };

            let storage_mb = if let Some(storage) = &config.storage_size {
                storage.parse::<u32>().unwrap_or(0)
            } else {
                0
            };

            // Create AvdId format: Pixel_9_Pro_Fold
            let avd_id = device_display_name.replace(" ", "_");

            // Add or update avd.ini.displayname (with spaces)
            if !device_display_name.is_empty() {
                if config_content.contains("avd.ini.displayname=") {
                    // Update existing line
                    if let Some(start) = config_content.find("avd.ini.displayname=") {
                        if let Some(end) = config_content[start..].find('\n') {
                            let line_end = start + end;
                            config_content.replace_range(
                                start..line_end,
                                &format!("avd.ini.displayname={}", device_display_name),
                            );
                        }
                    }
                } else {
                    // Add new line after encoding line
                    if let Some(encoding_pos) = config_content.find("avd.ini.encoding=UTF-8\n") {
                        let insert_pos = encoding_pos + "avd.ini.encoding=UTF-8\n".len();
                        config_content.insert_str(
                            insert_pos,
                            &format!("avd.ini.displayname={}\n", device_display_name),
                        );
                    } else {
                        // Add at the beginning
                        config_content = format!(
                            "avd.ini.displayname={}\navd.ini.encoding=UTF-8\n{}",
                            device_display_name, config_content
                        );
                    }
                }
            }

            // Add or update AvdId (with underscores)
            if !avd_id.is_empty() {
                if config_content.contains("AvdId=") {
                    // Update existing line
                    if let Some(start) = config_content.find("AvdId=") {
                        if let Some(end) = config_content[start..].find('\n') {
                            let line_end = start + end;
                            config_content
                                .replace_range(start..line_end, &format!("AvdId={}", avd_id));
                        }
                    }
                } else {
                    // Add new line after displayname
                    if let Some(displayname_pos) = config_content.find("avd.ini.displayname=") {
                        if let Some(line_end) = config_content[displayname_pos..].find('\n') {
                            let insert_pos = displayname_pos + line_end + 1;
                            config_content.insert_str(insert_pos, &format!("AvdId={}\n", avd_id));
                        }
                    }
                }
            }

            // Update RAM size if specified
            if ram_mb > 0 {
                if let Some(start) = config_content.find("hw.ramSize=") {
                    if let Some(end) = config_content[start..].find('\n') {
                        let line_end = start + end;
                        config_content
                            .replace_range(start..line_end, &format!("hw.ramSize={}", ram_mb));
                    }
                }
            }

            // Update storage size if specified
            if storage_mb > 0 {
                if let Some(start) = config_content.find("disk.dataPartition.size=") {
                    if let Some(end) = config_content[start..].find('\n') {
                        let line_end = start + end;
                        config_content.replace_range(
                            start..line_end,
                            &format!("disk.dataPartition.size={}G", storage_mb / 1024),
                        );
                    }
                }
            }

            // Ensure image.sysdir.1 has trailing slash for consistency
            if config_content.contains("image.sysdir.1=")
                && !config_content.contains("image.sysdir.1=system-images/android-")
            {
                // This should not happen, but add safety check
            } else if let Some(start) = config_content.find("image.sysdir.1=") {
                if let Some(end) = config_content[start..].find('\n') {
                    let line = &config_content[start..start + end];
                    if !line.ends_with('/') {
                        let line_end = start + end;
                        config_content.replace_range(start..line_end, &format!("{}/", line));
                    }
                }
            }

            // Write the updated configuration
            fs::write(&config_path, config_content)
                .await
                .context("Failed to write updated AVD configuration")?;
        }
        Ok(())
    }

    /// Get detailed information for a specific AVD
    pub async fn get_device_details(
        &self,
        avd_name: &str,
    ) -> Result<crate::app::state::DeviceDetails> {
        // Get basic device info from list
        let devices = self.list_devices().await?;
        let device = devices
            .iter()
            .find(|d| d.name == avd_name)
            .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", avd_name))?;

        // Read AVD configuration for detailed info
        let mut details = crate::app::state::DeviceDetails {
            name: device.name.clone(),
            status: if device.is_running {
                "Running".to_string()
            } else {
                "Stopped".to_string()
            },
            platform: crate::app::Panel::Android,
            device_type: device.device_type.clone(),
            api_level_or_version: {
                // Try dynamic lookup first for better accuracy
                let version_name = if let Some(dynamic_version) = self
                    .get_dynamic_android_version_name(device.api_level)
                    .await
                {
                    dynamic_version
                } else {
                    // Fall back to hardcoded values
                    self.get_android_version_name(device.api_level)
                };
                format!("API {} (Android {})", device.api_level, version_name)
            },
            ram_size: None,
            storage_size: None,
            resolution: None,
            dpi: None,
            device_path: None,
            system_image: None,
            identifier: device.name.clone(),
        };

        // Read config.ini for additional details
        if let Ok(home_dir) = std::env::var("HOME") {
            let config_path = std::path::PathBuf::from(home_dir)
                .join(".android")
                .join("avd")
                .join(format!("{}.avd", avd_name))
                .join("config.ini");

            if config_path.exists() {
                if let Ok(config_content) = tokio::fs::read_to_string(&config_path).await {
                    // Parse configuration values
                    for line in config_content.lines() {
                        if let Some((key, value)) = line.split_once('=') {
                            match key.trim() {
                                "hw.ramSize" => {
                                    if let Ok(ram_mb) = value.trim().parse::<u64>() {
                                        details.ram_size = Some(format!("{} MB", ram_mb));
                                    }
                                }
                                "disk.dataPartition.size" => {
                                    // Parse storage size (e.g., "8192M", "4G")
                                    let value = value.trim();
                                    if let Some(size_str) = value.strip_suffix('M') {
                                        if let Ok(size_mb) = size_str.parse::<u64>() {
                                            details.storage_size = Some(format!("{} MB", size_mb));
                                        }
                                    } else if let Some(size_str) = value.strip_suffix('G') {
                                        if let Ok(size_gb) = size_str.parse::<u64>() {
                                            details.storage_size =
                                                Some(format!("{} MB", size_gb * 1024));
                                        }
                                    }
                                }
                                "hw.lcd.width" => {
                                    if let Ok(width) = value.trim().parse::<u32>() {
                                        // Need to also get height to form resolution
                                        details.resolution = Some(format!("{}x?", width));
                                    }
                                }
                                "hw.lcd.height" => {
                                    if let Ok(height) = value.trim().parse::<u32>() {
                                        // Combine with width if available
                                        if let Some(ref res) = details.resolution {
                                            if res.contains("x?") {
                                                let width = res.replace("x?", "");
                                                details.resolution =
                                                    Some(format!("{}x{}", width, height));
                                            }
                                        } else {
                                            details.resolution = Some(format!("?x{}", height));
                                        }
                                    }
                                }
                                "hw.lcd.density" => {
                                    details.dpi = Some(format!("{} DPI", value.trim()));
                                }
                                "image.sysdir.1" => {
                                    details.system_image = Some(value.trim().to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Set device path
                details.device_path =
                    Some(config_path.parent().unwrap().to_string_lossy().to_string());
            }
        }

        // Clean up resolution if we only got partial info
        if let Some(ref res) = details.resolution {
            if res.contains("?") {
                details.resolution = None;
            }
        }

        Ok(details)
    }

    /// Get Android version name from API level (with accurate mapping)
    fn get_android_version_name(&self, api_level: u32) -> String {
        match api_level {
            36 => "Android 16 Preview".to_string(), // Preview/Beta version
            35 => "Android 15".to_string(),
            34 => "Android 14".to_string(),
            33 => "Android 13".to_string(),
            32 => "Android 12L".to_string(), // Fixed: was showing "Android 32"
            31 => "Android 12".to_string(),
            30 => "Android 11".to_string(),
            29 => "Android 10".to_string(),
            28 => "Android 9".to_string(),
            27 => "Android 8.1".to_string(),
            26 => "Android 8.0".to_string(),
            25 => "Android 7.1".to_string(),
            24 => "Android 7.0".to_string(),
            23 => "Android 6.0".to_string(),
            22 => "Android 5.1".to_string(),
            21 => "Android 5.0".to_string(),
            20 => "Android 4.4W".to_string(),
            19 => "Android 4.4".to_string(),
            18 => "Android 4.3".to_string(),
            17 => "Android 4.2".to_string(),
            16 => "Android 4.1".to_string(),
            15 => "Android 4.0.3".to_string(),
            14 => "Android 4.0".to_string(),
            _ => format!("API {}", api_level), // For unknown versions, just show API level
        }
    }

    /// Get Android version name from SDK dynamically
    async fn get_dynamic_android_version_name(&self, api_level: u32) -> Option<String> {
        // Try to get from available targets list
        if let Ok(targets) = self.list_available_targets().await {
            for (level_str, display) in targets {
                if let Ok(level) = level_str.parse::<u32>() {
                    if level == api_level {
                        // Extract version from display string like "API 34 - Android 14"
                        if let Some(dash_pos) = display.find(" - Android ") {
                            return Some(display[dash_pos + 11..].to_string());
                        }
                    }
                }
            }
        }

        // Try to parse from sdkmanager output
        if let Ok(sdkmanager_path) = Self::find_tool(&self.android_home, "sdkmanager") {
            if let Ok(output) = self.command_runner.run(&sdkmanager_path, &["--list"]).await {
                // Look for platform entries like "platforms;android-34 | 1 | Android SDK Platform 34"
                let pattern = format!(
                    r"platforms;android-{}\s*\|\s*\d+\s*\|\s*Android SDK Platform",
                    api_level
                );
                if let Ok(regex) = Regex::new(&pattern) {
                    if regex.is_match(&output) {
                        // Try to extract more detailed version info from subsequent lines
                        for line in output.lines() {
                            if line.contains(&format!("android-{}", api_level))
                                && line.contains("Android")
                            {
                                // Extract version number if present
                                if let Some(version_match) = line.split("Android").nth(1) {
                                    let version = version_match
                                        .split_whitespace()
                                        .next()
                                        .unwrap_or("")
                                        .trim_matches(|c: char| !c.is_numeric() && c != '.');
                                    if !version.is_empty() {
                                        return Some(version.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get appropriate skin name for device type using dynamic lookup
    async fn get_appropriate_skin(&self, device_id: &str, device_display: &str) -> Option<String> {
        if device_id.is_empty() {
            return None;
        }

        // First, try using the device ID directly as the skin name (most common case)
        let primary_skin = device_id.to_string();

        // Dynamically get available skins from SDK
        let available_skins = self
            .get_available_skins_from_sdk(device_id)
            .await
            .unwrap_or_default();

        // 1. If there's a skin that exactly matches the device ID
        if available_skins.iter().any(|skin| skin == &primary_skin) {
            return Some(primary_skin);
        }

        // 2. Generate candidates from device display name and check
        let display_based_skin = device_display
            .split('(')
            .next() // Only the part before parentheses
            .unwrap_or(device_display)
            .trim()
            .replace(' ', "_")
            .to_lowercase();

        if available_skins
            .iter()
            .any(|skin| skin == &display_based_skin)
        {
            return Some(display_based_skin);
        }

        // 3. Search for skins with partial match
        let device_lower = device_id.to_lowercase();
        for skin in &available_skins {
            let skin_lower = skin.to_lowercase();
            // Main part of device ID is contained in skin name, or vice versa
            if (device_lower.len() > 3 && skin_lower.contains(&device_lower))
                || (skin_lower.len() > 3 && device_lower.contains(&skin_lower))
            {
                return Some(skin.clone());
            }
        }

        // 4. If all fail, return device ID as-is (handled by fallback strategy)
        Some(primary_skin)
    }

    /// Dynamically get available skins for device from SDK
    async fn get_available_skins_from_sdk(&self, _device_id: &str) -> Result<Vec<String>> {
        let mut skins = Vec::new();

        // Dynamically scan Android SDK skin directories
        if let Ok(android_home) = std::env::var(env_vars::ANDROID_HOME) {
            let android_path = std::path::PathBuf::from(&android_home);

            // 1. 標準スキンディレクトリ
            let standard_skins = android_path.join("skins");
            if standard_skins.exists() {
                self.scan_skin_directory(&standard_skins, &mut skins).await;
            }

            // 2. プラットフォーム別スキンディレクトリを動的にスキャン
            let platforms_dir = android_path.join("platforms");
            if platforms_dir.exists() {
                if let Ok(mut platform_entries) = fs::read_dir(&platforms_dir).await {
                    while let Some(platform_entry) =
                        platform_entries.next_entry().await.ok().flatten()
                    {
                        if let Ok(file_type) = platform_entry.file_type().await {
                            if file_type.is_dir() {
                                let platform_skins = platform_entry.path().join("skins");
                                if platform_skins.exists() {
                                    self.scan_skin_directory(&platform_skins, &mut skins).await;
                                }
                            }
                        }
                    }
                }
            }

            // 3. システムイメージ別スキンディレクトリを動的にスキャン
            let system_images_dir = android_path.join("system-images");
            if system_images_dir.exists() {
                self.scan_system_images_for_skins(&system_images_dir, &mut skins)
                    .await;
            }
        }

        // 4. Get available device IDs from avdmanager (these are also skin candidates)
        if let Ok(available_devices) = self.list_available_devices().await {
            for (id, _display) in available_devices {
                skins.push(id);
            }
        }

        // Remove duplicates and sort
        skins.sort();
        skins.dedup();

        Ok(skins)
    }

    /// Scan skin directory and collect skin names
    async fn scan_skin_directory(&self, skin_dir: &std::path::Path, skins: &mut Vec<String>) {
        if let Ok(mut entries) = fs::read_dir(skin_dir).await {
            while let Some(entry) = entries.next_entry().await.ok().flatten() {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        if let Some(skin_name) = entry.file_name().to_str() {
                            skins.push(skin_name.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Recursively scan system image directories to find skins
    async fn scan_system_images_for_skins(
        &self,
        system_images_dir: &std::path::Path,
        skins: &mut Vec<String>,
    ) {
        if let Ok(mut api_entries) = fs::read_dir(system_images_dir).await {
            while let Some(api_entry) = api_entries.next_entry().await.ok().flatten() {
                if let Ok(file_type) = api_entry.file_type().await {
                    if file_type.is_dir() {
                        let api_dir = api_entry.path();
                        if let Ok(mut tag_entries) = fs::read_dir(&api_dir).await {
                            while let Some(tag_entry) =
                                tag_entries.next_entry().await.ok().flatten()
                            {
                                if let Ok(file_type) = tag_entry.file_type().await {
                                    if file_type.is_dir() {
                                        let tag_dir = tag_entry.path();
                                        if let Ok(mut abi_entries) = fs::read_dir(&tag_dir).await {
                                            while let Some(abi_entry) =
                                                abi_entries.next_entry().await.ok().flatten()
                                            {
                                                if let Ok(file_type) = abi_entry.file_type().await {
                                                    if file_type.is_dir() {
                                                        let skins_dir =
                                                            abi_entry.path().join("skins");
                                                        if skins_dir.exists() {
                                                            self.scan_skin_directory(
                                                                &skins_dir, skins,
                                                            )
                                                            .await;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl DynamicDeviceProvider for AndroidManager {
    async fn get_available_devices(&self) -> Result<Vec<DeviceInfo>> {
        let devices = self.list_available_devices().await?;

        let mut device_infos = Vec::new();
        for (id, display_name) in devices {
            // Use basic categorization to avoid deep async nesting
            let category = DeviceCategory::Unknown; // Will be determined later if needed

            // Extract OEM from display name for now (to avoid async issues)
            let oem = if display_name.contains('(') && display_name.contains(')') {
                let start = display_name.find('(').unwrap() + 1;
                let end = display_name.find(')').unwrap();
                Some(display_name[start..end].to_string())
            } else {
                None
            };

            device_infos.push(DeviceInfo {
                id,
                display_name,
                oem,
                category,
            });
        }

        Ok(device_infos)
    }

    async fn get_available_api_levels(&self) -> Result<Vec<ApiLevelInfo>> {
        let targets = self.list_available_targets().await?;

        let mut api_infos = Vec::new();
        for (api_level_str, display) in targets {
            if let Ok(level) = api_level_str.parse::<u32>() {
                // Extract version name from display (e.g., "API 34 - Android 14" -> "Android 14")
                let version_name = if let Some(dash_pos) = display.find(" - ") {
                    display[dash_pos + 3..].to_string()
                } else {
                    self.get_dynamic_android_version_name(level)
                        .await
                        .unwrap_or_else(|| format!("API {}", level))
                };

                // Get available tags for this API level
                let available_tags = self
                    .get_available_tags_for_api_level(level)
                    .await
                    .unwrap_or_default();

                api_infos.push(ApiLevelInfo {
                    level,
                    version_name,
                    available_tags,
                });
            }
        }

        // Sort by API level (newest first)
        api_infos.sort_by(|a, b| b.level.cmp(&a.level));

        Ok(api_infos)
    }

    async fn get_available_skins(&self, device_id: &str) -> Result<Vec<String>> {
        // Dynamically get skins
        self.get_available_skins_from_sdk(device_id).await
    }

    async fn get_device_priority(&self, device_id: &str) -> Result<u32> {
        // Use basic priority calculation to avoid async issues
        Ok(DynamicDeviceConfig::calculate_android_device_priority(
            device_id, "",
        ))
    }
}

impl AndroidManager {
    /// Diagnose AVD creation issues and provide specific solutions
    pub async fn diagnose_avd_creation_issues(&self, config: &DeviceConfig) -> Result<String> {
        let mut diagnosis = Vec::new();

        // Check 1: Android SDK availability
        diagnosis.push("=== Android SDK Diagnosis ===".to_string());

        // Check 2: Available system images
        let available_images = self.list_available_system_images().await?;
        diagnosis.push(format!(
            "Available system images: {}",
            available_images.len()
        ));
        if available_images.is_empty() {
            diagnosis.push("❌ No system images found! Install with: sdkmanager \"system-images;android-XX;google_apis_playstore;arm64-v8a\"".to_string());
        } else {
            diagnosis.push("✅ System images available".to_string());
            diagnosis.push(format!(
                "First 3: {}",
                available_images
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Check 3: Available device types
        let available_devices = self.list_available_devices().await?;
        diagnosis.push(format!(
            "Available device types: {}",
            available_devices.len()
        ));
        if available_devices.is_empty() {
            diagnosis.push("❌ No device types found! Check Android SDK installation".to_string());
        } else {
            diagnosis.push("✅ Device types available".to_string());
            diagnosis.push(format!(
                "First 3: {}",
                available_devices
                    .iter()
                    .take(3)
                    .map(|(id, display)| format!("{} ({})", display, id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Check 4: Specific system image for this config
        let (tag, abi) = if let Some((found_tag, found_abi)) = self
            .get_first_available_system_image(&config.version)
            .await?
        {
            (found_tag, found_abi)
        } else {
            (
                "google_apis_playstore".to_string(),
                defaults::default_abi().to_string(),
            )
        };

        let package_path = format!("system-images;android-{};{};{}", config.version, tag, abi);
        let image_available = self
            .check_system_image_available(&config.version, &tag, &abi)
            .await
            .unwrap_or(false);

        diagnosis.push(format!("Required system image: {package_path}"));
        if image_available {
            diagnosis.push("✅ Required system image is available".to_string());
        } else {
            diagnosis.push("❌ Required system image NOT available".to_string());
            diagnosis.push(format!("Install with: sdkmanager \"{}\"", package_path));
        }

        // Check 5: Device type availability
        let device_id = Self::find_matching_device_id(&available_devices, &config.device_type);
        diagnosis.push(format!(
            "Required device type: {} ({})",
            config.device_type,
            device_id.as_deref().unwrap_or("NOT FOUND")
        ));
        if device_id.is_some() {
            diagnosis.push("✅ Required device type is available".to_string());
        } else {
            diagnosis.push("❌ Required device type NOT found".to_string());
            diagnosis.push("Suggestion: Use one of the available device types above".to_string());
        }

        Ok(diagnosis.join("\n"))
    }

    /// Find matching device ID from available devices list
    fn find_matching_device_id(
        available_devices: &[(String, String)],
        device_type: &str,
    ) -> Option<String> {
        // Try exact ID match first
        if let Some((id, _)) = available_devices.iter().find(|(id, _)| id == device_type) {
            return Some(id.clone());
        }

        // Try exact display name match
        if let Some((id, _)) = available_devices
            .iter()
            .find(|(_, display)| display == device_type)
        {
            return Some(id.clone());
        }

        // Try partial match for display names (handles variations like quotes)
        let cleaned_config = device_type
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_lowercase();

        // More flexible matching
        available_devices.iter().find_map(|(id, display)| {
            let cleaned_display = display
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect::<String>()
                .to_lowercase();

            // Exact match
            if cleaned_config == cleaned_display {
                return Some(id.clone());
            }

            // Match by main keywords contained in device type name
            let config_words: Vec<&str> = cleaned_config.split_whitespace().collect();
            let display_words: Vec<&str> = cleaned_display.split_whitespace().collect();

            // Check if main keywords match (e.g., "Galaxy", "Pixel", "Nexus")
            let important_words = ["galaxy", "pixel", "nexus", "tv", "wear", "automotive"];
            for word in &important_words {
                if cleaned_config.contains(word) && cleaned_display.contains(word) {
                    // Further detailed check (e.g., "Galaxy S24" vs "Galaxy Nexus")
                    let config_specific: Vec<&str> = config_words
                        .iter()
                        .filter(|w| w.chars().any(|c| c.is_ascii_digit()) || w.len() > 4)
                        .cloned()
                        .collect();
                    let display_specific: Vec<&str> = display_words
                        .iter()
                        .filter(|w| w.chars().any(|c| c.is_ascii_digit()) || w.len() > 4)
                        .cloned()
                        .collect();

                    if !config_specific.is_empty() && !display_specific.is_empty() {
                        // Only when specific keywords match
                        if config_specific.iter().any(|w| display_specific.contains(w)) {
                            return Some(id.clone());
                        }
                    } else if config_specific.is_empty() && display_specific.is_empty() {
                        // Basic matching when both have generic names
                        return Some(id.clone());
                    }
                }
            }

            None
        })
    }

    async fn get_available_tags_for_api_level(&self, api_level: u32) -> Result<Vec<String>> {
        let images = self.list_available_system_images().await?;
        let mut tags = HashSet::new();

        for image in images {
            if image.contains(&format!("android-{api_level}")) {
                let parts: Vec<&str> = image.split(';').collect();
                if parts.len() >= 3 {
                    tags.insert(parts[2].to_string());
                }
            }
        }

        Ok(tags.into_iter().collect())
    }
}

impl DeviceManager for AndroidManager {
    type Device = AndroidDevice;

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        let avd_output = self
            .command_runner
            .run(&self.avdmanager_path, &["list", "avd"])
            .await
            .context("Failed to list Android AVDs")?;

        // log::debug!("AVD list output:\n{}", avd_output);

        // Get running AVD names mapped to their emulator IDs
        let running_avds = self.get_running_avd_names().await?;

        let mut devices = Vec::new();
        let mut current_device_info: Option<(String, String, String, String, String)> = None; // Name, Path, Target, Tag/ABI, Device

        let mut current_target_full = String::new();

        for line in avd_output.lines() {
            let trimmed_line = line.trim();

            // Capture multi-line target information
            if current_device_info.is_some() && line.starts_with("          Based on:") {
                current_target_full.push(' ');
                current_target_full.push_str(trimmed_line);
            }

            if trimmed_line.starts_with("---") || trimmed_line.is_empty() {
                if let Some((name, _path, mut target, _abi, device)) = current_device_info.take() {
                    // Append any additional target info
                    if !current_target_full.is_empty() {
                        target.push_str(&current_target_full);
                        current_target_full.clear();
                    }

                    let is_running = running_avds.contains_key(&name);

                    // Extract API level - simplified approach that tries multiple methods
                    let api_level = {
                        let mut api = 0u32;

                        // Method 1: Try to read from config.ini in standard location
                        if let Ok(home) = std::env::var("HOME") {
                            let config_path = PathBuf::from(home)
                                .join(".android")
                                .join("avd")
                                .join(format!("{}.avd", &name))
                                .join("config.ini");

                            if let Ok(config_content) = fs::read_to_string(&config_path).await {
                                // Try regex first (with or without trailing slash)
                                if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(&config_content) {
                                    if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                        api = parsed_api;
                                    }
                                }
                                // Fallback to target line
                                else if let Some(caps) =
                                    TARGET_CONFIG_REGEX.captures(&config_content)
                                {
                                    if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                        api = parsed_api;
                                    }
                                }
                            }
                        }

                        // Method 2: If still no API found, try get_avd_path method
                        if api == 0 {
                            if let Ok(Some(avd_path)) = self.get_avd_path(&name).await {
                                let config_path = avd_path.join("config.ini");
                                if let Ok(config_content) = fs::read_to_string(&config_path).await {
                                    if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(&config_content)
                                    {
                                        if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                            api = parsed_api;
                                        }
                                    } else if let Some(caps) =
                                        TARGET_CONFIG_REGEX.captures(&config_content)
                                    {
                                        if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                            api = parsed_api;
                                        }
                                    }
                                }
                            }
                        }

                        // Method 3: Fallback to parsing from avdmanager target string
                        if api == 0 {
                            if let Some(caps) = BASED_ON_REGEX.captures(&target) {
                                let version = &caps[1];
                                api = Self::parse_android_version_to_api_level(version);
                            } else if let Some(caps) = API_LEVEL_REGEX.captures(&target) {
                                api = caps[1].parse().unwrap_or(0);
                            } else if let Some(caps) = ANDROID_VERSION_REGEX.captures(&target) {
                                api = caps[1].parse().unwrap_or(0);
                            }
                        }

                        api
                    };

                    // Get device display name from config.ini or fallback to device mapping
                    let device_type_str = {
                        let mut display_name = String::new();

                        // Try to read displayname from config.ini
                        if let Ok(home) = std::env::var("HOME") {
                            let config_path = PathBuf::from(home)
                                .join(".android")
                                .join("avd")
                                .join(format!("{}.avd", &name))
                                .join("config.ini");

                            if let Ok(config_content) = fs::read_to_string(&config_path).await {
                                if let Some(caps) = AVD_DISPLAYNAME_REGEX.captures(&config_content)
                                {
                                    display_name = caps[1].trim().to_string();
                                }
                            }
                        }

                        // Fallback to device mapping if no displayname found
                        if display_name.is_empty() && !device.is_empty() {
                            let mapped_name = self.get_device_display_name(&device);
                            display_name = mapped_name;
                        }

                        // Final fallback
                        if display_name.is_empty() {
                            display_name = "Unknown".to_string();
                        }

                        display_name
                    };

                    devices.push(AndroidDevice {
                        name: name.to_string(),
                        device_type: device_type_str,
                        api_level,
                        is_running,
                        status: if is_running {
                            DeviceStatus::Running
                        } else {
                            DeviceStatus::Stopped
                        },
                        ..Default::default()
                    });
                }
                continue;
            }

            if let Some(caps) = AVD_NAME_REGEX.captures(trimmed_line) {
                if let Some((name, _path, mut target, _abi, device)) = current_device_info.take() {
                    // Push previous device if any
                    // Append any additional target info
                    if !current_target_full.is_empty() {
                        target.push_str(&current_target_full);
                        current_target_full.clear();
                    }

                    let is_running = running_avds.contains_key(&name);

                    let api_level = {
                        // First try: extract from "Based on: Android X.Y"
                        if let Some(caps) = BASED_ON_REGEX.captures(&target) {
                            let version = &caps[1];
                            Self::parse_android_version_to_api_level(version)
                        }
                        // Second try: extract from "API level XX"
                        else if let Some(caps) = API_LEVEL_REGEX.captures(&target) {
                            caps[1].parse().unwrap_or(0)
                        }
                        // Third try: extract from "android-XX"
                        else if let Some(caps) = ANDROID_VERSION_REGEX.captures(&target) {
                            caps[1].parse().unwrap_or(0)
                        }
                        // Fourth try: extract from any number pattern
                        else if let Some(caps) = NUMBER_PATTERN_REGEX.captures(&target) {
                            let potential_api = caps[1].parse().unwrap_or(0);
                            // Validate that it's a reasonable API level (21-35)
                            if (21..=40).contains(&potential_api) {
                                potential_api
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    };

                    // Get device display name from config.ini or fallback to device mapping
                    let device_type_str = {
                        let mut display_name = String::new();

                        // Try to read displayname from config.ini
                        if let Ok(home) = std::env::var("HOME") {
                            let config_path = PathBuf::from(home)
                                .join(".android")
                                .join("avd")
                                .join(format!("{}.avd", &name))
                                .join("config.ini");

                            if let Ok(config_content) = fs::read_to_string(&config_path).await {
                                if let Some(caps) = AVD_DISPLAYNAME_REGEX.captures(&config_content)
                                {
                                    display_name = caps[1].trim().to_string();
                                }
                            }
                        }

                        // Fallback to device mapping if no displayname found
                        if display_name.is_empty() && !device.is_empty() {
                            let mapped_name = self.get_device_display_name(&device);
                            display_name = mapped_name;
                        }

                        // Final fallback
                        if display_name.is_empty() {
                            display_name = "Unknown".to_string();
                        }

                        display_name
                    };

                    devices.push(AndroidDevice {
                        name: name.to_string(),
                        device_type: device_type_str,
                        api_level,
                        is_running,
                        status: if is_running {
                            DeviceStatus::Running
                        } else {
                            DeviceStatus::Stopped
                        },
                        ..Default::default()
                    });
                }
                current_device_info = Some((
                    caps[1].to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                ));
            } else if let Some(ref mut info) = current_device_info {
                if let Some(caps) = PATH_REGEX.captures(trimmed_line) {
                    info.1 = caps[1].to_string();
                } else if let Some(caps) = TARGET_REGEX.captures(trimmed_line) {
                    info.2 = caps[1].to_string();
                } else if let Some(caps) = ABI_REGEX.captures(trimmed_line) {
                    info.3 = caps[1].to_string();
                } else if let Some(caps) = DEVICE_REGEX.captures(trimmed_line) {
                    info.4 = caps[1].to_string();
                }
            }
        }
        if let Some((name, _path, mut target, _abi, device)) = current_device_info.take() {
            // Push the last device
            // Append any additional target info
            if !current_target_full.is_empty() {
                target.push_str(&current_target_full);
                current_target_full.clear();
            }

            let is_running = running_avds.contains_key(&name);

            // Use the same API level detection logic as the other paths
            let api_level = {
                let mut api = 0u32;

                // Method 1: Try to read from config.ini in standard location
                if let Ok(home) = std::env::var("HOME") {
                    let config_path = PathBuf::from(home)
                        .join(".android")
                        .join("avd")
                        .join(format!("{}.avd", &name))
                        .join("config.ini");

                    if let Ok(config_content) = fs::read_to_string(&config_path).await {
                        // Try regex first (with or without trailing slash)
                        if let Some(caps) = IMAGE_SYSDIR_REGEX.captures(&config_content) {
                            if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                api = parsed_api;
                            }
                        }
                        // Fallback to target line
                        else if let Some(caps) = TARGET_CONFIG_REGEX.captures(&config_content) {
                            if let Ok(parsed_api) = caps[1].parse::<u32>() {
                                api = parsed_api;
                            }
                        }
                    }
                }

                // Fallback to avdmanager target string parsing
                if api == 0 {
                    if let Some(caps) = BASED_ON_REGEX.captures(&target) {
                        let version = &caps[1];
                        api = Self::parse_android_version_to_api_level(version);
                    } else if let Some(caps) = API_OR_ANDROID_REGEX.captures(&target) {
                        api = caps[1].parse().unwrap_or(0);
                    }
                }

                api
            };

            let device_type_str = if !device.is_empty() {
                device
            } else {
                "Unknown".to_string()
            };

            // log::info!("Device '{}' - running: {}, API: {}", name, is_running, api_level);

            devices.push(AndroidDevice {
                name: name.to_string(),
                device_type: device_type_str,
                api_level,
                is_running,
                status: if is_running {
                    DeviceStatus::Running
                } else {
                    DeviceStatus::Stopped
                },
                ..Default::default()
            });
        }

        // log::info!("Total AVDs listed: {}", devices.len());

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

            // Use a graceful shutdown method instead of killing the emulator process
            // This sends a shutdown command to the Android OS, not the emulator itself
            // Important: This approach allows the emulator process to remain running
            // even when the emu TUI application exits, preventing accidental data loss
            // First try to send a shutdown broadcast to Android
            let shutdown_result = self
                .command_runner
                .run(
                    "adb",
                    &[
                        "-s",
                        emulator_id,
                        "shell",
                        "am",
                        "broadcast",
                        "-a",
                        "android.intent.action.ACTION_SHUTDOWN",
                    ],
                )
                .await;

            if shutdown_result.is_ok() {
                // Give the OS a moment to process the shutdown
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Then use reboot -p as a fallback to power off
                let _ = self
                    .command_runner
                    .run("adb", &["-s", emulator_id, "shell", "reboot", "-p"])
                    .await;
            } else {
                // If the graceful shutdown failed, fall back to emu kill
                // but only as a last resort
                self.command_runner
                    .run("adb", &["-s", emulator_id, "emu", "kill"])
                    .await
                    .context(format!("Failed to stop emulator {}", emulator_id))?;
            }
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
                .map_or(defaults::default_abi(), |s| s.as_str());
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

        // Use default if device parameter is not found
        if let Some(ref device_id) = device_param {
            args.push("--device");
            args.push(device_id);
        } else {
            // Omit device parameter - avdmanager will use default device
            log::warn!(
                "Device type '{}' not found, using default device",
                config.device_type
            );
        }

        // Dynamically get and specify skin (use fallback strategy on error)
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

        // Retry without skin if skin error occurs
        let result = if result.is_err() && skin_name.is_some() {
            let error_str = result.as_ref().unwrap_err().to_string();
            if error_str.to_lowercase().contains("skin") {
                log::warn!(
                    "Skin '{}' failed, retrying without skin",
                    skin_name.as_ref().unwrap()
                );
                // Retry after removing skin parameter
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
                // Shortened name (max 20 characters)
                let short_name = if safe_name.len() > 20 {
                    format!("{}...", &safe_name[..17])
                } else {
                    safe_name.clone()
                };
                diagnostic_info.push(format!("AVD: {}", short_name));
                diagnostic_info.push(format!("API: {}", config.version));

                // Create concise error message with important information at the beginning
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
                    // Generic error - only the most important information
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

impl AndroidManager {
    /// Lists available API levels with their installation status and Android version names.
    /// Returns a comprehensive list of system images with accurate version mapping.
    pub async fn list_api_levels(&self) -> Result<Vec<crate::models::ApiLevel>> {
        use crate::models::{ApiLevel, SystemImageVariant};

        // Get all available system images from SDK
        let sdkmanager_path = Self::find_tool(&self.android_home, "sdkmanager")?;
        let output = tokio::process::Command::new(&sdkmanager_path)
            .args(["--list", "--verbose"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to list system images: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut api_levels_map: std::collections::HashMap<u32, ApiLevel> =
            std::collections::HashMap::new();

        // Track which section we're in and parse system images
        let mut in_installed_section = false;
        let mut found_system_images = false;

        for line in output_str.lines() {
            let line = line.trim();

            // Track sections - use more flexible matching
            if line.contains("Installed packages") || line.contains("Installed Packages") {
                in_installed_section = true;
                continue;
            } else if line.contains("Available Packages") || line.contains("Available Updates") {
                in_installed_section = false;
                continue;
            }

            // Parse system image lines
            if line.starts_with("system-images;android-") {
                found_system_images = true;

                // Extract package ID (first column before whitespace)
                let package_id = line.split_whitespace().next().unwrap_or(line);

                if let Some(api_level) = self.parse_api_level_from_package(package_id) {
                    let is_installed = in_installed_section;

                    // Parse package components
                    let parts: Vec<&str> = package_id.split(';').collect();
                    if parts.len() >= 4 {
                        let variant = parts[2].to_string();
                        let architecture = parts[3].to_string();

                        let system_variant = SystemImageVariant::new(
                            variant.clone(),
                            architecture,
                            package_id.to_string(),
                        );

                        // Get or create API level entry
                        let api_entry = api_levels_map.entry(api_level).or_insert_with(|| {
                            let version_name = self.get_android_version_name(api_level);
                            ApiLevel::new(
                                api_level,
                                version_name,
                                format!("system-images;android-{};google_apis;x86_64", api_level),
                            )
                        });

                        // Add variant with installation status
                        let mut variant_clone = system_variant;
                        variant_clone.is_installed = is_installed;
                        api_entry.variants.push(variant_clone);

                        // Update overall installation status (if any variant is installed)
                        if is_installed {
                            api_entry.is_installed = true;
                        }
                    }
                }
            }
        }

        // If no system images found, create a default list with common API levels
        if !found_system_images {
            let default_apis = vec![35, 34, 33, 32, 31, 30, 29, 28];
            for api in default_apis {
                let version_name = self.get_android_version_name(api);
                let api_level = ApiLevel::new(
                    api,
                    version_name,
                    format!("system-images;android-{};google_apis;x86_64", api),
                );
                api_levels_map.insert(api, api_level);
            }
        }

        // Convert to sorted vector
        let mut api_levels: Vec<ApiLevel> = api_levels_map.into_values().collect();
        api_levels.sort_by(|a, b| b.api.cmp(&a.api)); // Sort by API level descending

        Ok(api_levels)
    }

    /// Installs a system image with progress callback.
    pub async fn install_system_image<F>(
        &self,
        package_id: &str,
        progress_callback: F,
    ) -> Result<()>
    where
        F: Fn(crate::models::InstallProgress) + Send + Sync + 'static,
    {
        use crate::models::InstallProgress;

        // Initial progress
        progress_callback(InstallProgress {
            operation: "Preparing installation...".to_string(),
            percentage: 0,
            eta_seconds: None,
        });

        let sdkmanager_path = Self::find_tool(&self.android_home, "sdkmanager")?;
        let mut child = tokio::process::Command::new(&sdkmanager_path)
            .args([package_id])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Send 'y' to accept license
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(b"y\n").await?;
            stdin.flush().await?;
        }

        // Set progress to show we're starting
        progress_callback(InstallProgress {
            operation: "Starting installation process...".to_string(),
            percentage: 5,
            eta_seconds: None,
        });

        // Simulate progress updates with timer since sdkmanager doesn't provide reliable progress
        let progress_callback = std::sync::Arc::new(progress_callback);
        let progress_clone = progress_callback.clone();

        // Start a timer-based progress update
        tokio::spawn(async move {
            let mut progress = 10u8;
            let mut stage = 0;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                match stage {
                    0 => {
                        // Initial loading phase (10-20%)
                        progress_clone(InstallProgress {
                            operation: "Loading package information...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += 5;
                        if progress >= 20 {
                            stage = 1;
                            progress = 20;
                        }
                    }
                    1 => {
                        // Download phase (20-70%)
                        progress_clone(InstallProgress {
                            operation: "Downloading system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += 3;
                        if progress >= 70 {
                            stage = 2;
                            progress = 70;
                        }
                    }
                    2 => {
                        // Extract phase (70-90%)
                        progress_clone(InstallProgress {
                            operation: "Extracting system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += 4;
                        if progress >= 90 {
                            stage = 3;
                            progress = 90;
                        }
                    }
                    3 => {
                        // Install phase (90-95%)
                        progress_clone(InstallProgress {
                            operation: "Installing system image...".to_string(),
                            percentage: progress,
                            eta_seconds: None,
                        });
                        progress += 2;
                        if progress >= 95 {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        // Monitor stdout for real progress (sdkmanager outputs to stdout)
        if let Some(stdout) = child.stdout.take() {
            let progress_stdout = progress_callback.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    // Look for download progress indicators
                    if line.contains("Downloading") {
                        // Try to extract size information
                        if line.contains(" MiB") || line.contains(" MB") {
                            // Extract percentage if present (e.g., "(45%)")
                            if let Some(start) = line.find('(') {
                                if let Some(end) = line.find('%') {
                                    if let Ok(pct) = line[start + 1..end].trim().parse::<u8>() {
                                        progress_stdout(InstallProgress {
                                            operation: "Downloading system image...".to_string(),
                                            percentage: (20 + (pct * 50 / 100)).min(70),
                                            eta_seconds: None,
                                        });
                                    }
                                }
                            }
                        }
                    } else if line.contains("Unzipping") || line.contains("Extracting") {
                        progress_stdout(InstallProgress {
                            operation: "Extracting system image...".to_string(),
                            percentage: 75,
                            eta_seconds: None,
                        });
                    } else if line.contains("Installing") {
                        progress_stdout(InstallProgress {
                            operation: "Installing system image...".to_string(),
                            percentage: 85,
                            eta_seconds: None,
                        });
                    }
                }
            });
        }

        // Also monitor stderr for any error messages
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    // Log errors for debugging
                    if line.contains("Error") || line.contains("error") || line.contains("Failed") {
                        eprintln!("sdkmanager error: {}", line);
                    }
                }
            });
        }

        let output = child.wait_with_output().await?;

        if output.status.success() {
            // Don't send final progress update - let the caller handle completion
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to install system image: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Uninstalls a system image.
    pub async fn uninstall_system_image(&self, package_id: &str) -> Result<()> {
        let sdkmanager_path = Self::find_tool(&self.android_home, "sdkmanager")?;
        let output = tokio::process::Command::new(&sdkmanager_path)
            .args(["--uninstall", package_id])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to uninstall system image: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Parses API level from package ID.
    fn parse_api_level_from_package(&self, package_id: &str) -> Option<u32> {
        if let Some(start) = package_id.find("android-") {
            let api_part = &package_id[start + 8..];
            if let Some(end) = api_part.find(';') {
                api_part[..end].parse().ok()
            } else {
                api_part.parse().ok()
            }
        } else {
            None
        }
    }
}
