# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test suite with 15 test files and 31+ test functions
- Performance benchmarks and validation tests
- Device operation status tracking and notifications
- Device details panel with real-time information display
- Circular navigation in device lists (wrap-around)
- Ctrl+hjkl keyboard support matching arrow key behavior
- Background device loading for fast startup (~100ms)
- Smart caching system for device metadata
- Debounced UI updates for smooth navigation
- Three-panel layout (Android | iOS | Device Details)
- Dynamic Android version lookup capability for API levels
- Support for Android 16 (API 36)
- Initial device details loading on startup
- Hybrid approach for Android version mapping (hardcoded + dynamic SDK lookup)

### Changed
- **BREAKING**: Device creation navigation - up/down keys now only move between fields
- RAM and storage units displayed in MB instead of raw values
- Device paths displayed in full without truncation
- Improved startup performance from 1s+ to ~104ms average
- Enhanced panel switching responsiveness with 100ms delays
- Updated UI layout to 30% | 30% | 40% panel distribution
- Updated all dependencies to latest versions:
  - ratatui: 0.26 → 0.29
  - crossterm: 0.27 → 0.28
  - thiserror: 1.0 → 2.0
  - dirs: 5.0 → 6.0
  - which: 6.0 → 7.0
  - mockall: 0.12 → 0.13
- Updated deprecated ratatui API calls (frame.size() → frame.area())
- Changed repository from "emu-project/emu" to "wasabeef/emu"
- Changed author from "The Emu Authors" to "Daichi Furiya"

### Fixed
- Fixed 'w' key crash by implementing device wipe functionality
- Resolved Android device state detection inaccuracies
- Fixed device details not updating when device status changes
- Corrected log scrolling behavior with proper scroll state management
- Fixed memory leaks in background task management
- Fixed "Android Unknown" display for API 36
- Fixed initial device details not loading when first Android device is selected
- Improved Android version detection with fallback to SDK properties

### Performance
- **Startup time**: Reduced from 1000ms+ to ~104ms average
- **Panel switching**: Optimized to < 100ms response time
- **Device navigation**: Sub-50ms response for device selection
- **Memory usage**: Implemented automatic log rotation and cache expiration
- **Background loading**: Non-blocking device list initialization

### Testing
- Added comprehensive integration tests for complete workflows
- Performance validation tests with startup time requirements
- UI navigation and focus management tests
- Device operation status and cache management tests
- Error handling and recovery scenario tests

## [0.1.0] - 2024-01-XX

### Added
- Initial release of Emu TUI
- Android Virtual Device (AVD) management
  - Create, start, stop, delete, and wipe devices
  - Support for all device categories (Phone, Tablet, TV, Wear OS, Automotive)
  - Custom RAM and storage configuration
  - API level selection from available system images
- iOS Simulator management (macOS only)
  - Basic simulator control via `xcrun simctl`
  - Device type and runtime selection
  - Start and stop operations
- Terminal User Interface
  - Two-panel layout for Android and iOS devices
  - Real-time log streaming with color-coded levels
  - Keyboard navigation with vim-like keybindings
  - Modal dialogs for device creation and confirmation
- Real-time features
  - Live device status monitoring
  - Automatic device list refresh
  - Log filtering and search capabilities
- Cross-platform support
  - Windows, macOS, and Linux for Android development
  - macOS-specific iOS simulator support
- Configuration system
  - TOML-based configuration structure
  - Theme support (dark theme implemented)
  - Platform-specific defaults

### Technical Features
- Async-first architecture using Tokio
- Trait-based abstraction for platform-specific operations
- Thread-safe state management with Arc<Mutex<>>
- Error handling with anyhow and thiserror
- Comprehensive logging with structured output
- Command execution utilities with proper error handling

### Dependencies
- **UI Framework**: Ratatui 0.26 for terminal interface
- **Async Runtime**: Tokio 1.45 with full features
- **CLI Framework**: Clap 4.5 with derive macros
- **Error Handling**: anyhow 1.0, thiserror 1.0
- **Serialization**: serde 1.0 with JSON/TOML support
- **Testing**: assert_cmd, predicates, tempfile, mockall
- **Time**: chrono 0.4 for timestamps

### Known Limitations
- iOS device creation uses hardcoded runtime IDs
- Limited iOS device information compared to Android
- Configuration file loading not fully implemented
- Android emulator state detection occasionally inaccurate

---

## Release Notes

### Version 0.1.0 Highlights

This initial release establishes Emu as a comprehensive TUI for mobile device management. Key achievements include:

#### Core Functionality
- **Unified Interface**: Single application for both Android and iOS device management
- **Complete Lifecycle**: Full device management from creation to deletion
- **Real-time Monitoring**: Live status updates and log streaming
- **Performance Optimized**: Fast startup and responsive navigation

#### Platform Support
- **Android**: Complete AVD management with all device types
- **iOS**: Basic simulator control with room for enhancement
- **Cross-platform**: Supports all major desktop platforms

#### Architecture Excellence
- **Clean Design**: Layered architecture with clear separation of concerns
- **Type Safety**: Leverages Rust's type system for reliability
- **Async Performance**: Non-blocking operations throughout
- **Comprehensive Testing**: Extensive test coverage for reliability

### Migration Guide

This is the initial release, so no migration is required. For setup instructions, see the [README.md](README.md).

### Upgrade Path

Future versions will maintain backward compatibility where possible. Breaking changes will be clearly documented and migration guides provided.

### Support

For issues, bug reports, or feature requests, please use the [GitHub Issues](https://github.com/wasabeef/emu/issues) page.