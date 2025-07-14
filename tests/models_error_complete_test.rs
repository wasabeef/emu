//! Comprehensive tests for models::error module
//!
//! These tests ensure complete coverage of DeviceError enum, error constructors,
//! format_user_error function, and all error handling patterns.

use emu::models::error::{format_user_error, DeviceError, DeviceResult};
use std::io::{Error as IoError, ErrorKind};

#[test]
fn test_device_error_not_found() {
    let error = DeviceError::not_found("test_device");

    assert_eq!(error.to_string(), "Device not found: test_device");
    assert_eq!(error.error_title(), "Device Not Found");
    assert_eq!(
        error.user_friendly_message(),
        "Device 'test_device' not found"
    );
}

#[test]
fn test_device_error_already_running() {
    let error = DeviceError::already_running("running_device");

    assert_eq!(
        error.to_string(),
        "Device running_device is already running"
    );
    assert_eq!(error.error_title(), "Device Running");
    assert_eq!(
        error.user_friendly_message(),
        "Device 'running_device' is already running"
    );
}

#[test]
fn test_device_error_not_running() {
    let error = DeviceError::not_running("stopped_device");

    assert_eq!(error.to_string(), "Device stopped_device is not running");
    assert_eq!(error.error_title(), "Device Stopped");
    assert_eq!(
        error.user_friendly_message(),
        "Device 'stopped_device' is not running"
    );
}

#[test]
fn test_device_error_start_failed_basic() {
    let error = DeviceError::start_failed("test_device", "Unknown reason");

    assert_eq!(
        error.to_string(),
        "Failed to start device test_device: Unknown reason"
    );
    assert_eq!(error.error_title(), "Start Error");
    assert_eq!(
        error.user_friendly_message(),
        "Failed to start device 'test_device'"
    );
}

#[test]
fn test_device_error_start_failed_licenses() {
    let error = DeviceError::start_failed("test_device", "licenses not accepted");

    assert_eq!(error.error_title(), "Start Error");
    assert_eq!(
        error.user_friendly_message(),
        "Android SDK licenses not accepted. Run 'sdkmanager --licenses'"
    );
}

#[test]
fn test_device_error_start_failed_system_image() {
    let error = DeviceError::start_failed("test_device", "system image not installed");

    assert_eq!(error.error_title(), "Start Error");
    assert_eq!(
        error.user_friendly_message(),
        "Required system image not installed"
    );
}

#[test]
fn test_device_error_stop_failed() {
    let error = DeviceError::stop_failed("test_device", "Timeout occurred");

    assert_eq!(
        error.to_string(),
        "Failed to stop device test_device: Timeout occurred"
    );
    assert_eq!(error.error_title(), "Stop Error");
    assert_eq!(
        error.user_friendly_message(),
        "Failed to stop device 'test_device'"
    );
}

#[test]
fn test_device_error_create_failed_basic() {
    let error = DeviceError::CreateFailed {
        name: "test_device".to_string(),
        reason: "Unknown error".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "Failed to create device test_device: Unknown error"
    );
    assert_eq!(error.error_title(), "Creation Error");
    assert_eq!(
        error.user_friendly_message(),
        "Failed to create device 'test_device'"
    );
}

#[test]
fn test_device_error_create_failed_licenses() {
    let error = DeviceError::CreateFailed {
        name: "test_device".to_string(),
        reason: "licenses must be accepted".to_string(),
    };

    assert_eq!(
        error.user_friendly_message(),
        "Android SDK licenses not accepted. Run 'sdkmanager --licenses'"
    );
}

#[test]
fn test_device_error_create_failed_system_image() {
    let error = DeviceError::CreateFailed {
        name: "test_device".to_string(),
        reason: "system image not installed".to_string(),
    };

    assert_eq!(
        error.user_friendly_message(),
        "Required system image not installed"
    );
}

#[test]
fn test_device_error_create_failed_already_exists() {
    let error = DeviceError::CreateFailed {
        name: "test_device".to_string(),
        reason: "Device already exists".to_string(),
    };

    assert_eq!(
        error.user_friendly_message(),
        "Device 'test_device' already exists"
    );
}

#[test]
fn test_device_error_create_failed_device_not_found() {
    let error = DeviceError::CreateFailed {
        name: "test_device".to_string(),
        reason: "device type not found".to_string(),
    };

    assert_eq!(
        error.user_friendly_message(),
        "Specified device type not found"
    );
}

