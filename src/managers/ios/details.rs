#[cfg(target_os = "macos")]
use super::IosManager;
#[cfg(target_os = "macos")]
use crate::constants::ios::{
    IOS_DEVICE_STATUS_BOOTED, IOS_DEVICE_STATUS_CREATING, IOS_DEVICE_STATUS_SHUTDOWN,
    IOS_RUNTIME_PREFIX,
};
#[cfg(target_os = "macos")]
use crate::constants::{
    defaults::UNKNOWN_VALUE,
    ios_devices::{
        DEVICE_KEYWORD_AIR, DEVICE_KEYWORD_IPAD, DEVICE_KEYWORD_IPHONE, DEVICE_KEYWORD_MINI,
        DEVICE_KEYWORD_PLUS, DEVICE_KEYWORD_PRO, DEVICE_KEYWORD_PRO_MAX, DEVICE_KEYWORD_SE,
        DEVICE_SIZE_11, DEVICE_SIZE_12_9, DEVICE_VERSION_13, DEVICE_VERSION_14, DEVICE_VERSION_15,
        DEVICE_VERSION_16,
    },
    numeric::BYTES_PER_MB,
    resolutions::*,
};
#[cfg(target_os = "macos")]
use crate::models::{DeviceStatus, IosDevice};
#[cfg(target_os = "macos")]
use anyhow::{Context, Result};
#[cfg(target_os = "macos")]
use serde_json::Value;
#[cfg(target_os = "macos")]
use std::path::Path;

#[cfg(target_os = "macos")]
impl IosManager {
    pub(super) fn parse_device_from_json(
        &self,
        device_json: &Value,
        runtime_str: &str,
    ) -> Result<Option<IosDevice>> {
        let device_name = device_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(UNKNOWN_VALUE);
        let udid = device_json
            .get("udid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if udid.is_empty() {
            return Ok(None);
        }

        let state_str = device_json
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or(UNKNOWN_VALUE);
        let is_available_json = device_json
            .get("isAvailable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let device_type_identifier = device_json
            .get("deviceTypeIdentifier")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let ios_version_str = runtime_str
            .replace(IOS_RUNTIME_PREFIX, "")
            .replace("-", ".");

        let ios_version_display = ios_version_str.replace("iOS.", "");
        let name = format!("{device_name} (iOS {ios_version_display})");

        let status = match state_str {
            IOS_DEVICE_STATUS_BOOTED => DeviceStatus::Running,
            IOS_DEVICE_STATUS_SHUTDOWN => DeviceStatus::Stopped,
            IOS_DEVICE_STATUS_CREATING => DeviceStatus::Creating,
            _ => DeviceStatus::Unknown,
        };
        let is_running_bool = state_str == IOS_DEVICE_STATUS_BOOTED;

        Ok(Some(IosDevice {
            name,
            udid,
            device_type: device_type_identifier,
            ios_version: ios_version_str.clone(),
            runtime_version: ios_version_str,
            status,
            is_running: is_running_bool,
            is_available: is_available_json,
        }))
    }

    pub async fn get_device_details(&self, udid: &str) -> Result<crate::models::DeviceDetails> {
        let device_output = self
            .command_executor
            .run(
                Path::new(crate::constants::commands::XCRUN),
                &[crate::constants::commands::SIMCTL, "list", "devices", "-j"],
            )
            .await
            .context("Failed to get device list")?;

        let json: Value =
            serde_json::from_str(&device_output).context("Failed to parse device JSON")?;

        let mut device_details = None;

        if let Some(devices) = json.get("devices").and_then(|v| v.as_object()) {
            for (runtime, device_list) in devices {
                if let Some(devices_array) = device_list.as_array() {
                    for device in devices_array {
                        if let Some(device_udid) = device.get("udid").and_then(|v| v.as_str()) {
                            if device_udid == udid {
                                let name = device
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(UNKNOWN_VALUE)
                                    .to_string();

                                let state = device
                                    .get("state")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(UNKNOWN_VALUE)
                                    .to_string();

                                let version = runtime
                                    .replace("com.apple.CoreSimulator.SimRuntime.iOS-", "")
                                    .replace("-", ".");

                                let device_type = device
                                    .get("deviceTypeIdentifier")
                                    .and_then(|v| v.as_str())
                                    .map(Self::parse_device_type_display_name)
                                    .unwrap_or_else(|| "Unknown".to_string());

                                let storage_size = device
                                    .get("dataPathSize")
                                    .and_then(|v| v.as_u64())
                                    .map(|size| format!("{} MB", size / BYTES_PER_MB));

                                let device_path = device
                                    .get("dataPath")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());

                                let resolution = self.get_device_resolution(&device_type);

                                device_details = Some(crate::models::DeviceDetails {
                                    name: name.clone(),
                                    status: state,
                                    platform: crate::models::Platform::Ios,
                                    device_type,
                                    api_level_or_version: format!("iOS {version}"),
                                    ram_size: None,
                                    storage_size,
                                    resolution,
                                    dpi: Some(RETINA_DISPLAY.to_string()),
                                    device_path,
                                    system_image: None,
                                    identifier: udid.to_string(),
                                });

                                break;
                            }
                        }
                    }
                    if device_details.is_some() {
                        break;
                    }
                }
            }
        }

        device_details.ok_or_else(|| anyhow::anyhow!("Device with UDID {udid} not found"))
    }

    pub(super) fn get_device_resolution(&self, device_type: &str) -> Option<String> {
        let device_lower = device_type.to_lowercase();

        if device_lower.contains(DEVICE_KEYWORD_IPHONE) {
            if device_lower.contains(DEVICE_VERSION_16)
                || device_lower.contains(DEVICE_VERSION_15)
                || device_lower.contains(DEVICE_VERSION_14)
            {
                if device_lower.contains(DEVICE_KEYWORD_PRO_MAX) {
                    return Some(IPHONE_15_PRO_MAX_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_KEYWORD_PRO) {
                    return Some(IPHONE_15_PRO_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_KEYWORD_PLUS) {
                    return Some(IPHONE_15_PRO_MAX_RESOLUTION.to_string());
                } else {
                    return Some(IPHONE_15_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_SE) {
                return Some(IPHONE_SE_RESOLUTION.to_string());
            }
        }

        if device_lower.contains(DEVICE_KEYWORD_IPAD) {
            if device_lower.contains(DEVICE_KEYWORD_PRO) {
                if device_lower.contains(DEVICE_VERSION_13)
                    || device_lower.contains(DEVICE_SIZE_12_9)
                {
                    return Some(IPAD_PRO_12_9_RESOLUTION.to_string());
                } else if device_lower.contains(DEVICE_SIZE_11) {
                    return Some(IPAD_PRO_11_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_AIR) {
                if device_lower.contains(DEVICE_VERSION_13) {
                    return Some(IPAD_AIR_13_RESOLUTION.to_string());
                } else {
                    return Some(IPAD_AIR_RESOLUTION.to_string());
                }
            } else if device_lower.contains(DEVICE_KEYWORD_MINI) {
                return Some(IPAD_MINI_RESOLUTION.to_string());
            } else {
                return Some(IPAD_RESOLUTION.to_string());
            }
        }

        None
    }
}
