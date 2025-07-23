//! Mock implementation of DeviceManager for testing purposes.
//!
//! This module provides a mock implementation that doesn't require
//! actual Android SDK or iOS tools, enabling true emulator-independent testing.

use crate::constants::android::{EMULATOR_PORT_BASE, EMULATOR_PORT_INCREMENT};
use crate::managers::common::{DeviceConfig, DeviceManager};
use crate::models::{AndroidDevice, DeviceStatus, IosDevice};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents an operation performed on the mock device manager
#[derive(Debug, Clone, PartialEq)]
pub enum MockOperation {
    ListDevices,
    StartDevice(String),
    StopDevice(String),
    CreateDevice { name: String, device_type: String },
    DeleteDevice(String),
    WipeDevice(String),
    GetDeviceDetails(String),
}

/// Mock implementation of DeviceManager for testing
#[derive(Clone)]
pub struct MockDeviceManager {
    /// Platform type (Android or iOS)
    platform: String,
    /// Mock devices stored by ID
    devices: Arc<Mutex<HashMap<String, MockDevice>>>,
    /// Record of operations performed
    operations: Arc<Mutex<Vec<MockOperation>>>,
    /// Configurable behavior for operations
    behavior: Arc<Mutex<MockBehavior>>,
}

/// Represents a mock device
#[derive(Debug, Clone)]
pub struct MockDevice {
    pub id: String,
    pub name: String,
    pub status: DeviceStatus,
    pub api_level: Option<String>,
    pub device_type: String,
}

/// Configurable behavior for mock operations
#[derive(Debug, Default)]
pub struct MockBehavior {
    /// Operations that should fail with an error
    pub failing_operations: HashMap<String, String>,
    /// Delay for operations (in milliseconds)
    pub operation_delays: HashMap<String, u64>,
}

impl MockDeviceManager {
    /// Create a new mock Android manager with default devices
    pub fn new_android() -> Self {
        let mut devices = HashMap::new();

        // Add some default Android devices
        devices.insert(
            "emulator-5554".to_string(),
            MockDevice {
                id: "emulator-5554".to_string(),
                name: "Pixel_4_API_30".to_string(),
                status: DeviceStatus::Stopped,
                api_level: Some("30".to_string()),
                device_type: "pixel_4".to_string(),
            },
        );

        devices.insert(
            "emulator-5556".to_string(),
            MockDevice {
                id: "emulator-5556".to_string(),
                name: "Pixel_6_API_33".to_string(),
                status: DeviceStatus::Stopped,
                api_level: Some("33".to_string()),
                device_type: "pixel_6".to_string(),
            },
        );

        Self {
            platform: "android".to_string(),
            devices: Arc::new(Mutex::new(devices)),
            operations: Arc::new(Mutex::new(Vec::new())),
            behavior: Arc::new(Mutex::new(MockBehavior::default())),
        }
    }

    /// Create a new mock iOS manager with default devices
    pub fn new_ios() -> Self {
        let mut devices = HashMap::new();

        // Add some default iOS devices
        devices.insert(
            "12345678-1234-1234-1234-123456789012".to_string(),
            MockDevice {
                id: "12345678-1234-1234-1234-123456789012".to_string(),
                name: "iPhone 14".to_string(),
                status: DeviceStatus::Stopped,
                api_level: Some("16.0".to_string()),
                device_type: "iPhone 14".to_string(),
            },
        );

        devices.insert(
            "87654321-4321-4321-4321-210987654321".to_string(),
            MockDevice {
                id: "87654321-4321-4321-4321-210987654321".to_string(),
                name: "iPad Pro".to_string(),
                status: DeviceStatus::Stopped,
                api_level: Some("16.0".to_string()),
                device_type: "iPad Pro (12.9-inch)".to_string(),
            },
        );

        Self {
            platform: "ios".to_string(),
            devices: Arc::new(Mutex::new(devices)),
            operations: Arc::new(Mutex::new(Vec::new())),
            behavior: Arc::new(Mutex::new(MockBehavior::default())),
        }
    }

    /// Configure a specific operation to fail
    pub fn configure_failure(&self, operation: &str, error_message: &str) {
        let mut behavior = self.behavior.lock().unwrap();
        behavior
            .failing_operations
            .insert(operation.to_string(), error_message.to_string());
    }

    /// Configure a delay for a specific operation
    pub fn configure_delay(&self, operation: &str, delay_ms: u64) {
        let mut behavior = self.behavior.lock().unwrap();
        behavior
            .operation_delays
            .insert(operation.to_string(), delay_ms);
    }

