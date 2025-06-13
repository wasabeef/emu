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
- Debounced UI updates (50-100ms delays) for smooth navigation

## Current Features

### Device Management
- **Complete Lifecycle**: Create, start, stop, delete, and wipe devices
- **Real-time Status**: Live device state monitoring with automatic updates
- **Cross-platform**: Android (all platforms), iOS (macOS only)

### Android Features
- Full AVD management with all device categories (Phone, Tablet, TV, Wear OS, Automotive)
- Custom configuration: RAM (512MB-8GB), Storage (1GB-64GB)
- API level installation with progress tracking (5% increments)
- System image selection with priorities (Google Play > Google APIs > AOSP)
- Background API installation with main screen progress display
- Real-time logcat streaming with color-coded log levels and filtering

### iOS Features (macOS only)
- iOS simulator management via `xcrun simctl`
- Device type selection (iPhone, iPad models)
- Runtime version selection
- Basic device operations with status monitoring
- Intelligent log streaming with multiple fallback methods

### User Interface
- **Three-panel layout**: Android devices (30%) | iOS devices (30%) | Device details (40%)
- **Device details panel**: Shows device specifications, status, RAM/Storage in MB, full paths
- **Modal workflows**: Guided device creation with field validation
- **Keyboard navigation**: Vim-like keybindings (jk for up/down, q to quit) with circular navigation and page scrolling
- **Real-time feedback**: Status notifications, operation progress, log streaming

### Performance Optimizations
- **Fast startup**: UI renders immediately (~50ms), device data loads in background (~104ms average total)
- **High-performance input handling**: 
  - Designed for 60+ FPS operation as baseline
  - 125 FPS consistent rendering (8ms frame time)
  - Advanced event batching: up to 50 events per frame
  - 4ms event polling for sub-8ms input latency
  - Intelligent navigation debouncing with accumulation
- **Optimized event processing**:
  - Event queue management prevents input lag during rapid navigation
  - Background task debouncing (50ms delay) prevents UI stuttering
  - Batched navigation processing for smooth scrolling
- **Smart caching**: Device metadata and details cached with context-aware invalidation
- **Optimized log streaming**: 
  - Small buffer sizes (256 bytes) for low latency
  - Device-specific stream validation to prevent cross-device logs
  - Immediate task cancellation on device switch
- **Memory management**: Automatic log rotation (1000 entries max), cache expiration, background task cleanup

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

### Recent Improvements (Latest)
- ✅ **API Installation**: Background installation with 5% progress increments
- ✅ **System Image Display**: Shows image type (Google Play/APIs/AOSP) in selection
- ✅ **Log Streaming**: Fixed device-specific streaming with validation
- ✅ **Performance**: Reduced event polling to 10ms, instant key response
- ✅ **Device Details**: Immediate updates on navigation
- ✅ **UI Polish**: Status messages moved to bottom, made more prominent

### Completed Features
- ✅ Complete Android AVD lifecycle management
- ✅ iOS simulator basic operations (macOS)
- ✅ Three-panel UI layout with real-time device details
- ✅ Real-time log streaming with filtering and fullscreen mode
- ✅ Device creation wizard with validation
- ✅ Confirmation dialogs for destructive operations
- ✅ Background loading and performance optimizations
- ✅ Circular navigation with page scrolling
- ✅ Device details with MB units, system images, and full paths
- ✅ Comprehensive test suite (15+ files, 30+ tests)
- ✅ Operation status tracking and notifications

### Known Issues & Limitations
- **Android state detection**: Occasional inaccuracy in AVD name to emulator serial mapping (improved with multiple fallbacks)
- **iOS device details**: Limited device information display compared to Android
- **Platform differences**: Some features are Android-only (API installation) or iOS-only (simulator logs)

### Architecture Strengths
- Clean separation of concerns with trait-based abstractions
- Async-first design with proper task coordination
- Comprehensive error handling with user-friendly messages
- Performance-optimized with intelligent caching and background operations
- Extensive test coverage ensuring reliability
- Real-time updates without sacrificing responsiveness

## Key Files & Functions

### Application Core
- `src/app/mod.rs` - Main application logic, event loop, background task coordination
- `src/app/state.rs` - Application state, device management, UI state coordination
- `src/app/events.rs` - Event type definitions
- `src/app/actions.rs` - User action handlers

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

## Code Conventions

### Error Handling
- Use `anyhow` for error propagation with context
- Use `thiserror` for custom error types
- Provide user-friendly error messages via `format_user_error`
- Never use `.unwrap()` or `.expect()` in user-facing code

### Async Patterns
- Use `impl Future + Send` for trait methods to avoid async trait limitations
- Background tasks with `tokio::spawn` and proper cancellation
- State access with `Arc<Mutex<>>` for thread safety
- Debounced operations for UI responsiveness

### State Management
- Centralized state in `AppState` with method-based access
- Background loading patterns for non-blocking operations
- Smart caching with invalidation strategies
- UI state coordination (focus, selection, modals)

### Performance Patterns
- Background device loading on startup
- Debounced UI updates (50-100ms delays)
- Smart cache invalidation based on context
- Memory-efficient log management with rotation

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

### Performance Considerations
- Keep startup time under 150ms
- Use background loading for heavy operations
- Implement debouncing for rapid user interactions
- Cache expensive operations appropriately
- Test with performance benchmarks

This codebase represents a well-architected, performant TUI application with comprehensive testing and clean abstractions. The async-first design and trait-based architecture provide excellent foundations for continued development.