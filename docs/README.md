# Emu: Technical Documentation

This document provides a more detailed technical overview of the Emu project, based on the initial project brief.

## 1. Project Overview

**Project Name:** Emu

**Goal:** To create a Terminal User Interface (TUI) tool, inspired by lazygit, for unified management of Android emulators and iOS simulators. The primary aim is to enhance mobile developer productivity by offering intuitive, keyboard-driven navigation and operations.

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

- **API Level Selection:** Allow users to choose from a list of available Android API levels.
- **Device Type Selection:** Support for various device types (Phone, Tablet, TV, Wear OS, etc.).
- **AVD Name:** Auto-generate AVD names or allow custom names.
- **Advanced Configuration Options:** Provide options for setting storage size, RAM, and other AVD parameters.

#### 3.1.2. Listing and Operations

- **List AVDs:** Display a list of all created AVDs, showing:
  - Name
  - API Level
  - Device Type
  - Current Status (e.g., Running, Stopped)
- **Start/Stop:** Manage the running state of AVDs.
- **Delete AVD:** Functionality to delete AVDs, including a confirmation dialog.
- **Wipe Data (Cold Boot):** Option to wipe data from an AVD.

### 3.2. iOS Simulator Management (via `simctl` command on macOS)

#### 3.2.1. Simulator Launch

- **List Available Runtimes:** Show available iOS versions/runtimes.
- **Device Type Selection:** Allow selection from various iPhone and iPad models.
- **Display Status:** Indicate the running state of simulators.
- _(Note: iOS simulator creation is typically handled by Xcode installation and runtime downloads. Emu will focus on listing and launching existing simulator configurations.)_

### 3.3. Common Features

- **Tabbed Interface:** Separate views for Android emulators, iOS simulators, and potentially settings.
- **Keyboard Navigation:** Intuitive keybindings for all major operations.
- **Status Updates:** Clear feedback on ongoing operations and their results.
- **Configuration:** Allow users to customize some aspects of the tool via a configuration file.
- **Logging:** Implement logging for debugging and issue tracking.
- **Error Handling:** Robust error handling with user-friendly messages.

## 4. UI/UX Requirements

### 4.1. Screen Layout (Conceptual)

A multi-pane layout, potentially with tabs:

```
┌──────────────────────────────────────────────────┐
│ Emu v0.1.0                                       │
├──────────────────────────────────────────────────┤
│ [Android]  [iOS]  [Settings]  [Logs]             │
├──────────────────────────────────────────────────┤
│ ┌─ Android Emulators (Tab) ────────────────────┐ │
│ │ ► Pixel_6_API_33      (Running)              │ │
│ │   Nexus_5_API_28      (Stopped)              │ │
│ │   Resizable_API_30    (Stopped)              │ │
│ │                                                │ │
│ └────────────────────────────────────────────────┘ │
│ ┌─ Device Details / Output (Pane) ──────────────┐ │
│ │ Name: Pixel_6_API_33                           │ │
│ │ State: Running                                 │ │
│ │ API: 33                                        │ │
│ │ Target: android-33                             │ │
│ │ Path: /Users/user/.android/avd/Pixel_6_API_33  │ │
│ └────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────┤
│ [s]tart/stop [c]reate [d]elete [w]ipe [r]efresh  │
│ [f]ilter logs [Page Up/Down] scroll [q]uit       │
└──────────────────────────────────────────────────┘
```

### 4.2. Keybindings (Examples)

- **Global:**
  - `Tab` / `Shift+Tab`: Cycle through main tabs/panels.
  - `q`: Quit application.
  - `?`: Show help/keybindings.
- **List Navigation:**
  - `↑` / `k`: Move up.
  - `↓` / `j`: Move down.
  - `Page Up` / `Page Down`: Scroll device lists.
- **Emulator/Simulator Actions:**
  - `Enter`: Start/Stop selected device.
  - `c`: Initiate creation wizard for the current platform.
  - `d`: Delete selected device (with confirmation).
  - `w`: Wipe data for selected Android AVD (with confirmation).
  - `r`: Refresh device list for the current platform.
