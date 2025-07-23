//! Test helper functions for creating mock data and assertions
//!
//! This module provides utility functions used across multiple test files
//! to create consistent test data and perform common assertions.

use emu::constants::{
    TEST_ANDROID_DEVICE_TYPE, TEST_API_LEVEL_30, TEST_IOS_DEVICE_TYPE, TEST_IOS_RUNTIME_17,
    TEST_IOS_VERSION_17, TEST_RAM_SIZE_DEFAULT, TEST_STORAGE_SIZE_DEFAULT,
};
use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

/// Create a test AndroidDevice with sane defaults
#[allow(dead_code)]
pub fn create_test_android_device(name: &str) -> AndroidDevice {
    AndroidDevice {
        name: name.to_string(),
        device_type: TEST_ANDROID_DEVICE_TYPE.to_string(),
        api_level: TEST_API_LEVEL_30,
        android_version_name: "11".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: TEST_RAM_SIZE_DEFAULT.to_string(),
        storage_size: TEST_STORAGE_SIZE_DEFAULT.to_string(),
    }
}

/// Create a test AndroidDevice with specific status
#[allow(dead_code)]
pub fn create_android_device_with_status(name: &str, status: DeviceStatus) -> AndroidDevice {
    AndroidDevice {
        name: name.to_string(),
        device_type: TEST_ANDROID_DEVICE_TYPE.to_string(),
        api_level: TEST_API_LEVEL_30,
        android_version_name: "11".to_string(),
        status,
        is_running: status == DeviceStatus::Running,
        ram_size: TEST_RAM_SIZE_DEFAULT.to_string(),
        storage_size: TEST_STORAGE_SIZE_DEFAULT.to_string(),
    }
}

/// Create a test iOSDevice with sane defaults
#[allow(dead_code)]
pub fn create_test_ios_device(name: &str) -> IosDevice {
    IosDevice {
        name: name.to_string(),
        udid: {
            let clean_name = name.replace(' ', "-");
            format!("test-udid-{clean_name}")
        },
        device_type: TEST_IOS_DEVICE_TYPE.to_string(),
        ios_version: TEST_IOS_VERSION_17.to_string(),
        runtime_version: TEST_IOS_RUNTIME_17.to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }
}

/// Create a test iOSDevice with specific status
#[allow(dead_code)]
pub fn create_ios_device_with_status(name: &str, status: DeviceStatus) -> IosDevice {
    IosDevice {
        name: name.to_string(),
        udid: {
            let clean_name = name.replace(' ', "-");
            format!("test-udid-{clean_name}")
        },
        device_type: TEST_IOS_DEVICE_TYPE.to_string(),
        ios_version: TEST_IOS_VERSION_17.to_string(),
        runtime_version: TEST_IOS_RUNTIME_17.to_string(),
        status,
        is_running: status == DeviceStatus::Running,
        is_available: true,
    }
}

/// Validate RAM size is within acceptable limits
#[allow(dead_code)]
pub fn assert_valid_ram_size(ram_mb: u32) {
    const MIN_RAM: u32 = 512;
    const MAX_RAM: u32 = 8192;

    assert!(
        (MIN_RAM..=MAX_RAM).contains(&ram_mb),
        "RAM size {ram_mb} MB is out of valid range [{MIN_RAM}, {MAX_RAM}] MB"
    );
}

/// Simple benchmark recorder for performance tests
#[allow(dead_code)]
pub struct BenchmarkRecorder {
    start_time: std::time::Instant,
}

impl BenchmarkRecorder {
    #[allow(dead_code)]
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    #[allow(dead_code)]
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_android_device() {
        let device = create_test_android_device("TestDevice");
        assert_eq!(device.name, "TestDevice");
        assert_eq!(device.device_type, "pixel_4");
        assert_eq!(device.api_level, 30);
        assert!(!device.is_running);
    }

    #[test]
    fn test_device_name_validation() {
        let device = create_test_android_device("ValidName123");
        assert!(!device.name.is_empty());
        assert!(!device.name.contains(' '));
    }

    #[test]
    fn test_ram_size_validation() {
        assert_valid_ram_size(2048); // Valid
    }

    #[test]
    #[should_panic(expected = "RAM size")]
    fn test_invalid_ram_size() {
        assert_valid_ram_size(128); // Too small
    }

    #[test]
    fn test_benchmark_recorder() {
        let recorder = BenchmarkRecorder::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let duration = recorder.elapsed();
        assert!(duration.as_millis() >= 10);
    }
}
