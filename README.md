# 🦤 Emu - Device Manager

[![Build Status](https://img.shields.io/github/actions/workflow/status/wasabeef/emu/ci.yml?branch=main)](https://github.com/wasabeef/emu/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)

A TUI for managing Android emulators and iOS simulators.

<https://github.com/user-attachments/assets/3f4bfde6-6e54-45bd-9a7c-0cb2eeb2f861>

## Features

### Core Functionality

- 🤖 **Android AVD Management**: Create, start, stop, delete, and wipe Android Virtual Devices
  - Dynamic API level detection with installed system images
  - Support for Phone, Tablet, TV, Wear OS, Automotive, Desktop device types
  - Advanced configuration: RAM (512MB-8GB), Storage (1GB-64GB)
  - Automatic placeholder naming (e.g., "Pixel 9 Pro Fold API 36")
  - Real-time system image installation with progress tracking
- 🍎 **iOS Simulator Management** (macOS only): Manage simulators via `xcrun simctl`
  - Device type selection (iPhone, iPad models)
  - Runtime version selection with dynamic detection
  - Basic device operations with status monitoring

### User Experience

- **Real-time Monitoring**: Live device status and log streaming with < 10ms latency
- **High Performance**: Instant startup (<150ms, typical ~104ms) with background loading
- **Keyboard-driven**: Vim-like keybindings with circular navigation
- **Three-panel layout**: Android devices (30%) | iOS devices (30%) | Device details (40%)
- **Comprehensive Details**: Device specifications, status, RAM/Storage in MB, full paths
- **Smart Caching**: Platform-aware cache invalidation and background loading
- **Robust Testing**: 15 test files with 31+ test functions ensuring reliability
- **API Level Management**: Install/uninstall system images directly from TUI

## Installation

### Homebrew

```bash
brew install wasabeef/emu-tap/emu
```

### Build from Source

```bash
# Clone and install
git clone https://github.com/wasabeef/emu.git
cd emu
cargo install --path .
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
```

### Keyboard Shortcuts

| Key                   | Action                       |
| --------------------- | ---------------------------- |
| `Tab`                 | Switch panels                |
| `↑`/`↓`               | Navigate devices             |
| `j`/`k`               | Navigate devices (vim-style) |
| `Enter`               | Start/Stop device            |
| `c`                   | Create new device            |
| `i`                   | Manage API levels (Android)  |
| `d`                   | Delete device                |
| `w`                   | Wipe device                  |
| `r`                   | Refresh                      |
| `f`                   | Cycle log filter             |
| `Page Up`/`Page Down` | Scroll device lists/logs     |
| `Ctrl+u`/`Ctrl+d`     | Page up/down in lists        |
| `g`/`G`               | Go to top/bottom of list     |
| `q` or `Ctrl+q`       | Quit                         |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT - see [LICENSE](LICENSE)
