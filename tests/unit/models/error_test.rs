//! Basic tests for models/error.rs
//!
//! Tests error type functionality, user-friendly message conversion, and error handling.

use anyhow::anyhow;
use emu::models::error::{format_user_error, DeviceError, DeviceResult};

#[test]
fn test_device_error_constructors() {
    let err = DeviceError::not_found("test_device");
    assert!(matches!(err, DeviceError::NotFound { name } if name == "test_device"));

    let err = DeviceError::already_running("device1");
    assert!(matches!(err, DeviceError::AlreadyRunning { name } if name == "device1"));

    let err = DeviceError::not_running("device2");
    assert!(matches!(err, DeviceError::NotRunning { name } if name == "device2"));

    let err = DeviceError::start_failed("device3", "reason");
    assert!(
        matches!(err, DeviceError::StartFailed { name, reason } if name == "device3" && reason == "reason")
    );

    let err = DeviceError::stop_failed("device4", "failed");
    assert!(
        matches!(err, DeviceError::StopFailed { name, reason } if name == "device4" && reason == "failed")
    );

    let err = DeviceError::command_failed("adb devices");
    assert!(matches!(err, DeviceError::CommandFailed { command } if command == "adb devices"));

    let err = DeviceError::other("custom error");
    assert!(matches!(err, DeviceError::Other { message } if message == "custom error"));
}

#[test]
fn test_user_friendly_messages() {
    assert_eq!(
        DeviceError::not_found("test").user_friendly_message(),
        "Device 'test' not found"
    );

    assert_eq!(
        DeviceError::already_running("test").user_friendly_message(),
        "Device 'test' is already running"
    );

    assert_eq!(
        DeviceError::not_running("test").user_friendly_message(),
        "Device 'test' is not running"
    );

    assert_eq!(
        DeviceError::start_failed("test", "unknown").user_friendly_message(),
        "Failed to start device 'test'"
    );

    assert_eq!(
        DeviceError::start_failed("test", "licenses not accepted").user_friendly_message(),
        "Android SDK licenses not accepted. Run 'sdkmanager --licenses'"
    );

    assert_eq!(
        DeviceError::start_failed("test", "system image not found").user_friendly_message(),
        "Required system image not installed"
    );

    assert_eq!(
        DeviceError::CreateFailed {
            name: "test".to_string(),
            reason: "already exists".to_string()
        }
        .user_friendly_message(),
        "Device 'test' already exists"
    );

    assert_eq!(
        DeviceError::PlatformNotSupported {
            platform: "windows".to_string()
        }
        .user_friendly_message(),
        "Platform 'windows' not supported"
    );

    assert_eq!(
        DeviceError::SdkNotFound {
            sdk: "Android".to_string()
        }
        .user_friendly_message(),
        "Android SDK not found. Check environment variables"
    );
}

#[test]
fn test_error_title() {
    assert_eq!(
        DeviceError::not_found("test").error_title(),
        "Device Not Found"
    );
    assert_eq!(
        DeviceError::already_running("test").error_title(),
        "Device Running"
    );
    assert_eq!(
        DeviceError::not_running("test").error_title(),
        "Device Stopped"
    );
    assert_eq!(
        DeviceError::start_failed("test", "reason").error_title(),
        "Start Error"
    );
    assert_eq!(
        DeviceError::stop_failed("test", "reason").error_title(),
        "Stop Error"
    );
    assert_eq!(
        DeviceError::CreateFailed {
            name: "test".to_string(),
            reason: "reason".to_string()
        }
        .error_title(),
        "Creation Error"
    );
    assert_eq!(
        DeviceError::DeleteFailed {
            name: "test".to_string(),
            reason: "reason".to_string()
        }
        .error_title(),
        "Deletion Error"
    );
    assert_eq!(
        DeviceError::command_failed("cmd").error_title(),
        "Command Error"
    );
    assert_eq!(
        DeviceError::PlatformNotSupported {
            platform: "win".to_string()
        }
        .error_title(),
        "Platform Error"
    );
    assert_eq!(
        DeviceError::SdkNotFound {
            sdk: "Android".to_string()
        }
        .error_title(),
        "SDK Error"
    );
    assert_eq!(
        DeviceError::InvalidConfig {
            message: "msg".to_string()
        }
        .error_title(),
        "Config Error"
    );
    assert_eq!(DeviceError::other("msg").error_title(), "Error");
}

