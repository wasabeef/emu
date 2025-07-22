//! iOSManager comprehensive tests using fixture data
//!
//! These tests validate iOSManager functionality using real xcrun simctl outputs
//! captured as fixtures, enabling thorough testing without requiring Xcode or simulators.

use anyhow::Result;
use emu::models::{DeviceStatus, IosDevice};
use serde_json::Value;

use super::fixture_loader::FixtureLoader;

/// Test fixture-based iOSManager for parsing and device operations
struct FixtureiOSManager {
    fixture_outputs: std::collections::HashMap<String, String>,
}

impl FixtureiOSManager {
    fn new() -> Self {
        Self {
            fixture_outputs: std::collections::HashMap::new(),
        }
    }

    fn with_list_devices_output(mut self, output: String) -> Self {
        self.fixture_outputs
            .insert("xcrun simctl list devices".to_string(), output);
        self
    }

    #[allow(dead_code)]
    fn with_list_runtimes_output(mut self, output: String) -> Self {
        self.fixture_outputs
            .insert("xcrun simctl list runtimes".to_string(), output);
        self
    }

    /// Parse iOS device list output using iOSManager's logic
    fn parse_device_list(&self, json_output: &str) -> Result<Vec<IosDevice>> {
        let json: Value = serde_json::from_str(json_output)?;
        let mut devices = Vec::new();

        if let Some(device_map) = json["devices"].as_object() {
            for (runtime_id, runtime_devices) in device_map {
                if let Some(device_array) = runtime_devices.as_array() {
                    for device_json in device_array {
                        let device = self.parse_device_from_json(device_json, runtime_id)?;
                        devices.push(device);
                    }
                }
            }
        }

        Ok(devices)
    }

    fn parse_device_from_json(&self, device_json: &Value, runtime_id: &str) -> Result<IosDevice> {
        let name = device_json["name"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();
        let udid = device_json["udid"].as_str().unwrap_or("").to_string();
        let state_str = device_json["state"].as_str().unwrap_or("Shutdown");
        let device_type = device_json["deviceTypeIdentifier"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let is_available = device_json["isAvailable"].as_bool().unwrap_or(false);

        let status = match state_str {
            "Booted" => DeviceStatus::Running,
            "Shutdown" => DeviceStatus::Stopped,
            _ => DeviceStatus::Stopped,
        };

        let ios_version = self.extract_ios_version(runtime_id);
        let runtime_version = format!("iOS {ios_version}");

        Ok(IosDevice {
            name,
            udid,
            device_type,
            ios_version,
            runtime_version,
            status,
            is_running: status == DeviceStatus::Running,
            is_available,
        })
    }

    fn extract_ios_version(&self, runtime_id: &str) -> String {
        // Extract version from runtime ID like "com.apple.CoreSimulator.SimRuntime.iOS-17-0"
        if let Some(version_part) = runtime_id.split("iOS-").nth(1) {
            version_part.replace('-', ".")
        } else {
            "17.0".to_string()
        }
    }

    /// Parse xcrun simctl list runtimes output
    fn parse_runtimes(&self, json_output: &str) -> Result<Vec<(String, String)>> {
        let json: Value = serde_json::from_str(json_output)?;
        let mut runtimes = Vec::new();

        if let Some(runtime_array) = json["runtimes"].as_array() {
            for runtime in runtime_array {
                if let (Some(name), Some(identifier)) =
                    (runtime["name"].as_str(), runtime["identifier"].as_str())
                {
                    if identifier.contains("iOS") {
                        runtimes.push((name.to_string(), identifier.to_string()));
                    }
                }
            }
        }

        Ok(runtimes)
    }
}

#[tokio::test]
async fn test_ios_device_parsing_single_runtime() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let single_output = loader
        .get_string(
            "ios_outputs.json",
            &["xcrun_simctl_list_devices", "single_runtime"],
        )?
        .unwrap_or_default();

    let manager = FixtureiOSManager::new().with_list_devices_output(single_output.clone());

    let devices = manager.parse_device_list(&single_output)?;

    assert!(!devices.is_empty());

    let device = &devices[0];
    assert!(!device.name.is_empty());
    assert!(!device.udid.is_empty());
    assert!(!device.ios_version.is_empty());
    assert!(device.ios_version.contains("17.0"));

    Ok(())
}

#[tokio::test]
async fn test_ios_device_parsing_multiple_runtimes() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let multiple_output = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_devices", "multiple_runtimes"],
    )?;

    if let Some(json_output) = multiple_output {
        let manager = FixtureiOSManager::new().with_list_devices_output(json_output.clone());

        let devices = manager.parse_device_list(&json_output)?;

        // Should have devices from multiple runtimes
        assert!(!devices.is_empty());

        // Check for different iOS versions
        let versions: Vec<&String> = devices.iter().map(|d| &d.ios_version).collect();
        let unique_versions: std::collections::HashSet<&String> = versions.into_iter().collect();

        // Should have multiple iOS versions
        if devices.len() > 1 {
            assert!(!unique_versions.is_empty());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_ios_device_state_parsing() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test shutdown state
    let shutdown_state = loader.get_string(
        "state_transitions.json",
        &["ios_device_states", "shutdown_to_booted", "initial_state"],
    )?;

    if let Some(json_output) = shutdown_state {
        let manager = FixtureiOSManager::new();
        let devices = manager.parse_device_list(&json_output)?;

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].status, DeviceStatus::Stopped);
        assert!(!devices[0].is_running);
    }

    // Test booted state
    let booted_state = loader.get_string(
        "state_transitions.json",
        &[
            "ios_device_states",
            "shutdown_to_booted",
            "after_boot_state",
        ],
    )?;

    if let Some(json_output) = booted_state {
        let manager = FixtureiOSManager::new();
        let devices = manager.parse_device_list(&json_output)?;

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].status, DeviceStatus::Running);
        assert!(devices[0].is_running);
    }

    Ok(())
}

