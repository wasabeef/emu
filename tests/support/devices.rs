//! Device factory functions for tests.
//!
//! Consolidated from tests/common/helpers.rs and tests/unit/common/mod.rs.
//! Single source of truth for creating test devices.

use emu::models::{AndroidDevice, DeviceStatus, IosDevice};

/// Create an Android device with sensible defaults.
pub fn android_device(name: &str) -> AndroidDevice {
    AndroidDevice {
        name: name.to_string(),
        device_type: "pixel_7".to_string(),
        api_level: 34,
        android_version_name: "14".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192M".to_string(),
    }
}

/// Create an Android device with a specific status.
pub fn android_device_with_status(name: &str, status: DeviceStatus) -> AndroidDevice {
    AndroidDevice {
        status,
        is_running: status == DeviceStatus::Running,
        ..android_device(name)
    }
}

/// Create an Android device with a specific API level.
pub fn android_device_with_api(name: &str, api_level: u32) -> AndroidDevice {
    AndroidDevice {
        api_level,
        android_version_name: format!("API {api_level}"),
        ..android_device(name)
    }
}

/// Create a list of Android devices.
pub fn android_device_list(count: usize) -> Vec<AndroidDevice> {
    (0..count)
        .map(|i| android_device(&format!("Device{i}")))
        .collect()
}

/// Create an iOS device with sensible defaults.
pub fn ios_device(name: &str) -> IosDevice {
    let clean = name.replace(' ', "-");
    IosDevice {
        name: name.to_string(),
        udid: format!("test-udid-{clean}"),
        device_type: "iPhone 15".to_string(),
        ios_version: "17.0".to_string(),
        runtime_version: "iOS 17.0".to_string(),
        status: DeviceStatus::Stopped,
        is_running: false,
        is_available: true,
    }
}

/// Create an iOS device with a specific status.
pub fn ios_device_with_status(name: &str, status: DeviceStatus) -> IosDevice {
    IosDevice {
        status,
        is_running: status == DeviceStatus::Running,
        ..ios_device(name)
    }
}

/// Create a list of iOS devices.
pub fn ios_device_list(count: usize) -> Vec<IosDevice> {
    (0..count)
        .map(|i| ios_device(&format!("iPhone{i}")))
        .collect()
}
