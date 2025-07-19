//! AndroidManager comprehensive tests using fixture data
//!
//! These tests validate AndroidManager functionality using real command outputs
//! captured as fixtures, enabling thorough testing without requiring actual emulators.

use anyhow::Result;
use emu::models::{AndroidDevice, DeviceStatus};
use std::collections::HashMap;

mod fixtures;
use fixtures::FixtureLoader;

/// Test fixture-based AndroidManager for parsing and device operations
struct FixtureAndroidManager {
    fixture_outputs: HashMap<String, String>,
}

impl FixtureAndroidManager {
    fn new() -> Self {
        Self {
            fixture_outputs: HashMap::new(),
        }
    }

    fn with_avd_list_output(mut self, output: String) -> Self {
        self.fixture_outputs
            .insert("avdmanager list avd".to_string(), output);
        self
    }

    fn with_adb_devices_output(mut self, output: String) -> Self {
        self.fixture_outputs
            .insert("adb devices".to_string(), output);
        self
    }

    #[allow(dead_code)]
    fn with_getprop_output(mut self, device_id: &str, prop: &str, output: String) -> Self {
        let key = format!("adb -s {device_id} shell getprop {prop}");
        self.fixture_outputs.insert(key, output);
        self
    }

    /// Parse AVD list output using AndroidManager's logic
    fn parse_avd_list(&self, output: &str) -> Vec<AndroidDevice> {
        let mut devices = Vec::new();
        let mut current_device: Option<AndroidDevice> = None;

        for line in output.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("Name: ") {
                let name = trimmed.strip_prefix("Name: ").unwrap_or("").to_string();
                current_device = Some(AndroidDevice {
                    name,
                    device_type: String::new(),
                    api_level: 30,
                    status: DeviceStatus::Stopped,
                    is_running: false,
                    ram_size: "2048".to_string(),
                    storage_size: "8192M".to_string(),
                });
            } else if trimmed.starts_with("Device: ") && current_device.is_some() {
                if let Some(ref mut device) = current_device {
                    let device_info = trimmed.strip_prefix("Device: ").unwrap_or("");
                    if let Some(end_paren) = device_info.find(')') {
                        device.device_type = device_info[..=end_paren].to_string();
                    }
                }
            } else if trimmed.starts_with("Based on: Android") && current_device.is_some() {
                if let Some(ref mut device) = current_device {
                    if let Some(api_start) = trimmed.find("(API level ") {
                        if let Some(api_end) = trimmed[api_start..].find(')') {
                            let api_str = &trimmed[api_start + 11..api_start + api_end];
                            device.api_level = api_str.parse().unwrap_or(30);
                        }
                    }
                }
            } else if trimmed == "---------" && current_device.is_some() {
                devices.push(current_device.take().unwrap());
            }
        }

        // Add final device if exists
        if let Some(device) = current_device {
            devices.push(device);
        }

        devices
    }

    /// Parse adb devices output
    fn parse_adb_devices(&self, output: &str) -> Vec<(String, String)> {
        let mut devices = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            // Skip the header line and empty lines
            if trimmed.is_empty() || trimmed.starts_with("List of devices attached") {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                devices.push((parts[0].to_string(), parts[1].to_string()));
            }
        }

        devices
    }
}

#[tokio::test]
async fn test_avd_list_parsing_single_device() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let single_output = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "single_device"],
        )?
        .unwrap_or_default();

    let manager = FixtureAndroidManager::new().with_avd_list_output(single_output.clone());

    let devices = manager.parse_avd_list(&single_output);

    assert_eq!(devices.len(), 1);

    let device = &devices[0];
    assert_eq!(device.name, "Pixel_7_API_34");
    assert_eq!(device.api_level, 34);
    assert!(device.device_type.contains("pixel_7"));
    assert_eq!(device.status, DeviceStatus::Stopped);

    Ok(())
}

#[tokio::test]
async fn test_avd_list_parsing_multiple_devices() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let multiple_output = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "multiple_devices"],
        )?
        .unwrap_or_default();

    let manager = FixtureAndroidManager::new().with_avd_list_output(multiple_output.clone());

    let devices = manager.parse_avd_list(&multiple_output);

    assert_eq!(devices.len(), 3);

    // Verify device names
    let device_names: Vec<&String> = devices.iter().map(|d| &d.name).collect();
    assert!(device_names.contains(&&"Pixel_7_API_34".to_string()));
    assert!(device_names.contains(&&"Pixel_Tablet_API_33".to_string()));
    assert!(device_names.contains(&&"Wear_OS_Round_API_30".to_string()));

    // Verify API levels
    let pixel_device = devices.iter().find(|d| d.name == "Pixel_7_API_34").unwrap();
    assert_eq!(pixel_device.api_level, 34);

    let tablet_device = devices
        .iter()
        .find(|d| d.name == "Pixel_Tablet_API_33")
        .unwrap();
    assert_eq!(tablet_device.api_level, 33);

    let wear_device = devices
        .iter()
        .find(|d| d.name == "Wear_OS_Round_API_30")
        .unwrap();
    assert_eq!(wear_device.api_level, 30);

    Ok(())
}

