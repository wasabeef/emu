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
    constants::commands,
    managers::common::{DeviceConfig, DeviceManager},
    models::AndroidDevice,
    utils::command::CommandRunner,
    utils::command_executor::CommandExecutor,
};
use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        })
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
mod tests {
    use super::*;
    use crate::managers::android::parser::AvdListParser;
    use crate::managers::common::DeviceConfig;
    use crate::models::device_info::DynamicDeviceProvider;
    use crate::utils::command_executor::mock::MockCommandExecutor;
    use std::collections::HashMap;
    use std::env;
    use std::ffi::OsString;
    use std::sync::OnceLock;
    use tokio::sync::{Mutex, MutexGuard};

    struct EnvVarGuard {
        key: &'static str,
        original: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set<K, V>(key: K, value: V) -> Self
        where
            K: Into<&'static str>,
            V: Into<OsString>,
        {
            let key = key.into();
            let original = env::var_os(key);
            env::set_var(key, value.into());
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    fn test_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    async fn acquire_test_env_lock() -> MutexGuard<'static, ()> {
        test_env_lock().lock().await
    }

    /// Set up Android SDK environment for testing
    fn setup_test_android_sdk() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let sdk_path = temp_dir.path();

        // Create necessary directory structure
        std::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin")).unwrap();
        std::fs::create_dir_all(sdk_path.join("tools/bin")).unwrap();
        std::fs::create_dir_all(sdk_path.join("emulator")).unwrap();
        std::fs::create_dir_all(sdk_path.join("platform-tools")).unwrap();

        // Create necessary tool scripts
        let tools_to_create = [
            (
                "cmdline-tools/latest/bin/avdmanager",
                "#!/bin/sh\necho 'avdmanager mock'\n",
            ),
            (
                "tools/bin/avdmanager",
                "#!/bin/sh\necho 'avdmanager mock'\n",
            ),
            (
                "cmdline-tools/latest/bin/sdkmanager",
                "#!/bin/sh\necho 'sdkmanager mock'\n",
            ),
            (
                "tools/bin/sdkmanager",
                "#!/bin/sh\necho 'sdkmanager mock'\n",
            ),
            ("emulator/emulator", "#!/bin/sh\necho 'emulator mock'\n"),
            ("platform-tools/adb", "#!/bin/sh\necho 'adb mock'\n"),
        ];

        for (tool_path, script_content) in &tools_to_create {
            let full_path = sdk_path.join(tool_path);
            std::fs::write(&full_path, script_content).unwrap();

            // Grant execute permission on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&full_path).unwrap().permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&full_path, perms).unwrap();
            }
        }

