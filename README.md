# 🦤 Emu - Terminal UI for Mobile Device Management

[![Build Status](https://img.shields.io/github/actions/workflow/status/wasabeef/emu/ci.yml?branch=main)](https://github.com/wasabeef/emu/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

A lazygit-inspired Terminal User Interface (TUI) for managing Android emulators and iOS simulators. Built with Rust for performance and reliability.

https://github.com/user-attachments/assets/0ff745ce-7329-4af8-b529-6c5b30d3c48e

## Features

### 🤖 Android AVD Management
- **Complete Lifecycle**: Create, start, stop, delete, and wipe Android Virtual Devices
- **API Level Management**: 
  - Install new Android API levels with real-time progress (5% increments)
  - Automatic system image selection (Google Play > Google APIs > AOSP)
  - Background installation with progress display on main screen
- **Device Categories**: Phone, Tablet, TV, Wear OS, Automotive, Desktop
- **Advanced Configuration**: 
  - RAM: 512MB to 8GB
  - Storage: 1GB to 64GB
  - Automatic naming (e.g., "Pixel 9 Pro Fold API 36")
- **Real-time Logcat**: Color-coded log streaming with filtering

### 🍎 iOS Simulator Management (macOS only)
- **Device Management**: Create, start, stop, delete simulators via `xcrun simctl`
- **Dynamic Detection**: Automatic runtime and device type discovery
- **Device Types**: All iPhone and iPad models
- **Real-time Logs**: System log streaming with intelligent fallback methods

### ⚡ Performance & UX
- **Instant Startup**: < 150ms (typical: ~104ms) with background device loading
- **High-Performance Rendering**: 
  - 125 FPS baseline for ultra-smooth operation
  - Optimized for 60+ FPS environments
  - Consistent performance during animations
  - Event-driven rendering with intelligent batching
- **Responsive Input Handling**:
  - < 8ms input latency for immediate feedback
  - Advanced key repeat handling for smooth navigation
  - Debounced background updates to prevent UI stuttering
  - Up to 50 events processed per frame
- **Real-time Updates**: 
  - Log streaming with < 10ms latency
  - Immediate device status updates
- **Keyboard-driven**: Vim-like keybindings with circular navigation
- **Three-panel Layout**: 
  - Android devices (30%)
  - iOS devices (30%)
  - Device details with live updates (40%)
- **Smart Features**:
  - Intelligent caching with context-aware invalidation
  - Debounced operations for UI responsiveness
  - Background task coordination
  - Memory-efficient log rotation (1000 entries max)

### 🛡️ Reliability
- **Comprehensive Testing**: 15+ test files with 30+ test functions
- **Error Handling**: User-friendly error messages with recovery suggestions
- **Platform Support**: Cross-platform (Android), macOS-specific features for iOS

## Installation

```bash
brew install wasabeef/emu-tap/emu
```

### Build from source
```bash
git clone https://github.com/wasabeef/emu.git
cd emu
cargo build --release
./target/release/emu
```

## Requirements

### Android
- Android SDK with `ANDROID_HOME` set
- `avdmanager`, `emulator`, and `adb` in PATH

### iOS (macOS only)
- Xcode and command line tools
- At least one iOS runtime installed

## Usage

```bash
# Start Emu
emu

# Debug mode
emu --debug
```

### Keyboard Shortcuts

#### Navigation
| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between panels |
| `↑`/`↓` or `j`/`k` | Navigate devices (with circular wrap) |
| `g` / `G` | Jump to top/bottom of list |
| `Ctrl+u` / `Ctrl+d` | Page up/down (half screen) |
| `Page Up` / `Page Down` | Scroll full page |

#### Device Operations
| Key | Action |
|-----|--------|
| `Enter` | Start/Stop selected device |
| `c` | Create new device |
| `d` | Delete device (with confirmation) |
| `w` | Wipe device data (with confirmation) |
| `i` | Install API Level (Android only) |
| `r` | Refresh device list |

#### Log Management
| Key | Action |
|-----|--------|
| `f` | Cycle log filter (ALL → ERROR → WARN → INFO → DEBUG) |
| `F` (Shift+F) | Toggle fullscreen logs |
| `L` (Shift+L) | Clear all logs |
| `s` | Toggle auto-scroll |
| `↑`/`↓` in logs | Manual scroll (disables auto-scroll) |

#### Application
| Key | Action |
|-----|--------|
| `q` or `Ctrl+q` | Quit |
| `Esc` | Cancel operation / Close dialog |


## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT - see [LICENSE](LICENSE)