- _(Specific keybindings will be refined during development and documented within the app.)_

## 5. Architecture (Conceptual)

```
src/
├── main.rs           // Entry point, CLI parsing, TUI/headless mode dispatch
├── app.rs            // Main application state and logic for TUI
├── ui/
│   ├── mod.rs
│   ├── layout.rs     // Defines TUI layout and panes
│   ├── event.rs      // Handles input events (keyboard, mouse)
│   ├── components/   // Reusable TUI components (lists, popups, etc.)
│   │   ├── mod.rs
│   │   ├── device_list.rs
│   │   └── ...
│   ├── views/        // Specific views for Android, iOS, Settings
│   │   ├── mod.rs
│   │   ├── android_view.rs
│   │   ├── ios_view.rs
│   │   └── settings_view.rs
│   └── theme.rs      // Colors, styles
├── managers/
│   ├── mod.rs
│   ├── android_manager.rs // Logic for interacting with `avdmanager`, `emulator`
│   ├── ios_manager.rs     // Logic for interacting with `simctl`
│   └── common.rs          // Shared utilities for managers
├── models/
│   ├── mod.rs
│   ├── device.rs     // Data structures for emulators/simulators
│   ├── config.rs     // Application configuration structure
│   └── error.rs      // Custom error types
└── utils/
    ├── mod.rs
    ├── command.rs    // Wrapper for executing external commands
    └── logger.rs     // Logging setup and utilities
```

### 5.1. External Command Interaction

- **Android:** Wrap `avdmanager` (for listing, creating, deleting AVDs) and `emulator` (for starting, stopping AVDs).
- **iOS (macOS):** Wrap `simctl` (for listing devices/runtimes, launching, shutting down simulators).
- All external command calls should be asynchronous and handle `stdout`, `stderr`, and exit codes properly.

### 5.2. Error Handling

- Define custom error types (`Error` enum in `models/error.rs`) that can wrap I/O errors, command execution errors, parsing errors, etc.
- Provide clear, user-facing error messages in both TUI and headless modes.

## 6. Implementation Phases (Suggested)

### Phase 1: Core Functionality & Android Support

1.  **Basic TUI Structure:** Implement main layout, tabs, and basic event loop.
2.  **Android AVD Listing:** Implement `android_manager.rs` to list existing AVDs.
3.  **Android AVD Start/Stop:** Implement basic start/stop functionality.
4.  **Headless Mode Basics:** Implement CLI parsing and headless equivalents for list, start, stop.
5.  **Logging Setup:** Basic logging infrastructure.

### Phase 2: Advanced Android Features & iOS Support

1.  **Android AVD Creation Wizard:** TUI flow for creating new AVDs.
2.  **Android AVD Delete/Wipe:** Implement these operations with confirmations.
3.  **iOS Simulator Listing & Launch (macOS):** Implement `ios_manager.rs` and UI components.

### Phase 3: Refinement and Polish

1.  **Error Handling:** Comprehensive error reporting and recovery.
2.  **Performance Optimization:** Especially for listing devices and long-running commands.
3.  **UI/UX Enhancements:** Improve visual appeal, add more informative widgets, refine keybindings.
4.  **Cross-Platform Testing:** Thorough testing on macOS, Windows, and Linux.
5.  **Documentation:** User guides, contribution guidelines.
6.  **Settings View:** Implement a TUI view for managing application settings.

## 7. Other Requirements

### 7.1. Logging

- **Operational Logs:** Record key actions performed by the user or system.
- **Error Details:** Output detailed error information for troubleshooting.
- **Debug Mode:** A more verbose logging level activated by a flag or setting.
- Log files should be stored in a standard user-specific directory (e.g., `$XDG_CACHE_HOME/emu/app.log`).

---

This document serves as a detailed starting point. Specific implementation details may evolve during development.