        temp_dir
    }

    #[test]
    fn test_parse_android_version_to_api_level() {
        // Normal cases: only exact, unambiguous mappings are supported
        assert_eq!(AndroidManager::parse_android_version_to_api_level("15"), 35);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("14"), 34);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("13"), 33);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("11"), 30);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("10"), 29);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("9"), 28);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("6"), 23);
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("8.1"),
            27
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("8.0"),
            26
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("7.1"),
            25
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("7.0"),
            24
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("5.1"),
            22
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("5.0"),
            21
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("4.4"),
            19
        );
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("4.1"),
            16
        );

        // Test version strings (with decimal points)
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("14.0"),
            34
        );

        // Ambiguous or unknown versions should remain unknown rather than guessed
        assert_eq!(AndroidManager::parse_android_version_to_api_level("12"), 0);
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("12.0"),
            0
        );
        assert_eq!(AndroidManager::parse_android_version_to_api_level("8"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("7"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("5"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("4"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("16"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("20"), 0);

        // Error case: Invalid input
        assert_eq!(AndroidManager::parse_android_version_to_api_level(""), 0);
        assert_eq!(
            AndroidManager::parse_android_version_to_api_level("invalid"),
            0
        );
        assert_eq!(AndroidManager::parse_android_version_to_api_level("abc"), 0);

        // Boundary value: unsupported versions also remain unknown
        assert_eq!(AndroidManager::parse_android_version_to_api_level("3"), 0);
        assert_eq!(AndroidManager::parse_android_version_to_api_level("2"), 0);
    }

    #[test]
    fn test_find_android_home_with_env_var() {
        // Test when environment variable is set
        let temp_dir = setup_test_android_sdk();
        let android_home = temp_dir.path().to_path_buf();

        // Temporarily set environment variable
        env::set_var("ANDROID_HOME", &android_home);

        let result = AndroidManager::find_android_home();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), android_home);

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[test]
    fn test_find_android_home_not_set() {
        // Test when environment variable is not set
        env::remove_var("ANDROID_HOME");
        env::remove_var("ANDROID_SDK_ROOT");

        let result = AndroidManager::find_android_home();
        // Depends on environment, returns error or standard path
        // Usually errors in CI environment
        if result.is_err() {
            assert!(result.unwrap_err().to_string().contains("Android"));
        }
    }

    #[test]
    fn test_find_tool_success() {
        // Tool search success case
        let temp_dir = setup_test_android_sdk();
        let android_home = temp_dir.path();

        // Create mock tool file structure
        let tool_path = android_home.join("tools").join("bin").join("avdmanager");
        std::fs::create_dir_all(tool_path.parent().unwrap()).expect("Failed to create dirs");
        std::fs::write(&tool_path, "#!/bin/bash\necho 'mock avdmanager'")
            .expect("Failed to write tool");

        // Grant execute permission (Unix systems only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&tool_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&tool_path, perms).unwrap();
        }

        let result = AndroidManager::find_tool(android_home, "avdmanager");
        assert!(result.is_ok());
        // Tools in cmdline-tools/latest/bin/ are prioritized
        let expected_path = android_home
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("avdmanager");
        assert_eq!(result.unwrap(), expected_path);
    }

    #[test]
    fn test_find_tool_not_found() {
        // Tool search failure case
        let temp_dir = setup_test_android_sdk();
        let android_home = temp_dir.path();

        let result = AndroidManager::find_tool(android_home, "nonexistent_tool");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_get_device_category() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let executor = std::sync::Arc::new(MockCommandExecutor::new());
        let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

        // Test Phone category
        assert_eq!(manager.get_device_category("pixel_7", "Pixel 7"), "phone");
        assert_eq!(manager.get_device_category("pixel_6", "Pixel 6"), "phone");
        assert_eq!(manager.get_device_category("nexus_5x", "Nexus 5X"), "phone");

        // Test Tablet category
        assert_eq!(
            manager.get_device_category("nexus_10", "Nexus 10 inch"),
            "tablet"
        );
        assert_eq!(
            manager.get_device_category("pixel_tablet", "Pixel Tablet"),
            "tablet"
        );

        // Test TV category
        assert_eq!(
            manager.get_device_category("tv_1080p", "Android TV (1080p)"),
            "tv"
        );
        assert_eq!(
            manager.get_device_category("tv_720p", "Android TV (720p)"),
            "tv"
        );

        // Test Wear category
        assert_eq!(
            manager.get_device_category("wear_round", "Android Wear Round"),
            "wear"
        );
        assert_eq!(
            manager.get_device_category("wear_square", "Android Wear Square"),
            "wear"
        );

        // Test Automotive category
        assert_eq!(
            manager.get_device_category("automotive_1024p", "Automotive (1024p landscape)"),
            "automotive"
        );

        // Unknown device (default: phone)
        assert_eq!(
            manager.get_device_category("unknown_device", "Unknown Device"),
            "phone"
        );
        assert_eq!(manager.get_device_category("", ""), "phone");

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[test]
    fn test_get_android_version_name() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let executor = std::sync::Arc::new(MockCommandExecutor::new());
        let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

        // Test dynamic API level fallback format
        assert_eq!(manager.get_android_version_name(34), "API 34");
        assert_eq!(manager.get_android_version_name(33), "API 33");
        assert_eq!(manager.get_android_version_name(31), "API 31");
        assert_eq!(manager.get_android_version_name(30), "API 30");
        assert_eq!(manager.get_android_version_name(29), "API 29");
        assert_eq!(manager.get_android_version_name(28), "API 28");

        // Old API levels - now use dynamic format
        assert_eq!(manager.get_android_version_name(21), "API 21");
        assert_eq!(manager.get_android_version_name(16), "API 16");

        // Unknown API levels (high values) - all use dynamic format
        assert_eq!(manager.get_android_version_name(40), "API 40");
        assert_eq!(manager.get_android_version_name(100), "API 100");

        // Boundary values - all use dynamic format
        assert_eq!(manager.get_android_version_name(1), "API 1");
        assert_eq!(manager.get_android_version_name(0), "API 0");

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[test]
    fn test_parse_api_level_from_package() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let executor = std::sync::Arc::new(MockCommandExecutor::new());
        let manager = AndroidManager::with_executor(executor).expect("Failed to create manager");

        // Extract API level from standard package ID
        assert_eq!(
            manager.parse_api_level_from_package("system-images;android-34;google_apis;arm64-v8a"),
            Some(34)
        );
        assert_eq!(
            manager.parse_api_level_from_package(
                "system-images;android-33;google_apis_playstore;arm64-v8a"
            ),
            Some(33)
        );
        assert_eq!(
            manager.parse_api_level_from_package("system-images;android-31;default;x86_64"),
            Some(31)
        );
        assert_eq!(
            manager.parse_api_level_from_package("system-images;android-28;google_apis;x86"),
            Some(28)
        );

        // Platform packages
        assert_eq!(
            manager.parse_api_level_from_package("platforms;android-34"),
            Some(34)
        );
        assert_eq!(
            manager.parse_api_level_from_package("platforms;android-21"),
            Some(21)
        );

        // Error cases: Pattern mismatch
        assert_eq!(
            manager.parse_api_level_from_package("invalid-package"),
            None
        );
        assert_eq!(manager.parse_api_level_from_package(""), None);
        assert_eq!(
            manager.parse_api_level_from_package("system-images;invalid;google_apis;arm64-v8a"),
            None
        );

        // Edge case: Contains non-numeric characters
        assert_eq!(
            manager.parse_api_level_from_package("system-images;android-abc;google_apis;arm64-v8a"),
            None
        );
        assert_eq!(
            manager.parse_api_level_from_package("system-images;android-;google_apis;arm64-v8a"),
            None
        );

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[test]
    fn test_find_matching_device_id() {
        // Prepare test data
        let available_devices = vec![
            ("pixel_7".to_string(), "Pixel 7".to_string()),
            ("pixel_6".to_string(), "Pixel 6".to_string()),
            ("galaxy_s22".to_string(), "Galaxy S22".to_string()),
            ("nexus_5x".to_string(), "Nexus 5X".to_string()),
            ("tv_1080p".to_string(), "Android TV (1080p)".to_string()),
            ("wear_round".to_string(), "Android Wear Round".to_string()),
            (
                "automotive_1024p".to_string(),
                "Automotive (1024p landscape)".to_string(),
            ),
        ];

        // Complete ID match
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "pixel_7"),
            Some("pixel_7".to_string())
        );

        // Complete display name match
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "Pixel 7"),
            Some("pixel_7".to_string())
        );

        // Partial match (after alphanumeric cleanup)
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "Galaxy S22"),
            Some("galaxy_s22".to_string())
        );

        // Important keyword match
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "pixel"),
            Some("pixel_7".to_string()) // First pixel found
        );

        // No match case
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "unknown_device"),
            None
        );

        // Empty string
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, ""),
            None
        );

        // Empty device list
        let empty_devices: Vec<(String, String)> = vec![];
        assert_eq!(
            AndroidManager::find_matching_device_id(&empty_devices, "pixel_7"),
            None
        );

        // TV device matching (more specific search)
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "Android TV"),
            Some("tv_1080p".to_string())
        );

        // Wear device matching (more specific search)
        assert_eq!(
            AndroidManager::find_matching_device_id(&available_devices, "Android Wear"),
            Some("wear_round".to_string())
        );
    }

    #[tokio::test]
    async fn test_run_commands_parallel() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let mock_executor = MockCommandExecutor::new()
            .with_success("cmd1", &[], "output1")
            .with_success("cmd2", &[], "output2")
            .with_success("cmd3", &["arg1"], "output3 with arg1")
            .with_error("cmd4", &[], "Command failed");

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Prepare parallel commands
        let commands = vec![
            ("cmd1".to_string(), vec![]),
            ("cmd2".to_string(), vec![]),
            ("cmd3".to_string(), vec!["arg1".to_string()]),
            ("cmd4".to_string(), vec![]), // This command will error
        ];

        // Execute in parallel
        let results = manager.run_commands_parallel(commands).await;

        // Verify results
        assert_eq!(results.len(), 4);

        // Success cases
        assert!(results[0].is_ok());
        assert_eq!(results[0].as_ref().unwrap(), "output1");

        assert!(results[1].is_ok());
        assert_eq!(results[1].as_ref().unwrap(), "output2");

        assert!(results[2].is_ok());
        assert_eq!(results[2].as_ref().unwrap(), "output3 with arg1");

        // Failure case
        assert!(results[3].is_err());
        assert!(results[3]
            .as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("Command failed"));

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[test]
    fn test_avd_list_parser_new() {
        let output = "Sample AVD list output";
        let parser = AvdListParser::new(output);

        // Confirm parser initializes correctly
        assert!(parser.current_device_info.is_none());
        assert!(parser.current_target_full.is_empty());
    }

    #[test]
    fn test_avd_list_parser_parse_single_device() {
        let avd_output = r#"
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/user/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
"#;

        let mut parser = AvdListParser::new(avd_output);

        // Parse first device
        let device = parser.parse_next_device();
        assert!(device.is_some());

        let (name, path, target, abi, device_id) = device.unwrap();
        assert_eq!(name, "Pixel_7_API_34");
        assert_eq!(path, "/Users/user/.android/avd/Pixel_7_API_34.avd");
        assert_eq!(target, "Google APIs (Google Inc.)");
        assert_eq!(abi, "google_apis/arm64-v8a");
        assert_eq!(device_id, "pixel_7 (Google)");

        // No second device
        assert!(parser.parse_next_device().is_none());
    }

    #[test]
    fn test_avd_list_parser_parse_multiple_devices() {
        let avd_output = r#"
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/user/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Galaxy_S22_API_33
    Device: galaxy_s22 (Samsung)
    Path: /Users/user/.android/avd/Galaxy_S22_API_33.avd
    Target: Android API 33
    Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/x86_64
---------
"#;

        let mut parser = AvdListParser::new(avd_output);

        // First device
        let device1 = parser.parse_next_device();
        assert!(device1.is_some());
        let (name1, _, _, _, _) = device1.unwrap();
        assert_eq!(name1, "Pixel_7_API_34");

        // Second device
        let device2 = parser.parse_next_device();
        assert!(device2.is_some());
        let (name2, _, _, _, _) = device2.unwrap();
        assert_eq!(name2, "Galaxy_S22_API_33");

        // No third device
        assert!(parser.parse_next_device().is_none());
    }

    #[test]
    fn test_avd_list_parser_empty_input() {
        let mut parser = AvdListParser::new("");
        assert!(parser.parse_next_device().is_none());
    }

    #[test]
    fn test_avd_list_parser_malformed_input() {
        let malformed_output = r#"
Some random text that doesn't match any patterns
Another line without proper formatting
---------
"#;

        let mut parser = AvdListParser::new(malformed_output);
        // Returns None when pattern doesn't match
        assert!(parser.parse_next_device().is_none());
    }

    #[tokio::test]
    async fn test_detect_api_level_for_device() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let mock_executor = MockCommandExecutor::new();
        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Method 3 fallback test: Parse from target string
        // "Based on: Android 14.0 (API level 34)" format
        let api_level = manager
            .detect_api_level_for_device(
                "test_device",
                "Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a",
            )
            .await;
        assert_eq!(api_level, 34);

        // "API level 33" format
        let api_level = manager
            .detect_api_level_for_device("test_device2", "Google APIs (API level 33)")
            .await;
        assert_eq!(api_level, 33);

        // Parse from Android version number
        let api_level = manager
            .detect_api_level_for_device(
                "test_device3",
                "Based on: Android 13 Tag/ABI: google_apis/arm64-v8a",
            )
            .await;
        assert_eq!(api_level, 33); // Android 13 -> API 33

        // Returns 0 when parsing fails
        let api_level = manager
            .detect_api_level_for_device("test_device4", "Some unknown target format")
            .await;
        assert_eq!(api_level, 0);

        // Empty string
        let api_level = manager
            .detect_api_level_for_device("test_device5", "")
            .await;
        assert_eq!(api_level, 0);

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[tokio::test]
    async fn test_get_avd_path() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        // Mock AVD list output
        let avd_list_output = r#"