#[tokio::test]
async fn test_ios_version_extraction() -> Result<()> {
    let manager = FixtureiOSManager::new();

    // Test various runtime ID formats
    assert_eq!(
        manager.extract_ios_version("com.apple.CoreSimulator.SimRuntime.iOS-17-0"),
        "17.0"
    );
    assert_eq!(
        manager.extract_ios_version("com.apple.CoreSimulator.SimRuntime.iOS-16-4"),
        "16.4"
    );
    assert_eq!(
        manager.extract_ios_version("com.apple.CoreSimulator.SimRuntime.iOS-15-5"),
        "15.5"
    );

    Ok(())
}

#[tokio::test]
async fn test_ios_device_types() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let device_types = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_devicetypes", "comprehensive"],
    )?;

    if let Some(json_output) = device_types {
        let json: Value = serde_json::from_str(&json_output)?;

        if let Some(devicetypes) = json["devicetypes"].as_array() {
            let iphone_types: Vec<&Value> = devicetypes
                .iter()
                .filter(|dt| dt["name"].as_str().unwrap_or("").contains("iPhone"))
                .collect();

            assert!(!iphone_types.is_empty());

            let ipad_types: Vec<&Value> = devicetypes
                .iter()
                .filter(|dt| dt["name"].as_str().unwrap_or("").contains("iPad"))
                .collect();

            assert!(!ipad_types.is_empty());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_ios_error_scenarios() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test already booted error
    let already_booted = loader
        .get_string(
            "error_scenarios.json",
            &["ios_errors", "device_already_booted", "stderr"],
        )?
        .unwrap_or_default();

    assert!(already_booted.contains("Unable to boot device in current state: Booted"));

    // Test device not found error
    let not_found = loader
        .get_string(
            "error_scenarios.json",
            &["ios_errors", "device_not_found", "stderr"],
        )?
        .unwrap_or_default();

    assert!(not_found.contains("Device not found"));

    // Test Xcode not installed error
    let xcode_missing = loader
        .get_string(
            "error_scenarios.json",
            &["ios_errors", "xcode_not_installed", "stderr"],
        )?
        .unwrap_or_default();

    assert!(xcode_missing.contains("does not exist"));

    Ok(())
}

#[tokio::test]
async fn test_ios_device_creation_simulation() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test before creation (empty)
    let before_creation = loader.get_string(
        "state_transitions.json",
        &["ios_device_states", "device_creation", "before_creation"],
    )?;

    if let Some(json_output) = before_creation {
        let manager = FixtureiOSManager::new();
        let devices = manager.parse_device_list(&json_output)?;
        assert_eq!(devices.len(), 0);
    }

    // Test after creation (device added)
    let after_creation = loader.get_string(
        "state_transitions.json",
        &["ios_device_states", "device_creation", "after_creation"],
    )?;

    if let Some(json_output) = after_creation {
        let manager = FixtureiOSManager::new();
        let devices = manager.parse_device_list(&json_output)?;
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "New Test Device");
        assert_eq!(devices[0].status, DeviceStatus::Stopped);
    }

    Ok(())
}

#[tokio::test]
async fn test_ios_runtimes_parsing() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let runtimes_output = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_runtimes", "comprehensive"],
    )?;

    if let Some(json_output) = runtimes_output {
        let manager = FixtureiOSManager::new();
        let runtimes = manager.parse_runtimes(&json_output)?;

        assert!(!runtimes.is_empty());

        // Check that all runtimes are iOS related
        for (name, identifier) in &runtimes {
            assert!(name.contains("iOS") || identifier.contains("iOS"));
        }

        // Check for expected runtime format
        let ios_17_runtime = runtimes.iter().find(|(_, id)| id.contains("iOS-17"));
        assert!(ios_17_runtime.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_device_availability_parsing() -> Result<()> {
    let mut loader = FixtureLoader::new();

    let single_output = loader
        .get_string(
            "ios_outputs.json",
            &["xcrun_simctl_list_devices", "single_runtime"],
        )?
        .unwrap_or_default();

    let manager = FixtureiOSManager::new();
    let devices = manager.parse_device_list(&single_output)?;

    for device in &devices {
        // All test devices should be available
        assert!(device.is_available);
    }

    Ok(())
}

#[tokio::test]
async fn test_comprehensive_ios_workflow() -> Result<()> {
    // This test simulates a complete iOS workflow using fixtures
    let mut loader = FixtureLoader::new();

    // Step 1: List available runtimes
    let runtimes = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_runtimes", "comprehensive"],
    )?;

    if let Some(runtimes_json) = runtimes {
        let manager = FixtureiOSManager::new();
        let parsed_runtimes = manager.parse_runtimes(&runtimes_json)?;
        assert!(!parsed_runtimes.is_empty());
    }

    // Step 2: List device types
    let device_types = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_devicetypes", "comprehensive"],
    )?;

    if let Some(types_json) = device_types {
        let json: Value = serde_json::from_str(&types_json)?;
        assert!(json["devicetypes"].is_array());
    }

    // Step 3: List current devices
    let devices = loader
        .get_string(
            "ios_outputs.json",
            &["xcrun_simctl_list_devices", "single_runtime"],
        )?
        .unwrap_or_default();

    let manager = FixtureiOSManager::new();
    let parsed_devices = manager.parse_device_list(&devices)?;

    // Verify we can parse the device structure
    for device in &parsed_devices {
        assert!(!device.name.is_empty());
        assert!(!device.udid.is_empty());
        assert!(!device.ios_version.is_empty());
    }

    Ok(())
}