    /// Get the list of operations performed
    pub fn get_operations(&self) -> Vec<MockOperation> {
        self.operations.lock().unwrap().clone()
    }

    /// Clear the operation history
    pub fn clear_operations(&self) {
        self.operations.lock().unwrap().clear();
    }

    /// Clear all behavior configurations (failures and delays)
    pub fn clear_behavior(&self) {
        let mut behavior = self.behavior.lock().unwrap();
        behavior.failing_operations.clear();
        behavior.operation_delays.clear();
    }

    /// Assert that a specific operation was called
    pub fn assert_operation_called(&self, expected: &MockOperation) -> bool {
        self.operations.lock().unwrap().contains(expected)
    }

    /// Add a mock device
    pub fn add_device(&self, device: MockDevice) {
        let mut devices = self.devices.lock().unwrap();
        devices.insert(device.id.clone(), device);
    }

    /// Record an operation
    fn record_operation(&self, op: MockOperation) {
        self.operations.lock().unwrap().push(op);
    }

    /// Check if an operation should fail
    fn check_failure(&self, operation: &str) -> Result<()> {
        let behavior = self.behavior.lock().unwrap();
        if let Some(error_msg) = behavior.failing_operations.get(operation) {
            return Err(anyhow::anyhow!("{}", error_msg));
        }
        Ok(())
    }

    /// Apply configured delay for an operation
    async fn apply_delay(&self, operation: &str) {
        let delay_ms = {
            let behavior = self.behavior.lock().unwrap();
            behavior.operation_delays.get(operation).copied()
        };
        if let Some(delay_ms) = delay_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }
}

impl DeviceManager for MockDeviceManager {
    type Device = Box<dyn crate::models::device::Device>;

    async fn list_devices(&self) -> Result<Vec<Self::Device>> {
        self.record_operation(MockOperation::ListDevices);
        self.check_failure("list_devices")?;
        self.apply_delay("list_devices").await;

        let devices = self.devices.lock().unwrap();
        let mut result: Vec<Self::Device> = Vec::new();

        for device in devices.values() {
            if self.platform == "android" {
                let api_level = device
                    .api_level
                    .clone()
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(30);
                result.push(Box::new(AndroidDevice {
                    name: device.name.clone(),
                    device_type: device.device_type.clone(),
                    api_level,
                    android_version_name: format!("API {api_level}"),
                    status: device.status,
                    is_running: device.status == DeviceStatus::Running,
                    ram_size: "2048".to_string(),
                    storage_size: "8192M".to_string(),
                }));
            } else {
                result.push(Box::new(IosDevice {
                    name: device.name.clone(),
                    udid: device.id.clone(),
                    device_type: device.device_type.clone(),
                    ios_version: device.api_level.clone().unwrap_or_default(),
                    runtime_version: format!(
                        "iOS {}",
                        device.api_level.clone().unwrap_or_default()
                    ),
                    status: device.status,
                    is_running: device.status == DeviceStatus::Running,
                    is_available: true,
                }));
            }
        }

        Ok(result)
    }