Available Android Virtual Devices:
    Name: Pixel_7_API_34
    Device: pixel_7 (Google)
    Path: /Users/test/.android/avd/Pixel_7_API_34.avd
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
    Name: Galaxy_S22_API_33
    Device: galaxy_s22 (Samsung)
    Path: /Users/test/.android/avd/Galaxy_S22_API_33.avd
    Target: Android API 33
    Based on: Android 13.0 (API level 33) Tag/ABI: google_apis_playstore/x86_64
---------
"#;

        let mock_executor = MockCommandExecutor::new().with_success(
            "avdmanager",
            &["list", "avd"],
            avd_list_output,
        );

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Get path for existing AVD
        let path = manager.get_avd_path("Pixel_7_API_34").await.unwrap();
        assert!(path.is_some());
        assert_eq!(
            path.unwrap().to_str().unwrap(),
            "/Users/test/.android/avd/Pixel_7_API_34.avd"
        );

        // Check second AVD too
        let path = manager.get_avd_path("Galaxy_S22_API_33").await.unwrap();
        assert!(path.is_some());
        assert_eq!(
            path.unwrap().to_str().unwrap(),
            "/Users/test/.android/avd/Galaxy_S22_API_33.avd"
        );

        // Returns None for non-existent AVD
        let path = manager.get_avd_path("NonExistent_AVD").await.unwrap();
        assert!(path.is_none());

        // Empty string
        let path = manager.get_avd_path("").await.unwrap();
        assert!(path.is_none());

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[tokio::test]
    async fn test_fine_tune_avd_config() {
        // Save current environment variables
        let original_android_home = env::var("ANDROID_HOME").ok();

        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        // Create AVD directory structure
        let avd_dir = temp_dir.path().join("test_avd.avd");
        tokio::fs::create_dir_all(&avd_dir).await.unwrap();

        // Create initial config.ini file
        let config_path = avd_dir.join("config.ini");
        let initial_config = r#"avd.ini.encoding=UTF-8
hw.accelerometer=no
hw.audioInput=yes
hw.battery=yes
vm.heapSize=256
"#;
        tokio::fs::write(&config_path, initial_config)
            .await
            .unwrap();

        // Mock AVD list output
        let avd_list_output = format!(
            r#"
Available Android Virtual Devices:
    Name: test_avd
    Device: pixel_7 (Google)
    Path: {}
    Target: Google APIs (Google Inc.)
    Based on: Android 14.0 (API level 34) Tag/ABI: google_apis/arm64-v8a
---------
"#,
            avd_dir.to_str().unwrap()
        );

        let mock_executor = MockCommandExecutor::new().with_success(
            "avdmanager",
            &["list", "avd"],
            &avd_list_output,
        );

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Prepare DeviceConfig
        let device_config = DeviceConfig {
            name: "Test Pixel 7".to_string(),
            device_type: "pixel_7".to_string(),
            version: "14".to_string(),
            ram_size: Some("2048".to_string()),
            storage_size: Some("4096".to_string()),
            additional_options: HashMap::new(),
        };

        // Execute fine_tune_avd_config
        manager
            .fine_tune_avd_config("test_avd", &device_config, "google_apis", "arm64-v8a")
            .await
            .expect("Failed to fine tune AVD config");

        // Confirm config file was updated
        let updated_config = tokio::fs::read_to_string(&config_path).await.unwrap();

        // Confirm added/updated config items
        assert!(updated_config.contains("avd.ini.displayname=Test Pixel 7"));
        assert!(updated_config.contains("AvdId=Test_Pixel_7"));

        // Confirm original settings are preserved
        assert!(updated_config.contains("avd.ini.encoding=UTF-8"));
        assert!(updated_config.contains("hw.accelerometer=no"));
        assert!(updated_config.contains("hw.audioInput=yes"));

        // Restore environment variables
        match original_android_home {
            Some(value) => env::set_var("ANDROID_HOME", value),
            None => env::remove_var("ANDROID_HOME"),
        }
    }

    #[tokio::test]
    async fn test_fine_tune_avd_config_avd_not_found() {
        // Save current environment variables
        let original_android_home = env::var("ANDROID_HOME").ok();

        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        // Mock empty AVD list output
        let mock_executor =
            MockCommandExecutor::new().with_success("avdmanager", &["list", "avd"], "");

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        let device_config = DeviceConfig {
            name: "Test Device".to_string(),
            device_type: "pixel_7".to_string(),
            version: "14".to_string(),
            ram_size: None,
            storage_size: None,
            additional_options: HashMap::new(),
        };

        // Do nothing for non-existent AVD (normal completion)
        let result = manager
            .fine_tune_avd_config(
                "nonexistent_avd",
                &device_config,
                "google_apis",
                "arm64-v8a",
            )
            .await;
        assert!(result.is_ok());

        // Restore environment variables
        match original_android_home {
            Some(value) => env::set_var("ANDROID_HOME", value),
            None => env::remove_var("ANDROID_HOME"),
        }
    }

    #[tokio::test]
    async fn test_get_dynamic_android_version_name() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());
        let sdkmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

        let platforms_output = r#"
