//! Mock manager factories for integration tests.
//!
//! Provides pre-configured MockDeviceManager instances.
//! When the mock API changes, only this file needs updating.

#[cfg(feature = "test-utils")]
use emu::managers::mock::{MockDevice, MockDeviceManager};
#[cfg(feature = "test-utils")]
use emu::models::DeviceStatus;

/// Create a MockDeviceManager pre-loaded with default Android devices.
#[cfg(feature = "test-utils")]
pub fn mock_android_manager() -> MockDeviceManager {
    let manager = MockDeviceManager::new_android();
    // new_android() already includes Pixel_4_API_30 and Pixel_4_API_33
    manager
}

/// Create a MockDeviceManager pre-loaded with default iOS devices.
#[cfg(feature = "test-utils")]
pub fn mock_ios_manager() -> MockDeviceManager {
    let manager = MockDeviceManager::new_ios();
    // new_ios() already includes iPhone 14 and iPad Air
    manager
}

/// Create an Android manager with a custom device set.
#[cfg(feature = "test-utils")]
pub fn mock_android_manager_with(devices: Vec<(&str, &str, DeviceStatus)>) -> MockDeviceManager {
    let manager = MockDeviceManager::new_android();
    for (id, name, status) in devices {
        manager.add_device(MockDevice {
            id: id.to_string(),
            name: name.to_string(),
            status,
            api_level: Some("34".to_string()),
            device_type: "pixel_7".to_string(),
        });
    }
    manager
}

/// Create an iOS manager with a custom device set.
#[cfg(feature = "test-utils")]
pub fn mock_ios_manager_with(devices: Vec<(&str, &str, DeviceStatus)>) -> MockDeviceManager {
    let manager = MockDeviceManager::new_ios();
    for (id, name, status) in devices {
        manager.add_device(MockDevice {
            id: id.to_string(),
            name: name.to_string(),
            status,
            api_level: Some("17.0".to_string()),
            device_type: "iPhone 15".to_string(),
        });
    }
    manager
}