    async fn start_device(&self, device_id: &str) -> Result<()> {
        self.record_operation(MockOperation::StartDevice(device_id.to_string()));
        self.apply_delay("start_device").await;
        self.check_failure("start_device")?;

        let mut devices = self.devices.lock().unwrap();

        // Try to find by ID first, then by name
        let key_to_update = devices
            .iter()
            .find(|(_, device)| device.id == device_id || device.name == device_id)
            .map(|(key, _)| key.clone());

        if let Some(key) = key_to_update {
            if let Some(device) = devices.get_mut(&key) {
                device.status = DeviceStatus::Running;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Device not found: {}", device_id))
        }
    }

    async fn stop_device(&self, device_id: &str) -> Result<()> {
        self.record_operation(MockOperation::StopDevice(device_id.to_string()));
        self.apply_delay("stop_device").await;
        self.check_failure("stop_device")?;

        let mut devices = self.devices.lock().unwrap();

        // Try to find by ID first, then by name
        let key_to_update = devices
            .iter()
            .find(|(_, device)| device.id == device_id || device.name == device_id)
            .map(|(key, _)| key.clone());

        if let Some(key) = key_to_update {
            if let Some(device) = devices.get_mut(&key) {
                device.status = DeviceStatus::Stopped;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Device not found: {}", device_id))
        }
    }

    async fn create_device(&self, config: &DeviceConfig) -> Result<()> {
        self.record_operation(MockOperation::CreateDevice {
            name: config.name.clone(),
            device_type: config.device_type.clone(),
        });
        self.apply_delay("create_device").await;
        self.check_failure("create_device")?;

        let new_id = if self.platform == "android" {
            format!(
                "emulator-{}",
                self.devices.lock().unwrap().len() * EMULATOR_PORT_INCREMENT as usize
                    + EMULATOR_PORT_BASE as usize
            )
        } else {
            format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                rand::random::<u32>(),
                rand::random::<u16>(),
                rand::random::<u16>(),
                rand::random::<u16>(),
                rand::random::<u64>() & 0xffffffffffff
            )
        };

        let device = MockDevice {
            id: new_id.clone(),
            name: config.name.clone(),
            status: DeviceStatus::Stopped,
            api_level: Some(config.version.clone()),
            device_type: config.device_type.clone(),
        };

        self.add_device(device);
        Ok(())
    }

    async fn delete_device(&self, device_id: &str) -> Result<()> {
        self.record_operation(MockOperation::DeleteDevice(device_id.to_string()));
        self.apply_delay("delete_device").await;
        self.check_failure("delete_device")?;

        let mut devices = self.devices.lock().unwrap();

        // Try to find by ID first, then by name
        let key_to_remove = devices
            .iter()
            .find(|(_, device)| device.id == device_id || device.name == device_id)
            .map(|(key, _)| key.clone());

        if let Some(key) = key_to_remove {
            devices.remove(&key);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Device not found: {}", device_id))
        }
    }

    async fn wipe_device(&self, device_id: &str) -> Result<()> {
        self.record_operation(MockOperation::WipeDevice(device_id.to_string()));
        self.apply_delay("wipe_device").await;
        self.check_failure("wipe_device")?;

        let devices = self.devices.lock().unwrap();

        // Try to find by ID first, then by name
        let device_exists = devices
            .iter()
            .any(|(_, device)| device.id == device_id || device.name == device_id);

        if device_exists {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Device not found: {}", device_id))
        }
    }

    async fn is_available(&self) -> bool {
        // Mock managers are always available
        true
    }
}

/// Implementation of UnifiedDeviceManager for MockDeviceManager
#[cfg(any(test, feature = "test-utils"))]
#[async_trait::async_trait]
impl crate::managers::common::UnifiedDeviceManager for MockDeviceManager {
    async fn list_devices(&self) -> Result<Vec<Box<dyn crate::models::device::Device>>> {
        <Self as DeviceManager>::list_devices(self).await
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
    use crate::managers::common::DeviceConfig;

    #[tokio::test]
    async fn test_mock_device_manager_operations() {
        let manager = MockDeviceManager::new_android();

        // Test list devices
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 2);

        // Test operation recording
        assert!(manager.assert_operation_called(&MockOperation::ListDevices));

        // Test start device
        manager.start_device("emulator-5554").await.unwrap();
        assert!(manager
            .assert_operation_called(&MockOperation::StartDevice("emulator-5554".to_string())));

        // Verify device status changed
        let devices = manager.list_devices().await.unwrap();
        let device = devices
            .iter()
            .find(|d| d.name() == "Pixel_4_API_30")
            .unwrap();
        assert_eq!(device.status(), &DeviceStatus::Running);
    }

    #[tokio::test]
    async fn test_mock_failure_configuration() {
        let manager = MockDeviceManager::new_ios();

        // Configure start_device to fail
        manager.configure_failure("start_device", "Simulated failure");

        // Attempt to start device should fail
        let result = manager
            .start_device("12345678-1234-1234-1234-123456789012")
            .await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Simulated failure");
    }

    #[tokio::test]
    async fn test_create_and_delete_device() {
        let manager = MockDeviceManager::new_android();

        // Create a new device
        let config = DeviceConfig::new(
            "Test Device".to_string(),
            "pixel_5".to_string(),
            "31".to_string(),
        );
        manager.create_device(&config).await.unwrap();

        // Verify creation
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 3);

        // Find and delete the new device
        let new_device = devices.iter().find(|d| d.name() == "Test Device").unwrap();
        manager.delete_device(new_device.id()).await.unwrap();

        // Verify deletion
        let devices = manager.list_devices().await.unwrap();
        assert_eq!(devices.len(), 2);
    }
}
