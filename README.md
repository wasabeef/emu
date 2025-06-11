# Emu 📱

[![Build Status](https://img.shields.io/github/actions/workflow/status/wasabeef/emu/ci.yml?branch=main)](https://github.com/wasabeef/emu/actions)
[![Crates.io](https://img.shields.io/crates/v/emu.svg)](https://crates.io/crates/emu)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**Emu** is a lazygit-inspired Terminal User Interface (TUI) for managing Android emulators and iOS simulators. Built with Rust, it provides a unified, intuitive interface for mobile developers to manage virtual devices directly from the terminal.

## ✨ Features

### Core Functionality
- **Unified Interface**: Single TUI for both Android AVDs and iOS simulators
- **Real-time Device Management**: Start, stop, create, delete, and wipe devices
- **Live Log Streaming**: Real-time logcat/device logs with color-coded levels
- **Background Operations**: Non-blocking UI with async task management
- **Cross-Platform Support**: macOS (with full iOS support), Windows, and Linux

### Android Emulator Management
- **Complete AVD Lifecycle**: Create, start, stop, delete, and wipe Android Virtual Devices
- **Device Categories**: Support for Phone, Tablet, TV, Wear OS, and Automotive devices
- **Custom Configuration**: Configurable RAM (512MB-8GB) and storage (1GB-64GB)
- **API Level Selection**: Choose from available Android system images
- **Real-time Status**: Live device state monitoring with automatic updates
- **Logcat Integration**: Real-time Android log streaming with filtering

### iOS Simulator Management (macOS only)
- **Simulator Control**: Launch, stop, and manage iOS simulators via `xcrun simctl`
- **Device Types**: Support for all iPhone and iPad models
- **Runtime Selection**: Choose from available iOS versions
- **Status Monitoring**: Real-time simulator state tracking

### User Interface
- **Three-Panel Layout**: Android devices (30%) | iOS devices (30%) | Device details (40%)
- **Device Details Panel**: Display device specifications, status, and configuration
- **Modal Workflows**: Guided device creation with field validation
- **Status Notifications**: Success/error/warning messages with auto-dismiss
- **Keyboard Navigation**: Vim-like keybindings (hjkl) and arrow key support
- **Focus Indicators**: Visual feedback for active panels and selections

### Performance Optimizations
- **Fast Startup**: UI renders immediately (~100ms), data loads in background
- **Smart Caching**: Device metadata cached for instant access
- **Debounced Updates**: 50-100ms delays prevent UI stuttering during navigation
- **Incremental Refresh**: Targeted updates instead of full device list rebuilds
- **Memory Management**: Automatic log rotation and cache expiration

## 📋 Requirements

### System Requirements
- **Rust**: Version 1.70 or later
- **Terminal**: Modern terminal with color support (256+ colors recommended)

### Android Development
- **Android SDK**: Complete Android SDK installation
- **Environment**: `ANDROID_HOME` environment variable set
- **Tools**: `avdmanager`, `emulator`, and `adb` in system PATH

### iOS Development (macOS only)
- **Xcode**: Latest version recommended
- **Command Line Tools**: `xcode-select --install`
- **Simulator Runtime**: At least one iOS runtime installed

### Verification Commands
```bash
# Android
echo $ANDROID_HOME
avdmanager list avd
adb version

# iOS (macOS)
xcrun simctl list devices
```

## 🚀 Installation

### Homebrew (macOS/Linux)
```bash
brew tap wasabeef/tap
brew install emu
```

### From Crates.io
```bash
cargo install emu
```

### From Source
```bash
git clone https://github.com/wasabeef/emu.git
cd emu
cargo build --release
./target/release/emu
```

### Using Cargo Run
```bash
git clone https://github.com/wasabeef/emu.git
cd emu
cargo run
```

## 🛠️ Usage

### Basic Usage
```bash
# Start Emu
emu

# Debug mode (command line output, no TUI)
emu --debug

# Show help
emu --help
```

### Interface Overview

### Keyboard Shortcuts

#### Navigation
- `Tab` / `Shift+Tab`: Switch between Android and iOS panels
- `h` / `l` / `Left` / `Right`: Switch between panels
- `↑` / `↓` / `k` / `j`: Navigate device lists (circular)
- `Ctrl+h/j/k/l`: Same as arrow keys in device creation

#### Device Operations
- `Enter`: Start/Stop selected device
- `c`: Create new device
- `d`: Delete selected device (with confirmation)
- `w`: Wipe device data (with confirmation)
- `r`: Refresh device lists

#### Log Management
- `f`: Cycle log filter (ERROR → WARN → INFO → DEBUG → ALL)
- `Shift+F`: Toggle fullscreen logs
- `Shift+L`: Clear all logs

#### Application
- `Ctrl+q`: Quit application
- `Esc`: Dismiss notifications/cancel operations

### Device Creation Workflow
1. Press `c` to open device creation modal
2. Navigate fields with `Tab`/`Shift+Tab` or `↑`/`↓`
3. Use `←`/`→` to change selections for dropdowns
4. Type directly for name, RAM, and storage fields
5. Press `Enter` to create device
6. Press `Esc` to cancel

#### Android Device Creation Fields
- **API Level**: Available Android system images
- **Category**: Phone, Tablet, TV, Wear OS, Automotive, Desktop
- **Device Type**: Specific device models (filtered by category)
- **RAM Size**: 512, 1024, 2048, 4096, 8192 (MB)
- **Storage Size**: 1024, 2048, 4096, 8192, 16384, 32768 (MB)
- **Name**: Custom device name (auto-generated suggestion)

#### iOS Device Creation Fields
- **Runtime**: Available iOS versions
- **Device Type**: iPhone and iPad models
- **Name**: Custom device name

## ⚙️ Configuration

### Configuration File
Emu searches for `config.toml` in the following locations:
1. Path from `--config` CLI argument
2. `EMU_CONFIG` environment variable
3. Platform-specific config directories:
   - **Linux**: `~/.config/emu/config.toml`
   - **macOS**: `~/Library/Application Support/emu/config.toml`
   - **Windows**: `%APPDATA%\emu\config.toml`

### Configuration Options
```toml
# config.toml
[theme]
# Theme selection: "dark" (default)
name = "dark"

[android]
# Default RAM size for new devices (MB)
default_ram = 2048
# Default storage size for new devices (MB)
default_storage = 8192

[ios]
# Default device type for new simulators
default_device = "iPhone 14"

[logging]
# Maximum number of log entries to keep in memory
max_entries = 1000
# Auto-scroll logs to bottom
auto_scroll = true

[performance]
# Device refresh interval (seconds)
refresh_interval = 3
# Log update delay for panel switching (ms)
log_update_delay = 100
```

## 🏗️ Architecture

### Project Structure
```
src/
├── app/                 # Application core
│   ├── mod.rs          # Main application logic
│   ├── state.rs        # Application state management
│   ├── events.rs       # Event handling
│   └── actions.rs      # User actions
├── managers/           # Platform-specific device management
│   ├── common.rs       # DeviceManager trait
│   ├── android.rs      # Android AVD management
│   └── ios.rs          # iOS Simulator management
├── models/             # Data structures
│   ├── device.rs       # Device models
│   ├── error.rs        # Error types
│   └── platform.rs     # Platform enums
├── ui/                 # Terminal user interface
│   ├── render.rs       # Main rendering logic
│   ├── theme.rs        # Color themes
│   └── widgets.rs      # Custom widgets
├── utils/              # Utilities
│   ├── command.rs      # Command execution
│   └── logger.rs       # Logging utilities
├── config.rs           # Configuration management
├── constants.rs        # Application constants
├── lib.rs             # Library root
└── main.rs            # Application entry point
```

### Key Design Patterns

#### Trait-Based Abstraction
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

#### Async State Management
```rust
pub struct App {
    state: Arc<Mutex<AppState>>,
    android_manager: AndroidManager,
    ios_manager: Option<IosManager>,
    // Background task handles
    log_update_handle: Option<JoinHandle<()>>,
    detail_update_handle: Option<JoinHandle<()>>,
}
```

#### Background Task Coordination
- Non-blocking device operations
- Debounced UI updates
- Smart cache invalidation
- Async log streaming

## 🧪 Testing

### Test Suite Overview
- **15 test files** with comprehensive coverage
- **31 test functions** covering all major functionality
- **Performance validation** with startup time requirements
- **Integration testing** for complete workflows

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test comprehensive_integration_test
cargo test --test device_creation_navigation_test
cargo test --test startup_performance_test

# Run with output
cargo test -- --nocapture

# Performance tests
cargo test startup_performance_test -- --nocapture
```

### Test Categories
- **Integration Tests**: Complete device lifecycle workflows
- **Performance Tests**: Startup time and responsiveness validation
- **UI Tests**: Navigation, focus management, theme handling
- **Unit Tests**: Device name validation, configuration parsing
- **Error Handling**: Invalid inputs, missing dependencies

### Performance Benchmarks
- **Startup Time**: < 150ms (typical: ~104ms)
- **Panel Switching**: < 100ms
- **Device Navigation**: < 50ms
- **Log Streaming**: Real-time with < 10ms latency

## 🐛 Troubleshooting

### Common Issues

#### Android SDK Not Found
```bash
# Set ANDROID_HOME
export ANDROID_HOME=/path/to/android/sdk
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

#### Emulator Command Not Found
```bash
# Add emulator to PATH
export PATH=$PATH:$ANDROID_HOME/emulator
```

#### iOS Simulator Not Available (macOS)
```bash
# Install Xcode command line tools
xcode-select --install

# Verify simctl
xcrun simctl list devices
```

#### Device List Empty
- Ensure Android SDK is properly installed
- Create at least one AVD using Android Studio
- For iOS, install at least one iOS runtime

#### Performance Issues
- Use `--debug` flag to identify bottlenecks
- Check system resources (emulators are resource-intensive)
- Reduce log verbosity in device settings

### Debug Mode
```bash
# Enable detailed logging
emu --debug

# Set log level
RUST_LOG=debug emu --debug
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup
```bash
# Clone repository
git clone https://github.com/wasabeef/emu.git
cd emu

# Install dependencies
cargo build

# Run tests
cargo test

# Run with live reload
cargo watch -x run

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Contribution Areas
- **iOS Feature Parity**: Enhanced iOS device management
- **Configuration System**: Advanced configuration options
- **Performance**: Further optimization opportunities
- **Platform Support**: Windows-specific enhancements
- **Device Operations**: Port forwarding, app installation
- **UI/UX**: Additional themes, layout options

## 📄 License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.

## 🙏 Acknowledgments

- Inspired by [lazygit](https://github.com/jesseduffield/lazygit) TUI design
- Built with [Ratatui](https://ratatui.rs/) for terminal UI
- Powered by [Tokio](https://tokio.rs/) async runtime
- CLI parsing with [Clap](https://clap.rs/)
- Configuration with [Serde](https://serde.rs/)

## 📊 Project Status

- **Current Version**: 0.1.0
- **Development Status**: Active
- **Platform Support**: Android (All platforms), iOS (macOS only)
- **Performance**: Optimized for responsiveness and low resource usage
- **Testing**: Comprehensive test suite with 90%+ coverage

---

*Built with ❤️ in Rust*