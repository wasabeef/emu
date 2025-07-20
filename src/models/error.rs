//! Error types and error handling utilities.
//!
//! This module defines custom error types for device management operations
//! and provides utilities for converting technical errors into user-friendly
//! messages suitable for display in the TUI.

use crate::constants::{
    env_vars::{ANDROID_HOME, ANDROID_SDK_ROOT},
    messages::error_formatting::{ERROR_MESSAGE_TRUNCATED_LENGTH, MAX_ERROR_MESSAGE_LENGTH},
};
use thiserror::Error;

/// Comprehensive error type for device management operations.
///
/// This enum covers all possible error conditions that can occur during
/// device lifecycle operations, from SDK issues to device-specific failures.
/// Each variant includes relevant context for debugging and user display.
#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {name}")]
    NotFound { name: String },

    #[error("Device {name} is already running")]
    AlreadyRunning { name: String },

    #[error("Device {name} is not running")]
    NotRunning { name: String },

    #[error("Failed to start device {name}: {reason}")]
    StartFailed { name: String, reason: String },

    #[error("Failed to stop device {name}: {reason}")]
    StopFailed { name: String, reason: String },

    #[error("Failed to create device {name}: {reason}")]
    CreateFailed { name: String, reason: String },

    #[error("Failed to delete device {name}: {reason}")]
    DeleteFailed { name: String, reason: String },

    #[error("Command execution failed: {command}")]
    CommandFailed { command: String },

    #[error("Platform not supported: {platform}")]
    PlatformNotSupported { platform: String },

    #[error("SDK not found: {sdk}")]
    SdkNotFound { sdk: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Other error: {message}")]
    Other { message: String },
}

impl DeviceError {
    /// Creates a NotFound error for the specified device name.
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound { name: name.into() }
    }

    pub fn already_running(name: impl Into<String>) -> Self {
        Self::AlreadyRunning { name: name.into() }
    }

    pub fn not_running(name: impl Into<String>) -> Self {
        Self::NotRunning { name: name.into() }
    }

    pub fn start_failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::StartFailed {
            name: name.into(),
            reason: reason.into(),
        }
    }

    pub fn stop_failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::StopFailed {
            name: name.into(),
            reason: reason.into(),
        }
    }

    pub fn command_failed(command: impl Into<String>) -> Self {
        Self::CommandFailed {
            command: command.into(),
        }
    }

    pub fn other(message: impl Into<String>) -> Self {
        Self::Other {
            message: message.into(),
        }
    }

    /// Convert an anyhow error to a user-friendly message for TUI display
    pub fn user_friendly_message(&self) -> String {
        match self {
            Self::NotFound { name } => format!("Device '{name}' not found"),
            Self::AlreadyRunning { name } => format!("Device '{name}' is already running"),
            Self::NotRunning { name } => format!("Device '{name}' is not running"),
            Self::StartFailed { name, reason } => {
                if reason.contains("licenses") {
                    "Android SDK licenses not accepted. Run 'sdkmanager --licenses'".to_string()
                } else if reason.contains("system image") || reason.contains("not installed") {
                    "Required system image not installed".to_string()
                } else {
                    format!("Failed to start device '{name}'")
                }
            }
            Self::StopFailed { name, .. } => format!("Failed to stop device '{name}'"),
            Self::CreateFailed { name, reason } => {
                if reason.contains("licenses") {
                    "Android SDK licenses not accepted. Run 'sdkmanager --licenses'".to_string()
                } else if reason.contains("system image") || reason.contains("not installed") {
                    "Required system image not installed".to_string()
                } else if reason.contains("already exists") {
                    format!("Device '{name}' already exists")
                } else if reason.contains("device") && reason.contains("not found") {
                    "Specified device type not found".to_string()
                } else {
                    format!("Failed to create device '{name}'")
                }
            }
            Self::DeleteFailed { name, .. } => format!("Failed to delete device '{name}'"),
            Self::CommandFailed { .. } => "Command execution failed".to_string(),
            Self::PlatformNotSupported { platform } => {
                format!("Platform '{platform}' not supported")
            }
            Self::SdkNotFound { sdk } => {
                format!("{sdk} SDK not found. Check environment variables")
            }
            Self::InvalidConfig { message } => format!("Configuration error: {message}"),
            Self::Io(_) => "File access error occurred".to_string(),
            Self::Parse(_) => "Data parsing failed".to_string(),
            Self::Regex(_) => "Pattern matching error occurred".to_string(),
            Self::Other { message } => message.clone(),
        }
    }

    /// Get a short error title for notifications
    pub fn error_title(&self) -> String {
        match self {
            Self::NotFound { .. } => "Device Not Found".to_string(),
            Self::AlreadyRunning { .. } => "Device Running".to_string(),
            Self::NotRunning { .. } => "Device Stopped".to_string(),
            Self::StartFailed { .. } => "Start Error".to_string(),
            Self::StopFailed { .. } => "Stop Error".to_string(),
            Self::CreateFailed { .. } => "Creation Error".to_string(),
            Self::DeleteFailed { .. } => "Deletion Error".to_string(),
            Self::CommandFailed { .. } => "Command Error".to_string(),
            Self::PlatformNotSupported { .. } => "Platform Error".to_string(),
            Self::SdkNotFound { .. } => "SDK Error".to_string(),
            Self::InvalidConfig { .. } => "Config Error".to_string(),
            Self::Io(_) => "IO Error".to_string(),
            Self::Parse(_) => "Parse Error".to_string(),
            Self::Regex(_) => "Regex Error".to_string(),
            Self::Other { .. } => "Error".to_string(),
        }
    }
}