#[test]
fn test_device_error_delete_failed() {
    let error = DeviceError::DeleteFailed {
        name: "test_device".to_string(),
        reason: "Permission denied".to_string(),
    };

    assert_eq!(
        error.to_string(),
        "Failed to delete device test_device: Permission denied"
    );
    assert_eq!(error.error_title(), "Deletion Error");
    assert_eq!(
        error.user_friendly_message(),
        "Failed to delete device 'test_device'"
    );
}

#[test]
fn test_device_error_command_failed() {
    let error = DeviceError::command_failed("adb devices");

    assert_eq!(error.to_string(), "Command execution failed: adb devices");
    assert_eq!(error.error_title(), "Command Error");
    assert_eq!(error.user_friendly_message(), "Command execution failed");
}

#[test]
fn test_device_error_platform_not_supported() {
    let error = DeviceError::PlatformNotSupported {
        platform: "Windows".to_string(),
    };

    assert_eq!(error.to_string(), "Platform not supported: Windows");
    assert_eq!(error.error_title(), "Platform Error");
    assert_eq!(
        error.user_friendly_message(),
        "Platform 'Windows' not supported"
    );
}

#[test]
fn test_device_error_sdk_not_found() {
    let error = DeviceError::SdkNotFound {
        sdk: "Android".to_string(),
    };

    assert_eq!(error.to_string(), "SDK not found: Android");
    assert_eq!(error.error_title(), "SDK Error");
    assert_eq!(
        error.user_friendly_message(),
        "Android SDK not found. Check environment variables"
    );
}

#[test]
fn test_device_error_invalid_config() {
    let error = DeviceError::InvalidConfig {
        message: "Invalid RAM size".to_string(),
    };

    assert_eq!(error.to_string(), "Invalid configuration: Invalid RAM size");
    assert_eq!(error.error_title(), "Config Error");
    assert_eq!(
        error.user_friendly_message(),
        "Configuration error: Invalid RAM size"
    );
}

#[test]
fn test_device_error_io_error() {
    let io_error = IoError::new(ErrorKind::PermissionDenied, "Access denied");
    let error = DeviceError::from(io_error);

    assert!(error.to_string().contains("IO error"));
    assert_eq!(error.error_title(), "IO Error");
    assert_eq!(error.user_friendly_message(), "File access error occurred");
}

#[test]
fn test_device_error_parse_error() {
    let json_error = serde_json::from_str::<i32>("invalid json").unwrap_err();
    let error = DeviceError::from(json_error);

    assert!(error.to_string().contains("Parse error"));
    assert_eq!(error.error_title(), "Parse Error");
    assert_eq!(error.user_friendly_message(), "Data parsing failed");
}

#[test]
fn test_device_error_regex_error() {
    // Create a regex error by using an invalid pattern that clippy won't detect
    let pattern = String::from("[") + "a-"; // Construct at runtime to avoid static analysis
    let regex_error = regex::Regex::new(&pattern).unwrap_err();
    let error = DeviceError::from(regex_error);

    assert!(error.to_string().contains("Regex error"));
    assert_eq!(error.error_title(), "Regex Error");
    assert_eq!(
        error.user_friendly_message(),
        "Pattern matching error occurred"
    );
}

#[test]
fn test_device_error_other() {
    let error = DeviceError::other("Custom error message");

    assert_eq!(error.to_string(), "Other error: Custom error message");
    assert_eq!(error.error_title(), "Error");
    assert_eq!(error.user_friendly_message(), "Custom error message");
}

#[test]
fn test_device_error_debug_formatting() {
    let error = DeviceError::not_found("debug_device");

    let debug_output = format!("{error:?}");
    assert!(debug_output.contains("NotFound"));
    assert!(debug_output.contains("debug_device"));
}

#[test]
fn test_format_user_error_licenses() {
    let error = anyhow::anyhow!("SDK licenses not accepted");
    let formatted = format_user_error(&error);

    assert_eq!(formatted, "Android SDK licenses not accepted. Run 'sdkmanager --licenses' in terminal to accept licenses.");
}

#[test]
fn test_format_user_error_accept() {
    let error = anyhow::anyhow!("You must accept the license agreements");
    let formatted = format_user_error(&error);

    assert_eq!(formatted, "Android SDK licenses not accepted. Run 'sdkmanager --licenses' in terminal to accept licenses.");
}

