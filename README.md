# 🦤 Emu - Device Manager

[![Build Status](https://img.shields.io/github/actions/workflow/status/wasabeef/emu/ci.yml?branch=main)](https://github.com/wasabeef/emu/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE-MIT)

A TUI for managing Android emulators and iOS simulators, inspired by lazygit.

https://github.com/user-attachments/assets/0ff745ce-7329-4af8-b529-6c5b30d3c48e

## Features

- 🤖 **Android**: Create, start, stop, delete, and wipe AVDs
- 🍎 **iOS** (macOS only): Manage simulators via xcrun simctl
- 📊 **Real-time**: Live device status and log streaming
- ⚡ **Fast**: Instant startup (<150ms, typical ~104ms) with background loading
- ⌨️ **Keyboard-driven**: Vim-like keybindings
- 📱 **Three-panel layout**: Android devices (30%) | iOS devices (30%) | Device details (40%)
- 🔍 **Device details**: Shows specifications, status, RAM/Storage in MB, full paths

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

| Key | Action |
|-----|--------|
| `Tab` | Switch panels |
| `↑`/`↓` | Navigate devices |
| `j`/`k` | Navigate devices (vim-style) |
| `Enter` | Start/Stop device |
| `n` | Create new device |
| `d` | Delete device |
| `w` | Wipe device |
| `r` | Refresh |
| `f` | Cycle log filter |
| `Page Up`/`Page Down` | Scroll device lists/logs |
| `Ctrl+u`/`Ctrl+d` | Page up/down in lists |
| `g`/`G` | Go to top/bottom of list |
| `q` or `Ctrl+q` | Quit |


## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

MIT - see [LICENSE](LICENSE)