/// Converts an anyhow::Error to a user-friendly message for TUI display.
///
/// This function analyzes error messages for common patterns and provides
/// helpful suggestions to users. It handles SDK configuration issues,
/// missing tools, permission errors, and other common problems.
///
/// # Arguments
/// * `error` - The anyhow error to format
///
/// # Returns
/// A user-friendly error message with actionable suggestions when possible.
///
/// # Examples
/// - "licenses not accepted" → "Run 'sdkmanager --licenses' to accept"
/// - "ANDROID_HOME not found" → "Set ANDROID_HOME environment variable"
/// - Long technical errors → Truncated to 150 characters
pub fn format_user_error(error: &anyhow::Error) -> String {
    let error_str = error.to_string();

    // Check for common error patterns and provide user-friendly messages
    if error_str.contains("licenses") || error_str.contains("accept") {
        return "Android SDK licenses not accepted. Run 'sdkmanager --licenses' in terminal to accept licenses.".to_string();
    }

    if error_str.contains("system image") || error_str.contains("not installed") {
        return "Required system image not installed. Install system images using SDK Manager."
            .to_string();
    }

    if error_str.contains(ANDROID_HOME) || error_str.contains(ANDROID_SDK_ROOT) {
        return format!("Android SDK not found. Set {ANDROID_HOME} environment variable.");
    }

    if error_str.contains("already exists") {
        return "Device with same name already exists. Choose different name or delete existing device.".to_string();
    }

    if error_str.contains("device") && error_str.contains("not found") {
        return "Specified device type not found. Select from available device types.".to_string();
    }

    if error_str.contains("emulator") && error_str.contains("not found") {
        return "Android emulator not found. Check if Android SDK is properly installed."
            .to_string();
    }

    if error_str.contains("adb")
        && (error_str.contains("not found") || error_str.contains("command not found"))
    {
        return "ADB command not found. Check if Android SDK path is properly set.".to_string();
    }

    if error_str.contains("xcrun") && error_str.contains("not found") {
        return "Xcode command line tools not found. Run 'xcode-select --install' to install."
            .to_string();
    }

    if error_str.contains("permission") || error_str.contains("denied") {
        return "Permission error occurred. Check file/directory access permissions.".to_string();
    }

    if error_str.contains("timeout") {
        return "Operation timed out. Please try again later.".to_string();
    }

    // Truncate very long error messages for display
    if error_str.len() > MAX_ERROR_MESSAGE_LENGTH {
        format!("{}...", &error_str[..ERROR_MESSAGE_TRUNCATED_LENGTH])
    } else {
        error_str
    }
}

/// Convenience type alias for Results with DeviceError.
///
/// This type alias simplifies function signatures throughout the codebase
/// when returning results that may contain device-specific errors.
pub type DeviceResult<T> = Result<T, DeviceError>;

#[cfg(test)]
mod tests {
    use super::*;

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
        let anyhow_error = anyhow::anyhow!("licenses not accepted");
        assert_eq!(
            format_user_error(&anyhow_error),
            "Android SDK licenses not accepted. Run 'sdkmanager --licenses' in terminal to accept licenses."
        );

        let anyhow_error = anyhow::anyhow!("system image not found");
        assert_eq!(
            format_user_error(&anyhow_error),
            "Required system image not installed. Install system images using SDK Manager."
        );

        let anyhow_error = anyhow::anyhow!("ANDROID_HOME not set");
        assert_eq!(
            format_user_error(&anyhow_error),
            "Android SDK not found. Set ANDROID_HOME environment variable."
        );

        let anyhow_error = anyhow::anyhow!("avdmanager: command not found");
        assert_eq!(
            format_user_error(&anyhow_error),
            "avdmanager: command not found"
        );

        let anyhow_error = anyhow::anyhow!("permission denied");
        assert_eq!(
            format_user_error(&anyhow_error),
            "Permission error occurred. Check file/directory access permissions."
        );

        let anyhow_error = anyhow::anyhow!("operation timeout");
        assert_eq!(
            format_user_error(&anyhow_error),
            "Operation timed out. Please try again later."
        );

        let long_error = "a".repeat(200);
        let anyhow_error = anyhow::anyhow!(long_error);
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
}
