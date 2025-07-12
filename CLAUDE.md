# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Emu is a lazygit-inspired Terminal User Interface (TUI) for managing Android emulators and iOS simulators. Built with Rust, it provides a unified, intuitive interface for mobile developers to manage virtual devices directly from the terminal. The application emphasizes performance, reliability, and user experience with comprehensive testing and async-first architecture.

## Current Architecture

### Layered Architecture
- `app/` - Application state management, event handling, and main application logic
- `managers/` - Platform-specific device management implementations (Android/iOS)
- `models/` - Core data structures, error types, and platform definitions
- `ui/` - Terminal UI rendering, themes, and custom widgets
- `utils/` - Shared utilities for command execution and logging

### Key Design Patterns

#### Trait-Based Abstraction
The `DeviceManager` trait (in `managers/common.rs`) provides a unified interface for device operations across platforms:
- `AndroidManager` - Manages Android AVDs using `avdmanager`, `emulator`, and `adb`
- `IosManager` - Manages iOS simulators using `xcrun simctl` (macOS only)

#### Async-First Architecture
- Uses Tokio runtime for all async operations
- Device operations return `impl Future + Send` to avoid async trait limitations
- Background task coordination with proper cancellation handling
- Real-time log streaming implemented as async tasks

#### State Management
- Centralized `AppState` with `Arc<Mutex<>>` for thread safety
- Background device loading for fast startup (~100ms)
- Smart caching system for device metadata and details
- Direct event processing for ultra-responsive input handling

## Current Features

### Device Management
- **Complete Lifecycle**: Create, start, stop, delete, and wipe devices
- **Real-time Status**: Live device state monitoring with automatic updates
- **Cross-platform**: Android (all platforms), iOS (macOS only)

### Android Features
- Full AVD management with all device categories (Phone, Tablet, TV, Wear OS, Automotive)
- Custom configuration: RAM (512MB-8GB), Storage (1GB-64GB)
- API level selection from available system images
- Real-time logcat streaming with color-coded log levels

### iOS Features (macOS only)
- iOS simulator management via `xcrun simctl`
- Device type selection (iPhone, iPad models)
- Runtime version selection
- Basic device operations with status monitoring
- Automatic Simulator.app lifecycle management (opens on start, quits when last device stops)

### User Interface
- **Three-panel layout**: Android devices (30%) | iOS devices (30%) | Device details (40%)
- **Device details panel**: Shows device specifications, status, RAM/Storage in MB, full paths
- **Modal workflows**: Guided device creation with field validation
- **Keyboard navigation**: Vim-like keybindings (jk for up/down, q to quit) with circular navigation and page scrolling
- **Real-time feedback**: Status notifications, operation progress, log streaming
- **Ultra-responsive input**: Direct event processing without debouncing ensures no key presses are ignored during rapid input or long holds

### Performance Optimizations
- **Fast startup**: UI renders immediately (~50ms), device data loads in background (~104ms average total)
- **Smart caching**: Device metadata and details cached with platform-aware invalidation
- **Ultra-responsive keyboard input**: Direct event processing with 8ms polling for 120fps responsiveness
- **Memory management**: Automatic log rotation (1000 entries max), cache expiration, background task cleanup
- **Optimized event loop**: Simplified architecture eliminates input lag and key press ignoring issues

### Advanced Performance Features (Always Active)
- **Fast Panel Switching**: 30.8% improvement in panel switching (250ms → 173ms)
  - Reduced update delays: log updates 50ms→25ms, device details 100ms→25ms
  - Parallel processing for simultaneous log stream and device detail updates
- **Smart Device Start**: Saves 200ms+ per device operation
  - Immediate UI updates without full device list refresh
  - Background status verification for accuracy
- **Incremental Refresh**: 10-40% faster device list updates
  - HashMap-based differential updates avoid unnecessary re-creation
- **Parallel Commands**: 42.2% improvement in device listing (986ms → 571ms)
  - Concurrent execution of `avdmanager` and status checking commands

## Development Commands

```bash
# Build the project
cargo build

# Run in development
cargo run

# Run with debug logging
cargo run -- --debug

# Run tests
cargo test

# Run specific test suite
cargo test --test comprehensive_integration_test

# Run with output
cargo test -- --nocapture

# Performance tests
cargo test startup_performance_test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Lint with CI-level strictness (treats warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Build optimized release binary
cargo build --release
```

## Testing Infrastructure

### Comprehensive Test Suite
The project has 15 test files with 31+ test functions covering:

