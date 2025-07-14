//! Integration tests for fixture-based testing
//!
//! This test demonstrates how to use the fixture system for testing
//! command output parsing and state transitions.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

mod fixtures;
use fixtures::FixtureLoader;

#[tokio::test]
async fn test_android_avd_parsing_with_fixtures() -> Result<()> {
    // Load Android fixtures
    let mut loader = FixtureLoader::new();

    // Test single device parsing
    if let Ok(Some(single_output)) = loader.get_string(
        "android_outputs.json",
        &["avdmanager_list_avd", "single_device"],
    ) {
        println!("Single device output length: {}", single_output.len());
        assert!(single_output.contains("Pixel_7_API_34"));
        assert!(single_output.contains("Based on: Android 14.0"));
    }

    // Test multiple devices parsing
    if let Ok(Some(multiple_output)) = loader.get_string(
        "android_outputs.json",
        &["avdmanager_list_avd", "multiple_devices"],
    ) {
        println!("Multiple devices output length: {}", multiple_output.len());
        assert!(multiple_output.contains("Pixel_7_API_34"));
        assert!(multiple_output.contains("Pixel_Tablet_API_33"));
        assert!(multiple_output.contains("Wear_OS_Round_API_30"));
    }

    Ok(())
}

#[tokio::test]
async fn test_ios_device_parsing_with_fixtures() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test iOS device JSON parsing
    if let Ok(Some(device_json)) = loader.get_string(
        "ios_outputs.json",
        &["xcrun_simctl_list_devices", "single_runtime"],
    ) {
        let parsed: Value = serde_json::from_str(&device_json)?;

        assert!(parsed["devices"].is_object());

        // Check for iOS 17.0 runtime
        if let Some(ios_17_devices) =
            parsed["devices"]["com.apple.CoreSimulator.SimRuntime.iOS-17-0"].as_array()
        {
            assert!(!ios_17_devices.is_empty());

            // Check first device
            let first_device = &ios_17_devices[0];
            assert!(first_device["name"].as_str().is_some());
            assert!(first_device["udid"].as_str().is_some());
            assert!(first_device["state"].as_str().is_some());
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_state_transitions_with_fixtures() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test Android state transitions
    if let Ok(Some(initial_state)) = loader.get_string(
        "state_transitions.json",
        &[
            "android_device_states",
            "stopped_to_running",
            "initial_adb_devices",
        ],
    ) {
        assert!(initial_state.contains("List of devices attached"));
        assert!(!initial_state.contains("emulator-"));
    }

    if let Ok(Some(after_start)) = loader.get_string(
        "state_transitions.json",
        &[
            "android_device_states",
            "stopped_to_running",
            "after_start_adb_devices",
        ],
    ) {
        assert!(after_start.contains("List of devices attached"));
        assert!(after_start.contains("emulator-5554"));
    }

    // Test iOS state transitions
    if let Ok(Some(shutdown_state)) = loader.get_string(
        "state_transitions.json",
        &["ios_device_states", "shutdown_to_booted", "initial_state"],
    ) {
        let parsed: Value = serde_json::from_str(&shutdown_state)?;

        if let Some(devices) =
            parsed["devices"]["com.apple.CoreSimulator.SimRuntime.iOS-17-0"].as_array()
        {
            if let Some(first_device) = devices.first() {
                assert_eq!(first_device["state"].as_str().unwrap(), "Shutdown");
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_error_scenarios_with_fixtures() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test Android license error
    if let Ok(Some(license_error)) = loader.get_string(
        "error_scenarios.json",
        &["android_errors", "license_not_accepted", "stderr"],
    ) {
        assert!(license_error.contains("have not been accepted"));
    }

    // Test iOS already booted error
    if let Ok(Some(booted_error)) = loader.get_string(
        "error_scenarios.json",
        &["ios_errors", "device_already_booted", "stderr"],
    ) {
        assert!(booted_error.contains("Unable to boot device in current state: Booted"));
    }

    Ok(())
}

#[tokio::test]
async fn test_environment_variations_with_fixtures() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test Android SDK environments
    if let Ok(Some(android_envs)) = loader.get(
        "environment_variations.json",
        &["android_environments", "sdk_locations"],
    ) {
        assert!(android_envs.is_object());
        assert!(android_envs.get("android_home").is_some());
        assert!(android_envs.get("android_sdk_root").is_some());
    }

    // Test iOS Xcode versions
    if let Ok(Some(xcode_versions)) = loader.get(
        "environment_variations.json",
        &["ios_environments", "xcode_versions"],
    ) {
        assert!(xcode_versions.is_object());
        assert!(xcode_versions.get("xcode_15").is_some());
        assert!(xcode_versions.get("xcode_14").is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_convenience_functions() -> Result<()> {
    // Test convenience functions
    let android_single = fixtures::fixtures::android_avd_list_single()?;
    assert!(android_single.contains("Pixel_7_API_34"));

    let android_multiple = fixtures::fixtures::android_avd_list_multiple()?;
    assert!(android_multiple.contains("Pixel_7_API_34"));
    assert!(android_multiple.contains("Pixel_Tablet_API_33"));

    let ios_single = fixtures::fixtures::ios_device_list_single()?;
    let parsed: Value = serde_json::from_str(&ios_single)?;
    assert!(parsed["devices"].is_object());

    Ok(())
}

#[tokio::test]
async fn test_fixture_loader_caching() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Load the same fixture twice - should use cache on second load
    let _fixture1 = loader.load("android_outputs.json")?;
    let _fixture2 = loader.load("android_outputs.json")?;

    // Cache should contain the fixture
    assert!(loader.cache.contains_key("android_outputs.json"));

    Ok(())
}

#[tokio::test]
async fn test_fixture_loader_error_handling() -> Result<()> {
    let mut loader = FixtureLoader::new();

    // Test loading non-existent file
    let result = loader.load("non_existent_file.json");
    assert!(result.is_err());

    // Test navigating to non-existent path
    if let Ok(fixture) = loader.load("android_outputs.json") {
        let fixture_copy = fixture.clone();
        let result = loader.navigate_json_path(&fixture_copy, &["non_existent", "path"]);
        assert!(result.is_none());
    }

    Ok(())
}

/// Example of how to create a mock scenario using fixtures
#[tokio::test]
async fn test_mock_scenario_with_fixtures() -> Result<()> {
    // This test demonstrates how fixtures can be used with MockDeviceManager
    // In a real implementation, this would integrate with the actual MockDeviceManager

    let mut scenario_commands = HashMap::new();

    // Load fixture data
    let avd_output = fixtures::fixtures::android_avd_list_multiple()?;
    let adb_output = fixtures::fixtures::adb_devices_running()?;

    // Build mock scenario
    scenario_commands.insert(("avdmanager", vec!["list", "avd"]), avd_output);
    scenario_commands.insert(("adb", vec!["devices"]), adb_output);

    // Verify the scenario data
    assert_eq!(scenario_commands.len(), 2);
    assert!(scenario_commands.contains_key(&("avdmanager", vec!["list", "avd"])));
    assert!(scenario_commands.contains_key(&("adb", vec!["devices"])));

    Ok(())
}

#[tokio::test]
async fn test_fixture_data_consistency() -> Result<()> {
    // Test that fixture data is consistent across files
    let mut loader = FixtureLoader::new();

    // Load device name from AVD list
    if let Ok(Some(avd_output)) = loader.get_string(
        "android_outputs.json",
        &["avdmanager_list_avd", "single_device"],
    ) {
        assert!(avd_output.contains("Pixel_7_API_34"));
    }

    // Load same device name from getprop output
    if let Ok(Some(getprop_output)) = loader.get_string(
        "android_outputs.json",
        &["adb_shell_getprop", "avd_name_pixel"],
    ) {
        assert_eq!(getprop_output, "Pixel_7_API_34");
    }

    // Load same device name from state transitions
    if let Ok(Some(transition_output)) = loader.get_string(
        "state_transitions.json",
        &[
            "android_device_states",
            "stopped_to_running",
            "getprop_avd_name",
        ],
    ) {
        assert_eq!(transition_output, "Pixel_7_API_34");
    }

    Ok(())
}