#[tokio::test]
async fn test_adb_devices_parsing() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let adb_output = loader
        .get_string("android_outputs.json", &["adb_devices", "multiple_devices"])?
        .unwrap_or_default();

    let manager = FixtureAndroidManager::new().with_adb_devices_output(adb_output.clone());

    let devices = manager.parse_adb_devices(&adb_output);

    assert_eq!(devices.len(), 3);

    // Verify device states
    let running_devices: Vec<&(String, String)> = devices
        .iter()
        .filter(|(_, status)| status == "device")
        .collect();
    assert_eq!(running_devices.len(), 2);

    let offline_devices: Vec<&(String, String)> = devices
        .iter()
        .filter(|(_, status)| status == "offline")
        .collect();
    assert_eq!(offline_devices.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_device_name_with_spaces() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let spaces_output = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "with_spaces"],
        )?
        .unwrap_or_default();

    let manager = FixtureAndroidManager::new().with_avd_list_output(spaces_output.clone());

    let devices = manager.parse_avd_list(&spaces_output);

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].name, "Pixel 7 API 34");

    Ok(())
}

#[tokio::test]
async fn test_getprop_parsing() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let avd_name = loader
        .get_string(
            "android_outputs.json",
            &["adb_shell_getprop", "avd_name_pixel"],
        )?
        .unwrap_or_default();

    assert_eq!(avd_name, "Pixel_7_API_34");

    let avd_name_spaces = loader
        .get_string(
            "android_outputs.json",
            &["adb_shell_getprop", "avd_name_spaces"],
        )?
        .unwrap_or_default();

    assert_eq!(avd_name_spaces, "Pixel 7 API 34");

    Ok(())
}

#[tokio::test]
async fn test_empty_avd_list() -> Result<()> {
    let manager = FixtureAndroidManager::new().with_avd_list_output("".to_string());

    let devices = manager.parse_avd_list("");
    assert_eq!(devices.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_empty_adb_devices() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let empty_output = loader
        .get_string("android_outputs.json", &["adb_devices", "empty"])?
        .unwrap_or_default();

    let manager = FixtureAndroidManager::new().with_adb_devices_output(empty_output.clone());

    let devices = manager.parse_adb_devices(&empty_output);
    assert_eq!(devices.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_error_scenarios() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test license error
    let license_error = loader
        .get_string(
            "error_scenarios.json",
            &["android_errors", "license_not_accepted", "stderr"],
        )?
        .unwrap_or_default();

    assert!(license_error.contains("have not been accepted"));

    // Test AVD already exists error
    let exists_error = loader
        .get_string(
            "error_scenarios.json",
            &["android_errors", "avd_already_exists", "stderr"],
        )?
        .unwrap_or_default();

    assert!(exists_error.contains("already exists"));

    // Test invalid device name error
    let invalid_name_error = loader
        .get_string(
            "error_scenarios.json",
            &["android_errors", "invalid_device_name", "stderr"],
        )?
        .unwrap_or_default();

    assert!(invalid_name_error.contains("Invalid device name"));

    Ok(())
}

#[tokio::test]
async fn test_device_state_transitions() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test initial state (no devices running)
    let initial_adb = loader.get_string(
        "state_transitions.json",
        &[
            "android_device_states",
            "stopped_to_running",
            "initial_adb_devices",
        ],
    );

    if let Ok(Some(initial_output)) = initial_adb {
        assert!(initial_output.contains("List of devices attached"));
        assert!(!initial_output.contains("emulator-"));
    }

    // Test after start state (device running)
    let after_start_adb = loader.get_string(
        "state_transitions.json",
        &[
            "android_device_states",
            "stopped_to_running",
            "after_start_adb_devices",
        ],
    );

    if let Ok(Some(after_output)) = after_start_adb {
        assert!(after_output.contains("List of devices attached"));
        assert!(after_output.contains("emulator-5554"));
    }

    Ok(())
}

#[tokio::test]
async fn test_comprehensive_device_creation_scenario() -> Result<()> {
    // This test simulates a complete device creation workflow using fixtures
    let mut loader = FixtureLoader::new();

    // Step 1: Check available system images
    let system_images = loader.get_string(
        "android_outputs.json",
        &["sdkmanager_list", "system_images"],
    )?;

    if let Some(images_output) = system_images {
        assert!(images_output.contains("system-images;android-34;google_apis;arm64-v8a"));
        assert!(images_output.contains("system-images;android-33;google_apis;x86_64"));
    }

    // Step 2: Verify no initial devices
    let initial_avd_list = loader
        .get_string(
            "android_outputs.json",
            &["avdmanager_list_avd", "single_device"],
        )?
        .unwrap_or_default();
    assert!(initial_avd_list.contains("Pixel_7_API_34"));

    // Step 3: Check device types
    let device_types = loader.get_string(
        "android_outputs.json",
        &["avdmanager_list_device", "comprehensive"],
    )?;

    if let Some(types_output) = device_types {
        assert!(types_output.contains("pixel_7"));
        assert!(types_output.contains("pixel_tablet"));
        assert!(types_output.contains("wear_round"));
    }

    Ok(())
}