#[test]
fn test_format_user_error_system_image() {
    let error = anyhow::anyhow!("system image not installed");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Required system image not installed. Install system images using SDK Manager."
    );
}

#[test]
fn test_format_user_error_not_installed() {
    let error = anyhow::anyhow!("Package not installed properly");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Required system image not installed. Install system images using SDK Manager."
    );
}

#[test]
fn test_format_user_error_android_home() {
    let error = anyhow::anyhow!("ANDROID_HOME environment variable not set");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Android SDK not found. Set ANDROID_HOME environment variable."
    );
}

#[test]
fn test_format_user_error_android_sdk_root() {
    let error = anyhow::anyhow!("ANDROID_SDK_ROOT is missing");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Android SDK not found. Set ANDROID_HOME environment variable."
    );
}

#[test]
fn test_format_user_error_already_exists() {
    let error = anyhow::anyhow!("Device name already exists");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Device with same name already exists. Choose different name or delete existing device."
    );
}

#[test]
fn test_format_user_error_device_not_found() {
    let error = anyhow::anyhow!("Specified device type not found");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Specified device type not found. Select from available device types."
    );
}

#[test]
fn test_format_user_error_emulator_not_found() {
    let error = anyhow::anyhow!("Android emulator command not found");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Android emulator not found. Check if Android SDK is properly installed."
    );
}

#[test]
fn test_format_user_error_adb_not_found() {
    let error = anyhow::anyhow!("adb command not found");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "ADB command not found. Check if Android SDK path is properly set."
    );
}

#[test]
fn test_format_user_error_adb_command_not_found() {
    let error = anyhow::anyhow!("adb: command not found");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "ADB command not found. Check if Android SDK path is properly set."
    );
}

#[test]
fn test_format_user_error_xcrun_not_found() {
    let error = anyhow::anyhow!("xcrun not found");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Xcode command line tools not found. Run 'xcode-select --install' to install."
    );
}

#[test]
fn test_format_user_error_permission_denied() {
    let error = anyhow::anyhow!("Permission denied to access file");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Permission error occurred. Check file/directory access permissions."
    );
}

#[test]
fn test_format_user_error_denied() {
    let error = anyhow::anyhow!("Access denied to resource");
    let formatted = format_user_error(&error);

    assert_eq!(
        formatted,
        "Permission error occurred. Check file/directory access permissions."
    );
}

#[test]
fn test_format_user_error_timeout() {
    let error = anyhow::anyhow!("Operation timeout exceeded");
    let formatted = format_user_error(&error);

    assert_eq!(formatted, "Operation timed out. Please try again later.");
}

#[test]
fn test_format_user_error_long_message() {
    let long_message = "A".repeat(200);
    let error = anyhow::anyhow!("{long_message}");
    let formatted = format_user_error(&error);

    // Should be truncated to 150 characters + "..."
    assert!(formatted.len() <= 153);
    assert!(formatted.ends_with("..."));
    assert!(formatted.starts_with("A"));
}

#[test]
fn test_format_user_error_exactly_max_length() {
    let message = "A".repeat(150);
    let error = anyhow::anyhow!("{message}");
    let formatted = format_user_error(&error);

    // Should not be truncated
    assert_eq!(formatted.len(), 150);
    assert!(!formatted.ends_with("..."));
}

#[test]
fn test_format_user_error_just_over_max_length() {
    let message = "A".repeat(151);
    let error = anyhow::anyhow!("{message}");
    let formatted = format_user_error(&error);

    // Should be truncated
    assert!(formatted.len() < 151);
    assert!(formatted.ends_with("..."));
}

#[test]
fn test_format_user_error_unknown_error() {
    let error = anyhow::anyhow!("Some unknown error occurred");
    let formatted = format_user_error(&error);

    // Should return the original message for unknown patterns
    assert_eq!(formatted, "Some unknown error occurred");
}