Installed packages:
  Path                                        | Version | Description                    | Location
  -------                                     | ------- | -------                        | -------
  platforms;android-34                        | 3       | Android SDK Platform 34        | platforms/android-34 | Android API 34, revision 2 | Android 14
  platforms;android-33                        | 3       | Android SDK Platform 33        | platforms/android-33 | Android API 33, revision 3 | Android 13
"#;

        let mock_executor = MockCommandExecutor::new()
            .with_error(
                &sdkmanager_path.to_string_lossy(),
                &["--list", "--verbose", "--include_obsolete"],
                "verbose list failed",
            )
            .with_success(
                &sdkmanager_path.to_string_lossy(),
                &["--list"],
                platforms_output,
            );

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Fall back to parsing sdkmanager --list output when targets are unavailable
        let version_name = manager.get_dynamic_android_version_name(34).await;
        assert_eq!(version_name, Some("14".to_string()));

        // Test non-existent API level
        let version_name = manager.get_dynamic_android_version_name(999).await;
        assert!(version_name.is_none());

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    #[tokio::test]
    async fn test_detect_api_level_for_device_prefers_explicit_api_level() {
        let _env_lock = acquire_test_env_lock().await;
        let temp_dir = setup_test_android_sdk();
        let _android_home = EnvVarGuard::set("ANDROID_HOME", temp_dir.path().as_os_str());

        let manager = AndroidManager::with_executor(Arc::new(MockCommandExecutor::new())).unwrap();

        let api_level = manager
            .detect_api_level_for_device(
                "Pixel_12_API",
                "Google APIs (Google Inc.) Based on: Android 12.0 (API level 31) Tag/ABI: google_apis/arm64-v8a",
            )
            .await;

        assert_eq!(api_level, 31);
    }

    #[tokio::test]
    async fn test_detect_api_level_for_device_keeps_ambiguous_version_unknown() {
        let _env_lock = acquire_test_env_lock().await;
        let temp_dir = setup_test_android_sdk();
        let _android_home = EnvVarGuard::set("ANDROID_HOME", temp_dir.path().as_os_str());

        let manager = AndroidManager::with_executor(Arc::new(MockCommandExecutor::new())).unwrap();

        let api_level = manager
            .detect_api_level_for_device(
                "Pixel_12_API",
                "Google APIs (Google Inc.) Based on: Android 12.0 Tag/ABI: google_apis/arm64-v8a",
            )
            .await;

        assert_eq!(api_level, 0);
    }

    #[tokio::test]
    async fn test_get_device_priority() {
        // Set up Android SDK environment for testing
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let mock_executor = MockCommandExecutor::new();
        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        // Phone device (high priority)
        let priority_phone = manager.get_device_priority("pixel_7").await.unwrap();
        assert!(priority_phone > 0); // High priority since it's phone category

        // TV device (medium priority)
        let priority_tv = manager.get_device_priority("tv_1080p").await.unwrap();
        assert!(priority_tv > priority_phone); // TV has higher number than Phone (indicates lower priority)

        // Unknown device (lowest priority)
        let priority_unknown = manager.get_device_priority("unknown_device").await.unwrap();
        assert!(priority_unknown > priority_tv); // Unknown has even higher number than TV (indicates lowest priority)

        // Confirm all priorities are positive values
        assert!(priority_phone > 0);
        assert!(priority_tv > 0);
        assert!(priority_unknown > 0);

        // Cleanup
        env::remove_var("ANDROID_HOME");
    }

    /// Test for get_available_devices function
    #[tokio::test]
    #[cfg(feature = "test-utils")]
    async fn test_get_available_devices() {
        let original_android_home = env::var("ANDROID_HOME").ok();
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        // Load directly from fixture
        let fixture_content = include_str!("../../../tests/fixtures/android_outputs.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_content).expect("Invalid JSON in fixture");
        let device_list_output = fixture["avdmanager_list_device"]["comprehensive"]
            .as_str()
            .expect("Device list fixture not found");

        let mock_executor = MockCommandExecutor::new().with_success(
            "avdmanager",
            &["list", "device"],
            device_list_output,
        );

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        let result = manager.get_available_devices().await;
        assert!(result.is_ok());

        let devices = result.unwrap();
        assert!(!devices.is_empty());

        // Basic validation only - not dependent on fixture data
        assert!(devices.iter().all(|d| !d.id.is_empty()));
        assert!(devices.iter().all(|d| !d.display_name.is_empty()));

        match original_android_home {
            Some(value) => env::set_var("ANDROID_HOME", value),
            None => env::remove_var("ANDROID_HOME"),
        }
    }

    /// Test for get_available_api_levels function
    #[tokio::test]
    #[cfg(feature = "test-utils")]
    async fn test_get_available_api_levels() {
        let original_android_home = env::var("ANDROID_HOME").ok();
        let temp_dir = setup_test_android_sdk();
        env::set_var("ANDROID_HOME", temp_dir.path());

        let sdkmanager_path = temp_dir.path().join("cmdline-tools/latest/bin/sdkmanager");

        // Load directly from fixture
        let fixture_content = include_str!("../../../tests/fixtures/android_outputs.json");
        let fixture: serde_json::Value =
            serde_json::from_str(fixture_content).expect("Invalid JSON in fixture");
        let sdkmanager_output = fixture["sdkmanager_list"]["system_images"]
            .as_str()
            .expect("System images fixture not found");

        let mock_executor = MockCommandExecutor::new()
            .with_success(
                "sdkmanager",
                &["--list", "--verbose", "--include_obsolete"],
                sdkmanager_output,
            )
            .with_success(
                &sdkmanager_path.to_string_lossy(),
                &["--list", "--verbose", "--include_obsolete"],
                sdkmanager_output,
            );

        let manager = match AndroidManager::with_executor(Arc::new(mock_executor)) {
            Ok(manager) => manager,
            Err(_) => {
                // Clean up environment variable
                env::remove_var("ANDROID_HOME");
                return; // Skip test if Android SDK setup fails
            }
        };

        let result = manager.get_available_api_levels().await;
        assert!(result.is_ok());

        let api_levels = result.unwrap();
        assert!(!api_levels.is_empty());

        // Basic validation not dependent on fixture
        assert!(api_levels.iter().all(|a| a.level > 0));
        assert!(api_levels.iter().all(|a| !a.version_name.is_empty()));

        match original_android_home {
            Some(value) => env::set_var("ANDROID_HOME", value),
            None => env::remove_var("ANDROID_HOME"),
        }
    }
}
