//! Common helpers for unit tests
//!
//! Provides test utilities and helper functions for unit tests.

pub mod helpers {
    use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

    /// Creates a test Android device with default values
    pub fn create_test_android_device(name: &str) -> AndroidDevice {
        AndroidDevice {
            name: name.to_string(),
            device_type: "pixel_7".to_string(),
            api_level: 33,
            android_version_name: "13".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "8192M".to_string(),
        }
    }

    /// Creates a test iOS device with default values
    pub fn create_test_ios_device(name: &str) -> IosDevice {
        IosDevice {
            name: name.to_string(),
            udid: format!("test-udid-{name}"),
            device_type: "iPhone 15".to_string(),
            ios_version: "17.0".to_string(),
            runtime_version: "iOS 17.0".to_string(),
            status: DeviceStatus::Stopped,
            is_running: false,
            is_available: true,
        }
    }

    /// Creates an Android device with a specific status
    pub fn create_android_device_with_status(name: &str, status: DeviceStatus) -> AndroidDevice {
        let mut device = create_test_android_device(name);
        device.status = status;
        device.is_running = matches!(status, DeviceStatus::Running);
        device
    }

    /// Creates an iOS device with a specific status
    pub fn create_ios_device_with_status(name: &str, status: DeviceStatus) -> IosDevice {
        let mut device = create_test_ios_device(name);
        device.status = status;
        device.is_running = matches!(status, DeviceStatus::Running);
        device
    }

    /// Creates an Android device with a specific API level
    pub fn create_android_device_with_api(name: &str, api_level: u32) -> AndroidDevice {
        let mut device = create_test_android_device(name);
        device.api_level = api_level;
        device.android_version_name = format!("API {api_level}");
        device
    }

    /// Creates a list of Android devices for testing
    pub fn create_android_device_list(count: usize) -> Vec<AndroidDevice> {
        (0..count)
            .map(|i| create_test_android_device(&format!("Device{i}")))
            .collect()
    }

    /// Creates a list of iOS devices for testing
    pub fn create_ios_device_list(count: usize) -> Vec<IosDevice> {
        (0..count)
            .map(|i| create_test_ios_device(&format!("iPhone{i}")))
            .collect()
    }

    /// Asserts that a device status matches expected
    pub fn assert_device_status(actual: DeviceStatus, expected: DeviceStatus) {
        assert_eq!(actual, expected, "Device status mismatch");
    }

    /// Asserts that an API level is within valid range
    pub fn assert_api_level_in_range(api_level: u32, min: u32, max: u32) {
        assert!(
            api_level >= min && api_level <= max,
            "API level {api_level} is not in range {min}-{max}"
        );
    }

    /// Asserts that a RAM size is valid
    pub fn assert_valid_ram_size(ram_mb: &str) {
        let ram_value = ram_mb.parse::<u32>().unwrap_or(0);
        assert!(
            (512..=16384).contains(&ram_value),
            "RAM size {ram_mb}MB is not in valid range 512-16384"
        );
    }
}

/// Helper function to create a mock Android SDK environment for testing
///
/// This function creates a temporary directory structure that mimics
/// an Android SDK installation.
#[allow(dead_code)]
pub fn setup_mock_android_sdk() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let sdk_path = temp_dir.path();

    // Create necessary directory structure
    std::fs::create_dir_all(sdk_path.join("cmdline-tools/latest/bin")).unwrap();
    std::fs::create_dir_all(sdk_path.join("emulator")).unwrap();
    std::fs::create_dir_all(sdk_path.join("platform-tools")).unwrap();

    // Create empty executable scripts
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let avdmanager = sdk_path.join("cmdline-tools/latest/bin/avdmanager");
        std::fs::write(&avdmanager, "#!/bin/sh\necho 'Mock avdmanager'\n").unwrap();
        std::fs::set_permissions(&avdmanager, std::fs::Permissions::from_mode(0o755)).unwrap();

        let adb = sdk_path.join("platform-tools/adb");
        std::fs::write(&adb, "#!/bin/sh\necho 'Mock adb'\n").unwrap();
        std::fs::set_permissions(&adb, std::fs::Permissions::from_mode(0o755)).unwrap();

        let emulator = sdk_path.join("emulator/emulator");
        std::fs::write(&emulator, "#!/bin/sh\necho 'Mock emulator'\n").unwrap();
        std::fs::set_permissions(&emulator, std::fs::Permissions::from_mode(0o755)).unwrap();

        let sdkmanager = sdk_path.join("cmdline-tools/latest/bin/sdkmanager");
        std::fs::write(&sdkmanager, "#!/bin/sh\necho 'Mock sdkmanager'\n").unwrap();
        std::fs::set_permissions(&sdkmanager, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(windows)]
    {
        let avdmanager = sdk_path.join("cmdline-tools/latest/bin/avdmanager.bat");
        std::fs::write(&avdmanager, "@echo off\necho Mock avdmanager\n").unwrap();

        let adb = sdk_path.join("platform-tools/adb.exe");
        std::fs::write(&adb, "@echo off\necho Mock adb\n").unwrap();

        let emulator = sdk_path.join("emulator/emulator.exe");
        std::fs::write(&emulator, "@echo off\necho Mock emulator\n").unwrap();

        let sdkmanager = sdk_path.join("cmdline-tools/latest/bin/sdkmanager.bat");
        std::fs::write(&sdkmanager, "@echo off\necho Mock sdkmanager\n").unwrap();
    }

    temp_dir
}
