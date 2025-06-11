//! Error types

use thiserror::Error;

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
            Self::NotFound { name } => format!("Device '{}' not found", name),
            Self::AlreadyRunning { name } => format!("Device '{}' is already running", name),
            Self::NotRunning { name } => format!("Device '{}' is not running", name),
            Self::StartFailed { name, reason } => {
                if reason.contains("licenses") {
                    "Android SDK licenses not accepted. Run 'sdkmanager --licenses'".to_string()
                } else if reason.contains("system image") || reason.contains("not installed") {
                    "Required system image not installed".to_string()
                } else {
                    format!("Failed to start device '{}'", name)
                }
            }
            Self::StopFailed { name, .. } => format!("Failed to stop device '{}'", name),
            Self::CreateFailed { name, reason } => {
                if reason.contains("licenses") {
                    "Android SDK licenses not accepted. Run 'sdkmanager --licenses'".to_string()
                } else if reason.contains("system image") || reason.contains("not installed") {
                    "Required system image not installed".to_string()
                } else if reason.contains("already exists") {
                    format!("Device '{}' already exists", name)
                } else if reason.contains("device") && reason.contains("not found") {
                    "Specified device type not found".to_string()
                } else {
                    format!("Failed to create device '{}'", name)
                }
            }
            Self::DeleteFailed { name, .. } => format!("Failed to delete device '{}'", name),
            Self::CommandFailed { .. } => "Command execution failed".to_string(),
            Self::PlatformNotSupported { platform } => {
                format!("Platform '{}' not supported", platform)
            }
            Self::SdkNotFound { sdk } => {
                format!("{} SDK not found. Check environment variables", sdk)
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

/// Helper function to convert anyhow::Error to user-friendly message
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

    if error_str.contains("ANDROID_HOME") || error_str.contains("ANDROID_SDK_ROOT") {
        return "Android SDK not found. Set ANDROID_HOME environment variable.".to_string();
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
    if error_str.len() > 150 {
        format!("{}...", &error_str[..147])
    } else {
        error_str
    }
}

pub type DeviceResult<T> = Result<T, DeviceError>;
