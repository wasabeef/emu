# Development Guide

This document provides comprehensive guidance for developers working on Emu, covering setup, workflows, best practices, and contribution guidelines.

## Table of Contents

- [Development Environment Setup](#development-environment-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing Strategy](#testing-strategy)
- [Performance Guidelines](#performance-guidelines)
- [Debugging and Troubleshooting](#debugging-and-troubleshooting)
- [Release Process](#release-process)

## Development Environment Setup

### Prerequisites

#### Required Tools

- **Rust**: Version 1.70 or later
- **Git**: For version control
- **Modern Terminal**: With 256+ color support

#### Platform-Specific Requirements

**Android Development (All Platforms)**:

```bash
# Verify Android SDK setup
echo $ANDROID_HOME
avdmanager list avd
adb version
emulator -version
```

**iOS Development (macOS Only)**:

```bash
# Verify Xcode setup
xcode-select -p
xcrun simctl list devices
```

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/wasabeef/emu.git
cd emu

# Install Rust dependencies
cargo build

# Run tests to verify setup
cargo test --bins --tests  # Recommended: excludes doctests
# cargo test              # Optional: includes doctests (may have import issues)

# Install development tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-audit

# Add Rust components
rustup component add clippy
rustup component add rustfmt
```

### IDE Configuration

#### VS Code Setup

Recommended extensions:

- **rust-analyzer**: Language server
- **CodeLLDB**: Debugging support
- **Even Better TOML**: Configuration file support
- **Error Lens**: Inline error display

`.vscode/settings.json`:

```json
{
  "rust-analyzer.cargo.features": "full",
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "files.associations": {
    "*.rs": "rust"
  }
}
```

#### RustRover/IntelliJ Setup

- Install Rust plugin
- Configure clippy integration
- Set up test runner
- Enable format on save

### Environment Variables

Environment variables for development:

```bash
# Debug logging
RUST_LOG=debug

# Android SDK (if not in system PATH)
ANDROID_HOME=/path/to/android/sdk
PATH=$PATH:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools
```

## Project Structure

### Source Organization

```
src/
├── app/                 # Application core
│   ├── mod.rs          # Main app logic and event loop
│   ├── state.rs        # Application state management
│   ├── events.rs       # Event type definitions
│   └── actions.rs      # User action handlers
├── managers/           # Platform-specific device management
│   ├── common.rs       # DeviceManager trait
│   ├── android.rs      # Android AVD management
│   ├── ios.rs          # iOS Simulator management
│   └── mod.rs          # Module exports
├── models/             # Data structures and types
│   ├── device.rs       # Device models
│   ├── device_config.rs # Device configuration
│   ├── device_info.rs  # Device information
│   ├── device_info.rs  # Device information
│   ├── error.rs        # Error types
│   ├── platform.rs     # Platform enums
│   └── mod.rs          # Module exports
├── ui/                 # Terminal user interface
│   ├── render.rs       # Main rendering logic (three-panel layout)
│   ├── theme.rs        # Color themes and focus indicators
│   ├── widgets.rs      # Custom UI widgets
│   └── mod.rs          # Module exports
├── utils/              # Shared utilities
│   ├── command.rs      # Command execution
│   ├── logger.rs       # Logging utilities
│   └── mod.rs          # Module exports
├── constants/          # Modular constants system
│   ├── mod.rs         # Module exports
│   ├── commands.rs    # CLI tool names and arguments
│   ├── defaults.rs    # Default values and configurations
│   ├── env_vars.rs    # Environment variable names
│   ├── files.rs       # File paths and extensions
│   ├── messages.rs    # User-facing strings
│   ├── patterns.rs    # Regular expressions
│   └── performance.rs # Performance tuning parameters
├── lib.rs             # Library root
└── main.rs            # Application entry point
```

### Test Organization

```
tests/
├── comprehensive_integration_test.rs  # Full workflow tests
├── device_creation_navigation_test.rs # UI navigation tests
├── device_operations_status_test.rs   # Operation status tests
├── ui_focus_and_theme_test.rs        # UI and theme tests
├── startup_performance_test.rs        # Performance benchmarks
├── device_creation_test.rs           # Device creation workflows
├── device_name_sanitization_test.rs  # Input validation
├── integration_test.rs               # General integration
└── ...                               # Additional test suites
```

### Documentation Structure

```
docs/
├── README.md           # Documentation overview
├── ARCHITECTURE.md     # System architecture
├── API.md             # API documentation
├── DEVELOPMENT.md     # This file
└── development_plan.md # Development roadmap
```

## Development Workflow

### Daily Development

#### Basic Commands

```bash
# Start development with live reload
cargo watch -x run

# Run tests with live reload
cargo watch -x test

# Run specific test
cargo test test_name -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Run all tests (recommended - excludes doctests)
cargo test --bins --tests

# Run all tests including doctests (may have import issues in examples)
cargo test
```

#### Advanced Commands

```bash
# Build optimized binary
cargo build --release

# Run with debug output
RUST_LOG=debug cargo run

# Profile performance
cargo build --release
time ./target/release/emu

# Check dependencies
cargo audit

# Generate documentation
cargo doc --open
```

### Git Workflow

#### Branch Naming

- **Feature**: `feature/device-details-panel`
- **Bug Fix**: `fix/android-state-detection`
- **Documentation**: `docs/development-guide-updates`
- **Performance**: `perf/startup-optimization`
- **Refactor**: `refactor/async-state-management`

#### Commit Messages

Follow conventional commit format:

```
type(scope): description

Detailed explanation if needed

Closes #issue-number
```

Examples:

```
feat(android): add device wipe functionality

Implement device data wiping for Android AVDs using direct
file deletion instead of AVD recreation for better performance.

Closes #45

fix(ui): resolve panel switching performance issue

Add debounced updates with 50ms delay to prevent UI stuttering
during rapid navigation between device panels.

test(integration): add comprehensive device lifecycle tests

Add tests covering complete device workflows including creation,
start, stop, delete, and error conditions.
```

### Code Review Process

#### Before Submitting

```bash
# Ensure code quality
cargo fmt
cargo clippy --all-targets --all-features
cargo test --bins --tests

# Check for common issues
cargo audit
```

#### PR Checklist

- [ ] Code follows style guidelines
- [ ] All tests pass
- [ ] New functionality has tests
- [ ] Documentation updated
- [ ] Performance impact considered
- [ ] Error handling implemented
- [ ] No breaking changes (or clearly documented)

## Testing Strategy

### Test Categories

#### Test Suite Overview

The project has 15 test files with 31+ test functions covering all major functionality.

#### Unit Tests

Located in source files with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_name_validation() {
        assert!(validate_device_name("Valid_Name"));
        assert!(!validate_device_name("Invalid Name!"));
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

#### Integration Tests

Located in `tests/` directory:

```rust
// tests/device_lifecycle_test.rs
use emu::managers::AndroidManager;
use emu::models::DeviceConfig;

#[tokio::test]
async fn test_complete_device_lifecycle() {
    let manager = AndroidManager::new().unwrap();

    // Test complete workflow
    let config = DeviceConfig::new("test_device", "pixel_7", "31");
    manager.create_device(&config).await.unwrap();
    manager.start_device("test_device").await.unwrap();
    manager.stop_device("test_device").await.unwrap();
    manager.delete_device("test_device").await.unwrap();
}
```

#### Performance Tests

```rust
#[tokio::test]
async fn test_startup_performance() {
    let start = std::time::Instant::now();
    let app = App::new(Config::default()).await.unwrap();
    let duration = start.elapsed();

    assert!(duration < std::time::Duration::from_millis(150));
    println!("Startup time: {:?}", duration);
}
```

### Running Tests

```bash
# Run all tests (recommended - excludes doctests)
cargo test --bins --tests

# Run all tests including doctests (may have import issues in examples)
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test file
cargo test --test comprehensive_integration_test

# Run specific test function
cargo test test_device_creation

# Run tests for specific module
cargo test android::

# Run performance tests
cargo test startup_performance_test -- --nocapture

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Guidelines

#### Writing Good Tests

1. **Descriptive Names**: Test names should clearly describe what is being tested
2. **Single Responsibility**: Each test should test one specific behavior
3. **Independent**: Tests should not depend on each other
4. **Fast**: Unit tests should run quickly
5. **Reliable**: Tests should pass consistently
6. **Coverage**: Aim for comprehensive coverage - current suite has 31+ test functions

#### Mock Usage

```rust
use mockall::predicate::*;
use mockall::mock;

mock! {
    AndroidManager {}

    #[async_trait]
    impl DeviceManager for AndroidManager {
        async fn list_devices(&self) -> Result<Vec<AndroidDevice>>;
        async fn start_device(&self, id: &str) -> Result<()>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock_manager = MockAndroidManager::new();

    mock_manager
        .expect_list_devices()
        .returning(|| Ok(vec![create_mock_device()]));

    let devices = mock_manager.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
}
```

## Performance Guidelines

### Performance Requirements

- **Startup Time**: < 150ms (typical: ~104ms)
- **Panel Switching**: < 100ms
- **Device Navigation**: < 50ms
- **Memory Usage**: < 50MB baseline
- **Log Streaming**: Real-time with < 10ms latency

### Optimization Strategies

#### Startup Optimization

```rust
// Good: Background loading
fn start_background_device_loading(&mut self) {
    let state_clone = Arc::clone(&self.state);
    let manager = self.android_manager.clone();

    tokio::spawn(async move {
        // Load devices in background
        let devices = manager.list_devices().await?;
        let mut state = state_clone.lock().await;
        state.android_devices = devices;
    });
}

// Avoid: Blocking operations
async fn new() -> Result<Self> {
    let devices = manager.list_devices().await?; // Blocks startup
    Ok(Self { devices })
}
```

#### UI Responsiveness

```rust
// Good: Debounced updates
async fn schedule_update(&mut self) {
    if let Some(handle) = self.update_handle.take() {
        handle.abort(); // Cancel previous update
    }

    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        // Perform update
    });

    self.update_handle = Some(handle);
}

// Avoid: Immediate expensive operations
fn on_selection_change(&mut self) {
    self.update_device_details(); // Blocks UI
}
```

#### Memory Management

```rust
// Good: Bounded collections
impl AppState {
    pub fn add_log(&mut self, level: String, message: String) {
        self.device_logs.push_back(LogEntry::new(level, message));

        // Limit log entries (max 1000)
        while self.device_logs.len() > self.max_log_entries {
            self.device_logs.pop_front();
        }
    }
}

// Avoid: Unbounded growth
fn add_log(&mut self, entry: LogEntry) {
    self.logs.push(entry); // Can grow indefinitely
}
```

### Performance Testing

```bash
# Measure startup time
time cargo run --release

# Profile with perf (Linux)
perf record --call-graph=dwarf cargo run --release
perf report

# Memory profiling with valgrind (Linux)
valgrind --tool=massif cargo run --release

# Custom benchmarks
cargo test startup_performance_test -- --nocapture
```

## Debugging and Troubleshooting

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- --debug

# Module-specific logging
RUST_LOG=emu::managers::android=debug cargo run

# Trace-level logging
RUST_LOG=trace cargo run -- --debug
```

### Common Issues

#### Android SDK Issues

```bash
# Verify SDK installation
echo $ANDROID_HOME
ls $ANDROID_HOME/emulator
ls $ANDROID_HOME/platform-tools

# Check PATH
which avdmanager
which emulator
which adb
```

#### iOS Issues (macOS)

```bash
# Verify Xcode installation
xcode-select -p
xcrun simctl list devices

# Install command line tools
xcode-select --install
```

#### Performance Issues

```bash
# Profile startup
time cargo run --release

# Check system resources
top -p $(pgrep emu)

# Memory usage
ps aux | grep emu
```

### Debugging Tools

#### Rust-Specific

```bash
# Debug with lldb
cargo build
lldb target/debug/emu

# Debug with gdb (Linux)
cargo build
gdb target/debug/emu
```

#### Application-Specific

```rust
// Add debug prints
log::debug!("Device operation: {:?}", operation);

// Assert invariants
debug_assert!(self.selected_android < self.android_devices.len());

// Conditional compilation
#[cfg(debug_assertions)]
fn validate_state(&self) {
    // Expensive validation only in debug builds
}
```

### Troubleshooting Guide

#### Build Issues

1. **Dependency conflicts**: `cargo clean && cargo build`
2. **Rust version**: `rustup update`
3. **Missing components**: `rustup component add clippy rustfmt`

#### Runtime Issues

1. **Android SDK**: Verify `ANDROID_HOME` and PATH
2. **iOS Simulator**: Check Xcode installation
3. **Permissions**: Ensure terminal has necessary permissions

#### Test Issues

1. **Flaky tests**: Check for race conditions
2. **Platform-specific**: Use conditional compilation
3. **Performance tests**: Run on consistent hardware

## Release Process

### Version Management

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

#### Version Update Process

1. Update `Cargo.toml` version
2. Update `CHANGELOG.md`
3. Create git tag
4. Publish release

### Pre-Release Checklist

```bash
# Code quality
cargo fmt --check
cargo clippy --all-targets --all-features
cargo audit

# Testing
cargo test --bins --tests
cargo test --test comprehensive_integration_test
cargo test startup_performance_test -- --nocapture

# Documentation
cargo doc
# Verify README is up to date
# Update CHANGELOG.md

# Build
cargo build --release
```

### Release Commands

```bash
# Create release build
cargo build --release

# Run final tests
cargo test --bins --tests --release

# Create git tag
git tag -a v1.0.0 -m "Release version 1.0.0"
git push origin v1.0.0

# Publish to crates.io (if applicable)
cargo publish
```

### Post-Release

1. **Update documentation**: Ensure docs reflect new version
2. **Monitor issues**: Watch for bug reports
3. **Plan next release**: Update development roadmap

## Constants Architecture

### Overview

The application uses a modular constants system to eliminate hardcoded strings and improve maintainability. Constants are organized by category in the `src/constants/` directory.

### Module Structure

```rust
// src/constants/mod.rs - Public API
pub mod commands;
pub mod defaults;
pub mod env_vars;
pub mod files;
pub mod messages;
pub mod patterns;
pub mod performance;

// Usage example
use crate::constants::{
    commands::ANDROID_COMMANDS,
    defaults::DEFAULT_RAM_SIZE,
    messages::MSG_DEVICE_CREATED,
};
```

### Categories

#### Commands (`commands.rs`)

- CLI tool names and paths
- Command arguments
- Platform-specific commands

```rust
pub const ADB: &str = "adb";
pub const EMULATOR: &str = "emulator";
pub const AVDMANAGER: &str = "avdmanager";
```

#### Defaults (`defaults.rs`)

- Default configuration values
- UI dimensions
- Resource limits

```rust
pub const DEFAULT_RAM_SIZE: u32 = 2048; // MB
pub const DEFAULT_STORAGE_SIZE: u32 = 8192; // MB
pub const MAX_LOG_ENTRIES: usize = 1000;
```

#### Environment Variables (`env_vars.rs`)

- System environment variable names
- Configuration paths

```rust
pub const ANDROID_HOME: &str = "ANDROID_HOME";
pub const RUST_LOG: &str = "RUST_LOG";
```

#### Files (`files.rs`)

- File paths and extensions
- Configuration file names

```rust
pub const AVD_CONFIG_FILE: &str = "config.ini";
pub const HARDWARE_QEMU_INI: &str = "hardware-qemu.ini";
```

#### Messages (`messages.rs`)

- User-facing strings
- Error messages
- Status notifications

```rust
pub const MSG_DEVICE_CREATED: &str = "Device created successfully";
pub const MSG_LOADING_DEVICES: &str = "Loading devices...";
```

#### Patterns (`patterns.rs`)

- Regular expressions
- Parsing patterns

```rust
pub const DEVICE_NAME_PATTERN: &str = r"^[a-zA-Z0-9_.-]+$";
pub const API_LEVEL_PATTERN: &str = r"API (\d+)";
```

#### Performance (`performance.rs`)

- Timing constants
- Debounce delays
- Cache durations

```rust
pub const STARTUP_MAX_TIME_MS: u64 = 150;
pub const UI_DEBOUNCE_MS: u64 = 50;
pub const CACHE_EXPIRY_SECS: u64 = 300; // 5 minutes
```

### Best Practices

1. **Naming Convention**: Use SCREAMING_SNAKE_CASE for constants
2. **Documentation**: Add doc comments explaining usage
3. **Organization**: Group related constants together
4. **Type Safety**: Use appropriate types (not just `&str`)
5. **Visibility**: Only export what's needed

### Adding New Constants

1. Determine the appropriate module based on category
2. Add the constant with documentation
3. Update module exports if needed
4. Replace hardcoded values throughout codebase

Example:

```rust
// In messages.rs
/// Message shown when device deletion is confirmed
pub const MSG_DEVICE_DELETED: &str = "Device deleted successfully";

// Usage
state.add_notification(MSG_DEVICE_DELETED, NotificationType::Success);
```

## Best Practices Summary

### Code Quality

- Run `cargo fmt` and `cargo clippy` before committing
- Write comprehensive tests for new functionality (follow the pattern of 31+ existing tests)
- Use meaningful variable and function names
- Add documentation for public APIs
- Follow the trait-based abstraction pattern for platform-specific code

### Performance

- Keep startup time under 150ms (typical: ~104ms)
- Use background loading for heavy operations
- Implement debouncing for UI operations (50-100ms delays)
- Monitor memory usage and prevent leaks (log rotation at 1000 entries)
- Smart caching with platform-aware invalidation

### Error Handling

- Use `Result<T, E>` for fallible operations
- Provide helpful error messages with context
- Never use `.unwrap()` in user-facing code
- Implement graceful error recovery

### Testing

- Write tests for all new functionality
- Include performance tests for critical paths
- Test error conditions and edge cases
- Use mocks for external dependencies

### Documentation

- Keep documentation up to date
- Include examples in API documentation
- Document architectural decisions
- Maintain comprehensive README

## API Reference

### Core Traits

#### DeviceManager Trait

The `DeviceManager` trait provides a unified interface for device operations across platforms:

```rust
pub trait DeviceManager: Send + Sync + Clone {
    /// List all available devices
    fn list_devices(&self) -> impl Future<Output = Result<Vec<Device>>> + Send;

    /// Start a device by identifier
    fn start_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;

    /// Stop a device by identifier
    fn stop_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;

    /// Create a new device with the given configuration
    fn create_device(&self, config: &DeviceConfig) -> impl Future<Output = Result<()>> + Send;

    /// Delete a device by identifier
    fn delete_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;

    /// Wipe device data (cold boot)
    fn wipe_device(&self, id: &str) -> impl Future<Output = Result<()>> + Send;

    /// Get detailed information about a device
    fn get_device_details(&self, id: &str) -> impl Future<Output = Result<DeviceDetails>> + Send;

    /// List available device types for creation
    fn list_device_types(&self) -> impl Future<Output = Result<Vec<(String, String)>>> + Send;

    /// List available system images or runtimes
    fn list_available_targets(&self) -> impl Future<Output = Result<Vec<(String, String)>>> + Send;
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

    // Create a new device
    let config = DeviceConfig::new("MyDevice", "pixel_7", "31");
    manager.create_device(&config).await?;

    // Start the device
    manager.start_device("MyDevice").await?;

    Ok(())
}
```

### Data Models

#### Device Types

```rust
// Android Device
pub struct AndroidDevice {
    pub name: String,
    pub target: String,
    pub status: DeviceStatus,
    pub path: String,
}

// iOS Device
pub struct IosDevice {
    pub udid: String,
    pub name: String,
    pub runtime: String,
    pub status: DeviceStatus,
}

// Device Status
#[derive(Debug, Clone, PartialEq)]
pub enum DeviceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Unknown,
}
```

#### Configuration Types

```rust
pub struct DeviceConfig {
    pub name: String,
    pub device_type: String,
    pub target: String,
    pub ram_size: Option<u32>,    // In MB
    pub storage_size: Option<u32>, // In MB
}

impl DeviceConfig {
    pub fn new(name: &str, device_type: &str, target: &str) -> Self {
        Self {
            name: name.to_string(),
            device_type: device_type.to_string(),
            target: target.to_string(),
            ram_size: None,
            storage_size: None,
        }
    }
}
```

## Release Process

### Prerequisites

1. Ensure all tests are passing:

   ```bash
   cargo test --bins --tests --all-features  # Recommended: excludes doctests
   # cargo test --all-features              # Optional: includes doctests (may have import issues)
   cargo fmt -- --check
   cargo clippy --all-features -- -D warnings
   ```

2. Update version in `Cargo.toml`:

   ```toml
   [package]
   version = "0.2.2"  # Update this
   ```

3. Update `CHANGELOG.md` (optional - automated release notes are generated):
   - The release process will automatically generate release notes using git-cliff
   - Manual updates are only needed for major releases or special announcements

4. Commit the version bump:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to v0.2.2"
   ```

### Creating a Release

1. Create and push a tag:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. The GitHub Actions workflow will automatically:
   - Generate release notes from commit history using git-cliff
   - Build binaries for all platforms
   - Create a GitHub release with auto-generated release notes
   - Upload the binaries

3. Homebrew formula update:
   - The release workflow automatically updates the formula
   - It calculates SHA256 for all platform binaries
   - Creates a commit in wasabeef/homebrew-emu-tap
   - No manual intervention required

### Post-Release

1. Verify the release on GitHub
2. Test installation via Homebrew:

   ```bash
   brew tap wasabeef/tap
   brew install emu
   ```

3. Announce the release:
   - Twitter/X
   - Reddit (r/rust, r/androiddev)
   - Hacker News (if significant release)

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes

### Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) for automatic changelog generation:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `perf:` Performance improvements
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Maintenance tasks

Example:

```bash
git commit -m "feat: add iOS simulator log streaming"
git commit -m "fix: prevent crash on small terminal sizes"
git commit -m "docs: update installation instructions"
```

## Troubleshooting

### Android SDK Issues

```bash
# Verify SDK installation
echo $ANDROID_HOME
ls $ANDROID_HOME/emulator
ls $ANDROID_HOME/platform-tools

# Check PATH
which avdmanager
which emulator
which adb
```

### iOS Issues (macOS)

```bash
# Verify Xcode installation
xcode-select -p
xcrun simctl list devices

# Install command line tools
xcode-select --install
```

### AVD Creation Debug

If AVD creation fails, check the following:

1. **System Image Availability**: Ensure the target system image is installed
2. **Device Definition**: Verify the device type exists in the SDK
3. **Storage Space**: Check available disk space for AVD creation
4. **Permissions**: Ensure write permissions to AVD directory

```bash
# Debug AVD creation
avdmanager list target
avdmanager list device
emulator -list-avds
```

### Common Issues

#### Build Issues

1. **Dependency conflicts**: `cargo clean && cargo build`
2. **Rust version**: `rustup update`
3. **Missing components**: `rustup component add clippy rustfmt`

#### Runtime Issues

1. **Android SDK**: Verify `ANDROID_HOME` and PATH
2. **iOS Simulator**: Check Xcode installation
3. **Permissions**: Ensure terminal has necessary permissions

#### Test Issues

1. **Flaky tests**: Check for race conditions
2. **Platform-specific**: Use conditional compilation
3. **Performance tests**: Run on consistent hardware

This comprehensive development guide provides everything needed for contributing to Emu. For questions or clarifications, please create an issue or start a discussion.
