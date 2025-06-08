//! Device models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDevice {
    pub name: String,
    pub device_type: String,
    pub api_level: u32,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub ram_size: String,
    pub storage_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosDevice {
    pub name: String,
    pub udid: String,
    pub device_type: String,
    pub ios_version: String,
    pub runtime_version: String,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeviceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Creating,
    Error,
    Unknown,
}

impl Default for AndroidDevice {
    fn default() -> Self {
        Self {
            name: String::new(),
            device_type: String::new(),
            api_level: 0,
            status: DeviceStatus::Stopped,
            is_running: false,
            ram_size: "2048".to_string(),
            storage_size: "512M".to_string(),
        }
    }
}
