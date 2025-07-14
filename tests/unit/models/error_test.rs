use emu::models::error::{EmuError, format_user_error};
use anyhow::anyhow;

#[test]
fn test_emu_error_formatting() {
    let errors = vec![
        (
            EmuError::DeviceNotFound("test-device".to_string()),
            "Device 'test-device' not found"
        ),
        (
            EmuError::OperationFailed("start".to_string(), "timeout".to_string()),
            "Failed to start: timeout"
        ),
        (
            EmuError::InvalidConfiguration("invalid config".to_string()),
            "Invalid configuration: invalid config"
        ),
        (
            EmuError::CommandExecutionFailed("emulator".to_string()),
            "Command execution failed: emulator"
        ),
        (
            EmuError::ParsingError("Invalid format".to_string()),
            "Parsing error: Invalid format"
        ),
    ];
    
    for (error, expected) in errors {
        let error_string = error.to_string();
        assert!(error_string.contains(expected), 
                "Error '{error_string}' should contain '{expected}'");
    }
}

#[test]
fn test_format_user_error() {
    let error = anyhow!("Command failed").context("Failed to start device");
    let formatted = format_user_error(&error);
    
    assert!(formatted.contains("Failed to start device"));
    assert!(formatted.contains("Command failed"));
}

#[test]
fn test_format_user_error_with_chain() {
    let root_error = anyhow!("No such file or directory");
    let mid_error = root_error.context("Failed to read config");
    let top_error = mid_error.context("Device initialization failed");
    
    let formatted = format_user_error(&top_error);
    
    // Verify that all error levels are included
    assert!(formatted.contains("Device initialization failed"));
    assert!(formatted.contains("Failed to read config"));
    assert!(formatted.contains("No such file or directory"));
}

#[test]
fn test_emu_error_debug() {
    let error = EmuError::DeviceNotFound("test-device".to_string());
    let debug_str = format!("{error:?}");
    assert!(debug_str.contains("DeviceNotFound"));
    assert!(debug_str.contains("test-device"));
}

#[test]
fn test_emu_error_clone() {
    let original = EmuError::OperationFailed("start".to_string(), "timeout".to_string());
    let cloned = original.clone();
    
    assert_eq!(original.to_string(), cloned.to_string());
}

#[test]
fn test_emu_error_equality() {
    let error1 = EmuError::DeviceNotFound("device1".to_string());
    let error2 = EmuError::DeviceNotFound("device1".to_string());
    let error3 = EmuError::DeviceNotFound("device2".to_string());
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_operation_failed_with_empty_message() {
    let error = EmuError::OperationFailed("delete".to_string(), "".to_string());
    let error_string = error.to_string();
    assert!(error_string.contains("Failed to delete"));
}

#[test]
fn test_invalid_configuration_with_special_characters() {
    let config_msg = "Invalid character: ';rm -rf /'".to_string();
    let error = EmuError::InvalidConfiguration(config_msg.clone());
    let error_string = error.to_string();
    assert!(error_string.contains(&config_msg));
}

#[test]
fn test_command_execution_failed_with_path() {
    let command = "/usr/bin/emulator".to_string();
    let error = EmuError::CommandExecutionFailed(command.clone());
    let error_string = error.to_string();
    assert!(error_string.contains(&command));
}

#[test]
fn test_parsing_error_with_json() {
    let json_error = "Expected closing bracket at line 5".to_string();
    let error = EmuError::ParsingError(json_error.clone());
    let error_string = error.to_string();
    assert!(error_string.contains(&json_error));
}

#[test]
fn test_format_user_error_with_simple_error() {
    let error = anyhow!("Simple error message");
    let formatted = format_user_error(&error);
    assert!(formatted.contains("Simple error message"));
}

#[test]
fn test_format_user_error_preserves_formatting() {
    let error = anyhow!("Error with\nmultiple\nlines");
    let formatted = format_user_error(&error);
    assert!(formatted.contains("Error with"));
    assert!(formatted.contains("multiple"));
    assert!(formatted.contains("lines"));
}

#[test]
fn test_emu_error_from_anyhow() {
    let anyhow_error = anyhow!("Test error");
    let emu_error = EmuError::CommandExecutionFailed(anyhow_error.to_string());
    assert!(emu_error.to_string().contains("Test error"));
}

#[test]
fn test_error_display_consistency() {
    let device_name = "pixel-8-api-34";
    let error = EmuError::DeviceNotFound(device_name.to_string());
    
    // Verify that Display and Debug contain consistent information
    let display_str = format!("{error}");
    let debug_str = format!("{error:?}");
    
    assert!(display_str.contains(device_name));
    assert!(debug_str.contains(device_name));
}