#### Test Categories
- **Integration Tests**: Complete device lifecycle workflows (`comprehensive_integration_test.rs`)
- **Performance Tests**: Startup time and responsiveness validation (`startup_performance_test.rs`)
- **UI Tests**: Navigation, focus management, theme handling (`ui_focus_and_theme_test.rs`)
- **Device Operations**: Creation, status tracking, operations (`device_operations_status_test.rs`)
- **Navigation Tests**: Field navigation, circular navigation (`device_creation_navigation_test.rs`)

#### Performance Benchmarks
- **Startup Time**: < 150ms (typical: ~104ms)
- **Panel Switching**: < 100ms
- **Device Navigation**: < 50ms
- **Log Streaming**: Real-time with < 10ms latency

### Running Tests
```bash
# Run all tests (recommended - excludes doctests)
cargo test --bins --tests

# Run all tests including doctests (may have import issues in examples)
cargo test

# Run main test suites
cargo test --test comprehensive_integration_test
cargo test --test device_creation_navigation_test
cargo test --test device_operations_status_test
cargo test --test ui_focus_and_theme_test

# Performance validation
cargo test startup_performance_test -- --nocapture
```

## Current Implementation Status

### Completed Features
- ✅ Complete Android AVD lifecycle management
- ✅ iOS simulator basic operations (macOS)
- ✅ Three-panel UI layout with device details
- ✅ Real-time log streaming with filtering
- ✅ Device creation wizard with validation
- ✅ Confirmation dialogs for destructive operations
- ✅ Background loading and performance optimizations
- ✅ Circular navigation in device lists
- ✅ Device details with MB units and full paths
- ✅ Comprehensive test suite
- ✅ Operation status tracking and notifications
- ✅ API level management with real-time installation
- ✅ Device creation cache system (5-minute expiration)
- ✅ Modular constants architecture (`src/constants/`)
- ✅ Form validation framework (`src/utils/validation.rs`)
- ✅ Enhanced command execution utilities with retry and error ignoring
- ✅ iOS Simulator.app automatic lifecycle management
- ✅ Ultra-responsive keyboard input handling eliminating all input lag and key press ignoring
- ✅ **Simplified architecture**: Removed environment variable complexity, all optimizations always active

### Known Issues & Limitations
- **Android state detection**: Occasional inaccuracy in AVD name to emulator serial mapping (improved but not perfect)
- **iOS device details**: Limited device information display compared to Android
- **Device operations**: Some platform-specific edge cases in device start/stop operations

### Architecture Strengths
- Clean separation of concerns with trait-based abstractions
- Async-first design with proper task coordination
- Comprehensive error handling with user-friendly messages
- Performance-optimized with background loading and caching
- Extensive test coverage ensuring reliability
- **Simplified codebase**: No environment variable complexity, always-optimal performance

## Key Files & Functions

### Application Core
- `src/app/mod.rs` - Main application logic, ultra-responsive event loop, background task coordination
- `src/app/state.rs` - Application state, device management, UI state coordination
- `src/app/events.rs` - Event type definitions
- `src/app/actions.rs` - User action handlers
- `src/app/event_processing.rs` - Event processing utilities (legacy, now using direct processing)

### Device Management
- `src/managers/common.rs` - `DeviceManager` trait definition
- `src/managers/android.rs` - Android AVD management, logcat streaming, device details
- `src/managers/ios.rs` - iOS simulator management via `xcrun simctl`

### User Interface
- `src/ui/render.rs` - Main rendering logic, three-panel layout
- `src/ui/theme.rs` - Color themes, focus indicators
- `src/ui/widgets.rs` - Custom UI widgets

### Models & Types
- `src/models/device.rs` - `AndroidDevice`, `IosDevice`, `DeviceStatus` definitions
- `src/models/error.rs` - Error types and user-friendly formatting
- `src/models/platform.rs` - Platform enums and device configurations

### Constants & Utilities
- `src/constants/` - Modular constants system (NO HARDCODED VALUES)
  - `commands.rs` - CLI tool names and arguments
  - `defaults.rs` - Default values and configurations
  - `env_vars.rs` - Environment variable names (legacy)
  - `files.rs` - File paths and extensions
  - `limits.rs` - Size limits, validation ranges, array indices
  - `messages.rs` - User-facing strings
  - `patterns.rs` - Regular expressions
  - `performance.rs` - Performance tuning parameters (always optimal)
  - `priorities.rs` - Device sorting priority values
  - `progress.rs` - Progress tracking and phase increments
  - `ui_layout.rs` - UI dimensions, spacing, padding, animations
