# Emu: Technical Documentation

This document provides a more detailed technical overview of the Emu project, based on the initial project brief.

## 1. Project Overview

**Project Name:** Emu

**Goal:** To create a Terminal User Interface (TUI) tool, inspired by lazygit, for unified management of Android emulators and iOS simulators. The primary aim is to enhance mobile developer productivity by offering intuitive, keyboard-driven navigation and operations.

**Current Status:** Production-ready with comprehensive testing (15 test files, 31+ test functions) and performance optimization (startup < 150ms, typical ~104ms).

## 2. Technical Requirements

### 2.1. Core Technologies

- **Language:** Rust (chosen for performance, safety, and cross-platform capabilities)
- **Primary UI Library:** Ratatui
- **Async Runtime:** Tokio (for managing concurrent operations, especially I/O with external commands)

### 2.2. Supported Operating Systems

- **macOS:** Full support, including iOS Simulator management (mandatory) and Android Emulator management.
- **Windows:** Android Emulator management.
- **Linux:** Android Emulator management.

## 3. Functional Requirements

### 3.1. Android Emulator Management (via `avdmanager` and `emulator` commands)

#### 3.1.1. Emulator Creation

- **API Level Selection:** Dynamic detection of installed system images.
- **Device Type Selection:** Support for Phone, Tablet, TV, Wear OS, Automotive, Desktop.
- **AVD Name:** Automatic placeholder generation in "Pixel 9 Pro Fold API 36" format.
- **Advanced Configuration Options:** RAM (512MB-8GB), Storage (1GB-64GB), with proper skin detection and fallback.

#### 3.1.2. Listing and Operations

- **List AVDs:** Display a list of all created AVDs with accurate status detection.
- **Start/Stop:** Real-time device state monitoring with automatic updates.
- **Delete AVD:** Automatic device stopping before deletion (confirmation dialog pending).
- **Wipe Data (Cold Boot):** Android cold boot functionality implementation pending.
- **Device Details Panel:** Shows specifications, status, RAM/Storage in MB, full paths (40% of screen width).

### 3.2. iOS Simulator Management (via `simctl` command on macOS)

#### 3.2.1. Simulator Launch

- **List Available Runtimes:** Show available iOS versions/runtimes.
- **Device Type Selection:** Allow selection from various iPhone and iPad models.
- **Display Status:** Indicate the running state of simulators.
- _(Note: iOS simulator creation is typically handled by Xcode installation and runtime downloads. Emu will focus on listing and launching existing simulator configurations.)_

### 3.3. Common Features

- **Three-Panel Layout:** Android devices (30%) | iOS devices (30%) | Device details (40%).
- **Keyboard Navigation:** Vim-like keybindings with circular navigation, page scrolling.
- **Real-time Updates:** Live device status monitoring, log streaming with < 10ms latency.
- **Performance:** Fast startup (~104ms), smart caching, debounced updates (50-100ms).
- **Logging:** Color-coded log levels with automatic rotation (1000 entries max).
- **Error Handling:** Comprehensive error handling with user-friendly notifications.

## 4. UI/UX Requirements

### 4.1. Screen Layout

Three-panel layout (30% | 30% | 40%):

```
┌──────────────────────────────────────────────────────────────┐
│ Emu - Mobile Device Manager                                    │
├───────────────────┬───────────────────┬────────────────────────┤
│ Android Devices    │ iOS Devices       │ Device Details              │
├───────────────────┼───────────────────┼────────────────────────┤
│ ► Pixel 7 (API 33) │   iPhone 15       │ Name: Pixel 7               │
│   Nexus 5 (API 28) │   iPhone 14 Pro   │ Status: Running             │
│   Tablet (API 30)  │   iPad Pro        │ API Level: 33              │
│                    │                   │ RAM: 2048 MB               │
│                    │                   │ Storage: 8192 MB           │
│                    │                   │ Path: /Users/.android/avd  │
├───────────────────┴───────────────────┴────────────────────────┤
│ [Tab] Switch panels │ [↑↓/jk] Navigate │ [Enter] Start/Stop         │
│ [c] Create [d] Delete [w] Wipe [r] Refresh [f] Filter [q] Quit    │
└──────────────────────────────────────────────────────────────┘
```

