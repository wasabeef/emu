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
    
    // Background operations
    pub is_loading: bool,
    pub device_operation_status: Option<String>,
    
    // Caching
    pub cached_device_details: Option<DeviceDetails>,
    pub device_cache: Arc<RwLock<DeviceCache>>,
    
    // Logging and notifications
    pub device_logs: VecDeque<LogEntry>,
    pub notifications: VecDeque<Notification>,
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
#[async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    async fn list_devices(&self) -> Result<Vec<Device>>;
    async fn start_device(&self, id: &str) -> Result<()>;
    async fn stop_device(&self, id: &str) -> Result<()>;
    async fn create_device(&self, config: &DeviceConfig) -> Result<()>;
    async fn delete_device(&self, id: &str) -> Result<()>;
    async fn wipe_device(&self, id: &str) -> Result<()>;
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
#[async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    async fn list_devices(&self) -> Result<Vec<Device>>;
}

// Platform-specific implementations
impl DeviceManager for AndroidManager {
    async fn list_devices(&self) -> Result<Vec<AndroidDevice>> {
        // Android-specific implementation
    }
}

impl DeviceManager for IosManager {
    async fn list_devices(&self) -> Result<Vec<IosDevice>> {
        // iOS-specific implementation
    }
}
```

**Benefits**:
- Code reuse across platforms
- Easy testing with mock implementations
- Clear separation of platform-specific logic

### 2. Async State Management

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
4. **Cache Utilization**: Use cached data when available

### Runtime Optimization

1. **Debounced Updates**: Delay expensive operations during rapid input
2. **Smart Caching**: Cache expensive API calls and command outputs
3. **Selective Rendering**: Only update changed UI components
4. **Task Management**: Proper cleanup and cancellation of background tasks

### Memory Management

1. **Log Rotation**: Automatic cleanup of old log entries
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

#### Unit Tests (`src/`)
- **Location**: Alongside source code in `#[cfg(test)]` modules
- **Purpose**: Test individual functions and methods
- **Focus**: Logic validation, edge cases, error conditions

#### Integration Tests (`tests/`)
- **Device Lifecycle**: Complete device management workflows
- **Performance Tests**: Startup time and responsiveness validation
- **UI Tests**: Navigation, focus management, state coordination
- **Error Handling**: Error conditions and recovery scenarios

#### Performance Tests
- **Startup Benchmarks**: Application initialization time
- **Operation Benchmarks**: Device operation responsiveness
- **Memory Tests**: Memory usage and leak detection
- **Load Tests**: Behavior under high device counts

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

This architecture provides a solid foundation for building a responsive, maintainable, and cross-platform terminal application while ensuring reliability through comprehensive testing.