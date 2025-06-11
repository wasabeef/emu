# API Documentation

This document provides comprehensive documentation for Emu's internal APIs, traits, and public interfaces.

## Table of Contents

- [Core Traits](#core-traits)
- [Device Management API](#device-management-api)
- [Application State API](#application-state-api)
- [Configuration API](#configuration-api)
- [Error Types](#error-types)
- [Platform-Specific APIs](#platform-specific-apis)

## Core Traits

### DeviceManager Trait

The `DeviceManager` trait provides a unified interface for device operations across platforms.

```rust
#[async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    /// List all available devices
    async fn list_devices(&self) -> Result<Vec<Device>>;
    
    /// Start a device by identifier
    async fn start_device(&self, id: &str) -> Result<()>;
    
    /// Stop a device by identifier
    async fn stop_device(&self, id: &str) -> Result<()>;
    
    /// Create a new device with the given configuration
    async fn create_device(&self, config: &DeviceConfig) -> Result<()>;
    
    /// Delete a device by identifier
    async fn delete_device(&self, id: &str) -> Result<()>;
    
    /// Wipe device data (cold boot)
    async fn wipe_device(&self, id: &str) -> Result<()>;
    
    /// Get detailed information about a device
    async fn get_device_details(&self, id: &str) -> Result<DeviceDetails>;
    
    /// List available device types for creation
    async fn list_device_types(&self) -> Result<Vec<(String, String)>>;
    
    /// List available system images or runtimes
    async fn list_available_targets(&self) -> Result<Vec<(String, String)>>;
}
```

#### Usage Example

```rust
use emu::managers::{AndroidManager, DeviceManager};
use emu::models::DeviceConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = AndroidManager::new()?;
    
    // List all devices
    let devices = manager.list_devices().await?;
    for device in devices {
        println!("{}: {}", device.name, device.status);
    }
    
    // Create a new device
    let config = DeviceConfig::new("MyDevice", "pixel_7", "31")
        .with_ram("4096")
        .with_storage("8192");
    
    manager.create_device(&config).await?;
    
    Ok(())
}
```

## Device Management API

### AndroidManager

Android-specific device management implementation.

#### Constructor

```rust
impl AndroidManager {
    /// Create a new AndroidManager
    /// 
    /// # Errors
    /// Returns error if Android SDK is not properly configured
    pub fn new() -> Result<Self>;
}
```

#### Platform-Specific Methods

```rust
impl AndroidManager {
    /// Get running AVD name to emulator serial mapping
    pub async fn get_running_avd_names(&self) -> Result<HashMap<String, String>>;
    
    /// List devices filtered by category
    pub async fn list_devices_by_category(&self, category: Option<&str>) -> Result<Vec<(String, String)>>;
    
    /// Get device category (phone, tablet, tv, etc.)
    pub fn get_device_category(&self, device_id: &str, display_name: &str) -> String;
    
    /// Stream Android logcat for a device
    pub async fn stream_logs(&self, device_serial: &str) -> Result<LogStream>;
}
```

#### Usage Example

```rust
use emu::managers::AndroidManager;

let manager = AndroidManager::new()?;

// Get running devices
let running = manager.get_running_avd_names().await?;
for (avd_name, serial) in running {
    println!("{} is running on {}", avd_name, serial);
}

// List devices by category
let phones = manager.list_devices_by_category(Some("phone")).await?;
```

### IosManager (macOS only)

iOS Simulator management implementation.

#### Constructor

```rust
impl IosManager {
    /// Create a new IosManager
    /// 
    /// # Errors
    /// Returns error if Xcode is not installed or simctl is unavailable
    pub fn new() -> Result<Self>;
}
```

#### Platform-Specific Methods

```rust
impl IosManager {
    /// List available iOS runtimes
    pub async fn list_runtimes(&self) -> Result<Vec<(String, String)>>;
    
    /// List device types with human-readable names
    pub async fn list_device_types_with_names(&self) -> Result<Vec<(String, String)>>;
    
    /// Get simulator status
    pub async fn get_simulator_status(&self, udid: &str) -> Result<SimulatorStatus>;
}
```

## Application State API

### AppState

Central application state container.

#### Core State

```rust
pub struct AppState {
    // Device data
    pub android_devices: Vec<AndroidDevice>,
    pub ios_devices: Vec<IosDevice>,
    
    // UI state
    pub active_panel: Panel,
    pub selected_android: usize,
    pub selected_ios: usize,
    pub focused_panel: FocusedPanel,
    
    // Modal states
    pub mode: Mode,
    pub create_device_form: CreateDeviceForm,
    pub confirm_delete_dialog: Option<ConfirmDeleteDialog>,
    pub confirm_wipe_dialog: Option<ConfirmWipeDialog>,
}
```

#### Navigation Methods

```rust
impl AppState {
    /// Move selection up in current panel (circular)
    pub fn move_up(&mut self);
    
    /// Move selection down in current panel (circular)
    pub fn move_down(&mut self);
    
    /// Switch to next panel
    pub fn next_panel(&mut self);
    
    /// Get currently selected device details
    pub fn get_selected_device_details(&self) -> Option<DeviceDetails>;
}
```

#### Device Management

```rust
impl AppState {
    /// Add a notification
    pub fn add_notification(&mut self, notification: Notification);
    
    /// Add success notification
    pub fn add_success_notification(&mut self, message: String);
    
    /// Add error notification
    pub fn add_error_notification(&mut self, message: String);
    
    /// Add warning notification
    pub fn add_warning_notification(&mut self, message: String);
    
    /// Add info notification
    pub fn add_info_notification(&mut self, message: String);
    
    /// Dismiss all notifications
    pub fn dismiss_all_notifications(&mut self);
    
    /// Dismiss expired notifications
    pub fn dismiss_expired_notifications(&mut self);
}
```

#### Log Management

```rust
impl AppState {
    /// Add a log entry
    pub fn add_log(&mut self, level: String, message: String);
    
    /// Clear all logs
    pub fn clear_logs(&mut self);
    
    /// Get filtered logs
    pub fn get_filtered_logs(&self) -> Vec<&LogEntry>;
    
    /// Toggle log filter
    pub fn toggle_log_filter(&mut self, level: Option<String>);
    
    /// Toggle fullscreen logs
    pub fn toggle_fullscreen_logs(&mut self);
    
    /// Scroll logs up
    pub fn scroll_logs_up(&mut self);
    
    /// Scroll logs down
    pub fn scroll_logs_down(&mut self);
}
```

#### Cache Management

```rust
impl AppState {
    /// Update cached device details
    pub fn update_cached_device_details(&mut self, details: DeviceDetails);
    
    /// Clear cached device details
    pub fn clear_cached_device_details(&mut self);
    
    /// Smart clear cached details (only if different platform)
    pub fn smart_clear_cached_device_details(&mut self, new_panel: Panel);
}
```

#### Operation Status

```rust
impl AppState {
    /// Set device operation status
    pub fn set_device_operation_status(&mut self, status: String);
    
    /// Clear device operation status
    pub fn clear_device_operation_status(&mut self);
    
    /// Get current device operation status
    pub fn get_device_operation_status(&self) -> Option<&String>;
    
    /// Set pending device start
    pub fn set_pending_device_start(&mut self, device_name: String);
    
    /// Clear pending device start
    pub fn clear_pending_device_start(&mut self);
    
    /// Get pending device start
    pub fn get_pending_device_start(&self) -> Option<&String>;
}
```

## Configuration API

### Config

Application configuration structure (simplified).

```rust
pub struct Config {
    // Currently minimal configuration
    // The project focuses on simplicity
}
```

#### Methods

```rust
impl Config {
    /// Get default configuration
    pub fn default() -> Self;
    
    /// Get theme
    pub fn theme(&self) -> Theme;
}
```

### DeviceConfig

Device creation configuration.

```rust
pub struct DeviceConfig {
    pub name: String,
    pub device_type: String,
    pub version: String,
    pub ram_size: Option<String>,
    pub storage_size: Option<String>,
    pub additional_options: HashMap<String, String>,
}
```

#### Builder Methods

```rust
impl DeviceConfig {
    /// Create new device configuration
    pub fn new(name: String, device_type: String, version: String) -> Self;
    
    /// Set RAM size
    pub fn with_ram(mut self, ram: String) -> Self;
    
    /// Set storage size
    pub fn with_storage(mut self, storage: String) -> Self;
    
    /// Add additional option
    pub fn with_option(mut self, key: String, value: String) -> Self;
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()>;
}
```

## Error Types

### Primary Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {name}")]
    NotFound { name: String },
    
    #[error("Device already exists: {name}")]
    AlreadyExists { name: String },
    
    #[error("Invalid device configuration: {reason}")]
    InvalidConfig { reason: String },
    
    #[error("Platform not supported: {platform}")]
    UnsupportedPlatform { platform: String },
    
    #[error("Command execution failed: {command}")]
    CommandFailed { command: String },
    
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,
    
    #[error("Invalid configuration format: {reason}")]
    InvalidFormat { reason: String },
    
    #[error("Missing required field: {field}")]
    MissingField { field: String },
}
```

### Error Utilities

```rust
/// Format error for user display
pub fn format_user_error(error: &anyhow::Error) -> String;

/// Check if error is retriable
pub fn is_retriable_error(error: &anyhow::Error) -> bool;

/// Get error category for logging
pub fn error_category(error: &anyhow::Error) -> ErrorCategory;

#[derive(Debug, Clone)]
pub enum ErrorCategory {
    Network,
    FileSystem,
    Configuration,
    Platform,
    User,
}
```

## Platform-Specific APIs

### Android-Specific Types

```rust
#[derive(Debug, Clone)]
pub struct AndroidDevice {
    pub name: String,
    pub device_type: String,
    pub api_level: u32,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub ram_size: String,
    pub storage_size: String,
}

#[derive(Debug, Clone)]
pub enum DeviceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error(String),
}
```

### iOS-Specific Types

```rust
#[derive(Debug, Clone)]
pub struct IosDevice {
    pub name: String,
    pub udid: String,
    pub device_type: String,
    pub ios_version: String,
    pub runtime_version: String,
    pub status: DeviceStatus,
    pub is_running: bool,
    pub is_available: bool,
}

#[derive(Debug, Clone)]
pub enum SimulatorStatus {
    Shutdown,
    Booted,
    ShuttingDown,
    Booting,
}
```

### Device Details

```rust
#[derive(Debug, Clone)]
pub struct DeviceDetails {
    pub name: String,
    pub status: String,
    pub platform: Panel,
    pub device_type: String,
    pub api_level_or_version: String,
    pub ram_size: Option<String>,
    pub storage_size: Option<String>,
    pub resolution: Option<String>,
    pub dpi: Option<String>,
    pub device_path: Option<String>,
    pub system_image: Option<String>,
    pub identifier: String,
}
```

## Usage Examples

### Complete Device Management Workflow

```rust
use emu::managers::{AndroidManager, DeviceManager};
use emu::models::{DeviceConfig, DeviceStatus};
use emu::app::{App, AppState};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize managers
    let android_manager = AndroidManager::new()?;
    
    // Create device
    let config = DeviceConfig::new("TestDevice", "pixel_7", "31")
        .with_ram("4096")
        .with_storage("8192");
    
    android_manager.create_device(&config).await?;
    
    // List devices
    let devices = android_manager.list_devices().await?;
    let device = devices.iter()
        .find(|d| d.name == "TestDevice")
        .expect("Device should exist");
    
    // Start device
    android_manager.start_device(&device.name).await?;
    
    // Wait for device to start
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Get device details
    let details = android_manager.get_device_details(&device.name).await?;
    println!("Device details: {:?}", details);
    
    // Stop device
    android_manager.stop_device(&device.name).await?;
    
    // Delete device
    android_manager.delete_device(&device.name).await?;
    
    Ok(())
}
```

### Application State Management

```rust
use emu::app::AppState;
use emu::models::{AndroidDevice, DeviceStatus, Panel};

fn manage_application_state() {
    let mut state = AppState::new();
    
    // Add devices
    state.android_devices.push(AndroidDevice {
        name: "Pixel_7_API_31".to_string(),
        device_type: "phone".to_string(),
        api_level: 31,
        status: DeviceStatus::Stopped,
        is_running: false,
        ram_size: "2048".to_string(),
        storage_size: "8192".to_string(),
    });
    
    // Navigate
    state.active_panel = Panel::Android;
    state.move_down(); // Select device
    
    // Get selected device
    if let Some(details) = state.get_selected_device_details() {
        println!("Selected: {}", details.name);
    }
    
    // Add notification
    state.add_success_notification("Device created successfully".to_string());
    
    // Manage logs
    state.add_log("INFO".to_string(), "Device started".to_string());
    state.toggle_log_filter(Some("INFO".to_string()));
}
```

This API documentation provides comprehensive coverage of Emu's internal interfaces and usage patterns. For more examples and detailed usage, refer to the test files in the `tests/` directory.