#[test]
fn test_device_result_type_alias() {
    // Test that DeviceResult is a proper type alias
    let success: DeviceResult<String> = Ok("success".to_string());
    let failure: DeviceResult<String> = Err(DeviceError::not_found("test"));

    assert!(success.is_ok());
    assert!(failure.is_err());

    match failure {
        Err(DeviceError::NotFound { name }) => assert_eq!(name, "test"),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_device_error_constructor_string_conversion() {
    // Test that Into<String> is properly handled
    let error1 = DeviceError::not_found("test");
    let error2 = DeviceError::not_found(String::from("test"));

    assert_eq!(error1.to_string(), error2.to_string());

    let error3 = DeviceError::start_failed("device", "reason");
    let error4 = DeviceError::start_failed(String::from("device"), String::from("reason"));

    assert_eq!(error3.to_string(), error4.to_string());
}

#[test]
fn test_all_error_titles() {
    let errors = vec![
        DeviceError::not_found("test"),
        DeviceError::already_running("test"),
        DeviceError::not_running("test"),
        DeviceError::start_failed("test", "reason"),
        DeviceError::stop_failed("test", "reason"),
        DeviceError::CreateFailed {
            name: "test".to_string(),
            reason: "reason".to_string(),
        },
        DeviceError::DeleteFailed {
            name: "test".to_string(),
            reason: "reason".to_string(),
        },
        DeviceError::command_failed("test"),
        DeviceError::PlatformNotSupported {
            platform: "test".to_string(),
        },
        DeviceError::SdkNotFound {
            sdk: "test".to_string(),
        },
        DeviceError::InvalidConfig {
            message: "test".to_string(),
        },
        DeviceError::Io(IoError::other("test")),
        DeviceError::Parse(serde_json::from_str::<i32>("invalid").unwrap_err()),
        DeviceError::Regex(regex::Regex::new(&(String::from("[") + "a-")).unwrap_err()),
        DeviceError::other("test"),
    ];

    let expected_titles = vec![
        "Device Not Found",
        "Device Running",
        "Device Stopped",
        "Start Error",
        "Stop Error",
        "Creation Error",
        "Deletion Error",
        "Command Error",
        "Platform Error",
        "SDK Error",
        "Config Error",
        "IO Error",
        "Parse Error",
        "Regex Error",
        "Error",
    ];

    for (error, expected_title) in errors.iter().zip(expected_titles.iter()) {
        assert_eq!(error.error_title(), *expected_title);
    }
}

#[test]
fn test_format_user_error_pattern_combinations() {
    // Test combinations of error patterns
    let error = anyhow::anyhow!("System image not installed and licenses not accepted");
    let formatted = format_user_error(&error);

    // Should match first pattern (licenses)
    assert!(formatted.contains("licenses"));

    let error2 = anyhow::anyhow!("Unknown pattern for testing fallback");
    let formatted2 = format_user_error(&error2);

    // Should return original message for unknown patterns
    assert_eq!(formatted2, "Unknown pattern for testing fallback");
}

#[test]
fn test_device_error_comprehensive_matching() {
    // Test that user_friendly_message matches all error variants correctly
    let test_cases = vec![
        (DeviceError::not_found("test"), "Device 'test' not found"),
        (
            DeviceError::already_running("test"),
            "Device 'test' is already running",
        ),
        (
            DeviceError::not_running("test"),
            "Device 'test' is not running",
        ),
        (
            DeviceError::stop_failed("test", "reason"),
            "Failed to stop device 'test'",
        ),
        (
            DeviceError::DeleteFailed {
                name: "test".to_string(),
                reason: "reason".to_string(),
            },
            "Failed to delete device 'test'",
        ),
        (
            DeviceError::command_failed("test"),
            "Command execution failed",
        ),
        (
            DeviceError::PlatformNotSupported {
                platform: "Linux".to_string(),
            },
            "Platform 'Linux' not supported",
        ),
        (
            DeviceError::SdkNotFound {
                sdk: "iOS".to_string(),
            },
            "iOS SDK not found. Check environment variables",
        ),
        (
            DeviceError::InvalidConfig {
                message: "bad config".to_string(),
            },
            "Configuration error: bad config",
        ),
        (
            DeviceError::Io(IoError::other("test")),
            "File access error occurred",
        ),
        (
            DeviceError::Parse(serde_json::from_str::<i32>("invalid").unwrap_err()),
            "Data parsing failed",
        ),
        (
            DeviceError::Regex(regex::Regex::new(&(String::from("[") + "a-")).unwrap_err()),
            "Pattern matching error occurred",
        ),
        (DeviceError::other("custom"), "custom"),
    ];

    for (error, expected_message) in test_cases {
        assert_eq!(error.user_friendly_message(), expected_message);
    }
}
