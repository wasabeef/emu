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

mod create;
mod details;
mod discovery;
mod install;
mod lifecycle;
mod parser;
mod sdk;
mod version;

use crate::{
    constants::{commands, performance::ANDROID_SDK_LIST_CACHE_TTL},
    managers::common::{DeviceConfig, DeviceManager},
    models::{AndroidDevice, ApiLevel},
    utils::command::CommandRunner,
    utils::command_executor::CommandExecutor,
};
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

type CachedTargets = Vec<(String, String)>;
type CachedAvailableDevices = Vec<(String, String)>;
type TimedTargetsCache = Arc<RwLock<Option<TimedCache<CachedTargets>>>>;
type TimedAvailableDevicesCache = Arc<RwLock<Option<TimedCache<CachedAvailableDevices>>>>;
type TimedStringCache = Arc<RwLock<Option<TimedCache<String>>>>;
type TimedApiLevelsCache = Arc<RwLock<Option<TimedCache<Vec<ApiLevel>>>>>;
type DeviceMetadataMap = std::collections::HashMap<String, CachedAndroidDeviceMetadata>;

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
    /// Command executor for executing Android SDK tools (abstracted for testability)
    command_executor: Arc<dyn CommandExecutor>,
    /// Path to Android SDK home directory (from ANDROID_HOME or ANDROID_SDK_ROOT)
    android_home: PathBuf,
    /// Path to avdmanager executable
    avdmanager_path: PathBuf,
    /// Path to emulator executable
    emulator_path: PathBuf,
    /// Session cache for Android target list derived from installed system images.
    available_targets_cache: TimedTargetsCache,
    /// Session cache for Android device definitions used by the create-device dialog.
    available_devices_cache: TimedAvailableDevicesCache,
    /// Session cache for raw sdkmanager verbose output reused across Android SDK-backed lists.
    sdkmanager_verbose_output_cache: TimedStringCache,
    /// Session cache for Android API levels used by the system-images dialog.
    api_levels_cache: TimedApiLevelsCache,
    /// Session cache for per-device metadata derived from config parsing.
    device_metadata_cache: Arc<RwLock<DeviceMetadataMap>>,
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
        Self::with_executor(Arc::new(CommandRunner::new()))
    }

    /// Creates a new AndroidManager instance with a custom command executor.
    /// This is primarily used for testing with mock executors.
    ///
    /// # Arguments
    /// - `executor` - The command executor to use for external commands
    ///
    /// # Returns
    /// - `Ok(AndroidManager)` - If Android SDK and tools are found
    /// - `Err` - If Android SDK is not installed or tools are missing
    pub fn with_executor(executor: Arc<dyn CommandExecutor>) -> Result<Self> {
        let android_home = Self::find_android_home()?;
        let avdmanager_path = Self::find_tool(&android_home, commands::AVDMANAGER)?;
        let emulator_path = Self::find_tool(&android_home, commands::EMULATOR)?;

        Ok(Self {
            command_executor: executor,
            android_home,
            avdmanager_path,
            emulator_path,
            available_targets_cache: Arc::new(RwLock::new(None)),
            available_devices_cache: Arc::new(RwLock::new(None)),
            sdkmanager_verbose_output_cache: Arc::new(RwLock::new(None)),
            api_levels_cache: Arc::new(RwLock::new(None)),
            device_metadata_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        })
    }

    async fn get_cached_available_targets(&self) -> Option<Vec<(String, String)>> {
        let cache = self.available_targets_cache.read().await;
        cache.as_ref().and_then(|cache| {
            cache
                .is_fresh(ANDROID_SDK_LIST_CACHE_TTL)
                .then(|| cache.value.clone())
        })
    }

    async fn set_cached_available_targets(&self, targets: Vec<(String, String)>) {
        let mut cache = self.available_targets_cache.write().await;
        *cache = Some(TimedCache::new(targets));
    }

    async fn get_cached_available_devices(&self) -> Option<Vec<(String, String)>> {
        let cache = self.available_devices_cache.read().await;
        cache.as_ref().and_then(|cache| {
            cache
                .is_fresh(ANDROID_SDK_LIST_CACHE_TTL)
                .then(|| cache.value.clone())
        })
    }

    async fn set_cached_available_devices(&self, devices: Vec<(String, String)>) {
        let mut cache = self.available_devices_cache.write().await;
        *cache = Some(TimedCache::new(devices));
    }

    async fn get_cached_sdkmanager_verbose_output(&self) -> Option<String> {
        let cache = self.sdkmanager_verbose_output_cache.read().await;
        cache.as_ref().and_then(|cache| {
            cache
                .is_fresh(ANDROID_SDK_LIST_CACHE_TTL)
                .then(|| cache.value.clone())
        })
    }

    async fn set_cached_sdkmanager_verbose_output(&self, output: String) {
        let mut cache = self.sdkmanager_verbose_output_cache.write().await;
        *cache = Some(TimedCache::new(output));
    }

    pub(crate) async fn get_sdkmanager_verbose_output(&self) -> Result<String> {
        if let Some(cached_output) = self.get_cached_sdkmanager_verbose_output().await {
            return Ok(cached_output);
        }

        let sdkmanager_path = Self::find_tool(&self.android_home, commands::SDKMANAGER)?;
        let output = self
            .command_executor
            .run(
                &sdkmanager_path,
                &[
                    commands::sdkmanager::LIST,
                    "--verbose",
                    "--include_obsolete",
                ],
            )
            .await?;
        self.set_cached_sdkmanager_verbose_output(output.clone())
            .await;
        Ok(output)
    }

    pub(crate) async fn get_cached_api_levels(&self) -> Option<Vec<ApiLevel>> {
        let cache = self.api_levels_cache.read().await;
        cache.as_ref().and_then(|cache| {
            cache
                .is_fresh(ANDROID_SDK_LIST_CACHE_TTL)
                .then(|| cache.value.clone())
        })
    }

    async fn set_cached_api_levels(&self, api_levels: Vec<ApiLevel>) {
        let mut cache = self.api_levels_cache.write().await;
        *cache = Some(TimedCache::new(api_levels));
    }

    async fn get_cached_device_metadata(
        &self,
        name: &str,
        target: &str,
    ) -> Option<CachedAndroidDeviceMetadata> {
        let cache = self.device_metadata_cache.read().await;
        cache
            .get(name)
            .and_then(|metadata| (metadata.target == target).then(|| metadata.clone()))
    }

    async fn set_cached_device_metadata(
        &self,
        name: String,
        metadata: CachedAndroidDeviceMetadata,
    ) {
        let mut cache = self.device_metadata_cache.write().await;
        cache.insert(name, metadata);
    }

    pub(crate) async fn invalidate_device_metadata_cache(&self, name: Option<&str>) {
        let mut cache = self.device_metadata_cache.write().await;
        if let Some(name) = name {
            cache.remove(name);
        } else {
            cache.clear();
        }
    }

    pub(crate) async fn invalidate_sdk_list_caches(&self) {
        {
            let mut cache = self.available_targets_cache.write().await;
            *cache = None;
        }
        {
            let mut cache = self.available_devices_cache.write().await;
            *cache = None;
        }
        {
            let mut cache = self.api_levels_cache.write().await;
            *cache = None;
        }
        {
            let mut cache = self.sdkmanager_verbose_output_cache.write().await;
            *cache = None;
        }
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
    /// Execute multiple commands in parallel and collect results
    #[allow(dead_code)]
    async fn run_commands_parallel<I, S>(&self, commands: I) -> Vec<Result<String>>
    where
        I: IntoIterator<Item = (S, Vec<S>)>,
        S: AsRef<str> + Send + 'static,
    {
        let command_executor = self.command_executor.clone();
        let handles: Vec<_> = commands
            .into_iter()
            .map(|(cmd, args)| {
                let executor = command_executor.clone();
                let cmd_str = cmd.as_ref().to_string();
                let args_vec: Vec<String> = args.iter().map(|s| s.as_ref().to_string()).collect();

                tokio::spawn(async move {
                    executor
                        .run(
                            Path::new(&cmd_str),
                            &args_vec.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        )
                        .await
                })
            })
            .collect();

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(anyhow::anyhow!("Task join error: {e}"))),
            }
        }
        results
    }
}

#[derive(Clone)]
struct TimedCache<T> {
    value: T,
    cached_at: Instant,
}

impl<T> TimedCache<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            cached_at: Instant::now(),
        }
    }

    fn is_fresh(&self, ttl: std::time::Duration) -> bool {
        self.cached_at.elapsed() < ttl
    }
}

#[derive(Clone)]
struct CachedAndroidDeviceMetadata {
    target: String,
    api_level: u32,
    android_version_name: String,
}

impl DeviceManager for AndroidManager {
    type Device = AndroidDevice;

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        self.list_devices_parallel().await
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
        // Availability is determined by `new()` succeeding (tools found).
        true
    }
}

/// Implementation of UnifiedDeviceManager for AndroidManager
#[async_trait::async_trait]
impl crate::managers::common::UnifiedDeviceManager for AndroidManager {
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

#[cfg(test)]
mod tests;
