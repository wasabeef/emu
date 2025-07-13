# Architecture Documentation

This document provides a comprehensive overview of Emu's architecture, design patterns, and implementation details.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Core Components](#core-components)
- [Design Patterns](#design-patterns)
- [Data Flow](#data-flow)
- [Performance Architecture](#performance-architecture)
- [Error Handling Strategy](#error-handling-strategy)
- [Testing Architecture](#testing-architecture)

## Overview

Emu is built using a layered, async-first architecture that prioritizes performance, maintainability, and cross-platform compatibility. The application uses Rust's type system and ownership model to ensure memory safety and thread safety while providing a responsive terminal user interface.

### Architectural Principles

1. **Separation of Concerns**: Clear boundaries between UI, business logic, and platform-specific code
2. **Async-First**: Non-blocking operations with proper task coordination
3. **Trait-Based Abstraction**: Platform-agnostic interfaces with concrete implementations
4. **Performance Optimization**: Background loading, caching, and debounced updates
5. **Comprehensive Testing**: Extensive test coverage for reliability

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Terminal UI Layer                        │
│         Three-panel layout: 30% | 30% | 40%               │
├─────────────────────────────────────────────────────────────┤
│                 Application Core Layer                      │
├─────────────────────────────────────────────────────────────┤
│              Device Management Layer                        │
├─────────────────────────────────────────────────────────────┤
│                   System Layer                              │
└─────────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### Terminal UI Layer (`ui/`)

- **Rendering**: Terminal UI rendering using Ratatui
- **Themes**: Color schemes and visual styling
- **Widgets**: Custom UI components and layouts
- **Input Handling**: Keyboard event processing

#### Application Core Layer (`app/`)

- **State Management**: Centralized application state
- **Event Processing**: User action handling and coordination
- **Background Tasks**: Async task management and coordination
- **Business Logic**: Core application workflows

#### Device Management Layer (`managers/`)

- **Platform Abstraction**: Unified device operation interface
- **Android Management**: AVD lifecycle and logcat streaming
- **iOS Management**: Simulator control via simctl
- **Caching**: Device metadata and detail caching

#### System Layer (`utils/`, `models/`)

- **Command Execution**: Safe system command execution
- **Error Handling**: Error types and user-friendly formatting
- **Configuration**: Application settings and platform detection
- **Logging**: Structured logging and debug output

## Core Components

### Application State (`app/state.rs`)

The `AppState` struct serves as the central state container:

```rust
pub struct AppState {
    // Device data
    pub android_devices: Vec<AndroidDevice>,
    pub ios_devices: Vec<IosDevice>,

    // UI state
    pub active_panel: Panel,
    pub selected_android: usize,
    pub selected_ios: usize,
    pub mode: Mode,

    // API Level Management (New in v2.0)
    pub api_level_management: Option<ApiLevelManagementState>,

    // Background operations
    pub is_loading: bool,
    pub device_operation_status: Option<String>,

    // Caching
    pub cached_device_details: Option<DeviceDetails>,
    pub device_cache: Arc<RwLock<DeviceCache>>,

    // Logging and notifications
    pub device_logs: VecDeque<LogEntry>,
    pub notifications: VecDeque<Notification>,

    // Dialogs
    pub create_device_form: CreateDeviceForm,
    pub confirm_delete_dialog: Option<ConfirmDeleteDialog>,
    pub confirm_wipe_dialog: Option<ConfirmWipeDialog>,
}
```

Key responsibilities:

- **Device State**: Tracks all device information and status
- **UI Coordination**: Manages panel focus, selection, and modal states
- **Cache Management**: Handles device detail and metadata caching
- **Notification System**: Manages user feedback and status messages

### Device Manager Trait (`managers/common.rs`)

The `DeviceManager` trait provides a unified interface for device operations:

```rust
pub trait DeviceManager: Send + Sync + Clone {
    fn list_devices(&self) -> impl Future<Output = Result<Vec<Device>>> + Send;
    fn start_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
    fn stop_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
    fn create_device(&self, config: &DeviceConfig) -> impl Future<Output = Result<()>> + Send;
    fn delete_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
    fn wipe_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;
}
```

Platform implementations:

- **AndroidManager**: Manages AVDs using Android SDK tools
- **IosManager**: Controls iOS simulators via Xcode simctl

### Application Controller (`app/mod.rs`)

The main `App` struct coordinates all application components:

```rust
pub struct App {
    state: Arc<Mutex<AppState>>,
    android_manager: AndroidManager,
    ios_manager: Option<IosManager>,
    log_update_handle: Option<JoinHandle<()>>,
    detail_update_handle: Option<JoinHandle<()>>,
}
```

Responsibilities:

- **Event Loop**: Processes user input and system events
- **Task Coordination**: Manages background tasks and cancellation
- **UI Coordination**: Coordinates between state and rendering
- **Platform Integration**: Manages platform-specific operations

## Design Patterns

### 1. Trait-Based Abstraction

**Purpose**: Provide platform-agnostic interfaces while maintaining type safety.

```rust
// Common interface
pub trait DeviceManager: Send + Sync + Clone {
    fn list_devices(&self) -> impl Future<Output = Result<Vec<Device>>> + Send;
}

// Platform-specific implementations
impl DeviceManager for AndroidManager {
    fn list_devices(&self) -> impl Future<Output = Result<Vec<AndroidDevice>>> + Send {
        async {
            // Android-specific implementation
        }
    }
}

impl DeviceManager for IosManager {
    fn list_devices(&self) -> impl Future<Output = Result<Vec<IosDevice>>> + Send {
        async {
            // iOS-specific implementation
        }
    }
}
```

**Benefits**:

- Code reuse across platforms
- Easy testing with mock implementations
- Clear separation of platform-specific logic

### 2. API Level Management System (New)

**Purpose**: Dynamic system image management for Android devices.

```rust
pub struct ApiLevelManagementState {
    pub api_levels: Vec<ApiLevel>,
    pub selected_index: usize,
    pub is_loading: bool,
    pub install_progress: Option<InstallProgress>,
    pub scroll_offset: usize,
}
```

**Features**:

- Real-time installation progress tracking
- Scrollable UI with keyboard navigation
- Automatic cache invalidation on changes
- Background installation with progress callbacks

### 3. Async State Management

**Purpose**: Provide thread-safe state access with non-blocking operations.

```rust
pub struct App {
    state: Arc<Mutex<AppState>>,
}

impl App {
    async fn update_device_status(&self, device_id: &str, status: DeviceStatus) {
        let mut state = self.state.lock().await;
        if let Some(device) = state.find_device_mut(device_id) {
            device.status = status;
        }
    }
}
```

**Benefits**:

- Safe concurrent access to shared state
- Non-blocking UI with background updates
- Proper task coordination and cancellation

### 3. Background Task Coordination

**Purpose**: Perform expensive operations without blocking the UI.

```rust
impl App {
    fn start_background_device_loading(&mut self) {
        let state_clone = Arc::clone(&self.state);
        let android_manager = self.android_manager.clone();

        tokio::spawn(async move {
            // Load devices in background
            let devices = android_manager.list_devices().await?;

            let mut state = state_clone.lock().await;
            state.android_devices = devices;
            state.is_loading = false;
        });
    }
}
```

**Benefits**:

- Fast application startup
- Responsive UI during heavy operations
- Proper resource cleanup and cancellation

### 4. Smart Caching Strategy

**Purpose**: Minimize expensive operations while maintaining data freshness.

```rust
impl AppState {
    pub fn smart_clear_cached_device_details(&mut self, new_panel: Panel) {
        if let Some(ref cached) = self.cached_device_details {
            if cached.platform != new_panel {
                self.clear_cached_device_details();
            }
        }
    }
}
```

**Benefits**:

- Reduced API calls and command executions
- Faster UI responsiveness
- Intelligent cache invalidation

### 5. Debounced Updates

**Purpose**: Prevent UI stuttering during rapid user interactions.

```rust
impl App {
    async fn schedule_device_details_update(&mut self) {
        // Cancel previous update
        if let Some(handle) = self.detail_update_handle.take() {
            handle.abort();
        }

        // Schedule new update with delay
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            // Perform update
        });

        self.detail_update_handle = Some(handle);
    }
}
```

**Benefits**:

- Smooth user experience during rapid navigation
- Reduced system load from excessive updates
- Proper task cancellation to prevent resource leaks

## Data Flow

### User Input Flow

```
User Input → Event Processing → State Update → UI Rendering
     ↓              ↓              ↓              ↑
Keyboard → App::run() → AppState → ui::render()
```

### Device Operation Flow

```
User Action → Device Manager → System Command → State Update → UI Update
     ↓              ↓               ↓              ↓            ↑
  Press 'c' → create_device() → avdmanager → update_state() → render()
```

### Background Loading Flow

```
App Start → Background Task → API Call → State Update → UI Refresh
    ↓              ↓             ↓           ↓            ↑
App::new() → tokio::spawn() → list_devices() → state.devices = ... → render()
```

## Performance Architecture

### Startup Optimization

1. **Immediate UI Rendering**: Show interface within ~50ms
2. **Background Data Loading**: Load device lists asynchronously
3. **Progressive Enhancement**: Add features as data becomes available
4. **Cache Preloading**: Preload device types and API levels at startup
5. **Target Performance**: Startup time < 150ms (typical: ~104ms)

### Runtime Optimization

1. **Debounced Updates**: 50-100ms delays prevent UI stuttering during rapid navigation
2. **Smart Caching**: Cache expensive API calls and command outputs with platform-aware invalidation
3. **Selective Rendering**: Only update changed UI components
4. **Task Management**: Proper cleanup and cancellation of background tasks
5. **Performance Benchmarks**: Panel switching < 100ms, device navigation < 50ms, log streaming < 10ms latency

### Memory Management

1. **Log Rotation**: Automatic cleanup of old log entries (1000 entries max)
2. **Cache Expiration**: Remove stale cached data
3. **Resource Cleanup**: Proper disposal of system resources
4. **Background Task Limits**: Prevent unlimited task spawning

## Error Handling Strategy

### Error Types Hierarchy

```rust
// Custom application errors
#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {name}")]
    NotFound { name: String },

    #[error("Invalid device configuration: {reason}")]
    InvalidConfig { reason: String },

    #[error("Platform not supported: {platform}")]
    UnsupportedPlatform { platform: String },
}

// Error propagation with context
fn create_device(config: &DeviceConfig) -> Result<()> {
    validate_config(config)
        .with_context(|| format!("Invalid config for device '{}'", config.name))?;

    execute_creation(config)
        .with_context(|| "Failed to create device")?;

    Ok(())
}
```

### Error Recovery Strategies

1. **Graceful Degradation**: Continue operation with reduced functionality
2. **User Feedback**: Provide clear, actionable error messages
3. **Automatic Retry**: Retry transient failures with backoff
4. **Fallback Options**: Provide alternative approaches when primary fails

### User-Friendly Error Formatting

```rust
pub fn format_user_error(error: &anyhow::Error) -> String {
    match error.downcast_ref::<DeviceError>() {
        Some(DeviceError::NotFound { name }) => {
            format!("Device '{}' not found. Please check if it exists.", name)
        }
        Some(DeviceError::InvalidConfig { reason }) => {
            format!("Configuration error: {}. Please check your settings.", reason)
        }
        _ => format!("An error occurred: {}", error),
    }
}
```

## Testing Architecture

### Test Categories

#### Test Suite Overview

The project has 15 test files with 31+ test functions covering:

#### Unit Tests (`src/`)

- **Location**: Alongside source code in `#[cfg(test)]` modules
- **Purpose**: Test individual functions and methods
- **Focus**: Logic validation, edge cases, error conditions

#### Integration Tests (`tests/`)

- **Device Lifecycle**: Complete device management workflows (`comprehensive_integration_test.rs`)
- **Performance Tests**: Startup time and responsiveness validation (`startup_performance_test.rs`)
- **UI Tests**: Navigation, focus management, state coordination (`ui_focus_and_theme_test.rs`)
- **Device Operations**: Creation, status tracking, operations (`device_operations_status_test.rs`)
- **Navigation Tests**: Field navigation, circular navigation (`device_creation_navigation_test.rs`)
- **Error Handling**: Error conditions and recovery scenarios

#### Performance Benchmarks

- **Startup Time**: < 150ms (typical: ~104ms)
- **Panel Switching**: < 100ms
- **Device Navigation**: < 50ms
- **Log Streaming**: Real-time with < 10ms latency

### Testing Patterns

#### Async Test Pattern

```rust
#[tokio::test]
async fn test_device_creation() {
    let manager = AndroidManager::new().unwrap();
    let config = DeviceConfig::new("test_device", "pixel_7", "31");

    let result = manager.create_device(&config).await;
    assert!(result.is_ok());
}
```

#### State Test Pattern

```rust
#[test]
fn test_state_consistency() {
    let mut state = AppState::new();

    // Setup initial state
    state.add_device(mock_device());

    // Perform operation
    state.select_device(0);

    // Verify state consistency
    assert_eq!(state.selected_android, 0);
    assert!(state.get_selected_device().is_some());
}
```

#### Performance Test Pattern

```rust
#[tokio::test]
async fn test_startup_performance() {
    let start = Instant::now();
    let app = App::new(Config::default()).await?;
    let duration = start.elapsed();

    assert!(duration < Duration::from_millis(150));
    println!("Startup time: {:?}", duration); // Typical: ~104ms
}
```

### Test Infrastructure

#### Mock Framework

- **mockall**: Mock external dependencies and system commands
- **Test Doubles**: Controlled test environments
- **Isolation**: Independent test execution

#### Assertion Framework

- **assert_cmd**: Command-line interface testing
- **predicates**: Complex assertion conditions
- **Custom Assertions**: Domain-specific test helpers

## New Features in v2.0

### API Level Management

- **Dynamic System Image Discovery**: Real-time detection of available system images
- **Installation Progress Tracking**: Live progress updates during installation
- **Smart Cache Invalidation**: Automatic cache refresh on system image changes
- **Architecture Detection**: Automatic selection of optimal architecture (x86_64/arm64)

### Enhanced Caching System

- **Device Creation Cache**: Pre-loaded device types and API levels
- **Background Refresh**: Automatic cache updates without blocking UI
- **Context-Aware Invalidation**: Cache cleared on relevant operations

### Improved User Experience

- **Scrollable Dialogs**: Better handling of long lists
- **Loading Indicators**: Clear feedback during async operations
- **Keyboard Navigation**: Circular navigation in device lists
- **Real-time Status Updates**: Live device status monitoring

### iOS Simulator Integration

- **Automatic App Lifecycle**: Simulator.app opens automatically when starting devices
- **Smart Cleanup**: Simulator.app quits automatically when last device stops
- **Graceful Shutdown**: Uses AppleScript for clean app termination with fallback
- **Dock Management**: Prevents Simulator.app icon from lingering in Dock

## Constants Architecture

The application uses a modular constants system (`constants/`):

```
constants/
├── commands.rs     # CLI tool names and arguments
├── defaults.rs     # Default values and configurations
├── env_vars.rs     # Environment variable names
├── files.rs        # File paths and extensions
├── messages.rs     # User-facing strings and messages
├── patterns.rs     # Regular expressions for parsing
└── performance.rs  # Performance tuning parameters
```

This architecture provides a solid foundation for building a responsive, maintainable, and cross-platform terminal application while ensuring reliability through comprehensive testing.