- `src/utils/validation.rs` - Form field validation framework
- `src/utils/command.rs` - Enhanced command execution with retry and error handling

## Code Conventions

### Constants Management
- **ALL hardcoded values must be defined as constants** in `src/constants/`
- Constants are organized by domain (ui_layout, limits, priorities, progress, etc.)
- Use descriptive constant names that clearly indicate purpose and unit
- Group related constants in the same module
- Add RustDoc comments for constant modules explaining their purpose
- Constants should have appropriate types (u8, u16, u32, usize)
- Re-export commonly used constants at module level for convenience
- Example: `pub const MAX_DEVICE_NAME_LENGTH: usize = 50;`

### Error Handling
- Use `anyhow` for error propagation with context
- Use `thiserror` for custom error types
- Provide user-friendly error messages via `format_user_error`
- Never use `.unwrap()` or `.expect()` in user-facing code

### String Formatting
- **ALWAYS use inline variable syntax in format! macros**: `format!("{variable}")` instead of `format!("{}", variable)`
- This applies to ALL format-like macros: `format!`, `println!`, `eprintln!`, `log::info!`, `log::warn!`, `log::error!`, etc.
- Examples:
  ```rust
  // ✅ Correct
  format!("Device {name} created successfully")
  println!("Found {count} devices")
  log::info!("Starting device {identifier}")
  
  // ❌ Incorrect
  format!("Device {} created successfully", name)
  println!("Found {} devices", count)
  log::info!("Starting device {}", identifier)
  ```
- This rule is enforced by `clippy::uninlined_format_args` which treats violations as errors in CI
- Apply this consistently across ALL files including main source, tests, examples, and binary targets

### Async Patterns
- Use `impl Future + Send` for trait methods to avoid async trait limitations
- Background tasks with `tokio::spawn` and proper cancellation
- State access with `Arc<Mutex<>>` for thread safety
- Direct event processing for ultra-responsive UI interactions

### State Management
- Centralized state in `AppState` with method-based access
- Background loading patterns for non-blocking operations
- Smart caching with invalidation strategies
- UI state coordination (focus, selection, modals)

### Performance Patterns
- Background device loading on startup
- Direct event processing for 120fps responsiveness
- Smart cache invalidation based on context
- Memory-efficient log management with rotation
- Simplified event loop architecture eliminating input lag
- **Always-optimal execution**: All performance features permanently enabled

## Development Workflow

### Adding New Features
1. Define data models in `models/` if needed
2. Implement device manager methods in `managers/`
3. Update application state in `app/state.rs`
4. Add UI rendering in `ui/render.rs`
5. Handle user actions in `app/mod.rs`
6. Add comprehensive tests

### Testing Requirements
- Write tests for new functionality
- Include performance tests for critical paths
- Test error conditions and edge cases
- Validate UI state management
- Ensure async operations work correctly
- **Test all new constants**: Validate ranges, ordering, and type consistency
- Use documentation tests for constants to verify runtime behavior
- Current test coverage: 146+ tests with 95%+ coverage

### Code Quality Requirements
- **ALL code must pass `cargo clippy --all-targets --all-features -- -D warnings`**
- Use inline variable syntax in ALL format! macros: `format!("{variable}")` not `format!("{}", variable)`
- Run `cargo fmt` before committing to maintain consistent formatting
- Ensure all tests pass with `cargo test --bins --tests`

### Performance Considerations
- Keep startup time under 150ms
- Use background loading for heavy operations
- Use direct event processing for ultra-responsive input (no debouncing needed)
- Cache expensive operations appropriately
- Test with performance benchmarks
- Maintain 120fps responsiveness for keyboard input
- **No configuration needed**: All optimizations are automatically applied

## Recent Architecture Improvements

### Simplification (v0.2.2+)
- **Removed environment variable complexity**: All performance optimizations are now permanently enabled
- **Eliminated conditional code paths**: Simplified codebase with single, optimized execution path
- **Enhanced maintainability**: Reduced technical debt and potential configuration issues
- **Improved user experience**: No setup required, optimal performance out-of-the-box

### Key Benefits of Simplification
- **Zero configuration**: Users get maximum performance without any setup
- **Consistent experience**: All users enjoy the same optimized performance
- **Reduced complexity**: Codebase is easier to understand and maintain
- **Lower risk**: Fewer conditional branches mean fewer potential bugs

This codebase represents a well-architected, performant TUI application with comprehensive testing and clean abstractions. The async-first design, trait-based architecture, ultra-responsive input handling, and simplified always-optimal execution provide excellent foundations for continued development.