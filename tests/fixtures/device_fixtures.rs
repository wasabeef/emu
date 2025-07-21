/// Device fixtures for testing
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFixture {
    pub name: String,
    pub device_type: String,
    pub api_level: String,
    pub expected_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandFixture {
    pub command: String,
    pub args: Vec<String>,
    pub output: String,
    pub exit_code: i32,
}

/// Load device creation scenarios
pub fn load_device_creation_fixtures() -> Vec<DeviceFixture> {
    vec![
        DeviceFixture {
            name: "pixel_7_api31".to_string(),
            device_type: "pixel_7".to_string(),
            api_level: "31".to_string(),
            expected_output: "Device created successfully".to_string(),
        },
        DeviceFixture {
            name: "tv_4k_api30".to_string(),
            device_type: "tv_4k".to_string(),
            api_level: "30".to_string(),
            expected_output: "Device created successfully".to_string(),
        },
    ]
}

/// Load command execution fixtures
pub fn load_command_fixtures() -> HashMap<String, CommandFixture> {
    let mut fixtures = HashMap::new();
    
    fixtures.insert("list_avd".to_string(), CommandFixture {
        command: "avdmanager".to_string(),
        args: vec!["list".to_string(), "avd".to_string()],
        output: include_str!("../fixtures/avd_list_output.txt").to_string(),
        exit_code: 0,
    });
    
    fixtures.insert("adb_devices".to_string(), CommandFixture {
        command: "adb".to_string(),
        args: vec!["devices".to_string()],
        output: "List of devices attached\nemulator-5554\tdevice\n".to_string(),
        exit_code: 0,
    });
    
    fixtures
}