#[test]
fn test_format_user_error() {
    let anyhow_error = anyhow!("licenses not accepted");
    assert_eq!(
        format_user_error(&anyhow_error),
        "Android SDK licenses not accepted. Run 'sdkmanager --licenses' in terminal to accept licenses."
    );

    let anyhow_error = anyhow!("system image not found");
    assert_eq!(
        format_user_error(&anyhow_error),
        "Required system image not installed. Install system images using SDK Manager."
    );

    let anyhow_error = anyhow!("ANDROID_HOME not set");
    assert_eq!(
        format_user_error(&anyhow_error),
        "Android SDK not found. Set ANDROID_HOME environment variable."
    );

    let anyhow_error = anyhow!("avdmanager: command not found");
    assert_eq!(
        format_user_error(&anyhow_error),
        "avdmanager: command not found"
    );

    let anyhow_error = anyhow!("permission denied");
    assert_eq!(
        format_user_error(&anyhow_error),
        "Permission error occurred. Check file/directory access permissions."
    );

    let anyhow_error = anyhow!("operation timeout");
    assert_eq!(
        format_user_error(&anyhow_error),
        "Operation timed out. Please try again later."
    );

    let long_error = "a".repeat(200);
    let anyhow_error = anyhow!(long_error);
    let result = format_user_error(&anyhow_error);
    assert!(result.ends_with("..."));
    assert_eq!(result.len(), 150); // Truncated to exactly 150 characters
}

#[test]
fn test_error_display() {
    let err = DeviceError::not_found("test");
    assert_eq!(err.to_string(), "Device not found: test");

    let err = DeviceError::CommandFailed {
        command: "adb".to_string(),
    };
    assert_eq!(err.to_string(), "Command execution failed: adb");

    let err = DeviceError::CreateFailed {
        name: "test".to_string(),
        reason: "invalid".to_string(),
    };
    assert_eq!(err.to_string(), "Failed to create device test: invalid");
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let device_err = DeviceError::from(io_err);
    assert!(matches!(device_err, DeviceError::Io(_)));
}

#[test]
fn test_error_from_serde() {
    let json = "{ invalid json";
    let parse_result: Result<serde_json::Value, _> = serde_json::from_str(json);
    if let Err(e) = parse_result {
        let device_err = DeviceError::from(e);
        assert!(matches!(device_err, DeviceError::Parse(_)));
    }
}

#[test]
fn test_device_result_type() {
    let success: DeviceResult<String> = Ok("success".to_string());
    assert!(success.is_ok());

    let error: DeviceResult<String> = Err(DeviceError::not_found("test"));
    assert!(error.is_err());
}

#[test]
fn test_error_chaining() {
    let root_cause = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let device_err = DeviceError::from(root_cause);

    // Check error chain
    assert!(matches!(device_err, DeviceError::Io(_)));
    assert!(device_err.to_string().contains("file not found"));
}

#[test]
fn test_error_context_preservation() {
    let device_name = "test_device";
    let reason = "insufficient RAM";

    let err = DeviceError::start_failed(device_name, reason);

    // Check if error context is preserved
    match err {
        DeviceError::StartFailed { name, reason: r } => {
            assert_eq!(name, device_name);
            assert_eq!(r, reason);
        }
        _ => panic!("Expected StartFailed error"),
    }
}

#[test]
fn test_error_user_friendly_truncation() {
    let long_reason = "a".repeat(200);
    let err = DeviceError::other(long_reason);

    // Check if user-friendly message is not too long
    let message = err.user_friendly_message();
    assert!(message.len() <= 250); // Appropriate length
}

#[test]
fn test_error_title_consistency() {
    let errors = vec![
        DeviceError::not_found("test"),
        DeviceError::already_running("test"),
        DeviceError::not_running("test"),
        DeviceError::start_failed("test", "reason"),
        DeviceError::stop_failed("test", "reason"),
        DeviceError::command_failed("cmd"),
        DeviceError::other("msg"),
    ];

    for error in errors {
        let title = error.error_title();
        // Check that title is not empty
        assert!(!title.is_empty());
        // Check that title has appropriate length
        assert!(title.len() <= 20);
    }
}

#[test]
fn test_error_debug_format() {
    let err = DeviceError::not_found("test");
    let debug_str = format!("{err:?}");

    // Check if device name is included in debug format
    assert!(debug_str.contains("test"));
    assert!(debug_str.contains("NotFound"));
}

#[test]
fn test_specific_error_patterns() {
    // Android SDK license error
    let license_err = DeviceError::start_failed("test", "licenses not accepted");
    assert!(license_err
        .user_friendly_message()
        .contains("sdkmanager --licenses"));

    // System image error
    let image_err = DeviceError::start_failed("test", "system image not installed");
    assert!(image_err.user_friendly_message().contains("system image"));

    // Existing device error
    let exists_err = DeviceError::CreateFailed {
        name: "test".to_string(),
        reason: "already exists".to_string(),
    };
    assert!(exists_err
        .user_friendly_message()
        .contains("already exists"));
}

#[test]
fn test_error_message_consistency() {
    // Check consistency across different contexts for the same error type
    let err1 = DeviceError::not_found("device1");
    let err2 = DeviceError::not_found("device2");

    assert_eq!(err1.error_title(), err2.error_title());
    assert_ne!(err1.user_friendly_message(), err2.user_friendly_message());
}