### 4.2. Keybindings

- **Global:**
  - `Tab`: Switch between panels (Android | iOS | Details).
  - `q` or `Ctrl+q`: Quit application.
- **Navigation:**
  - `↑/↓` or `j/k`: Move up/down (circular navigation).
  - `Page Up/Page Down`: Scroll device lists/logs.
  - `Ctrl+u/Ctrl+d`: Page up/down in lists.
  - `g/G`: Go to top/bottom of list.
- **Device Operations:**
  - `Enter`: Start/Stop selected device.
  - `c`: Create new device with guided form.
  - `d`: Delete selected device (confirmation pending).
  - `w`: Wipe device data (Android - pending implementation).
  - `r`: Refresh device lists.
- **Log Management:**
  - `f`: Cycle log filter (All → Error → Warning → Info → Debug).
  - Real-time log streaming with automatic rotation (1000 entries max).

## 5. Architecture

```
src/
├── main.rs           // Entry point
├── lib.rs            // Library root
├── app/              // Application core
│   ├── mod.rs        // Main app logic, event loop, background tasks
│   ├── state.rs      // Centralized state management
│   ├── events.rs     // Event type definitions
│   └── actions.rs    // User action handlers
├── managers/         // Platform-specific device management
│   ├── common.rs     // DeviceManager trait (impl Future + Send)
│   ├── android.rs    // Android AVD management
│   └── ios.rs        // iOS simulator management
├── models/           // Core data structures
│   ├── device.rs     // AndroidDevice, IosDevice, DeviceStatus
│   ├── device_config.rs // Device creation configuration
│   ├── error.rs      // Error types with user-friendly formatting
│   └── platform.rs   // Platform enums and configurations
├── ui/               // Terminal UI
│   ├── render.rs     // Three-panel layout rendering
│   ├── theme.rs      // Color themes, focus indicators
│   └── widgets.rs    // Custom UI widgets
└── utils/            // Shared utilities
    ├── command.rs    // Safe command execution
    └── logger.rs     // Structured logging
```

### 5.1. External Command Interaction

- **Android:** 
  - `avdmanager`: Create, delete, list AVDs with dynamic system image detection
  - `emulator`: Start AVDs with proper serial mapping
  - `adb`: Stop devices, logcat streaming, status detection
- **iOS (macOS):** 
  - `xcrun simctl`: List, create, boot, shutdown simulators
  - Dynamic runtime detection and device type mapping
- All commands use async execution with proper error handling and timeout management.

### 5.2. Error Handling

- Custom error types using `thiserror` for derive macros
- User-friendly error formatting with `format_user_error` utility
- Context propagation using `anyhow` with descriptive messages
- Notification system for displaying errors in the UI
- Never use `.unwrap()` or `.expect()` in user-facing code

## 6. Current Implementation Status

### Completed Features ✅

1. **Core Architecture:** Async-first design with trait-based abstractions
2. **Android Support:** Complete lifecycle management, real-time logs, device details
3. **iOS Support:** Basic operations on macOS with dynamic runtime detection
4. **Three-Panel UI:** 30% | 30% | 40% layout with device details panel
5. **Performance:** < 150ms startup (typical ~104ms), smart caching, debounced updates
6. **Testing:** 15 test files, 31+ test functions covering all major functionality
7. **Real-time Updates:** Live status monitoring, log streaming with < 10ms latency

### Pending Features 🚀

1. **Confirmation Dialogs:** Delete and wipe operations need user confirmation
2. **Android Wipe:** Cold boot functionality implementation
3. **Progress Indicators:** For long-running operations
4. **Additional Documentation:** Architecture diagrams, contribution guidelines

## 7. Other Requirements

### 7.1. Performance & Testing

- **Startup Performance:** < 150ms (typical: ~104ms)
- **UI Responsiveness:** Panel switching < 100ms, navigation < 50ms
- **Test Coverage:** 15 test files with 31+ test functions
- **Memory Management:** Automatic log rotation (1000 entries max)
- **Background Loading:** Non-blocking device data loading on startup
- **Smart Caching:** Platform-aware cache invalidation

---

This document serves as a detailed starting point. Specific implementation details may evolve during development.
